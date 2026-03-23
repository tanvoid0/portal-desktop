/**
 * SDK Configuration Commands
 * 
 * Tauri commands for SDK configuration functionality (all categories).
 * Backend processes configs and returns formatted data to frontend.
 */

use crate::domains::sdk::services::language_config_service::{
    SDKConfigService,
    ProcessedSDKConfig,
    ProcessedSDKManager,
};
use crate::domains::sdk::configs::SDKCategory;

/// Get processed SDK configuration
#[tauri::command]
pub async fn get_sdk_config(
    sdk_id: String,
) -> Result<Option<ProcessedSDKConfig>, String> {
    SDKConfigService::get_sdk_config(&sdk_id).await
}

/// Get all processed SDK configurations
#[tauri::command]
pub async fn get_all_sdk_configs() -> Result<Vec<ProcessedSDKConfig>, String> {
    SDKConfigService::get_all_sdk_configs().await
}

/// Get SDKs by category
#[tauri::command]
pub async fn get_sdks_by_category(
    category: String,
) -> Result<Vec<ProcessedSDKConfig>, String> {
    let sdk_category = SDKCategory::from_str(&category)
        .ok_or_else(|| format!("Invalid SDK category: {}", category))?;
    SDKConfigService::get_sdks_by_category(sdk_category).await
}

// Legacy aliases for backward compatibility
#[tauri::command]
pub async fn get_language_config(
    language_id: String,
) -> Result<Option<ProcessedSDKConfig>, String> {
    get_sdk_config(language_id).await
}

#[tauri::command]
pub async fn get_all_language_configs() -> Result<Vec<ProcessedSDKConfig>, String> {
    get_all_sdk_configs().await
}

/// Get all SDK managers from all SDK configs
#[tauri::command]
pub async fn get_all_sdk_managers() -> Result<Vec<ProcessedSDKManager>, String> {
    SDKConfigService::get_all_sdk_managers().await
}

