use tauri::State;
use std::sync::Arc;
use super::services::settings_service::{SettingsService, Settings, SettingsUpdate};

#[tauri::command]
pub async fn get_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
) -> Result<Settings, String> {
    _state.load_settings()
}

#[tauri::command]
pub async fn save_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
    settings: Settings,
) -> Result<(), String> {
    _state.save_settings(&settings)
}

#[tauri::command]
pub async fn update_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
    settings: Settings,
    updates: SettingsUpdate,
) -> Result<Settings, String> {
    _state.update_settings(settings, updates)
}

#[tauri::command]
pub async fn reset_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
) -> Result<Settings, String> {
    _state.reset_settings()
}

#[tauri::command]
pub async fn export_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
    settings: Settings,
) -> Result<String, String> {
    _state.export_settings(&settings)
}

#[tauri::command]
pub async fn import_settings_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, Arc<SettingsService>>,
    settings_json: String,
) -> Result<Settings, String> {
    _state.import_settings(&settings_json)
}
