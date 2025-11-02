/**
 * Version Fetcher
 * 
 * Fetches available versions from official sources for different SDKs
 */

use super::VersionInfo;
use crate::domains::sdk::SDKError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFetcher {
    sdk_type: String,
}

impl VersionFetcher {
    pub fn new(sdk_type: String) -> Self {
        Self { sdk_type }
    }

    /// Fetch all available versions for the SDK type
    pub async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        match self.sdk_type.as_str() {
            "node" => self.fetch_nodejs_versions().await,
            "python" => self.fetch_python_versions().await,
            "java" => self.fetch_java_versions().await,
            "rust" => self.fetch_rust_versions().await,
            "go" => self.fetch_go_versions().await,
            "php" => self.fetch_php_versions().await,
            "ruby" => self.fetch_ruby_versions().await,
            _ => Err(SDKError::ManagerNotFound(format!("Unsupported SDK type: {}", self.sdk_type))),
        }
    }

    /// Fetch Node.js versions from official source
    async fn fetch_nodejs_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://nodejs.org/dist/")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Node.js versions: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse response: {}", e)))?;

        let mut versions = Vec::new();
        let re = regex::Regex::new(r#"href="v([\d.]+?)/""#).unwrap();
        
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

                let download_urls = self.get_nodejs_download_urls(&version_str);
                
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
    fn get_nodejs_download_urls(&self, version: &str) -> HashMap<String, String> {
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

    /// Fetch Python versions from GitHub releases
    async fn fetch_python_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://api.github.com/repos/python/cpython/releases")
            .header("User-Agent", "Portal-Desktop")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Python versions: {}", e)))?;

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

    /// Fetch Java versions from Adoptium API
    async fn fetch_java_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://api.adoptium.net/v3/assets/latest/8,11,17,21,22/hotspot")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Java versions: {}", e)))?;

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

    /// Fetch Rust versions from official channel manifests
    async fn fetch_rust_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
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

    /// Fetch Go versions from official API
    async fn fetch_go_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://go.dev/dl/?mode=json")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Go versions: {}", e)))?;

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

    /// Fetch PHP versions from official releases API
    async fn fetch_php_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://www.php.net/releases/index.php?json")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch PHP versions: {}", e)))?;

        #[derive(Deserialize)]
        struct PhpRelease {
            version: String,
            date: String,
            source: Vec<PhpSource>,
        }

        #[derive(Deserialize)]
        struct PhpSource {
            filename: String,
            name: String,
            md5: String,
        }

        let releases: Vec<PhpRelease> = response
            .json()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse PHP releases: {}", e)))?;

        let mut versions = Vec::new();
        
        for release in releases {
            // Skip very old versions
            if let Some(version_parts) = release.version.split('.').next() {
                if let Ok(major) = version_parts.parse::<u32>() {
                    if major < 7 {
                        continue;
                    }
                }
            }

            let mut download_urls = HashMap::new();
            for source in &release.source {
                if source.filename.ends_with(".tar.gz") {
                    let url = format!("https://www.php.net/get/{}/from/this/mirror", source.filename);
                    download_urls.insert("source".to_string(), url);
                }
            }

            versions.push(VersionInfo {
                version: release.version.clone(),
                lts: false,
                release_date: Some(release.date),
                download_urls,
                checksum: release.source.first().map(|s| s.md5.clone()),
                description: Some(format!("PHP {}", release.version)),
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

    /// Fetch Ruby versions from official cache
    async fn fetch_ruby_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        use reqwest::Client;
        
        let client = Client::new();
        let response = client
            .get("https://cache.ruby-lang.org/pub/ruby/")
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to fetch Ruby versions: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse response: {}", e)))?;

        let mut versions = Vec::new();
        let re = regex::Regex::new(r#"href="(\d+\.\d+\.\d+)/""#).unwrap();
        
        for cap in re.captures_iter(&html) {
            if let Some(version) = cap.get(1) {
                let version_str = version.as_str().to_string();
                
                // Skip very old versions
                if let Some(version_parts) = version_str.split('.').next() {
                    if let Ok(major) = version_parts.parse::<u32>() {
                        if major < 2 {
                            continue;
                        }
                    }
                }

                let mut download_urls = HashMap::new();
                let (platform, arch) = self.get_platform_info();
                let platform_arch = format!("{}-{}", platform, arch);
                
                // Ruby source downloads
                let source_url = format!("https://cache.ruby-lang.org/pub/ruby/{}/ruby-{}.tar.gz", version_str, version_str);
                download_urls.insert(platform_arch, source_url);

                versions.push(VersionInfo {
                    version: version_str.clone(),
                    lts: false,
                    release_date: None,
                    download_urls,
                    checksum: None,
                    description: Some(format!("Ruby {}", version_str)),
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
