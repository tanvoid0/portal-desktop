use super::super::SDKError;
/**
 * Package Manager Trait
 *
 * This trait defines the common interface for all package managers (winget, scoop, chocolatey, etc.)
 * Each package manager implementation must provide these methods.
 */
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Package search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub source: String, // manager name
}

/// Installed package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub source: String,
}

/// Detailed package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetails {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
    pub source: String,
}

/// Package update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdate {
    pub id: String,
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub source: String,
}

/// Package Manager trait - all package managers implement this
#[async_trait]
pub trait PackageManager: Send + Sync {
    // === Core Identity ===
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn platform(&self) -> &'static str; // "windows", "linux", "macos", "cross-platform"

    // === Availability & Detection ===
    async fn is_available(&self) -> Result<bool, SDKError>;
    async fn get_version(&self) -> Result<String, SDKError>;

    // === Package Operations ===
    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError>;
    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError>;
    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError>;
    async fn install_package(&self, id: &str, version: Option<&str>) -> Result<(), SDKError>;
    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError>;
    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError>;
    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError>;

    // === Optional Features ===
    fn supports_search(&self) -> bool;
    fn supports_updates(&self) -> bool;
    fn requires_elevation(&self) -> bool;
}
