use crate::domains::projects::pipelines::services::{
    ExecutionRequestData, ExecutionService, PipelineService,
};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_pipeline_executions(
    pipeline_id: String,
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service
        .get_executions_by_pipeline(pipeline_id_int, limit)
        .await
}

#[tauri::command]
pub async fn get_project_pipeline_executions(
    project_id: i32,
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    service.get_executions_by_project(project_id, limit).await
}

#[tauri::command]
pub async fn get_all_pipeline_executions(
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    service.get_all_executions(limit).await
}

#[tauri::command]
pub async fn get_pipeline_variables(
    _scope: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    Err(
        "Pipeline variables are not implemented yet. Variables are embedded in pipeline definitions."
            .to_string(),
    )
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
    Err(
        "Pipeline secrets are not implemented yet. Use the credentials vault for secret storage."
            .to_string(),
    )
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
pub async fn get_blocks(service: State<'_, Arc<PipelineService>>) -> Result<Vec<Value>, String> {
    service.get_blocks().await
}

#[tauri::command]
pub async fn create_block(
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    service.create_block(request).await
}

#[tauri::command]
pub async fn update_block(
    block_id: String,
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    service.update_block(&block_id, request).await
}

#[tauri::command]
pub async fn delete_block(
    block_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    service.delete_block(&block_id).await
}

#[tauri::command]
pub async fn get_step_execution_logs(
    execution_id: String,
    step_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<String>, String> {
    service.get_step_logs(&execution_id, &step_id).await
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
    let pipeline_request =
        crate::domains::projects::pipelines::services::pipeline_service::parse_pipeline_request(
            request,
        )?;
    let pipeline_id = service.create_pipeline(pipeline_request).await?;
    service
        .get_pipeline(pipeline_id)
        .await?
        .ok_or_else(|| "Pipeline not found after create".to_string())
}

#[tauri::command]
pub async fn get_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Option<Value>, String> {
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
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
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    let pipeline_request =
        crate::domains::projects::pipelines::services::pipeline_service::parse_pipeline_request(
            request,
        )?;
    let id = service
        .update_pipeline(pipeline_id_int, pipeline_request)
        .await?;
    service
        .get_pipeline(id)
        .await?
        .ok_or_else(|| "Pipeline not found after update".to_string())
}

#[tauri::command]
pub async fn delete_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.delete_pipeline(pipeline_id_int).await
}

#[tauri::command]
pub async fn execute_pipeline(
    request: Value,
    app: tauri::AppHandle,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Value, String> {
    let execution_request = parse_execution_request(request)?;
    let execution_id = service.execute_pipeline(execution_request, app).await?;
    service
        .get_execution(&execution_id)
        .await?
        .ok_or_else(|| "Execution not found after start".to_string())
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
    app: tauri::AppHandle,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<(), String> {
    service.cancel_execution(&execution_id, Some(app)).await
}

fn parse_execution_request(value: Value) -> Result<ExecutionRequestData, String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FrontendExecutionRequest {
        pipeline_id: String,
        variables: Option<std::collections::HashMap<String, String>>,
        secrets: Option<std::collections::HashMap<String, String>>,
    }

    if let Ok(request) = serde_json::from_value::<ExecutionRequestData>(value.clone()) {
        return Ok(request);
    }

    let frontend: FrontendExecutionRequest = serde_json::from_value(value)
        .map_err(|e| format!("Failed to parse execution request: {}", e))?;

    Ok(ExecutionRequestData {
        pipeline_id: frontend.pipeline_id,
        variables: frontend.variables,
        secrets: frontend.secrets,
    })
}
