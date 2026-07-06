/**
 * Package Manager Commands
 *
 * Tauri commands for package manager operations (search, install, upgrade, etc.)
 */
use crate::domains::sdk::factory::PackageManagerFactory;
use crate::domains::sdk::traits::package_manager::{
    InstalledPackage, Package, PackageDetails, PackageUpdate,
};

/// Get list of available package managers on the system
#[tauri::command]
pub async fn get_available_package_managers() -> Result<Vec<String>, String> {
    let factory = PackageManagerFactory::new();
    let available = factory.detect_available_managers().await;
    Ok(available)
}

/// Search for packages using a specific package manager
#[tauri::command]
pub async fn package_manager_search(
    manager_name: String,
    query: String,
) -> Result<Vec<Package>, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.search_packages(&query).await {
            Ok(packages) => Ok(packages),
            Err(e) => Err(format!("Failed to search packages: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// List installed packages from a specific package manager
#[tauri::command]
pub async fn package_manager_list_installed(
    manager_name: String,
) -> Result<Vec<InstalledPackage>, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.get_installed_packages().await {
            Ok(packages) => Ok(packages),
            Err(e) => Err(format!("Failed to list installed packages: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Get detailed information about a package
#[tauri::command]
pub async fn package_manager_get_details(
    manager_name: String,
    package_id: String,
) -> Result<PackageDetails, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.get_package_details(&package_id).await {
            Ok(details) => Ok(details),
            Err(e) => Err(format!("Failed to get package details: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Install a package using a specific package manager
#[tauri::command]
pub async fn package_manager_install(
    manager_name: String,
    package_id: String,
    version: Option<String>,
) -> Result<String, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager
            .install_package(&package_id, version.as_deref())
            .await
        {
            Ok(_) => Ok(format!(
                "Successfully installed {} via {}",
                package_id, manager_name
            )),
            Err(e) => Err(format!("Failed to install package: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Upgrade a package using a specific package manager
#[tauri::command]
pub async fn package_manager_upgrade(
    manager_name: String,
    package_id: String,
) -> Result<String, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.upgrade_package(&package_id).await {
            Ok(_) => Ok(format!(
                "Successfully upgraded {} via {}",
                package_id, manager_name
            )),
            Err(e) => Err(format!("Failed to upgrade package: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Uninstall a package using a specific package manager
#[tauri::command]
pub async fn package_manager_uninstall(
    manager_name: String,
    package_id: String,
) -> Result<String, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.uninstall_package(&package_id).await {
            Ok(_) => Ok(format!(
                "Successfully uninstalled {} via {}",
                package_id, manager_name
            )),
            Err(e) => Err(format!("Failed to uninstall package: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Check for available updates using a specific package manager
#[tauri::command]
pub async fn package_manager_check_updates(
    manager_name: String,
) -> Result<Vec<PackageUpdate>, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.check_updates().await {
            Ok(updates) => Ok(updates),
            Err(e) => Err(format!("Failed to check updates: {}", e)),
        }
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}

/// Get information about a package manager
#[tauri::command]
pub async fn package_manager_info(manager_name: String) -> Result<serde_json::Value, String> {
    let factory = PackageManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        let available = manager.is_available().await.unwrap_or(false);
        let version = manager
            .get_version()
            .await
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(serde_json::json!({
            "name": manager.name(),
            "display_name": manager.display_name(),
            "platform": manager.platform(),
            "available": available,
            "version": version,
            "supports_search": manager.supports_search(),
            "supports_updates": manager.supports_updates(),
            "requires_elevation": manager.requires_elevation(),
        }))
    } else {
        Err(format!("Package manager '{}' not found", manager_name))
    }
}
