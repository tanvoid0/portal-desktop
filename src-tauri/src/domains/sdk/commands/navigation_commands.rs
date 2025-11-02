/**
 * Navigation Commands
 * 
 * Tauri commands for SDK navigation functionality
 */

use tauri::State;
use crate::domains::sdk::services::navigation_service::{NavigationService, NavigationResponse, SdkDetails};

/// Get SDK navigation items with installation status
#[tauri::command]
pub async fn get_sdk_navigation_items(
    navigation_service: State<'_, NavigationService>,
) -> Result<NavigationResponse, String> {
    navigation_service
        .get_sdk_navigation_items()
        .await
        .map_err(|e| e.to_string())
}

/// Get detailed information about a specific SDK
#[tauri::command]
pub async fn get_sdk_details(
    navigation_service: State<'_, NavigationService>,
    sdk_type: String,
) -> Result<Option<SdkDetails>, String> {
    navigation_service
        .get_sdk_details(&sdk_type)
        .await
        .map_err(|e| e.to_string())
}
