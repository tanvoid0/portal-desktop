use super::super::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageManager, PackageUpdate,
};
use super::super::SDKError;
use crate::command_executor::CommandExecutor;
/**
 * NPM Package Manager Implementation
 *
 * NPM (Node.js) implementation - cross-platform
 */
use async_trait::async_trait;
use serde_json::Value;

pub struct NpmManager;

impl NpmManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_npm(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("npm", args, None)
            .await
            .map_err(|e| SDKError::CommandFailed(format!("NPM command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!(
                "NPM error: {}",
                result.stderr
            )))
        }
    }
}

#[async_trait]
impl PackageManager for NpmManager {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn display_name(&self) -> &'static str {
        "NPM"
    }

    fn platform(&self) -> &'static str {
        "cross-platform"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("npm").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_npm(&["--version"]).await?;
        Ok(output.trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        let output = self.execute_npm(&["search", query, "--json"]).await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse NPM JSON: {}", e)))?;

        let mut packages = Vec::new();

        if let Some(array) = json.as_array() {
            for item in array {
                if let (Some(name), Some(desc)) = (
                    item.get("name").and_then(|v| v.as_str()),
                    item.get("description").and_then(|v| v.as_str()),
                ) {
                    let version = item
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let publisher = item
                        .get("publisher")
                        .and_then(|v| v.get("username"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    packages.push(Package {
                        id: name.to_string(),
                        name: name.to_string(),
                        version,
                        publisher,
                        description: Some(desc.to_string()),
                        homepage: None,
                        license: None,
                        source: "npm".to_string(),
                    });
                }
            }
        }

        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self
            .execute_npm(&["list", "-g", "--depth=0", "--json"])
            .await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse NPM JSON: {}", e)))?;

        let mut packages = Vec::new();

        if let Some(dependencies) = json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, info) in dependencies {
                let version = info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                packages.push(InstalledPackage {
                    id: name.clone(),
                    name: name.clone(),
                    version: version.clone(),
                    installed_version: Some(version),
                    available_version: None,
                    source: "npm".to_string(),
                });
            }
        }

        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_npm(&["view", id, "--json"]).await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse NPM JSON: {}", e)))?;

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();

        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let homepage = json
            .get("homepage")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let license = json
            .get("license")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let publisher = json
            .get("publisher")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let dependencies = if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object())
        {
            deps.keys().map(|k| k.clone()).collect()
        } else {
            Vec::new()
        };

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher,
            description,
            homepage,
            license,
            dependencies,
            source: "npm".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        self.execute_npm(&["install", "-g", id]).await?;
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_npm(&["update", "-g", id]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_npm(&["uninstall", "-g", id]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self.execute_npm(&["outdated", "-g", "--json"]).await?;

        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse NPM JSON: {}", e)))?;

        let mut updates = Vec::new();

        if let Some(obj) = json.as_object() {
            for (name, info) in obj {
                if let (Some(current), Some(latest)) = (
                    info.get("current").and_then(|v| v.as_str()),
                    info.get("latest").and_then(|v| v.as_str()),
                ) {
                    updates.push(PackageUpdate {
                        id: name.clone(),
                        name: name.clone(),
                        current_version: current.to_string(),
                        available_version: latest.to_string(),
                        source: "npm".to_string(),
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
