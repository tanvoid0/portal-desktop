/**
 * Package Manager Factory
 * 
 * Factory for creating and managing package managers (winget, scoop, etc.)
 */

use std::collections::HashMap;
use crate::domains::sdk::traits::package_manager::PackageManager;
use crate::domains::sdk::package_managers::{
    WingetManager, ScoopManager, ChocolateyManager, CargoManager,
    HomebrewManager, NpmManager, PipManager
};

/// Package manager factory that creates and manages all package managers
pub struct PackageManagerFactory {
    managers: HashMap<String, Box<dyn PackageManager>>,
}

impl PackageManagerFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            managers: HashMap::new(),
        };
        
        // Register all available package managers
        factory.register_manager("winget", Box::new(WingetManager::new()));
        factory.register_manager("scoop", Box::new(ScoopManager::new()));
        factory.register_manager("chocolatey", Box::new(ChocolateyManager::new()));
        factory.register_manager("cargo", Box::new(CargoManager::new()));
        factory.register_manager("homebrew", Box::new(HomebrewManager::new()));
        factory.register_manager("npm", Box::new(NpmManager::new()));
        factory.register_manager("pip", Box::new(PipManager::new()));
        
        factory
    }
    
    /// Register a new package manager
    pub fn register_manager(&mut self, name: &str, manager: Box<dyn PackageManager>) {
        self.managers.insert(name.to_string(), manager);
    }
    
    /// Get a specific package manager
    pub fn get_manager(&self, name: &str) -> Option<&Box<dyn PackageManager>> {
        self.managers.get(name)
    }
    
    /// Get all package managers
    pub fn get_all_managers(&self) -> &HashMap<String, Box<dyn PackageManager>> {
        &self.managers
    }
    
    /// Detect all available package managers on the system
    pub async fn detect_available_managers(&self) -> Vec<String> {
        let mut available = Vec::new();
        
        for (name, manager) in &self.managers {
            match manager.is_available().await {
                Ok(true) => {
                    available.push(name.clone());
                },
                _ => {}
            }
        }
        
        available
    }
    
    /// Get managers by platform
    pub fn get_managers_by_platform(&self, platform: &str) -> Vec<&Box<dyn PackageManager>> {
        self.managers
            .values()
            .filter(|manager| {
                manager.platform() == platform || manager.platform() == "cross-platform"
            })
            .collect()
    }
}

impl Default for PackageManagerFactory {
    fn default() -> Self {
        Self::new()
    }
}

