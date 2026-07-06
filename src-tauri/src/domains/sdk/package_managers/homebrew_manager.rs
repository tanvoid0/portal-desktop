use super::super::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageManager, PackageUpdate,
};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * Homebrew Package Manager Implementation
 *
 * Homebrew (macOS/Linux) implementation
 */
use async_trait::async_trait;

pub struct HomebrewManager;

impl HomebrewManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_brew(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("brew", args, None)
            .await
            .map_err(|e| SDKError::CommandFailed(format!("Homebrew command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "Homebrew error: {}",
                result.stderr
            )))
        }
    }
}

#[async_trait]
impl PackageManager for HomebrewManager {
    fn name(&self) -> &'static str {
        "homebrew"
    }

    fn display_name(&self) -> &'static str {
        "Homebrew"
    }

    fn platform(&self) -> &'static str {
        "macos"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("brew").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_brew(&["--version"]).await?;
        Ok(output.lines().next().unwrap_or("").trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        let output = self.execute_brew(&["search", query]).await?;

        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("==>") {
                continue;
            }

            let name = line.to_string();
            packages.push(Package {
                id: name.clone(),
                name,
                version: None,
                publisher: None,
                description: None,
                homepage: None,
                license: None,
                source: "homebrew".to_string(),
            });
        }

        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_brew(&["list", "--versions"]).await?;

        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].to_string();
                let version = parts.get(1).unwrap_or(&"unknown").to_string();

                packages.push(InstalledPackage {
                    id: name.clone(),
                    name,
                    version: version.clone(),
                    installed_version: Some(version),
                    available_version: None,
                    source: "homebrew".to_string(),
                });
            }
        }

        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_brew(&["info", id]).await?;

        let mut name = id.to_string();
        let mut description = None;
        let mut homepage = None;

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with(id) && line.contains(':') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    name = parts[0].to_string();
                    description = Some(parts[1].trim().to_string());
                }
            } else if line.starts_with("https://") || line.starts_with("http://") {
                homepage = Some(line.to_string());
            }
        }

        // Homebrew info doesn't provide version in the standard output format
        let version: Option<String> = None;

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher: None,
            description,
            homepage,
            license: None,
            dependencies: Vec::new(),
            source: "homebrew".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        self.execute_brew(&["install", id]).await?;
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_brew(&["upgrade", id]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_brew(&["uninstall", id]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self.execute_brew(&["outdated"]).await?;

        let mut updates = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let name = line.to_string();
            updates.push(PackageUpdate {
                id: name.clone(),
                name,
                current_version: "installed".to_string(),
                available_version: "latest".to_string(),
                source: "homebrew".to_string(),
            });
        }

        Ok(updates)
    }

    fn supports_search(&self) -> bool {
        true
    }

    fn supports_updates(&self) -> bool {
        true
    }

    fn requires_elevation(&self) -> bool {
        false // Homebrew typically doesn't need sudo for user installs
    }
}
