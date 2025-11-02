/**
 * Python Source Implementation
 * Fetches versions from GitHub releases API
 */

use super::super::VersionInfo;
use crate::domains::sdk::SDKError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub struct PythonSource {
    client: Client,
}

impl PythonSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch all available Python versions from GitHub releases
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        let response = self.client
            .get("https://api.github.com/repos/python/cpython/releases")
            .header("User-Agent", "Portal-Desktop")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Python versions: {}", e)))?;

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse Python releases: {}", e)))?;

        let mut versions = Vec::new();
        
        for release in releases {
            let version = release.tag_name.trim_start_matches('v').to_string();
            
            // Skip pre-releases and very old versions
            if version.contains("a") || version.contains("b") || version.contains("rc") {
                continue;
            }
            
            if let Some(version_parts) = version.split('.').next() {
                if let Ok(major) = version_parts.parse::<u32>() {
                    if major < 3 {
                        continue;
                    }
                }
            }

            let mut download_urls = HashMap::new();
            for asset in &release.assets {
                if let Some(platform_arch) = self.extract_python_platform(&asset.name) {
                    download_urls.insert(platform_arch, asset.browser_download_url.clone());
                }
            }

            versions.push(VersionInfo {
                version: version.clone(),
                lts: false, // Python doesn't have LTS releases
                release_date: Some(release.published_at),
                download_urls,
                checksum: None,
                description: Some(format!("Python {}", version)),
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

    /// Extract platform info from Python asset name
    fn extract_python_platform(&self, filename: &str) -> Option<String> {
        if filename.contains("macos") && filename.contains("x86_64") {
            Some("darwin-x64".to_string())
        } else if filename.contains("macos") && filename.contains("arm64") {
            Some("darwin-arm64".to_string())
        } else if filename.contains("linux") && filename.contains("x86_64") {
            Some("linux-x64".to_string())
        } else if filename.contains("linux") && filename.contains("aarch64") {
            Some("linux-arm64".to_string())
        } else if filename.contains("windows") && filename.contains("x86_64") {
            Some("win32-x64".to_string())
        } else if filename.contains("windows") && filename.contains("arm64") {
            Some("win32-arm64".to_string())
        } else {
            None
        }
    }
}
