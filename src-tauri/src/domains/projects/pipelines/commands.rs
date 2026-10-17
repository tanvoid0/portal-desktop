use crate::domains::projects::pipelines::services::{
    ExecutionRequestData, ExecutionService, PipelineService,
};
use crate::events::Events;
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[portal_rpc_macro::rpc]
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

#[portal_rpc_macro::rpc]
pub async fn get_project_pipeline_executions(
    project_id: i32,
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    service.get_executions_by_project(project_id, limit).await
}

#[portal_rpc_macro::rpc]
pub async fn get_all_pipeline_executions(
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    service.get_all_executions(limit).await
}

#[portal_rpc_macro::rpc]
pub async fn get_pipeline_variables(
    _scope: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    Err(
        "Pipeline variables are not implemented yet. Variables are embedded in pipeline definitions."
            .to_string(),
    )
}

#[portal_rpc_macro::rpc]
pub async fn set_pipeline_variable(
    _scope: Value,
    variable: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    // FUTURE: Pipeline variables management - requires database schema for variable storage
    // Planned for future release
    Ok(variable)
}

#[portal_rpc_macro::rpc]
pub async fn delete_pipeline_variable(
    _scope: Value,
    _variable_name: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline variables management - requires database schema for variable storage
    // Planned for future release
    Ok(())
}

