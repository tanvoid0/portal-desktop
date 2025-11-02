use tauri::State;
use serde_json::Value;
use std::sync::Arc;
use crate::domains::automation::services::automation_service::AutomationService;
use crate::domains::automation::entities::*;

#[tauri::command]
pub async fn trigger_n8n_workflow(
    workflow_id: String,
    project_data: Value,
    automation_service: State<'_, Arc<AutomationService>>,
) -> Result<WorkflowResult, String> {
    automation_service
        .trigger_workflow(&workflow_id, &project_data)
        .await
}

#[tauri::command]
pub async fn get_workflow_status(
    execution_id: String,
    automation_service: State<'_, Arc<AutomationService>>,
) -> Result<WorkflowExecution, String> {
    automation_service
        .get_workflow_status(&execution_id)
        .await
}

#[tauri::command]
pub async fn list_available_workflows(
    automation_service: State<'_, Arc<AutomationService>>,
) -> Result<Vec<AvailableWorkflow>, String> {
    automation_service
        .list_available_workflows()
        .await
}

#[tauri::command]
pub async fn get_suggested_workflows(
    framework: Option<String>,
    package_manager: Option<String>,
    automation_service: State<'_, Arc<AutomationService>>,
) -> Result<Vec<AvailableWorkflow>, String> {
    automation_service
        .get_suggested_workflows(
            framework.as_deref(),
            package_manager.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn check_n8n_health(
    automation_service: State<'_, Arc<AutomationService>>,
) -> Result<bool, String> {
    automation_service
        .check_n8n_health()
        .await
}
