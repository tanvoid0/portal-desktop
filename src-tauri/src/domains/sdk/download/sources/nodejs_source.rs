/**
 * Node.js Source Implementation
 * Fetches versions from official Node.js distribution site
 */

use super::super::VersionInfo;
use crate::domains::sdk::SDKError;
use reqwest::Client;
use std::collections::HashMap;
use regex::Regex;

pub struct NodejsSource {
    client: Client,
}

impl NodejsSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch all available Node.js versions from official source
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        let response = self.client
            .get("https://nodejs.org/dist/")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Node.js versions: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse response: {}", e)))?;

        let mut versions = Vec::new();
        let re = Regex::new(r#"href="v([\d.]+?)/""#).unwrap();
        
        for cap in re.captures_iter(&html) {
            if let Some(version) = cap.get(1) {
                let version_str = version.as_str().to_string();
                
                // Skip versions older than 7.x
                if let Some(version_parts) = version_str.split('.').next() {
                    if let Ok(major) = version_parts.parse::<u32>() {
                        if major <= 7 {
                            continue;
                        }
                    }
                }

                let download_urls = self.get_download_urls(&version_str);
                
                versions.push(VersionInfo {
                    version: version_str.clone(),
                    lts: version_str.contains("LTS"),
                    release_date: None,
                    download_urls,
                    checksum: None,
                    description: Some(format!("Node.js {}", version_str)),
                });
            }
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

    /// Get download URLs for Node.js version
    pub fn get_download_urls(&self, version: &str) -> HashMap<String, String> {
        let mut urls = HashMap::new();
        let base_url = format!("https://nodejs.org/dist/v{}/", version);
        
        // Determine platform and architecture
        let (platform, arch) = self.get_platform_info();
        
        let filename = match (platform.as_str(), arch.as_str()) {
            ("darwin", "x64") => format!("node-v{}-darwin-x64.tar.gz", version),
            ("darwin", "arm64") => format!("node-v{}-darwin-arm64.tar.gz", version),
            ("linux", "x64") => format!("node-v{}-linux-x64.tar.xz", version),
            ("linux", "arm64") => format!("node-v{}-linux-arm64.tar.xz", version),
            ("win32", "x64") => format!("node-v{}-win-x64.zip", version),
            ("win32", "arm64") => format!("node-v{}-win-arm64.zip", version),
            _ => return urls,
        };
        
        urls.insert(format!("{}-{}", platform, arch), format!("{}{}", base_url, filename));
        urls
    }

    /// Get current platform and architecture
    fn get_platform_info(&self) -> (String, String) {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        
        let platform = match os {
            "macos" => "darwin",
            "windows" => "win32",
            "linux" => "linux",
            _ => "unknown",
        };
        
        let architecture = match arch {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            _ => "unknown",
        };
        
        (platform.to_string(), architecture.to_string())
    }
}
