use crate::database::DatabaseManager;
use crate::domains::scripts::services::{
    ExecuteScriptRequest, ScriptExecutionInfo, ScriptExecutionService,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Shared state for script execution service
pub struct ScriptExecutionState {
    pub service: Arc<Mutex<Option<ScriptExecutionService>>>,
}

impl ScriptExecutionState {
    pub fn new() -> Self {
        Self {
            service: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_or_init(&self, db_manager: &Arc<DatabaseManager>) -> ScriptExecutionService {
        let mut service = self.service.lock().await;
        if service.is_none() {
            *service = Some(ScriptExecutionService::new(db_manager.clone()));
        }
        service.as_ref().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteScriptParams {
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: Option<HashMap<String, String>>,
    pub working_directory: Option<String>,
}

/// Execute a script
#[tauri::command]
pub async fn execute_script(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    params: ExecuteScriptParams,
) -> Result<String, String> {
    let service = execution_state.get_or_init(&db_manager).await;

    let request = ExecuteScriptRequest {
        block_id: params.block_id,
        command: params.command,
        parameters: params.parameters.unwrap_or_default(),
        working_directory: params.working_directory,
    };

    service.execute_script(request).await
}

/// Get execution details
#[tauri::command]
pub async fn get_script_execution(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    execution_id: String,
) -> Result<Option<ScriptExecutionInfo>, String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.get_execution(&execution_id).await
}

/// Get live output for a running execution
#[tauri::command]
pub async fn get_script_execution_live_output(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    execution_id: String,
) -> Result<Vec<String>, String> {
    let service = execution_state.get_or_init(&db_manager).await;
    Ok(service.get_live_output(&execution_id))
}

/// Cancel a running execution
#[tauri::command]
pub async fn cancel_script_execution(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    execution_id: String,
) -> Result<(), String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.cancel_execution(&execution_id).await
}

/// Get executions for a specific block/script
#[tauri::command]
pub async fn get_script_executions_by_block(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    block_id: String,
    limit: Option<u64>,
) -> Result<Vec<ScriptExecutionInfo>, String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.get_executions_by_block(&block_id, limit).await
}

/// Get all currently running executions
#[tauri::command]
pub async fn get_running_script_executions(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
) -> Result<Vec<ScriptExecutionInfo>, String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.get_running_executions().await
}

/// Get recent executions
#[tauri::command]
pub async fn get_recent_script_executions(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    limit: Option<u64>,
) -> Result<Vec<ScriptExecutionInfo>, String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.get_recent_executions(limit.unwrap_or(20)).await
}

/// Sync running executions on app startup
#[tauri::command]
pub async fn sync_script_executions(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
) -> Result<(), String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.sync_running_executions().await
}

/// Delete an execution record
#[tauri::command]
pub async fn delete_script_execution(
    db_manager: State<'_, Arc<DatabaseManager>>,
    execution_state: State<'_, ScriptExecutionState>,
    execution_id: String,
) -> Result<(), String> {
    let service = execution_state.get_or_init(&db_manager).await;
    service.delete_execution(&execution_id).await
}
