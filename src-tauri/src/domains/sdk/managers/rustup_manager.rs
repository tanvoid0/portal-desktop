/**
 * Rustup Manager Implementation
 * 
 * This is a concrete implementation of the SDK manager traits for Rustup.
 * It demonstrates how to implement all required methods with proper error handling.
 */

use async_trait::async_trait;
use std::collections::HashMap;
use super::super::SDKError;
use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use crate::command_executor::CommandExecutor;

/// Rustup Manager implementation
pub struct RustupManager {
    // Add any Rustup-specific configuration here
}

impl RustupManager {
    pub fn new() -> Self {
        Self {}
    }

    async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_with_args(command, args, None).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to execute command: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::ManagerNotFound(format!("Command failed: {}", result.stderr)))
        }
    }
}

#[async_trait]
impl SDKManager for RustupManager {
    fn name(&self) -> &'static str {
        "rustup"
    }
    
    fn display_name(&self) -> &'static str {
        "Rust Toolchain Manager"
    }
    
    fn sdk_type(&self) -> &'static str {
        "rust"
    }
    
    fn category(&self) -> &'static str {
        "language"
    }
    
    async fn is_installed(&self) -> Result<bool, SDKError> {
        match self.execute_command("rustup", &["--version"]).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let output = self.execute_command("rustup", &["--version"]).await?;
        Ok(output.trim().to_string())
    }
    
    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("rustup", &["show"]).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("rust-") {
                    Some(line.trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(versions)
    }
    
    async fn get_current_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("rustc", &["--version"]).await?;
        let version = output.trim().replace("rustc ", "");
        Ok(Some(version))
    }
    
    async fn switch_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["default", version]).await?;
        Ok(())
    }
    
    async fn switch_version_for_project(&self, version: &str, project_path: &str) -> Result<(), SDKError> {
        // Create rust-toolchain file for project-specific version
        let toolchain_file = format!("{}/rust-toolchain", project_path);
        std::fs::write(&toolchain_file, version)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write rust-toolchain: {}", e)))?;
        Ok(())
    }
    
    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError> {
        let output = self.execute_command("rustup", &["show"]).await?;
        Ok(output.contains(version))
    }
    
    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["install", version]).await?;
        Ok(())
    }
    
    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["uninstall", version]).await?;
        Ok(())
    }
    
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("rustup", &["update", "--list"]).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("rust-") {
                    Some(line.trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(versions)
    }
    
    fn supports_installation(&self) -> bool {
        true
    }
    
    // === Environment Management ===
    async fn create_project_environment(&self, version: &str, project_path: &str) -> Result<String, SDKError> {
        let script = format!(
            "#!/bin/bash\n# Rust Project Environment\n# Generated for project: {}\n# Rust version: {}\n\n# Set Rust toolchain for this project\nexport RUSTUP_TOOLCHAIN={}\n",
            project_path, version, version
        );
        Ok(script)
    }
    
    async fn get_environment_variables(&self, version: &str) -> Result<HashMap<String, String>, SDKError> {
        let mut env_vars = HashMap::new();
        env_vars.insert("RUSTUP_TOOLCHAIN".to_string(), version.to_string());
        Ok(env_vars)
    }
    
    // === Configuration ===
    async fn get_project_config(&self, project_path: &str) -> Result<HashMap<String, String>, SDKError> {
        let mut config = HashMap::new();
        // Check for rust-toolchain file
        let toolchain_path = format!("{}/rust-toolchain", project_path);
        if std::path::Path::new(&toolchain_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&toolchain_path) {
                config.insert("rust_toolchain".to_string(), content.trim().to_string());
            }
        }
        Ok(config)
    }
    
    async fn set_project_config(&self, project_path: &str, key: &str, value: &str) -> Result<(), SDKError> {
        if key == "rust_toolchain" {
            let toolchain_path = format!("{}/rust-toolchain", project_path);
            std::fs::write(&toolchain_path, value)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write rust-toolchain: {}", e)))?;
        }
        Ok(())
    }
    
    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok("Rustup - Rust toolchain installer and version manager\n\nUsage:\n  rustup install <toolchain>    Install a Rust toolchain\n  rustup default <toolchain>     Set default toolchain\n  rustup show                   Show installed toolchains\n  rustup update                 Update all toolchains".to_string())
    }
    
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![
            "rustup install stable".to_string(),
            "rustup default stable".to_string(),
            "rustup show".to_string(),
            "rustup update".to_string(),
        ])
    }
    
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        let mut issues = Vec::new();
        
        if !self.is_installed().await? {
            issues.push("Rustup is not installed".to_string());
        }
        
        Ok(issues)
    }
    
    // === Information ===
    async fn get_info(&self) -> Result<HashMap<String, String>, SDKError> {
        let mut info = HashMap::new();
        info.insert("name".to_string(), self.display_name().to_string());
        info.insert("version".to_string(), self.get_manager_version().await?);
        info.insert("sdk_type".to_string(), self.sdk_type().to_string());
        info.insert("category".to_string(), self.category().to_string());
        Ok(info)
    }
}

#[async_trait]
impl SDKManagerDefaults for RustupManager {}

#[async_trait]
impl SDKManagerHelpers for RustupManager {}