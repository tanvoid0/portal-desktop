/**
 * Environment Manager
 * 
 * Manages environment variables and PATH for SDK installations
 * Supports both app-managed and system-managed environments
 */

use crate::domains::sdk::SDKError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub scope: EnvironmentScope,
    pub is_exported: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentScope {
    Global,   // System-wide environment
    Session, // Current session only
    Project, // Project-specific
    Service, // Service-specific
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub path: String,
    pub sdk_type: String,
    pub version: String,
    pub scope: EnvironmentScope,
    pub is_active: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatus {
    pub sdk_type: String,
    pub current_version: Option<String>,
    pub path_managed_by: PathManagementType,
    pub binaries_in_path: Vec<String>,
    pub environment_variables: Vec<EnvironmentVariable>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathManagementType {
    App,    // Managed by portal_desktop
    System, // Managed by system
    None,   // Not in PATH
}

pub struct EnvironmentManager {
    environment_variables: HashMap<String, EnvironmentVariable>,
    path_entries: Vec<PathEntry>,
    shell_config_path: Option<PathBuf>,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        Self {
            environment_variables: HashMap::new(),
            path_entries: Vec::new(),
            shell_config_path: Self::detect_shell_config_path(),
        }
    }

    /// Set environment variable
    pub async fn set_environment_variable(
        &mut self,
        name: String,
        value: String,
        scope: EnvironmentScope,
    ) -> Result<(), SDKError> {
        let now = chrono::Utc::now().to_rfc3339();
        let env_var = EnvironmentVariable {
            name: name.clone(),
            value,
            scope,
            is_exported: true,
            created_at: now.clone(),
            updated_at: now,
        };

        self.environment_variables.insert(name, env_var);
        self.apply_environment_changes().await?;
        Ok(())
    }

    /// Remove environment variable
    pub async fn remove_environment_variable(&mut self, name: &str) -> Result<(), SDKError> {
        self.environment_variables.remove(name);
        self.apply_environment_changes().await?;
        Ok(())
    }

    /// Get all environment variables
    pub async fn get_environment_variables(&self) -> Vec<EnvironmentVariable> {
        self.environment_variables.values().cloned().collect()
    }

    /// Set SDK path in environment
    pub async fn set_sdk_path(
        &mut self,
        sdk_type: String,
        version: String,
        path: String,
        scope: EnvironmentScope,
    ) -> Result<(), SDKError> {
        // Remove existing entries for this SDK type
        self.path_entries.retain(|entry| entry.sdk_type != sdk_type);

        // Add new path entry
        let path_entry = PathEntry {
            path,
            sdk_type: sdk_type.clone(),
            version,
            scope,
            is_active: true,
            priority: self.get_next_priority(),
        };

        self.path_entries.push(path_entry);
        self.apply_path_changes().await?;
        Ok(())
    }

    /// Remove SDK path from environment
    pub async fn remove_sdk_path(&mut self, sdk_type: &str) -> Result<(), SDKError> {
        self.path_entries.retain(|entry| entry.sdk_type != sdk_type);
        self.apply_path_changes().await?;
        Ok(())
    }

    /// Get environment status for an SDK
    pub async fn get_environment_status(&self, sdk_type: &str) -> Result<EnvironmentStatus, SDKError> {
        let current_version = self.get_current_sdk_version(sdk_type).await?;
        let path_managed_by = self.determine_path_management_type(sdk_type).await;
        let binaries_in_path = self.get_sdk_binaries_in_path(sdk_type).await;
        let environment_variables = self.get_sdk_environment_variables(sdk_type).await;

        Ok(EnvironmentStatus {
            sdk_type: sdk_type.to_string(),
            current_version,
            path_managed_by,
            binaries_in_path,
            environment_variables,
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Apply environment changes to the system
    async fn apply_environment_changes(&self) -> Result<(), SDKError> {
        // For session scope, set in current process
        for env_var in self.environment_variables.values() {
            if matches!(env_var.scope, EnvironmentScope::Session) {
                std::env::set_var(&env_var.name, &env_var.value);
            }
        }

        // For global scope, update shell configuration files
        if let Some(config_path) = &self.shell_config_path {
            self.update_shell_config(config_path).await?;
        }

        Ok(())
    }

    /// Apply path changes to the system
    async fn apply_path_changes(&self) -> Result<(), SDKError> {
        let mut path_entries = self.path_entries.clone();
        path_entries.sort_by_key(|entry| entry.priority);

        // Build new PATH
        let mut new_path = String::new();
        for entry in path_entries {
            if entry.is_active {
                if !new_path.is_empty() {
                    new_path.push(Self::get_path_separator());
                }
                new_path.push_str(&entry.path);
            }
        }

        // Update PATH environment variable
        std::env::set_var("PATH", &new_path);

        // Update shell configuration if needed
        if let Some(config_path) = &self.shell_config_path {
            self.update_shell_path_config(config_path, &new_path).await?;
        }

        Ok(())
    }

    /// Update shell configuration file
    async fn update_shell_config(&self, config_path: &Path) -> Result<(), SDKError> {
        let mut config_content = if config_path.exists() {
            fs::read_to_string(config_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read shell config: {}", e)))?
        } else {
            String::new()
        };

        // Add environment variables
        for env_var in self.environment_variables.values() {
            if matches!(env_var.scope, EnvironmentScope::Global) {
                let export_line = format!("export {}={}\n", env_var.name, env_var.value);
                if !config_content.contains(&format!("export {}", env_var.name)) {
                    config_content.push_str(&export_line);
                }
            }
        }

        fs::write(config_path, config_content).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write shell config: {}", e)))?;

        Ok(())
    }

    /// Update shell PATH configuration
    async fn update_shell_path_config(&self, config_path: &Path, new_path: &str) -> Result<(), SDKError> {
        let config_content = if config_path.exists() {
            fs::read_to_string(config_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read shell config: {}", e)))?
        } else {
            String::new()
        };

        // Remove existing PATH exports
        let lines: Vec<&str> = config_content.lines().collect();
        let filtered_lines: Vec<&str> = lines
            .iter()
            .filter(|line| !line.starts_with("export PATH="))
            .cloned()
            .collect();

        let mut new_config = filtered_lines.join("\n");
        if !new_config.is_empty() {
            new_config.push('\n');
        }

        // Add new PATH export
        new_config.push_str(&format!("export PATH=\"{}\"\n", new_path));

        fs::write(config_path, new_config).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write shell config: {}", e)))?;

        Ok(())
    }

    /// Detect shell configuration file path
    fn detect_shell_config_path() -> Option<PathBuf> {
        let home_dir = dirs::home_dir()?;
        
        // Check for different shells
        if std::env::var("SHELL").unwrap_or_default().contains("zsh") {
            Some(home_dir.join(".zshrc"))
        } else if std::env::var("SHELL").unwrap_or_default().contains("bash") {
            Some(home_dir.join(".bashrc"))
        } else if std::env::var("SHELL").unwrap_or_default().contains("fish") {
            Some(home_dir.join(".config/fish/config.fish"))
        } else {
            // Default to bash
            Some(home_dir.join(".bashrc"))
        }
    }

    /// Get path separator for current OS
    fn get_path_separator() -> char {
        if cfg!(target_os = "windows") {
            ';'
        } else {
            ':'
        }
    }

    /// Get next priority for path entries
    fn get_next_priority(&self) -> u32 {
        self.path_entries.iter().map(|entry| entry.priority).max().unwrap_or(0) + 1
    }

    /// Get current SDK version from PATH
    async fn get_current_sdk_version(&self, sdk_type: &str) -> Result<Option<String>, SDKError> {
        // This would check the current PATH to determine which version is active
        // For now, return the first active version
        if let Some(entry) = self.path_entries.iter().find(|entry| entry.sdk_type == sdk_type && entry.is_active) {
            Ok(Some(entry.version.clone()))
        } else {
            Ok(None)
        }
    }

    /// Determine how PATH is managed for an SDK
    async fn determine_path_management_type(&self, sdk_type: &str) -> PathManagementType {
        if self.path_entries.iter().any(|entry| entry.sdk_type == sdk_type) {
            PathManagementType::App
        } else if self.is_sdk_in_system_path(sdk_type).await {
            PathManagementType::System
        } else {
            PathManagementType::None
        }
    }

    /// Check if SDK is in system PATH
    async fn is_sdk_in_system_path(&self, _sdk_type: &str) -> bool {
        let _system_path = std::env::var("PATH").unwrap_or_default();
        // This would check if SDK binaries are in the system PATH
        // For now, return false
        false
    }

    /// Get SDK binaries currently in PATH
    async fn get_sdk_binaries_in_path(&self, sdk_type: &str) -> Vec<String> {
        let mut binaries = Vec::new();
        
        for entry in &self.path_entries {
            if entry.sdk_type == sdk_type && entry.is_active {
                // This would scan the directory for SDK binaries
                // For now, return mock data
                binaries.push(format!("{}/bin", entry.path));
            }
        }

        binaries
    }

    /// Get environment variables for an SDK
    async fn get_sdk_environment_variables(&self, sdk_type: &str) -> Vec<EnvironmentVariable> {
        self.environment_variables
            .values()
            .filter(|env_var| env_var.name.to_lowercase().contains(&sdk_type.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Create shell integration script
    pub async fn create_shell_integration(&self, shell_type: &str) -> Result<String, SDKError> {
        let script = match shell_type {
            "bash" => self.create_bash_integration().await,
            "zsh" => self.create_zsh_integration().await,
            "fish" => self.create_fish_integration().await,
            "powershell" => self.create_powershell_integration().await,
            _ => return Err(SDKError::ManagerNotFound(format!("Unsupported shell: {}", shell_type))),
        };

        Ok(script)
    }

    /// Create bash integration script
    async fn create_bash_integration(&self) -> String {
        format!(
            r#"# Portal Desktop SDK Integration
export PORTAL_SDK_MANAGER="portal-desktop"
export PORTAL_SDK_PATH="{}"

# Function to activate SDK environment
activate_sdk() {{
    local sdk_type=$1
    local version=$2
    if [ -n "$sdk_type" ] && [ -n "$version" ]; then
        echo "Activating $sdk_type version $version"
        # SDK activation logic would go here
    else
        echo "Usage: activate_sdk <sdk_type> <version>"
    fi
}}

# Function to deactivate SDK environment
deactivate_sdk() {{
    echo "Deactivating SDK environment"
    # SDK deactivation logic would go here
}}
"#,
            self.get_portal_sdk_path()
        )
    }

    /// Create zsh integration script
    async fn create_zsh_integration(&self) -> String {
        self.create_bash_integration().await // Similar to bash
    }

    /// Create fish integration script
    async fn create_fish_integration(&self) -> String {
        format!(
            r#"# Portal Desktop SDK Integration for Fish
set -gx PORTAL_SDK_MANAGER "portal-desktop"
set -gx PORTAL_SDK_PATH "{}"

# Function to activate SDK environment
function activate_sdk
    set sdk_type $argv[1]
    set version $argv[2]
    if test -n "$sdk_type" -a -n "$version"
        echo "Activating $sdk_type version $version"
        # SDK activation logic would go here
    else
        echo "Usage: activate_sdk <sdk_type> <version>"
    end
end

# Function to deactivate SDK environment
function deactivate_sdk
    echo "Deactivating SDK environment"
    # SDK deactivation logic would go here
end
"#,
            self.get_portal_sdk_path()
        )
    }

    /// Create PowerShell integration script
    async fn create_powershell_integration(&self) -> String {
        format!(
            r#"# Portal Desktop SDK Integration for PowerShell
$env:PORTAL_SDK_MANAGER = "portal-desktop"
$env:PORTAL_SDK_PATH = "{}"

# Function to activate SDK environment
function Activate-Sdk {{
    param(
        [string]$SdkType,
        [string]$Version
    )
    if ($SdkType -and $Version) {{
        Write-Host "Activating $SdkType version $Version"
        # SDK activation logic would go here
    }} else {{
        Write-Host "Usage: Activate-Sdk -SdkType <type> -Version <version>"
    }}
}}

# Function to deactivate SDK environment
function Deactivate-Sdk {{
    Write-Host "Deactivating SDK environment"
    # SDK deactivation logic would go here
}}
"#,
            self.get_portal_sdk_path()
        )
    }

    /// Get portal SDK path
    fn get_portal_sdk_path(&self) -> String {
        // This would return the actual portal SDK installation path
        "/usr/local/portal/sdk".to_string()
    }
}