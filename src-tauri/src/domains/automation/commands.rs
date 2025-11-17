use tauri::State;
use serde_json::Value;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::domains::automation::services::automation_service::AutomationService;
use crate::domains::automation::services::workflow_engine::{WorkflowEngine, Workflow, WorkflowContext, WorkflowExecutionResult};
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

// Workflow Engine Commands
static WORKFLOW_ENGINES: OnceLock<Arc<Mutex<HashMap<String, WorkflowEngine>>>> = OnceLock::new();

fn get_workflow_engines() -> Arc<Mutex<HashMap<String, WorkflowEngine>>> {
    WORKFLOW_ENGINES.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

#[tauri::command]
pub async fn register_embedded_workflow(
    workflow_json: String,
) -> Result<String, String> {
    // Extract workflow ID from JSON first
    let workflow: Workflow = serde_json::from_str(&workflow_json)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;
    
    let workflow_id = workflow.id.clone();
    
    let engines = get_workflow_engines();
    let mut engines_guard = engines.lock().await;
    let engine = engines_guard.entry("default".to_string())
        .or_insert_with(WorkflowEngine::new);
    
    engine.load_from_json(&workflow_json)
        .map_err(|e| format!("Failed to load workflow: {}", e))?;
    
    Ok(workflow_id)
}

#[tauri::command]
pub async fn execute_embedded_workflow(
    workflow_id: String,
    project_path: Option<String>,
    variables: Option<HashMap<String, String>>,
) -> Result<WorkflowExecutionResult, String> {
    let context = WorkflowContext {
        project_path: project_path.map(PathBuf::from),
        variables: variables.unwrap_or_default(),
        trigger_data: None,
    };
    
    let engines = get_workflow_engines();
    let engines_guard = engines.lock().await;
    let engine = engines_guard.get("default")
        .ok_or("Workflow engine not initialized")?;
    
    engine.execute_workflow(&workflow_id, context).await
}

#[tauri::command]
pub async fn list_embedded_workflows() -> Result<Vec<Value>, String> {
    let engines = get_workflow_engines();
    let engines_guard = engines.lock().await;
    let engine = engines_guard.get("default")
        .ok_or("Workflow engine not initialized")?;
    
    let workflows: Vec<_> = engine.get_workflows().into_iter().collect();
    let mut result = Vec::new();
    
    for workflow in workflows {
        result.push(serde_json::json!({
            "id": workflow.id,
            "name": workflow.name,
            "description": workflow.description,
            "enabled": workflow.enabled,
            "trigger_count": workflow.triggers.len(),
            "step_count": workflow.steps.len(),
        }));
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn check_workflow_trigger(
    workflow_id: String,
    trigger_data: Value,
) -> Result<bool, String> {
    let engines = get_workflow_engines();
    let engines_guard = engines.lock().await;
    let engine = engines_guard.get("default")
        .ok_or("Workflow engine not initialized")?;
    
    let result = engine.should_trigger(&workflow_id, &trigger_data);
    Ok(result)
}
