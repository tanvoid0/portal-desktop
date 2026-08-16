use super::super::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageManager, PackageUpdate,
};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * Cargo Package Manager Implementation
 *
 * Cargo (Rust) implementation - cross-platform
 */
use async_trait::async_trait;

pub struct CargoManager;

impl CargoManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_cargo(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("cargo", args, None)
            .await
            .map_err(|e| SDKError::CommandFailed(format!("Cargo command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "Cargo error: {}",
                result.stderr
            )))
        }
    }
}

#[async_trait]
impl PackageManager for CargoManager {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn display_name(&self) -> &'static str {
        "Cargo"
    }

    fn platform(&self) -> &'static str {
        "cross-platform"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("cargo").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_cargo(&["--version"]).await?;
        Ok(output.trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        let output = self
            .execute_cargo(&["search", query, "--limit", "50"])
            .await?;

        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("    Updating") {
                continue;
            }

            // Cargo search format: package_name = "version" # description
            if let Some(equals_pos) = line.find('=') {
                let name = line[..equals_pos].trim().to_string();
                let rest = &line[equals_pos + 1..];

                let version = rest.split('"').nth(1).map(|s| s.to_string());

                let description = rest.split('#').nth(1).map(|s| s.trim().to_string());

                packages.push(Package {
                    id: name.clone(),
                    name,
                    version,
                    publisher: None,
                    description,
                    homepage: None,
                    license: None,
                    source: "cargo".to_string(),
                });
            }
        }

        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_cargo(&["install", "--list"]).await?;

        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Format: package_name version: ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version_part = parts[1];
                let version = version_part.trim_start_matches("v").to_string();

                packages.push(InstalledPackage {
                    id: name.clone(),
                    name,
                    version: version.clone(),
                    installed_version: Some(version),
                    available_version: None,
                    source: "cargo".to_string(),
                });
            }
        }

        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_cargo(&["search", id, "--limit", "1"]).await?;

        let mut name = id.to_string();
        let mut version = None;
        let mut description = None;

        for line in output.lines() {
            let line = line.trim();
            if line.contains('=') {
                if let Some(equals_pos) = line.find('=') {
                    name = line[..equals_pos].trim().to_string();
                    let rest = &line[equals_pos + 1..];
                    version = rest.split('"').nth(1).map(|s| s.to_string());
                    description = rest.split('#').nth(1).map(|s| s.trim().to_string());
                }
                break;
            }
        }

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher: None,
            description,
            homepage: None,
            license: None,
            dependencies: Vec::new(),
            source: "cargo".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        self.execute_cargo(&["install", id]).await?;
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        // Cargo doesn't have upgrade, reinstall to get latest
        self.execute_cargo(&["install", "--force", id]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_cargo(&["uninstall", id]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        // Cargo doesn't have a built-in update check, return empty for now
        Ok(Vec::new())
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn supports_updates(&self) -> bool {
        false // Cargo doesn't have built-in update checking
    }

    fn requires_elevation(&self) -> bool {
        false
    }
}
