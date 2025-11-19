use super::services::update_service::UpdateService;

#[tauri::command]
pub async fn get_app_version_command() -> Result<String, String> {
    Ok(UpdateService::get_current_version())
}

