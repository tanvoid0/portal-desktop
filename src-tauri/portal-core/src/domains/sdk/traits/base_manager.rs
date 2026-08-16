/**
 * SDK Manager Trait
 * 
 * This is the unified trait that all SDK managers must implement.
 * It provides all the essential functionality in a single, easy-to-implement interface.
 */

use async_trait::async_trait;
use std::collections::HashMap;
use super::super::SDKError;

/// SDK Manager trait - all managers implement this single trait
#[async_trait]
pub trait SDKManager: Send + Sync {
    // === Core Identity ===
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn sdk_type(&self) -> &'static str;
    fn category(&self) -> &'static str;
    
    // === Installation & Detection ===
    async fn is_installed(&self) -> Result<bool, SDKError>;
    async fn get_manager_version(&self) -> Result<String, SDKError>;
    
    // === Version Management ===
    async fn list_versions(&self) -> Result<Vec<String>, SDKError>;
    async fn get_current_version(&self) -> Result<Option<String>, SDKError>;
    async fn switch_version(&self, version: &str) -> Result<(), SDKError>;
    async fn switch_version_for_project(&self, version: &str, project_path: &str) -> Result<(), SDKError>;
    async fn is_version_installed(&self, version: &str) -> Result<bool, SDKError>;
    
    // === Installation (Optional) ===
    async fn install_version(&self, version: &str) -> Result<(), SDKError>;
    async fn uninstall_version(&self, version: &str) -> Result<(), SDKError>;
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError>;
    fn supports_installation(&self) -> bool;
    
    // === Environment Management ===
    async fn create_project_environment(&self, version: &str, project_path: &str) -> Result<String, SDKError>;
    async fn get_environment_variables(&self, version: &str) -> Result<HashMap<String, String>, SDKError>;
    
    // === Configuration ===
    async fn get_project_config(&self, project_path: &str) -> Result<HashMap<String, String>, SDKError>;
    async fn set_project_config(&self, project_path: &str, key: &str, value: &str) -> Result<(), SDKError>;
    
    // === Help & Validation ===
    async fn get_help(&self) -> Result<String, SDKError>;
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError>;
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError>;
}

/// Default implementations for optional methods
#[async_trait]
pub trait SDKManagerDefaults: SDKManager {
    // Default implementations that can be overridden
    async fn install_version(&self, _version: &str) -> Result<(), SDKError> {
        Err(SDKError::ManagerNotFound(format!("Installation not supported for {}", self.name())))
    }
    
    async fn uninstall_version(&self, _version: &str) -> Result<(), SDKError> {
        Err(SDKError::ManagerNotFound(format!("Uninstallation not supported for {}", self.name())))
    }
    
    async fn list_available_versions(&self) -> Result<Vec<String>, SDKError> {
        Err(SDKError::ManagerNotFound(format!("Remote version listing not supported for {}", self.name())))
    }
    
    fn supports_installation(&self) -> bool {
        false
    }
    
    async fn create_project_environment(&self, _version: &str, _project_path: &str) -> Result<String, SDKError> {
        Ok("#!/bin/bash\n# Project environment script\n".to_string())
    }
    
    async fn get_environment_variables(&self, _version: &str) -> Result<HashMap<String, String>, SDKError> {
        Ok(HashMap::new())
    }
    
    async fn get_project_config(&self, _project_path: &str) -> Result<HashMap<String, String>, SDKError> {
        Ok(HashMap::new())
    }
    
    async fn set_project_config(&self, _project_path: &str, _key: &str, _value: &str) -> Result<(), SDKError> {
        Err(SDKError::ManagerNotFound(format!("Project configuration not supported for {}", self.name())))
    }
    
    async fn get_help(&self) -> Result<String, SDKError> {
        Ok(format!("Help for {} - not implemented", self.name()))
    }
    
    async fn get_usage_examples(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![format!("{} --help", self.name())])
    }
    
    async fn validate_setup(&self) -> Result<Vec<String>, SDKError> {
        Ok(vec![])
    }
}

/// Helper trait for common functionality
pub trait SDKManagerHelpers: SDKManager {
    /// Get comprehensive info about this manager
    async fn get_info(&self) -> Result<HashMap<String, String>, SDKError> {
        let mut info = HashMap::new();
        info.insert("name".to_string(), self.name().to_string());
        info.insert("display_name".to_string(), self.display_name().to_string());
        info.insert("sdk_type".to_string(), self.sdk_type().to_string());
        info.insert("category".to_string(), self.category().to_string());
        info.insert("version".to_string(), self.get_manager_version().await?);
        info.insert("installed".to_string(), self.is_installed().await?.to_string());
        Ok(info)
    }
    
    /// Check if this manager is properly configured
    async fn is_configured(&self) -> Result<bool, SDKError> {
        let issues = self.validate_setup().await?;
        Ok(issues.is_empty())
    }
}
