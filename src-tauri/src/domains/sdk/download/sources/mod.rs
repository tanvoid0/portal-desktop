/**
 * Version Sources Module
 * 
 * Individual source implementations for different SDKs
 */

pub mod nodejs_source;
pub mod python_source;
pub mod java_source;
pub mod rust_source;
pub mod go_source;

pub use nodejs_source::NodejsSource;
pub use python_source::PythonSource;
pub use java_source::JavaSource;
pub use rust_source::RustSource;
pub use go_source::GoSource;

use super::super::SDKError;
use super::VersionInfo;

/// Trait for version sources
pub trait VersionSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError>;
    async fn get_download_url(&self, version: &str, os: &str, arch: &str) -> Result<String, SDKError>;
}

impl VersionSource for NodejsSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        self.fetch_versions().await
    }

    async fn get_download_url(&self, version: &str, os: &str, arch: &str) -> Result<String, SDKError> {
        let urls = self.get_download_urls(version);
        let platform_arch = format!("{}-{}", os, arch);
        urls.get(&platform_arch)
            .ok_or_else(|| SDKError::ManagerNotFound(format!("No download URL for platform {}", platform_arch)))
            .map(|url| url.clone())
    }
}

impl VersionSource for PythonSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        self.fetch_versions().await
    }

    async fn get_download_url(&self, _version: &str, _os: &str, _arch: &str) -> Result<String, SDKError> {
        Err(SDKError::ManagerNotFound("Python source doesn't support direct URL generation".to_string()))
    }
}

impl VersionSource for JavaSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        self.fetch_versions().await
    }

    async fn get_download_url(&self, _version: &str, _os: &str, _arch: &str) -> Result<String, SDKError> {
        Err(SDKError::ManagerNotFound("Java source doesn't support direct URL generation".to_string()))
    }
}

impl VersionSource for RustSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        self.fetch_versions().await
    }

    async fn get_download_url(&self, _version: &str, _os: &str, _arch: &str) -> Result<String, SDKError> {
        Err(SDKError::ManagerNotFound("Rust source doesn't support direct URL generation".to_string()))
    }
}

impl VersionSource for GoSource {
    async fn fetch_versions(&self) -> Result<Vec<VersionInfo>, SDKError> {
        self.fetch_versions().await
    }

    async fn get_download_url(&self, _version: &str, _os: &str, _arch: &str) -> Result<String, SDKError> {
        Err(SDKError::ManagerNotFound("Go source doesn't support direct URL generation".to_string()))
    }
}
