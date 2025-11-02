/**
 * NVM (Node Version Manager) Implementation
 * 
 * This is a concrete implementation of the SDK manager traits for NVM.
 * It demonstrates how to implement all required methods with proper error handling.
 */

use async_trait::async_trait;
use std::collections::HashMap;
use super::super::SDKError;
use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use crate::command_executor::CommandExecutor;

/// NVM Manager implementation
pub struct NvmManager {
    // Add any NVM-specific configuration here
}

impl NvmManager {
    pub fn new() -> Self {
        Self {}
    }

    async fn execute_shell_command(&self, command: &str) -> Result<String, SDKError> {
        let result = CommandExecutor::execute_shell(command, None).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to execute command: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::ManagerNotFound(format!("Command failed: {}", result.stderr)))
        }
    }
}

#[async_trait]
impl SDKManager for NvmManager {
    fn name(&self) -> &'static str {
        "nvm"
    }
    
    fn display_name(&self) -> &'static str {
        "Node Version Manager"
    }
    
    fn sdk_type(&self) -> &'static str {
        "node"
    }
    
    fn category(&self) -> &'static str {
        "language"
    }
    
    async fn is_installed(&self) -> Result<bool, SDKError> {
        // NVM is a shell function, so we need to check if it's available in the shell
        // First try to source NVM and then check if it's available
        let nvm_check_command = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm --version > /dev/null 2>&1
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm --version > /dev/null 2>&1
            else
                nvm --version > /dev/null 2>&1
            fi
        "#;
        
        match self.execute_shell_command(nvm_check_command).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let nvm_version_command = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm --version
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm --version
            else
                nvm --version
            fi
        "#;
        
        let output = self.execute_shell_command(nvm_version_command).await?;
        Ok(output.trim().to_string())
    }
    
    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> {
        let nvm_list_command = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm list --no-colors
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm list --no-colors
            else
                nvm list --no-colors
            fi
        "#;
        
        let output = self.execute_shell_command(nvm_list_command).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("v") {
                    Some(line.trim().replace("v", "").replace("*", "").trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(versions)
    }
    
    async fn get_current_version(&self) -> Result<Option<String>, SDKError> {
        let nvm_current_command = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm current
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm current
            else
                nvm current
            fi
        "#;
        
        let output = self.execute_shell_command(nvm_current_command).await?;
        let version = output.trim().replace("v", "");
        if version.is_empty() || version == "system" {
            Ok(None)
        } else {
            Ok(Some(version))
        }
    }
    
    async fn switch_version(&self, version: &str) -> Result<(), SDKError> {
        let nvm_use_command = format!(r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm use {}
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm use {}
            else
                nvm use {}
            fi
        "#, version, version, version);
        
        self.execute_shell_command(&nvm_use_command).await?;
        Ok(())
    }
    
    async fn switch_version_for_project(&self, version: &str, project_path: &str) -> Result<(), SDKError> {
        let nvm_use_project_command = format!(r#"
            cd {} && if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm use {}
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm use {}
            else
                nvm use {}
            fi
        "#, project_path, version, version, version);
        
        self.execute_shell_command(&nvm_use_project_command).await?;
        Ok(())
    }
    
    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError> {
        let nvm_list_version_command = format!(r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm list {}
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm list {}
            else
                nvm list {}
            fi
        "#, version, version, version);
        
        match self.execute_shell_command(&nvm_list_version_command).await {
            Ok(output) => Ok(output.contains(version)),
            Err(_) => Ok(false),
        }
    }
    
    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError> {
        let nvm_install_command = format!(r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm install {}
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm install {}
            else
                nvm install {}
            fi
        "#, version, version, version);
        
        self.execute_shell_command(&nvm_install_command).await?;
        Ok(())
    }
    
    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError> {
        let nvm_uninstall_command = format!(r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm uninstall {}
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm uninstall {}
            else
                nvm uninstall {}
            fi
        "#, version, version, version);
        
        self.execute_shell_command(&nvm_uninstall_command).await?;
        Ok(())
    }
    
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        let nvm_ls_remote_command = r#"
            if [ -s "$HOME/.nvm/nvm.sh" ]; then
                source "$HOME/.nvm/nvm.sh"
                nvm ls-remote --no-colors
            elif [ -s "/usr/local/opt/nvm/nvm.sh" ]; then
                source "/usr/local/opt/nvm/nvm.sh"
                nvm ls-remote --no-colors
            else
                nvm ls-remote --no-colors
            fi
        "#;
        
        let output = self.execute_shell_command(nvm_ls_remote_command).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("v") {
                    Some(line.trim().replace("v", "").trim().to_string())
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
            "#!/bin/bash\n# NVM Project Environment\n# Generated for project: {}\n# Node version: {}\n\n# Set NVM version for this project\nexport NVM_DIR=\"$HOME/.nvm\"\n[ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"\nnvm use {}\n",
            project_path, version, version
        );
        Ok(script)
    }
    
    async fn get_environment_variables(&self, version: &str) -> Result<HashMap<String, String>, SDKError> {
        let mut env_vars = HashMap::new();
        env_vars.insert("NODE_VERSION".to_string(), version.to_string());
        env_vars.insert("NVM_DIR".to_string(), "$HOME/.nvm".to_string());
        Ok(env_vars)
    }
    
    // === Configuration ===
    async fn get_project_config(&self, project_path: &str) -> Result<HashMap<String, String>, SDKError> {
        let mut config = HashMap::new();
        // Check for .nvmrc file
        let nvmrc_path = format!("{}/.nvmrc", project_path);
        if std::path::Path::new(&nvmrc_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&nvmrc_path) {
                config.insert("node_version".to_string(), content.trim().to_string());
            }
        }
        Ok(config)
    }
    
    async fn set_project_config(&self, project_path: &str, key: &str, value: &str) -> Result<(), SDKError> {
        if key == "node_version" {
            let nvmrc_path = format!("{}/.nvmrc", project_path);
            std::fs::write(&nvmrc_path, value)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write .nvmrc: {}", e)))?;
        }
        Ok(())
    }
    
    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok("NVM (Node Version Manager) - Manage multiple Node.js versions\n\nUsage:\n  nvm install <version>    Install a Node.js version\n  nvm use <version>       Switch to a version\n  nvm list               List installed versions\n  nvm current            Show current version".to_string())
    }
    
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![
            "nvm install 18.0.0".to_string(),
            "nvm use 18.0.0".to_string(),
            "nvm list".to_string(),
            "nvm current".to_string(),
        ])
    }
    
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        let mut issues = Vec::new();
        
        if !self.is_installed().await? {
            issues.push("NVM is not installed".to_string());
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
impl SDKManagerDefaults for NvmManager {}

#[async_trait]
impl SDKManagerHelpers for NvmManager {}