/**
 * Winget Package Manager Implementation
 * 
 * Windows Package Manager (winget) implementation
 */

use async_trait::async_trait;
use serde_json::Value;
use regex::Regex;
use crate::command_executor::CommandExecutor;
use super::super::SDKError;
use super::super::traits::package_manager::{PackageManager, Package, InstalledPackage, PackageDetails, PackageUpdate};

pub struct WingetManager;

impl WingetManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_winget(&self, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args("winget", args, None).await
            .map_err(|e| SDKError::CommandFailed(format!("Winget command failed: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::CommandFailed(format!("Winget error: {}", result.stderr)))
        }
    }

    #[allow(dead_code)]
    fn parse_package_from_json(&self, value: &Value, source: &str) -> Option<Package> {
        let id = value.get("Id")?.as_str()?.to_string();
        let name = value.get("Name")?.as_str()?.to_string();
        let version = value.get("Version").and_then(|v| v.as_str()).map(|s| s.to_string());
        let publisher = value.get("Publisher").and_then(|v| v.as_str()).map(|s| s.to_string());
        let description = value.get("Description").and_then(|v| v.as_str()).map(|s| s.to_string());

        Some(Package {
            id,
            name,
            version,
            publisher,
            description,
            homepage: None,
            license: None,
            source: source.to_string(),
        })
    }
}

#[async_trait]
impl PackageManager for WingetManager {
    fn name(&self) -> &'static str {
        "winget"
    }

    fn display_name(&self) -> &'static str {
        "Windows Package Manager"
    }

    fn platform(&self) -> &'static str {
        "windows"
    }

    async fn is_available(&self) -> Result<bool, SDKError> {
        match CommandExecutor::command_exists("winget").await {
            true => Ok(true),
            false => Ok(false),
        }
    }

    async fn get_version(&self) -> Result<String, SDKError> {
        let output = self.execute_winget(&["--version"]).await?;
        Ok(output.trim().to_string())
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<Package>, SDKError> {
        // Winget search doesn't support --output json, so we parse fixed-width table output
        // Columns are separated by 2+ spaces. Format: Name  Id  Version  Match  Source
        let output = self.execute_winget(&["search", query]).await?;
        
        let mut packages = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        // Find the header separator line (dashes)
        let mut header_index = None;
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("---") {
                header_index = Some(i);
                break;
            }
        }
        
        let header_index = match header_index {
            Some(idx) => idx,
            None => return Ok(packages), // No data found
        };
        
        // Use regex to split on 2+ spaces (column separator in fixed-width table)
        let re = Regex::new(r"\s{2,}").unwrap();
        
        // Parse data rows (skip header and separator)
        for line in lines.iter().skip(header_index + 1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Split by 2+ spaces to get columns
            let parts: Vec<&str> = re.split(trimmed).collect();
            
            // Winget search output: Name, Id, Version, Match, Source
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let id = parts[1].trim().to_string();
                let version = if parts.len() > 2 && parts[2].trim() != "Unknown" && !parts[2].trim().is_empty() {
                    Some(parts[2].trim().to_string())
                } else {
                    None
                };
                let source = if parts.len() > 4 {
                    parts[4].trim().to_string()
                } else if parts.len() > 3 {
                    parts[3].trim().to_string()
                } else {
                    "winget".to_string()
                };
                
                if !id.is_empty() && !name.is_empty() {
                    packages.push(Package {
                        id: id.clone(),
                        name,
                        version,
                        publisher: None,
                        description: None,
                        homepage: None,
                        license: None,
                        source: if source.is_empty() { "winget".to_string() } else { source },
                    });
                }
            }
        }

        Ok(packages)
    }

    async fn get_installed_packages(&self) -> Result<Vec<InstalledPackage>, SDKError> {
        let output = self.execute_winget(&["list", "--output", "json"]).await?;
        
        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse winget JSON: {}", e)))?;

        let mut packages = Vec::new();
        
        if let Some(array) = json.as_array() {
            for item in array {
                if let (Some(id), Some(name), Some(version)) = (
                    item.get("Id").and_then(|v| v.as_str()),
                    item.get("Name").and_then(|v| v.as_str()),
                    item.get("Version").and_then(|v| v.as_str()),
                ) {
                    packages.push(InstalledPackage {
                        id: id.to_string(),
                        name: name.to_string(),
                        version: version.to_string(),
                        installed_version: Some(version.to_string()),
                        available_version: None,
                        source: "winget".to_string(),
                    });
                }
            }
        }

        Ok(packages)
    }

    async fn get_package_details(&self, id: &str) -> Result<PackageDetails, SDKError> {
        // Winget show doesn't support --output json, so we parse text output
        // Format: "Found [Name] [Id]" followed by key-value pairs
        let output = self.execute_winget(&["show", id]).await?;
        
        let mut name = String::new();
        let mut version = None;
        let mut publisher = None;
        let mut description = None;
        let mut homepage = None;
        let mut license = None;
        let dependencies = Vec::new();
        
        let lines: Vec<&str> = output.lines().collect();
        let mut in_description = false;
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                in_description = false;
                continue;
            }
            
            // Parse "Found [Name] [Id]" line
            if trimmed.starts_with("Found ") {
                // Format: "Found Microsoft Visual Studio Code [Microsoft.VisualStudioCode]"
                if let Some(bracket_start) = trimmed.find('[') {
                    let found_part = &trimmed[6..bracket_start].trim(); // Skip "Found "
                    name = found_part.to_string();
                }
                continue;
            }
            
            // Parse key-value pairs (e.g., "Version: 1.106.3")
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();
                
                match key {
                    "Version" => {
                        version = Some(value.to_string());
                        in_description = false;
                    }
                    "Publisher" => {
                        publisher = Some(value.to_string());
                        in_description = false;
                    }
                    "Description" => {
                        description = Some(value.to_string());
                        in_description = true;
                    }
                    "Homepage" => {
                        homepage = Some(value.to_string());
                        in_description = false;
                    }
                    "Publisher Url" => {
                        if homepage.is_none() {
                            homepage = Some(value.to_string());
                        }
                        in_description = false;
                    }
                    "License" => {
                        license = Some(value.to_string());
                        in_description = false;
                    }
                    "License Url" => {
                        // Keep existing license text, just note we have a URL
                        in_description = false;
                    }
                    _ => {
                        in_description = false;
                    }
                }
            } else if in_description && !trimmed.starts_with("---") {
                // Continuation of description (multi-line)
                if let Some(desc) = &mut description {
                    desc.push_str(" ");
                    desc.push_str(trimmed);
                }
            } else {
                in_description = false;
            }
        }
        
        if name.is_empty() {
            return Err(SDKError::CommandFailed("Failed to parse package details: missing name".to_string()));
        }

        Ok(PackageDetails {
            id: id.to_string(),
            name,
            version,
            publisher,
            description,
            homepage,
            license,
            dependencies,
            source: "winget".to_string(),
        })
    }

    async fn install_package(&self, id: &str, _version: Option<&str>) -> Result<(), SDKError> {
        // Use -h for silent mode (--silent is not a valid flag)
        // Note: install commands are long-running, so we spawn and return immediately
        use std::process::Command;
        
        let mut cmd = Command::new("winget");
        cmd.args(&[
            "install",
            id,
            "--accept-package-agreements",
            "--accept-source-agreements",
            "-h"
        ]);
        
        // Spawn the process and don't wait for it to complete
        // This allows the install to run in the background
        match cmd.spawn() {
            Ok(_) => Ok(()),
            Err(e) => Err(SDKError::CommandFailed(format!("Failed to start winget install: {}", e)))
        }
    }

    async fn upgrade_package(&self, id: &str) -> Result<(), SDKError> {
        // Use -h for silent mode (--silent is not a valid flag)
        // Note: upgrade commands are long-running, so we spawn and return immediately
        use std::process::Command;
        
        let mut cmd = Command::new("winget");
        cmd.args(&[
            "upgrade",
            id,
            "--accept-package-agreements",
            "--accept-source-agreements",
            "-h"
        ]);
        
        match cmd.spawn() {
            Ok(_) => Ok(()),
            Err(e) => Err(SDKError::CommandFailed(format!("Failed to start winget upgrade: {}", e)))
        }
    }

    async fn uninstall_package(&self, id: &str) -> Result<(), SDKError> {
        // Use -h for silent mode (--silent is not a valid flag)
        // Note: uninstall commands may take time
        let result = CommandExecutor::execute_with_args("winget", &["uninstall", id, "-h"], None).await
            .map_err(|e| SDKError::CommandFailed(format!("Winget command failed: {}", e)))?;
        
        if !result.success && !result.stderr.trim().is_empty() {
            return Err(SDKError::CommandFailed(format!("Winget uninstall failed: {}", result.stderr)));
        }
        
        Ok(())
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>, SDKError> {
        let output = self.execute_winget(&["upgrade", "--list", "--output", "json"]).await?;
        
        let json: Value = serde_json::from_str(&output)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to parse winget JSON: {}", e)))?;

        let mut updates = Vec::new();
        
        if let Some(array) = json.as_array() {
            for item in array {
                if let (Some(id), Some(name), Some(current), Some(available)) = (
                    item.get("Id").and_then(|v| v.as_str()),
                    item.get("Name").and_then(|v| v.as_str()),
                    item.get("InstalledVersion").and_then(|v| v.as_str()),
                    item.get("AvailableVersion").and_then(|v| v.as_str()),
                ) {
                    updates.push(PackageUpdate {
                        id: id.to_string(),
                        name: name.to_string(),
                        current_version: current.to_string(),
                        available_version: available.to_string(),
                        source: "winget".to_string(),
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
        false // Winget can work without elevation for user-scoped installs
    }
}

