/**
 * SDKMAN Manager Implementation
 */

use async_trait::async_trait;
use std::collections::HashMap;
use super::super::SDKError;
use super::super::traits::sdk_manager::{SDKManager, SDKManagerDefaults, SDKManagerHelpers};
use crate::command_executor::CommandExecutor;

pub struct SdkmanManager;

impl SdkmanManager {
    pub fn new() -> Self {
        Self
    }

    async fn execute_shell_command(&self, command: &str) -> Result<String, SDKError> {
        // SDKMAN is typically installed in zsh, so we need to source it first
        let full_command = format!(
            "source ~/.sdkman/bin/sdkman-init.sh && {}",
            command
        );
        
        let result = CommandExecutor::execute_shell(&full_command, None).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to execute command: {}", e)))?;

        if result.success {
            Ok(result.stdout)
        } else {
            Err(SDKError::ManagerNotFound(format!("Command failed: {}", result.stderr)))
        }
    }
}

#[async_trait]
impl SDKManager for SdkmanManager {
    fn name(&self) -> &'static str { "sdk" }
    fn display_name(&self) -> &'static str { "SDKMAN" }
    fn sdk_type(&self) -> &'static str { "java" }
    fn category(&self) -> &'static str { "language" }
    async fn is_installed(&self) -> Result<bool, SDKError> {
        match self.execute_shell_command("sdk version").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    async fn get_manager_version(&self) -> Result<String, SDKError> {
        let output = self.execute_shell_command("sdk version").await?;
        Ok(output.trim().to_string())
    }
    
    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError> { Ok(vec![]) }
    async fn get_current_version(&self) -> Result<Option<String>, SDKError> { Ok(None) }
    async fn switch_version(&self, _version: &str) -> Result<(), SDKError> { Ok(()) }
    async fn switch_version_for_project(&self, _version: &str, _project_path: &str) -> Result<(), SDKError> { Ok(()) }
    async fn is_version_installed(&self, _version: &str) -> Result<bool, SDKError> { Ok(false) }
    
    // === Installation (Optional) ===
    async fn install_version(&self, _version: &str) -> Result<(), SDKError> { Ok(()) }
    async fn uninstall_version(&self, _version: &str) -> Result<(), SDKError> { Ok(()) }
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> { Ok(vec![]) }
    fn supports_installation(&self) -> bool { false }
    
    // === Environment Management ===
    async fn create_project_environment(&self, _version: &str, _project_path: &str) -> Result<String, SDKError> { Ok("".to_string()) }
    async fn get_environment_variables(&self, _version: &str) -> Result<HashMap<String, String>, SDKError> { Ok(HashMap::new()) }
    
    // === Configuration ===
    async fn get_project_config(&self, _project_path: &str) -> Result<HashMap<String, String>, SDKError> { Ok(HashMap::new()) }
    async fn set_project_config(&self, _project_path: &str, _key: &str, _value: &str) -> Result<(), SDKError> { Ok(()) }
    
    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError> { Ok("SDKMAN help - not implemented".to_string()) }
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> { Ok(vec!["sdk install java 11.0.0".to_string()]) }
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> { Ok(vec!["SDKMAN not implemented".to_string()]) }
    
    // === Information ===
    async fn get_info(&self) -> Result<HashMap<String, String>, SDKError> { Ok(HashMap::new()) }
}

#[async_trait]
impl SDKManagerDefaults for SdkmanManager {}

#[async_trait]
impl SDKManagerHelpers for SdkmanManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_sdkman_manager_creation() {
        let manager = SdkmanManager::new();
        assert_eq!(manager.name(), "sdk");
        assert_eq!(manager.display_name(), "SDKMAN");
        assert_eq!(manager.sdk_type(), "java");
        assert_eq!(manager.category(), "language");
    }

    #[tokio::test]
    async fn test_sdkman_is_installed() {
        let manager = SdkmanManager::new();
        let result = manager.is_installed().await;
        
        println!("SDKMAN is_installed result: {:?}", result);
        
        // This should return Ok(true) if SDKMAN is properly installed
        match result {
            Ok(true) => println!("✅ SDKMAN is detected as installed"),
            Ok(false) => println!("❌ SDKMAN is detected as NOT installed"),
            Err(e) => println!("❌ Error checking SDKMAN installation: {}", e),
        }
    }

    #[tokio::test]
    async fn test_sdkman_get_version() {
        let manager = SdkmanManager::new();
        let result = manager.get_manager_version().await;
        
        println!("SDKMAN get_manager_version result: {:?}", result);
        
        match result {
            Ok(version) => {
                println!("✅ SDKMAN version detected: {}", version);
                assert!(!version.is_empty(), "Version should not be empty");
            },
            Err(e) => {
                println!("❌ Error getting SDKMAN version: {}", e);
                panic!("Failed to get SDKMAN version: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_sdkman_shell_command_execution() {
        let manager = SdkmanManager::new();
        
        // Test the internal shell command execution
        let result = manager.execute_shell_command("sdk version").await;
        
        println!("SDKMAN shell command result: {:?}", result);
        
        match result {
            Ok(output) => {
                println!("✅ SDKMAN shell command successful: {}", output);
                assert!(!output.is_empty(), "Output should not be empty");
                assert!(output.contains("SDKMAN"), "Output should contain 'SDKMAN'");
            },
            Err(e) => {
                println!("❌ SDKMAN shell command failed: {}", e);
                panic!("SDKMAN shell command execution failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_sdkman_direct_shell_test() {
        // Test the exact command our manager uses
        let command = "source ~/.sdkman/bin/sdkman-init.sh && sdk version";
        
        let result = CommandExecutor::execute_shell(command, None).await;
        
        println!("Direct shell test result: {:?}", result);
        
        match result {
            Ok(cmd_result) => {
                println!("✅ Direct shell command successful");
                println!("Success: {}", cmd_result.success);
                println!("Stdout: {}", cmd_result.stdout);
                println!("Stderr: {}", cmd_result.stderr);
                
                if cmd_result.success {
                    assert!(!cmd_result.stdout.is_empty(), "Stdout should not be empty");
                    assert!(cmd_result.stdout.contains("SDKMAN"), "Output should contain 'SDKMAN'");
                } else {
                    panic!("Command failed with stderr: {}", cmd_result.stderr);
                }
            },
            Err(e) => {
                println!("❌ Direct shell command failed: {}", e);
                panic!("Direct shell command execution failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_sdkman_file_existence() {
        // Check if SDKMAN files exist
        let sdkman_dir = std::env::var("HOME").unwrap() + "/.sdkman";
        let init_script = sdkman_dir.clone() + "/bin/sdkman-init.sh";
        
        println!("Checking SDKMAN directory: {}", sdkman_dir);
        println!("Checking init script: {}", init_script);
        
        let dir_exists = std::path::Path::new(&sdkman_dir).exists();
        let script_exists = std::path::Path::new(&init_script).exists();
        
        println!("SDKMAN directory exists: {}", dir_exists);
        println!("SDKMAN init script exists: {}", script_exists);
        
        assert!(dir_exists, "SDKMAN directory should exist");
        assert!(script_exists, "SDKMAN init script should exist");
    }
}