use tauri::State;
use serde_json::Value;
use std::sync::Arc;
use crate::domains::projects::pipelines::services::{PipelineService, ExecutionService};

#[tauri::command]
pub async fn get_pipeline_executions(
    pipeline_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    let pipeline_id_int = pipeline_id.parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.get_executions_by_pipeline(pipeline_id_int).await
}

#[tauri::command]
pub async fn get_pipeline_variables(
    _scope: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    // FUTURE: Pipeline variables management - requires database schema for variable storage
    // Planned for future release - variables will be stored per pipeline/execution scope
    Ok(vec![])
}

#[tauri::command]
pub async fn set_pipeline_variable(
    _scope: Value,
    variable: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    // FUTURE: Pipeline variables management - requires database schema for variable storage
    // Planned for future release
    Ok(variable)
}

#[tauri::command]
pub async fn delete_pipeline_variable(
    _scope: Value,
    _variable_name: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline variables management - requires database schema for variable storage
    // Planned for future release
    Ok(())
}

#[tauri::command]
pub async fn get_pipeline_secrets(
    _scope: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<String>, String> {
    // FUTURE: Pipeline secrets management - requires integration with credentials domain
    // Planned for future release - secrets will reference credential vault entries
    Ok(vec![])
}

#[tauri::command]
pub async fn add_pipeline_secret(
    _scope: Value,
    _secret_id: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline secrets management - requires integration with credentials domain
    // Planned for future release
    Ok(())
}

#[tauri::command]
pub async fn remove_pipeline_secret(
    _scope: Value,
    _secret_id: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline secrets management - requires integration with credentials domain
    // Planned for future release
    Ok(())
}

#[tauri::command]
pub async fn get_blocks(
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    // FUTURE: Custom pipeline blocks - requires database schema for block storage
    // For now, default blocks are managed in frontend. Custom blocks planned for future release.
    Ok(vec![])
}

#[tauri::command]
pub async fn create_block(
    request: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    // FUTURE: Custom pipeline blocks - requires database schema for block storage
    // Planned for future release
    Ok(request)
}

#[tauri::command]
pub async fn update_block(
    _block_id: String,
    request: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    // FUTURE: Custom pipeline blocks - requires database schema for block storage
    // Planned for future release
    Ok(request)
}

#[tauri::command]
pub async fn delete_block(
    _block_id: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Custom pipeline blocks - requires database schema for block storage
    // Planned for future release
    Ok(())
}

#[tauri::command]
pub async fn get_step_execution_logs(
    _execution_id: String,
    _step_id: String,
    _service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<String>, String> {
    // FUTURE: Step execution logs - requires log storage system for pipeline steps
    // Planned for future release - logs will be stored per step execution
    Ok(vec![])
}

#[tauri::command]
pub async fn retry_step_execution(
    _execution_id: String,
    _step_id: String,
    _service: State<'_, Arc<ExecutionService>>,
) -> Result<(), String> {
    // FUTURE: Step retry functionality - requires execution state management
    // Planned for future release - will allow retrying failed steps from checkpoint
    Ok(())
}

#[tauri::command]
pub async fn create_pipeline(
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    let pipeline_request = serde_json::from_value(request)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    let pipeline_id = service.create_pipeline(pipeline_request).await?;
    Ok(serde_json::json!({ "id": pipeline_id.to_string() }))
}

#[tauri::command]
pub async fn get_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Option<Value>, String> {
    let pipeline_id_int = pipeline_id.parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.get_pipeline(pipeline_id_int).await
}

#[tauri::command]
pub async fn get_pipelines(
    project_id: i32,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    service.get_pipelines(project_id).await
}

#[tauri::command]
pub async fn update_pipeline(
    pipeline_id: String,
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    let pipeline_id_int = pipeline_id.parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    let pipeline_request = serde_json::from_value(request)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    let id = service.update_pipeline(pipeline_id_int, pipeline_request).await?;
    Ok(serde_json::json!({ "id": id.to_string() }))
}

#[tauri::command]
pub async fn delete_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    let pipeline_id_int = pipeline_id.parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.delete_pipeline(pipeline_id_int).await
}

#[tauri::command]
pub async fn execute_pipeline(
    request: Value,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Value, String> {
    let execution_request = serde_json::from_value(request)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    let execution_id = service.execute_pipeline(execution_request).await?;
    Ok(serde_json::json!({ "id": execution_id }))
}

#[tauri::command]
pub async fn get_pipeline_execution(
    execution_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Option<Value>, String> {
    service.get_execution(&execution_id).await
}

#[tauri::command]
pub async fn cancel_pipeline_execution(
    execution_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<(), String> {
    service.cancel_execution(&execution_id).await
}

