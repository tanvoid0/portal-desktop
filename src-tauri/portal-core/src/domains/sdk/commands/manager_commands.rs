//! Tauri adapters over `portal_core::domains::sdk::services::manager_workflow`.

use crate::domains::sdk::factory::SDKManagerFactory;
use crate::domains::sdk::services::manager_workflow::{
    execute_workflow, explain_missing_install_workflow, explain_missing_uninstall_workflow,
    find_manager_config, get_sdk_manager_workflow_support, resolve_install_workflow,
    resolve_uninstall_workflow,
};

/// Get installed versions for a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn get_manager_installed_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();

    match factory.list_versions(&manager_name).await {
        Ok(versions) => Ok(versions),
        Err(e) => Err(format!(
            "Failed to list versions for {}: {}",
            manager_name, e
        )),
    }
}

/// Get available (installable) versions for a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn get_manager_available_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.list_available_versions().await {
            Ok(versions) => Ok(versions),
            Err(e) => Err(format!(
                "Failed to list available versions for {}: {}",
                manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Get current active version for a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn get_manager_current_version(manager_name: String) -> Result<Option<String>, String> {
    let factory = SDKManagerFactory::new();

    match factory.get_current_version(&manager_name).await {
        Ok(version) => Ok(version),
        Err(e) => Err(format!(
            "Failed to get current version for {}: {}",
            manager_name, e
        )),
    }
}

/// Install a version using a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn install_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    match factory.install_version(&manager_name, &version).await {
        Ok(_) => Ok(format!(
            "Successfully installed {} version {}",
            manager_name, version
        )),
        Err(e) => Err(format!(
            "Failed to install version {} for {}: {}",
            version, manager_name, e
        )),
    }
}

/// Install an SDK manager itself rather than an SDK version.
#[portal_rpc_macro::rpc]
pub async fn install_sdk_manager(manager_name: String) -> Result<String, String> {
    let config = find_manager_config(&manager_name)
        .ok_or_else(|| format!("Manager '{}' not found", manager_name))?;
    let support = get_sdk_manager_workflow_support(&manager_name, false);

    if !support.install_available {
        return Err(support
            .install_unavailable_reason
            .unwrap_or_else(|| explain_missing_install_workflow(&config)));
    }

    let workflow = resolve_install_workflow(&config)
        .ok_or_else(|| explain_missing_install_workflow(&config))?;
    execute_workflow(&manager_name, "Installation", workflow).await?;

    Ok(format!("Successfully installed {}", config.display_name))
}

/// Switch to a version using a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn switch_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.switch_version(&version).await {
            Ok(_) => Ok(format!(
                "Successfully switched to version {} for {}",
                version, manager_name
            )),
            Err(e) => Err(format!(
                "Failed to switch to version {} for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Uninstall a version using a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn uninstall_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.uninstall_version(&version).await {
            Ok(_) => Ok(format!(
                "Successfully uninstalled {} version {}",
                manager_name, version
            )),
            Err(e) => Err(format!(
                "Failed to uninstall version {} for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Uninstall an SDK manager itself rather than an SDK version.
#[portal_rpc_macro::rpc]
pub async fn uninstall_sdk_manager(manager_name: String) -> Result<String, String> {
    let config = find_manager_config(&manager_name)
        .ok_or_else(|| format!("Manager '{}' not found", manager_name))?;
    let support = get_sdk_manager_workflow_support(&manager_name, true);

    if !support.uninstall_available {
        return Err(support
            .uninstall_unavailable_reason
            .unwrap_or_else(|| explain_missing_uninstall_workflow(&config)));
    }

    let workflow = resolve_uninstall_workflow(&config)
        .ok_or_else(|| explain_missing_uninstall_workflow(&config))?;
    execute_workflow(&manager_name, "Uninstall", workflow).await?;

    Ok(format!("Successfully uninstalled {}", config.display_name))
}

/// Check if a version is installed for a specific SDK manager
#[portal_rpc_macro::rpc]
pub async fn is_manager_version_installed(
    manager_name: String,
    version: String,
) -> Result<bool, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.is_version_installed(&version).await {
            Ok(installed) => Ok(installed),
            Err(e) => Err(format!(
                "Failed to check if version {} is installed for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}
