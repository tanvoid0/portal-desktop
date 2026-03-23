/**
 * SDK Manager Commands
 * 
 * Tauri commands for SDK manager-specific operations (version management, installation, etc.)
 */

use crate::domains::sdk::factory::SDKManagerFactory;

/// Get installed versions for a specific SDK manager
#[tauri::command]
pub async fn get_manager_installed_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();
    
    match factory.list_versions(&manager_name).await {
        Ok(versions) => Ok(versions),
        Err(e) => Err(format!("Failed to list versions for {}: {}", manager_name, e)),
    }
}

/// Get available (installable) versions for a specific SDK manager
#[tauri::command]
pub async fn get_manager_available_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();
    
    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.list_available_versions().await {
            Ok(versions) => Ok(versions),
            Err(e) => Err(format!("Failed to list available versions for {}: {}", manager_name, e)),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Get current active version for a specific SDK manager
#[tauri::command]
pub async fn get_manager_current_version(manager_name: String) -> Result<Option<String>, String> {
    let factory = SDKManagerFactory::new();
    
    match factory.get_current_version(&manager_name).await {
        Ok(version) => Ok(version),
        Err(e) => Err(format!("Failed to get current version for {}: {}", manager_name, e)),
    }
}

/// Install a version using a specific SDK manager
#[tauri::command]
pub async fn install_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();
    
    match factory.install_version(&manager_name, &version).await {
        Ok(_) => Ok(format!("Successfully installed {} version {}", manager_name, version)),
        Err(e) => Err(format!("Failed to install version {} for {}: {}", version, manager_name, e)),
    }
}

/// Switch to a version using a specific SDK manager
#[tauri::command]
pub async fn switch_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();
    
    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.switch_version(&version).await {
            Ok(_) => Ok(format!("Successfully switched to version {} for {}", version, manager_name)),
            Err(e) => Err(format!("Failed to switch to version {} for {}: {}", version, manager_name, e)),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Uninstall a version using a specific SDK manager
#[tauri::command]
pub async fn uninstall_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();
    
    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.uninstall_version(&version).await {
            Ok(_) => Ok(format!("Successfully uninstalled {} version {}", manager_name, version)),
            Err(e) => Err(format!("Failed to uninstall version {} for {}: {}", version, manager_name, e)),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Check if a version is installed for a specific SDK manager
#[tauri::command]
pub async fn is_manager_version_installed(
    manager_name: String,
    version: String,
) -> Result<bool, String> {
    let factory = SDKManagerFactory::new();
    
    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.is_version_installed(&version).await {
            Ok(installed) => Ok(installed),
            Err(e) => Err(format!("Failed to check if version {} is installed for {}: {}", version, manager_name, e)),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

