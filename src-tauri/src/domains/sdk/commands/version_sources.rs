/**
 * FlyEnv-style Version Sources Commands
 * 
 * These commands provide FlyEnv-style smart version detection and management
 * for any SDK type, with backend handling all the heavy lifting.
 */

use serde::{Deserialize, Serialize};
use crate::domains::sdk::factory::SDKManagerFactory;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionSource {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub versions: Vec<SDKVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SDKVersion {
    pub id: String,
    pub library: String,
    pub version: String,
    pub installed: bool,
    pub active: bool,
    pub source: String,
    pub description: Option<String>,
    pub download_url: Option<String>,
    pub release_date: Option<String>,
    pub lts: bool,
}

#[tauri::command]
pub async fn get_version_sources(sdk_type: String) -> Result<Vec<VersionSource>, String> {
    println!("[SDK] Getting version sources for: {}", sdk_type);
    
    let factory = SDKManagerFactory::new();
    
    // Get all managers that support this SDK type
    let managers = factory.get_managers_by_sdk_type(&sdk_type);
    
    if managers.is_empty() {
        return Ok(vec![]);
    }
    
    let mut sources = Vec::new();
    
    for manager in managers {
        if let Ok(installed) = manager.is_installed().await {
            if installed {
                // Get installed versions first
                let mut installed_versions = Vec::new();
                if let Ok(versions) = manager.list_versions().await {
                    for version in versions {
                        let is_active = manager.get_current_version().await
                            .map(|current| current.as_ref() == Some(&version))
                            .unwrap_or(false);
                        
                        installed_versions.push(SDKVersion {
                            id: format!("{}-{}", manager.name(), version),
                            library: format!("{}-{}", manager.name(), version),
                            version: version.clone(),
                            installed: true,
                            active: is_active,
                            source: manager.name().to_string(),
                            description: Some(format!("{} via {}", version, manager.display_name())),
                            download_url: None,
                            release_date: None,
                            lts: version.contains("LTS") || version.contains("lts"),
                        });
                    }
                }
                
                // Get available versions for installation
                let mut available_versions = Vec::new();
                if let Ok(versions) = manager.list_available_versions().await {
                    for version in versions {
                        // Only add if not already installed
                        if !installed_versions.iter().any(|v| v.version == version) {
                            available_versions.push(SDKVersion {
                                id: format!("{}-{}", manager.name(), version),
                                library: format!("{}-{}", manager.name(), version),
                                version: version.clone(),
                                installed: false,
                                active: false,
                                source: manager.name().to_string(),
                                description: Some(format!("{} via {}", version, manager.display_name())),
                                download_url: None,
                                release_date: None,
                                lts: version.contains("LTS") || version.contains("lts"),
                            });
                        }
                    }
                }
                
                // Combine installed and available versions
                let mut all_versions = installed_versions;
                all_versions.extend(available_versions);
                
                if !all_versions.is_empty() {
                    sources.push(VersionSource {
                        name: manager.name().to_string(),
                        display_name: manager.display_name().to_string(),
                        description: format!("{} versions via {}", sdk_type, manager.display_name()),
                        versions: all_versions,
                    });
                }
            }
        }
    }
    
    // Return empty array if no sources found - no mocking
    // This allows the UI to handle the "no sources" state gracefully
    
    println!("[SDK] Found {} version sources for {}", sources.len(), sdk_type);
    Ok(sources)
}

#[tauri::command]
pub async fn refresh_version_status(sdk_type: String) -> Result<(), String> {
    println!("[SDK] Refreshing version status for: {}", sdk_type);
    
    let factory = SDKManagerFactory::new();
    let managers = factory.get_managers_by_sdk_type(&sdk_type);
    
    for manager in managers {
        if let Ok(installed) = manager.is_installed().await {
            if installed {
                // Refresh the manager's version cache
                match manager.list_versions().await {
                    Ok(_) => {
                        println!("[SDK] Refreshed versions for {}", manager.name());
                    },
                    Err(e) => {
                        println!("[SDK] Failed to refresh {}: {}", manager.name(), e);
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn uninstall_sdk_version(
    sdk_type: String,
    version: String,
    manager: String,
) -> Result<(), String> {
    println!("[SDK] Uninstalling {} version {} via {}", sdk_type, version, manager);
    
    let factory = SDKManagerFactory::new();
    
    if let Some(sdk_manager) = factory.get_manager(&manager) {
        if sdk_manager.supports_installation() {
            match sdk_manager.uninstall_version(&version).await {
                Ok(_) => {
                    println!("[SDK] Successfully uninstalled {} version {}", sdk_type, version);
                    Ok(())
                },
                Err(e) => {
                    let error_msg = format!("Failed to uninstall {} version {}: {}", sdk_type, version, e);
                    println!("[SDK] {}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            Err(format!("Manager {} does not support installation", manager))
        }
    } else {
        Err(format!("Manager {} not found", manager))
    }
}