#[portal_rpc_macro::rpc]
pub async fn get_pipeline_secrets(
    _scope: Value,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<String>, String> {
    Err(
        "Pipeline secrets are not implemented yet. Use the credentials vault for secret storage."
            .to_string(),
    )
}

#[portal_rpc_macro::rpc]
pub async fn add_pipeline_secret(
    _scope: Value,
    _secret_id: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline secrets management - requires integration with credentials domain
    // Planned for future release
    Ok(())
}

#[portal_rpc_macro::rpc]
pub async fn remove_pipeline_secret(
    _scope: Value,
    _secret_id: String,
    _service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    // FUTURE: Pipeline secrets management - requires integration with credentials domain
    // Planned for future release
    Ok(())
}

#[portal_rpc_macro::rpc]
pub async fn get_blocks(service: State<'_, Arc<PipelineService>>) -> Result<Vec<Value>, String> {
    service.get_blocks().await
}

#[portal_rpc_macro::rpc]
pub async fn create_block(
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    service.create_block(request).await
}

#[portal_rpc_macro::rpc]
pub async fn update_block(
    block_id: String,
    request: Value,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Value, String> {
    service.update_block(&block_id, request).await
}

#[portal_rpc_macro::rpc]
pub async fn delete_block(
    block_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    service.delete_block(&block_id).await
}

#[portal_rpc_macro::rpc]
pub async fn get_step_execution_logs(
    execution_id: String,
    step_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<String>, String> {
    service.get_step_logs(&execution_id, &step_id).await
}

#[portal_rpc_macro::rpc]
pub async fn retry_step_execution(
    _execution_id: String,
    _step_id: String,
    _service: State<'_, Arc<ExecutionService>>,
) -> Result<(), String> {
    // FUTURE: Step retry functionality - requires execution state management
    // Planned for future release - will allow retrying failed steps from checkpoint
    Ok(())
}

#[portal_rpc_macro::rpc]
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

#[portal_rpc_macro::rpc]
pub async fn get_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Option<Value>, String> {
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.get_pipeline(pipeline_id_int).await
}

#[portal_rpc_macro::rpc]
pub async fn get_pipelines(
    project_id: i32,
    service: State<'_, Arc<PipelineService>>,
) -> Result<Vec<Value>, String> {
    service.get_pipelines(project_id).await
}

#[portal_rpc_macro::rpc]
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

#[portal_rpc_macro::rpc]
pub async fn delete_pipeline(
    pipeline_id: String,
    service: State<'_, Arc<PipelineService>>,
) -> Result<(), String> {
    let pipeline_id_int = pipeline_id
        .parse::<i32>()
        .map_err(|_| "Invalid pipeline ID".to_string())?;
    service.delete_pipeline(pipeline_id_int).await
}

#[portal_rpc_macro::rpc]
pub async fn execute_pipeline(
    request: Value,
    events: State<'_, Events>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Value, String> {
    let execution_request = parse_execution_request(request)?;
    let execution_id = service
        .execute_pipeline(execution_request, events.sink())
        .await?;
    service
        .get_execution(&execution_id)
        .await?
        .ok_or_else(|| "Execution not found after start".to_string())
}

#[portal_rpc_macro::rpc]
pub async fn get_pipeline_execution(
    execution_id: String,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Option<Value>, String> {
    service.get_execution(&execution_id).await
}

#[portal_rpc_macro::rpc]
pub async fn cancel_pipeline_execution(
    execution_id: String,
    events: State<'_, Events>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<(), String> {
    service
        .cancel_execution(&execution_id, Some(events.sink()))
        .await
}

// ── Releases ─────────────────────────────────────────────────────────────
// A release is a workflow from the project's `.portal/pipeline.yml` named by its
// `release:` block. Rehearsing is reachable by any client of the daemon; shipping
// is `local_only`, so it can only be started from this app's own window.

#[portal_rpc_macro::rpc]
pub async fn release_describe(
    project_id: i32,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Option<Value>, String> {
    service.describe_release(project_id).await
}

#[portal_rpc_macro::rpc]
pub async fn release_list(
    project_id: i32,
    limit: Option<u64>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Vec<Value>, String> {
    service.get_release_executions(project_id, limit).await
}

async fn start_release(
    project_id: i32,
    ship: bool,
    mut variables: std::collections::HashMap<String, String>,
    events: &Events,
    service: &ExecutionService,
) -> Result<Value, String> {
    let described = service
        .describe_release(project_id)
        .await?
        .ok_or_else(|| "This project declares no `release:` in .portal/pipeline.yml".to_string())?;
    let workflow = if ship {
        described["release"]["ship"].as_str()
    } else {
        described["release"]["rehearse"].as_str()
    }
    .ok_or_else(|| "No `rehearse` workflow declared; a rehearsal never runs the ship workflow".to_string())?
    .to_string();
    variables.insert("RELEASE_DRY_RUN".to_string(), if ship { "0" } else { "1" }.to_string());

    let id = service
        .execute_file_workflow(project_id, &workflow, variables, events.sink())
        .await?;
    service
        .get_execution(&id)
        .await?
        .ok_or_else(|| "Execution not found after start".to_string())
}

/// Dry run: the project's `rehearse` workflow, with `RELEASE_DRY_RUN=1`.
#[portal_rpc_macro::rpc]
pub async fn release_rehearse(
    project_id: i32,
    variables: Option<std::collections::HashMap<String, String>>,
    events: State<'_, Events>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Value, String> {
    start_release(project_id, false, variables.unwrap_or_default(), &events, &service).await
}

/// The real release. `local_only`: the daemon has no auth, and this deploys to
/// production, so no client of the HTTP port may start it. A Claude session
/// rehearses and watches; a person clicks Ship in the app.
#[portal_rpc_macro::rpc(local_only)]
pub async fn release_ship(
    project_id: i32,
    variables: Option<std::collections::HashMap<String, String>>,
    events: State<'_, Events>,
    service: State<'_, Arc<ExecutionService>>,
) -> Result<Value, String> {
    start_release(project_id, true, variables.unwrap_or_default(), &events, &service).await
}

// ── Release setup: propose, refine with AI, write, and health ────────────

async fn root_and_modules(
    db: &Arc<crate::database::DatabaseManager>,
    project_id: i32,
) -> Result<(std::path::PathBuf, Vec<crate::domains::projects::modules::DetectedModule>), String> {
    use crate::domains::projects::modules::{DetectedModule, ModuleService};
    let project = crate::domains::projects::repositories::project_repository::ProjectRepository::new(db.clone())
        .get_by_id(project_id)
        .await?
        .ok_or_else(|| "Project not found".to_string())?;
    let modules = ModuleService::new(db)
        .refresh(project_id)
        .await?
        .into_iter()
        .map(|m| DetectedModule { rel_path: m.rel_path, name: m.name, stack: m.stack, role: m.role })
        .collect();
    Ok((std::path::PathBuf::from(project.path), modules))
}

/// A `.portal/pipeline.yml` built from what the project contains. Read-only.
#[portal_rpc_macro::rpc]
pub async fn release_propose(
    project_id: i32,
    db_manager: State<'_, Arc<crate::database::DatabaseManager>>,
) -> Result<crate::domains::projects::pipelines::services::release_setup::Proposal, String> {
    let (root, modules) = root_and_modules(&db_manager, project_id).await?;
    crate::domains::projects::pipelines::services::release_setup::propose(&root, &modules)
}

/// Whether `yaml` is a usable pipeline file (parses; references resolve).
#[portal_rpc_macro::rpc]
pub async fn release_validate(yaml: String) -> Result<(), String> {
    use crate::domains::projects::pipelines::services::pipeline_file as pf;
    let file = pf::parse(&yaml)?;
    for id in file.workflows.keys() {
        pf::build_steps(&file, id)?;
    }
    Ok(())
}

/// What would make this project's release fail or quietly skip something.
#[portal_rpc_macro::rpc]
pub async fn release_health(
    project_id: i32,
    db_manager: State<'_, Arc<crate::database::DatabaseManager>>,
) -> Result<Vec<crate::domains::projects::pipelines::services::release_setup::Issue>, String> {
    let (root, modules) = root_and_modules(&db_manager, project_id).await?;
    tokio::task::spawn_blocking(move || {
        crate::domains::projects::pipelines::services::release_setup::health(&root, &modules)
    })
    .await
    .map_err(|e| e.to_string())
}

/// Writes `.portal/pipeline.yml`, keeping the old file as `.bak`.
///
/// `local_only`: the file's commands are what a release runs, and the daemon has
/// no auth, so no HTTP client may author them.
#[portal_rpc_macro::rpc(local_only)]
pub async fn release_write(
    project_id: i32,
    yaml: String,
    db_manager: State<'_, Arc<crate::database::DatabaseManager>>,
) -> Result<String, String> {
    release_validate(yaml.clone()).await?;
    let (root, _) = root_and_modules(&db_manager, project_id).await?;
    let dir = root.join(".portal");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("pipeline.yml");
    if path.is_file() {
        let _ = std::fs::copy(&path, dir.join("pipeline.yml.bak"));
    }
    std::fs::write(&path, yaml).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path.to_string_lossy().into_owned())
}

/// Asks the AI gateway to change the pipeline file as instructed, and returns the
/// new YAML once it validates (one retry, telling the model what was wrong).
/// `local_only`: it sends project details to the model and is a UI flow.
#[portal_rpc_macro::rpc(local_only)]
pub async fn release_refine(
    project_id: i32,
    yaml: String,
    instruction: String,
    db_manager: State<'_, Arc<crate::database::DatabaseManager>>,
    ai_service: State<'_, Arc<portal_core::domains::ai::services::AIService>>,
) -> Result<String, String> {
    use crate::domains::ai::providers::GenerationOptions;
    use crate::domains::projects::pipelines::services::release_setup as rs;
    let (_, modules) = root_and_modules(&db_manager, project_id).await?;
    let (system, mut user) = rs::refine_prompt(&yaml, &instruction, &modules);
    let options = GenerationOptions {
        temperature: Some(0.2),
        max_tokens: Some(4096),
        timeout_ms: Some(120_000),
        ..Default::default()
    };
    let mut last_error = String::new();
    for _ in 0..2 {
        let reply = ai_service
            .generate_with_system(&system, &user, Some(options.clone()), None)
            .await
            .map_err(|e| format!("AI request failed: {e}"))?;
        let candidate = rs::extract_yaml(&reply.content);
        match release_validate(candidate.clone()).await {
            Ok(()) => return Ok(candidate),
            Err(e) => {
                user = format!("{user}

Your last reply was rejected: {e}
Reply again with the corrected complete file.");
                last_error = e;
            }
        }
    }
    Err(format!("The AI reply was not a valid pipeline file: {last_error}"))
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
