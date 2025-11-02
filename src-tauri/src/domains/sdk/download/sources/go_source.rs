/**
 * Go Source Implementation
 * Fetches versions from official Go API
 */

use super::super::VersionInfo;
use crate::domains::sdk::SDKError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct GoRelease {
    version: String,
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize)]
struct GoFile {
    filename: String,
    os: String,
    arch: String,
    sha256: String,
}

pub struct GoSource {
    client: Client,
}

impl GoSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch all available Go versions from official API
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        let response = self.client
            .get("https://go.dev/dl/?mode=json")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Go versions: {}", e)))?;

        let releases: Vec<GoRelease> = response
            .json()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse Go releases: {}", e)))?;

        let mut versions = Vec::new();
        
        for release in releases {
            // Skip beta/rc versions
            if !release.stable || release.version.contains("beta") || release.version.contains("rc") {
                continue;
            }

            let mut download_urls = HashMap::new();
            for file in &release.files {
                let platform_arch = format!("{}-{}", file.os, file.arch);
                let url = format!("https://go.dev/dl/{}", file.filename);
                download_urls.insert(platform_arch, url);
            }

            versions.push(VersionInfo {
                version: release.version.trim_start_matches("go").to_string(),
                lts: false,
                release_date: None,
                download_urls,
                checksum: release.files.first().map(|f| f.sha256.clone()),
                description: Some(format!("Go {}", release.version)),
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
