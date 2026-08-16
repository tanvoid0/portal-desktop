use super::services::update_service::UpdateService;

#[portal_rpc_macro::rpc]
pub async fn get_app_version_command() -> Result<String, String> {
    Ok(UpdateService::get_current_version())
}
