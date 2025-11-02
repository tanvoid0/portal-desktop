/**
 * Java Source Implementation
 * Fetches versions from Adoptium API
 */

use super::super::VersionInfo;
use crate::domains::sdk::SDKError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct AdoptiumRelease {
    version: AdoptiumVersion,
    release_name: String,
    release_type: String,
    timestamp: String,
    binaries: Vec<AdoptiumBinary>,
}

#[derive(Deserialize)]
struct AdoptiumVersion {
    major: u32,
    minor: u32,
    security: u32,
    patch: u32,
}

#[derive(Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
    os: String,
    architecture: String,
}

#[derive(Deserialize)]
struct AdoptiumPackage {
    name: String,
    link: String,
    checksum: String,
}

pub struct JavaSource {
    client: Client,
}

impl JavaSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch all available Java versions from Adoptium API
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        let response = self.client
            .get("https://api.adoptium.net/v3/assets/latest/8,11,17,21,22/hotspot")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Java versions: {}", e)))?;

        let releases: Vec<AdoptiumRelease> = response
            .json()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse Java releases: {}", e)))?;

        let mut versions = Vec::new();
        
        for release in releases {
            let version = format!("{}.{}.{}.{}", 
                release.version.major, 
                release.version.minor, 
                release.version.security, 
                release.version.patch
            );

            let mut download_urls = HashMap::new();
            for binary in &release.binaries {
                let platform_arch = format!("{}-{}", binary.os, binary.architecture);
                download_urls.insert(platform_arch, binary.package.link.clone());
            }

            versions.push(VersionInfo {
                version: version.clone(),
                lts: release.release_type == "lts",
                release_date: Some(release.timestamp),
                download_urls,
                checksum: release.binaries.first().map(|b| b.package.checksum.clone()),
                description: Some(format!("OpenJDK {}", version)),
            });
        }

        // Sort versions (newest first)
        versions.sort_by(|a, b| {
            use version_compare::Version;
            let a_ver = Version::from(&a.version).unwrap_or(Version::from("0.0.0").unwrap());
            let b_ver = Version::from(&b.version).unwrap_or(Version::from("0.0.0").unwrap());
            b_ver.partial_cmp(&a_ver).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(versions)
    }
}
