use tauri::State;
use super::services::deployment_service::{DeploymentService, CreateDeploymentRequest, UpdateDeploymentRequest};
use super::services::docker_service::{DockerContainer, Deployment};

#[tauri::command]
pub async fn create_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    request: CreateDeploymentRequest,
) -> Result<Deployment, String> {
    _state.create_deployment(request).await
}

#[tauri::command]
pub async fn get_deployments_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
) -> Result<Vec<Deployment>, String> {
    _state.get_deployments()
}

#[tauri::command]
pub async fn get_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    deployment_id: String,
) -> Result<Option<Deployment>, String> {
    _state.get_deployment(&deployment_id)
}

#[tauri::command]
pub async fn start_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    deployment_id: String,
) -> Result<Deployment, String> {
    _state.start_deployment(&deployment_id).await
}

#[tauri::command]
pub async fn stop_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    deployment_id: String,
) -> Result<Deployment, String> {
    _state.stop_deployment(&deployment_id).await
}

#[tauri::command]
pub async fn delete_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    deployment_id: String,
) -> Result<(), String> {
    _state.delete_deployment(&deployment_id).await
}

#[tauri::command]
pub async fn get_deployment_logs_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    deployment_id: String,
    tail: Option<usize>,
) -> Result<Vec<String>, String> {
    _state.get_deployment_logs(&deployment_id, tail).await
}

#[tauri::command]
pub async fn update_deployment_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
    _deployment_id: String,
    request: UpdateDeploymentRequest,
) -> Result<Deployment, String> {
    _state.update_deployment(request).await
}

#[tauri::command]
pub async fn refresh_deployment_statuses_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
) -> Result<Vec<Deployment>, String> {
    _state.refresh_deployment_statuses().await
}

#[tauri::command]
pub async fn list_containers_command(
    _app_handle: tauri::AppHandle,
    _state: State<'_, DeploymentService>,
) -> Result<Vec<DockerContainer>, String> {
    _state.docker_service.list_containers().await
}
