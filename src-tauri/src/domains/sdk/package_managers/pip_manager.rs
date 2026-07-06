use super::super::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageManager, PackageUpdate,
};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * Pip Package Manager Implementation
 *
 * Pip (Python) implementation - cross-platform
 */
use async_trait::async_trait;
use serde_json::Value;

pub struct PipManager;

impl PipManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_pip(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("pip", args, None)
            .await
            .map_err(|e| SDKError::CommandFailed(format!("Pip command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "Pip error: {}",
                result.stderr
            )))
        }
    }
}

#[async_trait]
impl PackageManager for PipManager {
    fn name(&self) -> &'static str {
        "pip"
    }

    fn display_name(&self) -> &'static str {
        "Pip"
    }

    fn platform(&self) -> &'static str {
        "cross-platform"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("pip").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_pip(&["--version"]).await?;
        Ok(output.lines().next().unwrap_or("").trim().to_string())
    }

    async fn search_packages(&self, _query: &str) -> Result<Vec<Package>, SDKError> {
        // Pip search is deprecated, return empty for now
        // Could use PyPI API as alternative
        Ok(Vec::new())
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_pip(&["list", "--format=json"]).await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse pip JSON: {}", e)))?;

        let mut packages = Vec::new();

        if let Some(array) = json.as_array() {
            for item in array {
                if let (Some(name), Some(version)) = (
                    item.get("name").and_then(|v| v.as_str()),
                    item.get("version").and_then(|v| v.as_str()),
                ) {
                    packages.push(InstalledPackage {
                        id: name.to_string(),
                        name: name.to_string(),
                        version: version.to_string(),
                        installed_version: Some(version.to_string()),
                        available_version: None,
                        source: "pip".to_string(),
                    });
                }
            }
        }

        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_pip(&["show", id]).await?;

        let mut name = id.to_string();
        let mut version = None;
        let mut description = None;
        let mut homepage = None;
        let mut license = None;

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Name:") {
                name = line.replace("Name:", "").trim().to_string();
            } else if line.starts_with("Version:") {
                version = Some(line.replace("Version:", "").trim().to_string());
            } else if line.starts_with("Summary:") {
                description = Some(line.replace("Summary:", "").trim().to_string());
            } else if line.starts_with("Home-page:") {
                homepage = Some(line.replace("Home-page:", "").trim().to_string());
            } else if line.starts_with("License:") {
                license = Some(line.replace("License:", "").trim().to_string());
            }
        }

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher: None,
            description,
            homepage,
            license,
            dependencies: Vec::new(),
            source: "pip".to_string(),
        })
    }

    async fn install_package(&self, id: &str, version: Option<&str>) -> Result<(), SDKError> {
        if let Some(ver) = version {
            self.execute_pip(&["install", &format!("{}=={}", id, ver)])
                .await?;
        } else {
            self.execute_pip(&["install", id]).await?;
        }
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_pip(&["install", "--upgrade", id]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_pip(&["uninstall", id, "-y"]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self
            .execute_pip(&["list", "--outdated", "--format=json"])
            .await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse pip JSON: {}", e)))?;

        let mut updates = Vec::new();

        if let Some(array) = json.as_array() {
            for item in array {
                if let (Some(name), Some(current), Some(latest)) = (
                    item.get("name").and_then(|v| v.as_str()),
                    item.get("version").and_then(|v| v.as_str()),
                    item.get("latest_version").and_then(|v| v.as_str()),
                ) {
                    updates.push(PackageUpdate {
                        id: name.to_string(),
                        name: name.to_string(),
                        current_version: current.to_string(),
                        available_version: latest.to_string(),
                        source: "pip".to_string(),
                    });
                }
            }
        }

        Ok(updates)
    }

    fn supports_search(&self) -> bool {
        false // Pip search is deprecated
    }

    fn supports_updates(&self) -> bool {
        true
    }

    fn requires_elevation(&self) -> bool {
        false
    }
}
