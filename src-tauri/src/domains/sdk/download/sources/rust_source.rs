/**
 * Rust Source Implementation
 * Fetches versions from official channel manifests
 */

use super::super::VersionInfo;
use crate::domains::sdk::SDKError;
use reqwest::Client;
use std::collections::HashMap;

pub struct RustSource {
    client: Client,
}

impl RustSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch all available Rust versions from official channel manifests
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        let response = self.client
            .get("https://static.rust-lang.org/dist/channel-rust-stable.toml")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Rust versions: {}", e)))?;

        let manifest = response
            .text()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse Rust manifest: {}", e)))?;

        // Parse TOML manifest to extract version
        let version = self.extract_rust_version(&manifest)?;
        
        let mut download_urls = HashMap::new();
        let (platform, arch) = self.get_platform_info();
        let platform_arch = format!("{}-{}", platform, arch);
        
        // Rust uses different naming conventions
        let rust_platform = match (platform.as_str(), arch.as_str()) {
            ("darwin", "x64") => "x86_64-apple-darwin",
            ("darwin", "arm64") => "aarch64-apple-darwin",
            ("linux", "x64") => "x86_64-unknown-linux-gnu",
            ("linux", "arm64") => "aarch64-unknown-linux-gnu",
            ("win32", "x64") => "x86_64-pc-windows-msvc",
            ("win32", "arm64") => "aarch64-pc-windows-msvc",
            _ => return Ok(vec![]),
        };

        let base_url = format!("https://static.rust-lang.org/dist/rust-{}-{}.tar.gz", version, rust_platform);
        download_urls.insert(platform_arch, base_url);

        Ok(vec![VersionInfo {
            version: version.clone(),
            lts: false,
            release_date: None,
            download_urls,
            checksum: None,
            description: Some(format!("Rust {}", version)),
        }])
    }

    /// Extract version from Rust TOML manifest
    fn extract_rust_version(&self, manifest: &str) -> Result<String, SDKError> {
        for line in manifest.lines() {
            if line.starts_with("version = ") {
                let version = line
                    .trim_start_matches("version = ")
                    .trim_matches('"')
                    .to_string();
                return Ok(version);
            }
        }
        Err(SDKError::ManagerNotFound("Could not extract Rust version from manifest".to_string()))
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
