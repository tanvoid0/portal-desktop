/**
 * SDK Manager Factory
 * 
 * This module provides a simple factory for creating and managing SDK managers.
 * It eliminates the need for complex trait hierarchies and provides a unified interface.
 */

use std::collections::HashMap;
use std::fmt;
use super::SDKError;
use super::traits::sdk_manager::SDKManager;
use super::managers::{
    NvmManager, RustupManager, PyenvManager, SdkmanManager, RbenvManager, PhpenvManager
};

/// Manager factory that creates and manages all SDK managers
pub struct SDKManagerFactory {
    managers: HashMap<String, Box<dyn SDKManager>>,
}

impl SDKManagerFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            managers: HashMap::new(),
        };
        
        // Register all available managers
        factory.register_manager("nvm", Box::new(NvmManager::new()));
        factory.register_manager("rustup", Box::new(RustupManager::new()));
        factory.register_manager("pyenv", Box::new(PyenvManager::new()));
        factory.register_manager("sdk", Box::new(SdkmanManager::new()));
        factory.register_manager("rbenv", Box::new(RbenvManager::new()));
        factory.register_manager("phpenv", Box::new(PhpenvManager::new()));
        
        factory
    }
    
    /// Register a new manager
    pub fn register_manager(&mut self, name: &str, manager: Box<dyn SDKManager>) {
        self.managers.insert(name.to_string(), manager);
    }
    
    /// Get a specific manager
    pub fn get_manager(&self, name: &str) -> Option<&Box<dyn SDKManager>> {
        self.managers.get(name)
    }
    
    /// Get all managers
    pub fn get_all_managers(&self) -> &HashMap<String, Box<dyn SDKManager>> {
        &self.managers
    }
    
    /// Detect all installed managers
    pub async fn detect_installed_managers(&self) -> Result<Vec<HashMap<String, String>>, SDKError> {
        let mut installed = Vec::new();
        
        println!("[SDKManagerFactory] Starting detection of {} managers", self.managers.len());
        
        for (name, manager) in &self.managers {
            println!("[SDKManagerFactory] Checking manager: {}", name);
            
            match manager.is_installed().await {
                Ok(true) => {
                    println!("[SDKManagerFactory] Manager {} is installed", name);
                    let mut info = HashMap::new();
                    info.insert("name".to_string(), manager.name().to_string());
                    info.insert("display_name".to_string(), manager.display_name().to_string());
                    info.insert("sdk_type".to_string(), manager.sdk_type().to_string());
                    info.insert("category".to_string(), manager.category().to_string());
                    
                    match manager.get_manager_version().await {
                        Ok(version) => {
                            info.insert("version".to_string(), version.clone());
                            info.insert("installed".to_string(), "true".to_string());
                            info.insert("type".to_string(), name.clone()); // Add type field for frontend compatibility
                            info.insert("description".to_string(), format!("{} - {}", manager.display_name(), manager.sdk_type()));
                            println!("[SDKManagerFactory] Added manager {} with version {}", name, version);
                            installed.push(info);
                        },
                        Err(e) => {
                            println!("[SDKManagerFactory] Failed to get version for {}: {}", name, e);
                        }
                    }
                },
                Ok(false) => {
                    println!("[SDKManagerFactory] Manager {} is not installed", name);
                },
                Err(e) => {
                    println!("[SDKManagerFactory] Error checking {}: {}", name, e);
                }
            }
        }
        
        println!("[SDKManagerFactory] Detection complete. Found {} installed managers", installed.len());
        Ok(installed)
    }
    
    /// Get managers by category
    pub fn get_managers_by_category(&self, category: &str) -> Vec<&Box<dyn SDKManager>> {
        self.managers
            .values()
            .filter(|manager| manager.category() == category)
            .collect()
    }
    
    /// Get managers by SDK type
    pub fn get_managers_by_sdk_type(&self, sdk_type: &str) -> Vec<&Box<dyn SDKManager>> {
        self.managers
            .values()
            .filter(|manager| manager.sdk_type() == sdk_type)
            .collect()
    }
    
    /// Get all manager info
    pub async fn get_all_manager_info(&self) -> Result<HashMap<String, HashMap<String, String>>, SDKError> {
        let mut info = HashMap::new();
        
        for (name, manager) in &self.managers {
            if let Ok(manager_info) = manager.get_info().await {
                info.insert(name.clone(), manager_info);
            }
        }
        
        Ok(info)
    }
}

/// Helper functions for common operations
impl SDKManagerFactory {
    /// Switch version for a specific manager and project
    pub async fn switch_version_for_project(&self, manager_name: &str, version: &str, project_path: &str) -> Result<(), SDKError> {
        if let Some(manager) = self.get_manager(manager_name) {
            manager.switch_version_for_project(version, project_path).await
        } else {
            Err(SDKError::ManagerNotFound(manager_name.to_string()))
        }
    }
    
    /// Install a version for a specific manager
    pub async fn install_version(&self, manager_name: &str, version: &str) -> Result<(), SDKError> {
        if let Some(manager) = self.get_manager(manager_name) {
            if manager.supports_installation() {
                manager.install_version(version).await
            } else {
                Err(SDKError::ManagerNotFound(format!("Installation not supported for {}", manager_name)))
            }
        } else {
            Err(SDKError::ManagerNotFound(manager_name.to_string()))
        }
    }
    
    /// Get current version for a specific manager
    pub async fn get_current_version(&self, manager_name: &str) -> Result<Option<String>, SDKError> {
        if let Some(manager) = self.get_manager(manager_name) {
            manager.get_current_version().await
        } else {
            Err(SDKError::ManagerNotFound(manager_name.to_string()))
        }
    }
    
    /// List versions for a specific manager
    pub async fn list_versions(&self, manager_name: &str) -> Result<Vec<String>, SDKError> {
        if let Some(manager) = self.get_manager(manager_name) {
            manager.list_versions().await
        } else {
            Err(SDKError::ManagerNotFound(manager_name.to_string()))
        }
    }
}

impl fmt::Debug for SDKManagerFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SDKManagerFactory")
            .field("managers", &format!("{} managers", self.managers.len()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_factory_detection() {
        let factory = SDKManagerFactory::new();
        let result = factory.detect_installed_managers().await;
        
        println!("Factory detection result: {:?}", result);
        
        match result {
            Ok(managers) => {
                println!("✅ Factory detected {} managers", managers.len());
                for (i, manager) in managers.iter().enumerate() {
                    println!("Manager {}: {:?}", i, manager);
                }
                
                // Check if SDKMAN is in the results
                let sdkman_found = managers.iter().any(|m| {
                    m.get("name") == Some(&"sdk".to_string()) || 
                    m.get("display_name") == Some(&"SDKMAN".to_string())
                });
                
                if sdkman_found {
                    println!("✅ SDKMAN found in factory results!");
                } else {
                    println!("❌ SDKMAN NOT found in factory results");
                }
            },
            Err(e) => {
                println!("❌ Factory detection failed: {}", e);
                panic!("Factory detection failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_factory_sdkman_specific() {
        let factory = SDKManagerFactory::new();
        
        // Test SDKMAN manager specifically
        if let Some(sdkman_manager) = factory.get_manager("sdk") {
            println!("✅ SDKMAN manager found in factory");
            
            let is_installed = sdkman_manager.is_installed().await;
            println!("SDKMAN is_installed: {:?}", is_installed);
            
            let version = sdkman_manager.get_manager_version().await;
            println!("SDKMAN version: {:?}", version);
        } else {
            println!("❌ SDKMAN manager NOT found in factory");
            panic!("SDKMAN manager should be registered in factory");
        }
    }
}
