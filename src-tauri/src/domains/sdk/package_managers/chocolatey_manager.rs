/**
 * Chocolatey Package Manager Implementation
 * 
 * Chocolatey (Windows) implementation
 */

use async_trait::async_trait;
use crate::command_executor::CommandExecutor;
use super::super::SDKError;
use super::super::traits::package_manager::{PackageManager, Package, InstalledPackage, PackageDetails, PackageUpdate};

pub struct ChocolateyManager;

impl ChocolateyManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_choco(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("choco", args, None).await
            .map_err(|e| SDKError::CommandFailed(format!("Chocolatey command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!("Chocolatey error: {}", result.stderr)))
        }
    }
}

#[async_trait]
impl PackageManager for ChocolateyManager {
    fn name(&self) -> &'static str {
        "chocolatey"
    }

    fn display_name(&self) -> &'static str {
        "Chocolatey"
    }

    fn platform(&self) -> &'static str {
        "windows"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("choco").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_choco(&["--version"]).await?;
        Ok(output.trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        let output = self.execute_choco(&["search", query, "--limit-output"]).await?;
        
        let mut packages = Vec::new();
        
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // Chocolatey limit-output format: package|version
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 1 {
                let name = parts[0].to_string();
                let version = parts.get(1).map(|s| s.to_string());
                
                packages.push(Package {
                    id: name.clone(),
                    name,
                    version,
                    publisher: None,
                    description: None,
                    homepage: None,
                    license: None,
                    source: "chocolatey".to_string(),
                });
            }
        }
        
        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_choco(&["list", "--local-only", "--limit-output"]).await?;
        
        let mut packages = Vec::new();
        
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();
                
                packages.push(InstalledPackage {
                    id: name.clone(),
                    name,
                    version: version.clone(),
                    installed_version: Some(version),
                    available_version: None,
                    source: "chocolatey".to_string(),
                });
            }
        }
        
        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        let output = self.execute_choco(&["info", id]).await?;
        
        let mut name = id.to_string();
        let mut version = None;
        let mut description = None;
        
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with(id) && line.contains('|') {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    name = parts[0].to_string();
                    version = Some(parts[1].to_string());
                }
            } else if line.to_lowercase().contains("description") {
                description = Some(line.to_string());
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
            source: "chocolatey".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        self.execute_choco(&["install", id, "-y"]).await?;
        Ok(())
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_choco(&["upgrade", id, "-y"]).await?;
        Ok(())
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        self.execute_choco(&["uninstall", id, "-y"]).await?;
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self.execute_choco(&["outdated"]).await?;
        
        let mut updates = Vec::new();
        
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Chocolatey") || line.starts_with("---") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let current = parts.get(1).unwrap_or(&"unknown").to_string();
                let available = parts.get(2).unwrap_or(&"latest").to_string();
                
                updates.push(PackageUpdate {
                    id: name.clone(),
                    name,
                    current_version: current,
                    available_version: available,
                    source: "chocolatey".to_string(),
                });
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
        true // Chocolatey typically requires admin
    }
}

