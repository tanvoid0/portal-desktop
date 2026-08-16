use super::super::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageManager, PackageUpdate,
};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * Scoop Package Manager Implementation
 *
 * Scoop (Windows) implementation
 */
use async_trait::async_trait;

pub struct ScoopManager;

impl ScoopManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_scoop(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("scoop", args, None)
            .await
            .map_err(|e| SDKError::CommandFailed(format!("Scoop command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "Scoop error: {}",
                result.stderr
            )))
        }
    }

    fn parse_package_list(&self, output: &str) -> Vec<InstalledPackage> {
        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Name") || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();

                packages.push(InstalledPackage {
                    id: name.clone(),
                    name,
                    version: version.clone(),
                    installed_version: Some(version),
                    available_version: None,
                    source: "scoop".to_string(),
                });
            }
        }

        packages
    }
}

#[async_trait]
impl PackageManager for ScoopManager {
    fn name(&self) -> &'static str {
        "scoop"
    }

    fn display_name(&self) -> &'static str {
        "Scoop"
    }

    fn platform(&self) -> &'static str {
        "windows"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("scoop").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_scoop(&["--version"]).await?;
        Ok(output.trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        let output = self.execute_scoop(&["search", query]).await?;

        let mut packages = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Name") || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].to_string();

                packages.push(Package {
                    id: name.clone(),
                    name,
                    version: parts.get(1).map(|s| s.to_string()),
                    publisher: None,
                    description: None,
                    homepage: None,
                    license: None,
                    source: "scoop".to_string(),
                });
            }
        }

        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_scoop(&["list"]).await?;
        Ok(self.parse_package_list(&output))
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_scoop(&["info", id]).await?;

        // Parse scoop info output
        let mut name = id.to_string();
        let mut version = None;
        let mut description = None;
        let mut homepage = None;

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Name:") {
                name = line.replace("Name:", "").trim().to_string();
            } else if line.starts_with("Version:") {
                version = Some(line.replace("Version:", "").trim().to_string());
            } else if line.starts_with("Description:") {
                description = Some(line.replace("Description:", "").trim().to_string());
            } else if line.starts_with("Website:") {
                homepage = Some(line.replace("Website:", "").trim().to_string());
            }
        }

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher: None,
            description,
            homepage,
            license: None,
            dependencies: Vec::new(),
            source: "scoop".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        self.execute_scoop(&["install", id]).await?;
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_scoop(&["update", id]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_scoop(&["uninstall", id]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self.execute_scoop(&["status"]).await?;

        let mut updates = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.contains("WARN") || line.contains("Update available") {
                // Parse update information from status output
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    updates.push(PackageUpdate {
                        id: name.clone(),
                        name,
                        current_version: "unknown".to_string(),
                        available_version: "latest".to_string(),
                        source: "scoop".to_string(),
                    });
                }
            }
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
        false
    }
}
