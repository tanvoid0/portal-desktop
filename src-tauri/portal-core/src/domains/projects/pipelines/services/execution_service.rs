use crate::database::DatabaseManager;
use crate::domains::projects::entities::ProjectResponse;
use crate::domains::projects::pipelines::repositories::{ExecutionRepository, PipelineRepository};
use crate::domains::projects::pipelines::services::pipeline_file;
use crate::domains::projects::pipelines::utils::dependency_resolver::resolve_execution_order;
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use crate::events::{EventSink, EventSinkExt};
use crate::process_ext::NoWindowExt;
use crate::utils::pnpm_workspace::{prepare_shell_command, warn_if_broken_pnpm_workspace};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::watch;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequestData {
    pub pipeline_id: String,
    pub variables: Option<HashMap<String, String>>,
    pub secrets: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StepExecutionState {
    id: String,
    stepId: String,
    stepName: String,
    status: String,
    startedAt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    finishedAt: Option<String>,
    output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exitCode: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<i64>,
    retryCount: i32,
    logs: Vec<String>,
}

enum StepRunOutcome {
    Completed { exit_code: i32, success: bool },
    LongRunning,
}

struct RunningExecution {
    cancel_tx: watch::Sender<bool>,
    children: Arc<Mutex<Vec<Child>>>,
}

#[derive(Clone)]
pub struct ExecutionService {
    execution_repo: ExecutionRepository,
    pipeline_repo: PipelineRepository,
    project_repo: ProjectRepository,
    running: Arc<Mutex<HashMap<String, RunningExecution>>>,
}

impl ExecutionService {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            execution_repo: ExecutionRepository::new(db_manager.clone()),
            pipeline_repo: PipelineRepository::new(db_manager.clone()),
            project_repo: ProjectRepository::new(db_manager),
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn execute_pipeline(
        &self,
        request: ExecutionRequestData,
        events: Arc<dyn EventSink>,
    ) -> Result<String, String> {
        let execution_id = Uuid::new_v4().to_string();
        let pipeline_id = request
            .pipeline_id
            .parse::<i32>()
            .map_err(|_| "Invalid pipeline ID".to_string())?;

        let pipeline = self
            .pipeline_repo
            .get_by_id(pipeline_id)
            .await?
            .ok_or_else(|| "Pipeline not found".to_string())?;

        let project = self
            .project_repo
            .get_by_id(pipeline.project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;

        let steps: Vec<Value> = serde_json::from_str(&pipeline.steps_json).unwrap_or_default();

        let mut variables: HashMap<String, String> = request.variables.unwrap_or_default();
        if let Ok(pipeline_vars) = serde_json::from_str::<Vec<Value>>(&pipeline.variables_json) {
            for var in pipeline_vars {
                if let (Some(name), Some(value)) = (
                    var.get("name").and_then(|v| v.as_str()),
                    var.get("value").and_then(|v| v.as_str()),
                ) {
                    variables
                        .entry(name.to_string())
                        .or_insert(value.to_string());
                }
            }
        }

        variables.insert("PROJECT_PATH".to_string(), project.path.clone());
        variables.insert("PROJECT_NAME".to_string(), project.name.clone());
        let detected_pm = resolve_package_manager(&project);
        variables.insert("PACKAGE_MANAGER".to_string(), detected_pm.clone());
        let build_command = project.build_command.clone();

        let step_executions = build_initial_step_executions(&steps);
        let step_executions_json = serde_json::to_string(&step_executions)
            .map_err(|e| format!("Failed to serialize steps: {}", e))?;
        let variables_json = serde_json::to_string(&variables)
            .map_err(|e| format!("Failed to serialize variables: {}", e))?;

        self.execution_repo
            .create(
                execution_id.clone(),
                pipeline_id,
                pipeline.project_id,
                "pending".to_string(),
                "user".to_string(),
                step_executions_json,
                variables_json,
            )
            .await?;

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let children: Arc<Mutex<Vec<Child>>> = Arc::new(Mutex::new(Vec::new()));

        {
            let mut running = self.running.lock().unwrap();
            running.insert(
                execution_id.clone(),
                RunningExecution {
                    cancel_tx,
                    children: Arc::clone(&children),
                },
            );
        }

        self.execution_repo
            .update_status(&execution_id, "queued".to_string(), None)
            .await?;

        let service = self.clone();
        let exec_id = execution_id.clone();
        let events_handle = events.clone();

        tokio::spawn(async move {
            let result = service
                .run_execution(
                    exec_id.clone(),
                    steps,
                    project.path,
                    variables,
                    build_command,
                    detected_pm,
                    children,
                    cancel_rx,
                    events_handle.clone(),
                )
                .await;

            if let Err(e) = result {
                let _ = service
                    .execution_repo
                    .update_status(&exec_id, "failed".to_string(), Some(e))
                    .await;
                if let Ok(Some(exec)) = service.get_execution(&exec_id).await {
                    service.emit_execution_update(&events_handle, exec);
                }
            }

            let mut running = service.running.lock().unwrap();
            running.remove(&exec_id);
        });

        Ok(execution_id)
    }

    async fn run_execution(
        &self,
        execution_id: String,
        steps: Vec<Value>,
        project_path: String,
        variables: HashMap<String, String>,
        _build_command: Option<String>,
        detected_pm: String,
        children: Arc<Mutex<Vec<Child>>>,
        mut cancel_rx: watch::Receiver<bool>,
        events: Arc<dyn EventSink>,
    ) -> Result<(), String> {
        self.execution_repo
            .update_status(&execution_id, "running".to_string(), None)
            .await?;

        if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
            self.emit_execution_update(&events, exec);
        }

        let order = resolve_execution_order(&steps)?;
        let step_map: HashMap<String, Value> = steps
            .iter()
            .filter_map(|s| {
                s.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| (id.to_string(), s.clone()))
            })
            .collect();

        let mut pipeline_has_long_running = false;

        for group in order {
            for step_id in group {
                if *cancel_rx.borrow() {
                    kill_children_async(&children).await;
                    self.finalize_running_steps(&execution_id, "cancelled")
                        .await?;
                    self.mark_cancelled(&execution_id, &events).await;
                    return Ok(());
                }

                let step = step_map
                    .get(&step_id)
                    .ok_or_else(|| format!("Step {} not found", step_id))?;

                let step_name = step
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&step_id)
                    .to_string();

                let config = step.get("config").cloned().unwrap_or(json!({}));
                let command_template = config
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format!("Step {} missing command", step_id))?
                    .to_string();
                let long_running = config
                    .get("longRunning")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut command = substitute_variables(&command_template, &variables);

                // Honor the resolved command from the Actions catalog / FE.
                // Only normalize the package-manager binary name — do not replace
                // build/dev commands with project metadata heuristics.
                command = normalize_package_manager_command(&command, &detected_pm);

                // Optional per-step `cwd` (relative to the project) and `env`, from
                // `.portal/pipeline.yml`. Absent for steps built by the UI.
                let step_dir = match config.get("cwd").and_then(|v| v.as_str()) {
                    Some(rel) => Path::new(&project_path)
                        .join(rel)
                        .to_string_lossy()
                        .into_owned(),
                    None => project_path.clone(),
                };
                let step_env: HashMap<String, String> = config
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|o| {
                        o.iter()
                            .filter_map(|(k, v)| {
                                v.as_str()
                                    .map(|s| (k.clone(), substitute_variables(s, &variables)))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                self.set_step_running(&execution_id, &step_id).await?;
                if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
                    self.emit_execution_update(&events, exec);
                }

                let step_result = self
                    .run_step_command(
                        &execution_id,
                        &step_id,
                        &command,
                        &step_dir,
                        &step_env,
                        long_running,
                        Arc::clone(&children),
                        &mut cancel_rx,
                        &events,
                    )
                    .await;

                match step_result {
                    Ok(StepRunOutcome::Completed { exit_code, success }) => {
                        let status = if success { "success" } else { "failed" };
                        self.finalize_step(&execution_id, &step_id, status, Some(exit_code), None)
                            .await?;

                        if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
                            self.emit_execution_update(&events, exec);
                        }

                        if !success {
                            self.execution_repo
                                .update_status(
                                    &execution_id,
                                    "failed".to_string(),
                                    Some(format!(
                                        "Step '{}' failed with exit code {}",
                                        step_name, exit_code
                                    )),
                                )
                                .await?;
                            if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
                                self.emit_execution_update(&events, exec);
                            }
                            return Ok(());
                        }
                    }
                    Ok(StepRunOutcome::LongRunning) => {
                        pipeline_has_long_running = true;
                        loop {
                            if *cancel_rx.borrow() {
                                kill_children_async(&children).await;
                                self.finalize_step(
                                    &execution_id,
                                    &step_id,
                                    "cancelled",
                                    None,
                                    None,
                                )
                                .await?;
                                self.mark_cancelled(&execution_id, &events).await;
                                return Ok(());
                            }
                            tokio::select! {
                                changed = cancel_rx.changed() => {
                                    if changed.is_ok() && *cancel_rx.borrow() {
                                        kill_children_async(&children).await;
                                        self.finalize_step(
                                            &execution_id,
                                            &step_id,
                                            "cancelled",
                                            None,
                                            None,
                                        )
                                        .await?;
                                        self.mark_cancelled(&execution_id, &events).await;
                                        return Ok(());
                                    }
                                }
                                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {}
                            }
                        }
                    }
                    Err(e) if e == "Execution cancelled" => {
                        self.finalize_step(&execution_id, &step_id, "cancelled", None, None)
                            .await?;
                        self.mark_cancelled(&execution_id, &events).await;
                        return Ok(());
                    }
                    Err(e) => {
                        self.finalize_step(&execution_id, &step_id, "failed", None, Some(&e))
                            .await?;
                        self.execution_repo
                            .update_status(&execution_id, "failed".to_string(), Some(e))
                            .await?;
                        if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
                            self.emit_execution_update(&events, exec);
                        }
                        return Ok(());
                    }
                }
            }
        }

        if !pipeline_has_long_running {
            self.execution_repo
                .update_status(&execution_id, "success".to_string(), None)
                .await?;
            if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
                self.emit_execution_update(&events, exec);
            }
        }

        Ok(())
    }

    async fn run_step_command(
        &self,
        execution_id: &str,
        step_id: &str,
        command: &str,
        working_directory: &str,
        env: &HashMap<String, String>,
        long_running: bool,
        children: Arc<Mutex<Vec<Child>>>,
        cancel_rx: &mut watch::Receiver<bool>,
        events: &Arc<dyn EventSink>,
    ) -> Result<StepRunOutcome, String> {
        let exec_command = prepare_shell_command(command, working_directory);

        append_step_log(
            &self.execution_repo,
            execution_id,
            step_id,
            &format!("$ {}", exec_command),
            "stdout",
            events,
        )
        .await;
        append_step_log(
            &self.execution_repo,
            execution_id,
            step_id,
            &format!("cwd: {}", working_directory),
            "stdout",
            events,
        )
        .await;

        let pm_prefix = exec_command.split_whitespace().next();
        if matches!(pm_prefix, Some("npm" | "yarn" | "pnpm"))
            && !Path::new(working_directory).join("package.json").exists()
        {
            append_step_log(
                &self.execution_repo,
                execution_id,
                step_id,
                "warning: no package.json in working directory",
                "stderr",
                events,
            )
            .await;
        }

        if pm_prefix == Some("pnpm") {
            if let Some(warning) = warn_if_broken_pnpm_workspace(working_directory) {
                append_step_log(
                    &self.execution_repo,
                    execution_id,
                    step_id,
                    &warning,
                    "stderr",
                    events,
                )
                .await;
            }
        }

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.no_window();
            c.args(["/C", &exec_command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &exec_command]);
            c
        };

        cmd.current_dir(working_directory);
        cmd.envs(env);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut procs = children.lock().unwrap();
            procs.push(child);
        }

        let exec_id = execution_id.to_string();
        let sid = step_id.to_string();
        let repo = self.execution_repo.clone();
        let events_clone = events.clone();

        if let Some(stdout) = stdout {
            let exec_id = exec_id.clone();
            let sid = sid.clone();
            let repo = repo.clone();
            let events_clone = events_clone.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    append_step_log(&repo, &exec_id, &sid, &line, "stdout", &events_clone).await;
                }
            });
        }

        if let Some(stderr) = stderr {
            let exec_id = exec_id.clone();
            let sid = sid.clone();
            let repo = repo.clone();
            let events_clone = events_clone.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    append_step_log(&repo, &exec_id, &sid, &line, "stderr", &events_clone).await;
                }
            });
        }

        if long_running {
            return Ok(StepRunOutcome::LongRunning);
        }

        let exit_status = loop {
            if *cancel_rx.borrow() {
                kill_children_async(&children).await;
                return Err("Execution cancelled".to_string());
            }

            let wait_result = {
                let mut procs = children.lock().unwrap();
                if let Some(child) = procs.last_mut() {
                    child.try_wait()
                } else {
                    return Err("Process handle lost".to_string());
                }
            };

            match wait_result {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(e) => return Err(format!("Failed to wait for process: {}", e)),
            }

            tokio::select! {
                changed = cancel_rx.changed() => {
                    if changed.is_ok() && *cancel_rx.borrow() {
                        kill_children_async(&children).await;
                        return Err("Execution cancelled".to_string());
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(200)) => {}
            }
        };

        {
            let mut procs = children.lock().unwrap();
            procs.pop();
        }

        let code = exit_status.code().unwrap_or(-1);
        Ok(StepRunOutcome::Completed {
            exit_code: code,
            success: exit_status.success(),
        })
    }

    async fn set_step_running(&self, execution_id: &str, step_id: &str) -> Result<(), String> {
        self.update_step_fields(execution_id, step_id, |step| {
            step.status = "running".to_string();
            if step.startedAt.is_empty() {
                step.startedAt = Utc::now().to_rfc3339();
            }
        })
        .await
    }

    async fn finalize_step(
        &self,
        execution_id: &str,
        step_id: &str,
        status: &str,
        exit_code: Option<i32>,
        error: Option<&str>,
    ) -> Result<(), String> {
        self.update_step_fields(execution_id, step_id, |step| {
            step.status = status.to_string();
            step.finishedAt = Some(Utc::now().to_rfc3339());
            if let (Ok(started), Some(finished)) = (
                chrono::DateTime::parse_from_rfc3339(&step.startedAt),
                step.finishedAt.as_ref(),
            ) {
                if let Ok(finished_dt) = chrono::DateTime::parse_from_rfc3339(finished) {
                    step.duration = Some((finished_dt - started).num_milliseconds());
                }
            }
            if let Some(code) = exit_code {
                step.exitCode = Some(code);
            }
            if let Some(err) = error {
                step.error = Some(err.to_string());
            }
        })
        .await
    }

    async fn update_step_fields<F>(
        &self,
        execution_id: &str,
        step_id: &str,
        update: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut StepExecutionState),
    {
        let execution = self
            .execution_repo
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| "Execution not found".to_string())?;

        let mut steps: Vec<StepExecutionState> =
            serde_json::from_str(&execution.step_executions_json).unwrap_or_default();

        for step in steps.iter_mut() {
            if step.stepId == step_id {
                update(step);
                break;
            }
        }

        let json = serde_json::to_string(&steps)
            .map_err(|e| format!("Failed to serialize step executions: {}", e))?;
        self.execution_repo
            .update_step_executions(execution_id, json)
            .await?;
        Ok(())
    }

    async fn mark_cancelled(&self, execution_id: &str, events: &Arc<dyn EventSink>) {
        let _ = self
            .execution_repo
            .update_status(execution_id, "cancelled".to_string(), None)
            .await;
        if let Ok(Some(exec)) = self.get_execution(execution_id).await {
            self.emit_execution_update(events, exec);
        }
    }

    fn emit_execution_update(&self, events: &Arc<dyn EventSink>, execution: Value) {
        events.emit("pipeline-execution-update", execution);
    }

    fn emit_step_log(
        events: &Arc<dyn EventSink>,
        execution_id: &str,
        step_id: &str,
        line: &str,
        stream: &str,
    ) {
        events.emit(
            "pipeline-step-log",
            json!({
                "executionId": execution_id,
                "stepId": step_id,
                "line": line,
                "stream": stream,
            }),
        );
    }

    fn execution_to_json(e: &crate::entities::pipeline_execution::Model) -> Value {
        json!({
            "id": e.id,
            "pipelineId": e.pipeline_id.to_string(),
            "projectId": e.project_id.to_string(),
            "status": e.status,
            "startedAt": e.started_at.to_rfc3339(),
            "finishedAt": e.finished_at.map(|d| d.to_rfc3339()),
            "triggeredBy": e.triggered_by,
            "stepExecutions": serde_json::from_str::<Value>(&e.step_executions_json).unwrap_or(json!([])),
            "variables": serde_json::from_str::<Value>(&e.variables_json).unwrap_or(json!({})),
            "error": e.error,
        })
    }

    async fn enrich_execution_list_item(
        &self,
        e: crate::entities::pipeline_execution::Model,
    ) -> Result<Value, String> {
        let mut item = Self::execution_to_json(&e);

        if let Some(pipeline) = self.pipeline_repo.get_by_id(e.pipeline_id).await? {
            if let Some(obj) = item.as_object_mut() {
                obj.insert("pipelineName".to_string(), json!(pipeline.name));
            }
        }

        if let Some(project) = self.project_repo.get_by_id(e.project_id).await? {
            if let Some(obj) = item.as_object_mut() {
                obj.insert("projectName".to_string(), json!(project.name));
            }
        }

        Ok(item)
    }

    /// Run a workflow from the project's `.portal/pipeline.yml`.
    ///
    /// The runner is keyed to a `pipelines` row, so the workflow is materialised
    /// as one (`preset_key = "release:<workflow>"`, rewritten on every run so the
    /// file stays the source of truth) and handed to `execute_pipeline`.
    /// `variables` reach each step as env vars as well as `${NAME}` substitutions.
    pub async fn execute_file_workflow(
        &self,
        project_id: i32,
        workflow_id: &str,
        mut variables: HashMap<String, String>,
        events: Arc<dyn EventSink>,
    ) -> Result<String, String> {
        let project = self
            .project_repo
            .get_by_id(project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        let file = pipeline_file::load(&project.path)?
            .ok_or_else(|| "No .portal/pipeline.yml in this project".to_string())?;

        if let Some(bump) = file.release.as_ref().and_then(|r| r.bump.clone()) {
            variables.entry("BUMP".to_string()).or_insert(bump);
        }
        let mut steps = pipeline_file::build_steps(&file, workflow_id)?;
        for step in &mut steps {
            if let Some(env) = step["config"]["env"].as_object_mut() {
                for (k, v) in &variables {
                    env.entry(k.clone()).or_insert_with(|| json!(v));
                }
            }
        }
        let steps_json = serde_json::to_string(&steps).map_err(|e| e.to_string())?;

        let key = format!("release:{workflow_id}");
        let name = format!("Release: {workflow_id}");
        let pipeline_id = match self.pipeline_repo.get_by_preset_key(project_id, &key).await? {
            Some(p) => {
                self.pipeline_repo
                    .update(p.id, Some(name), None, Some(steps_json), None, None, None, None, None, None)
                    .await?
                    .id
            }
            None => {
                self.pipeline_repo
                    .create(
                        name,
                        None,
                        project_id,
                        steps_json,
                        "[]".to_string(),
                        "[]".to_string(),
                        "{}".to_string(),
                        true,
                        Some(key),
                        Some("release".to_string()),
                    )
                    .await?
                    .id
            }
        };

        self.execute_pipeline(
            ExecutionRequestData {
                pipeline_id: pipeline_id.to_string(),
                variables: Some(variables),
                secrets: None,
            },
            events,
        )
        .await
    }

    /// The project's `release:` block with each workflow's resolved steps, or
    /// `None` when there is no pipeline file or it declares no release.
    pub async fn describe_release(&self, project_id: i32) -> Result<Option<Value>, String> {
        let project = self
            .project_repo
            .get_by_id(project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        let Some(file) = pipeline_file::load(&project.path)? else {
            return Ok(None);
        };
        let Some(release) = file.release.clone() else {
            return Ok(None);
        };
        let plan = |id: &str| -> Result<Value, String> {
            let steps = pipeline_file::build_steps(&file, id)?;
            Ok(json!({ "id": id, "steps": steps.iter().map(|s| json!({
                "id": s["id"], "name": s["name"], "dependsOn": s["dependsOn"],
                "command": s["config"]["command"], "cwd": s["config"]["cwd"],
            })).collect::<Vec<_>>() }))
        };
        Ok(Some(json!({
            "release": release,
            "ship": plan(&release.ship)?,
            "rehearse": release.rehearse.as_deref().map(plan).transpose()?,
        })))
    }

    /// Executions of the project's release workflows, newest first.
    pub async fn get_release_executions(
        &self,
        project_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<Value>, String> {
        let mut items = Vec::new();
        for p in self.pipeline_repo.get_all_by_project(project_id).await? {
            if p.preset_key.as_deref().is_some_and(|k| k.starts_with("release:")) {
                items.extend(self.get_executions_by_pipeline(p.id, limit).await?);
            }
        }
        items.sort_by(|a, b| b["startedAt"].as_str().cmp(&a["startedAt"].as_str()));
        items.truncate(limit.unwrap_or(20) as usize);
        Ok(items)
    }

    pub async fn get_execution(&self, execution_id: &str) -> Result<Option<Value>, String> {
        let execution = self.execution_repo.get_by_id(execution_id).await?;
        Ok(execution.map(|e| Self::execution_to_json(&e)))
    }

    pub async fn get_step_logs(
        &self,
        execution_id: &str,
        step_id: &str,
    ) -> Result<Vec<String>, String> {
        let execution = self
            .execution_repo
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| "Execution not found".to_string())?;

        let steps: Vec<StepExecutionState> =
            serde_json::from_str(&execution.step_executions_json).unwrap_or_default();

        Ok(steps
            .into_iter()
            .find(|s| s.stepId == step_id)
            .map(|s| s.logs)
            .unwrap_or_default())
    }

    pub async fn cancel_execution(
        &self,
        execution_id: &str,
        events: Option<Arc<dyn EventSink>>,
    ) -> Result<(), String> {
        let runtime_snapshot = {
            let guard = self.running.lock().unwrap();
            guard
                .get(execution_id)
                .map(|r| (r.cancel_tx.clone(), Arc::clone(&r.children)))
        };

        if let Some((cancel_tx, children)) = runtime_snapshot {
            let _ = cancel_tx.send(true);
            kill_children_async(&children).await;
        }

        self.finalize_running_steps(execution_id, "cancelled")
            .await?;

        self.execution_repo
            .update_status(execution_id, "cancelled".to_string(), None)
            .await?;

        if let Some(events) = events {
            if let Ok(Some(exec)) = self.get_execution(execution_id).await {
                self.emit_execution_update(&events, exec);
            }
        }

        Ok(())
    }

    async fn finalize_running_steps(&self, execution_id: &str, status: &str) -> Result<(), String> {
        let execution = self
            .execution_repo
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| "Execution not found".to_string())?;

        let mut steps: Vec<StepExecutionState> =
            serde_json::from_str(&execution.step_executions_json).unwrap_or_default();

        let mut changed = false;
        for step in steps.iter_mut() {
            if step.status == "running" {
                step.status = status.to_string();
                if step.startedAt.is_empty() {
                    step.startedAt = Utc::now().to_rfc3339();
                }
                step.finishedAt = Some(Utc::now().to_rfc3339());
                if let (Ok(started), Some(finished)) = (
                    chrono::DateTime::parse_from_rfc3339(&step.startedAt),
                    step.finishedAt.as_ref(),
                ) {
                    if let Ok(finished_dt) = chrono::DateTime::parse_from_rfc3339(finished) {
                        step.duration = Some((finished_dt - started).num_milliseconds());
                    }
                }
                changed = true;
            }
        }

        if changed {
            let json = serde_json::to_string(&steps)
                .map_err(|e| format!("Failed to serialize step executions: {}", e))?;
            self.execution_repo
                .update_step_executions(execution_id, json)
                .await?;
        }

        Ok(())
    }

    pub async fn get_executions_by_pipeline(
        &self,
        pipeline_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<Value>, String> {
        let executions = self
            .execution_repo
            .get_by_pipeline(pipeline_id, limit)
            .await?;
        let mut items = Vec::with_capacity(executions.len());
        for execution in executions {
            items.push(self.enrich_execution_list_item(execution).await?);
        }
        Ok(items)
    }

    pub async fn get_executions_by_project(
        &self,
        project_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<Value>, String> {
        let executions = self
            .execution_repo
            .get_by_project(project_id, limit)
            .await?;
        let mut items = Vec::with_capacity(executions.len());
        for execution in executions {
            items.push(self.enrich_execution_list_item(execution).await?);
        }
        Ok(items)
    }

    pub async fn get_all_executions(&self, limit: Option<u64>) -> Result<Vec<Value>, String> {
        let executions = self.execution_repo.get_all(limit).await?;
        let mut items = Vec::with_capacity(executions.len());
        for execution in executions {
            items.push(self.enrich_execution_list_item(execution).await?);
        }
        Ok(items)
    }
}

async fn append_step_log(
    repo: &ExecutionRepository,
    execution_id: &str,
    step_id: &str,
    line: &str,
    stream: &str,
    events: &Arc<dyn EventSink>,
) {
    ExecutionService::emit_step_log(events, execution_id, step_id, line, stream);

    if let Ok(Some(execution)) = repo.get_by_id(execution_id).await {
        let mut steps: Vec<StepExecutionState> =
            serde_json::from_str(&execution.step_executions_json).unwrap_or_default();

        let formatted = if stream == "stderr" {
            format!("[stderr] {}", line)
        } else {
            line.to_string()
        };

        for step in steps.iter_mut() {
            if step.stepId == step_id {
                step.logs.push(formatted.clone());
                step.output.push_str(&formatted);
                step.output.push('\n');
                break;
            }
        }

        if let Ok(json) = serde_json::to_string(&steps) {
            let _ = repo.update_step_executions(execution_id, json).await;
        }
    }
}

fn build_initial_step_executions(steps: &[Value]) -> Vec<StepExecutionState> {
    steps
        .iter()
        .filter_map(|step| {
            let id = step.get("id")?.as_str()?;
            let name = step.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            Some(StepExecutionState {
                id: Uuid::new_v4().to_string(),
                stepId: id.to_string(),
                stepName: name.to_string(),
                status: "pending".to_string(),
                startedAt: String::new(),
                finishedAt: None,
                output: String::new(),
                error: None,
                exitCode: None,
                duration: None,
                retryCount: 0,
                logs: Vec::new(),
            })
        })
        .collect()
}

fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("${{{}}}", key), value);
    }
    result
}

fn detect_package_manager_from_path(project_path: &str) -> String {
    let path = Path::new(project_path);

    if path.join("pnpm-lock.yaml").exists() {
        return "pnpm".to_string();
    }
    if path.join("yarn.lock").exists() {
        return "yarn".to_string();
    }
    if path.join("package-lock.json").exists() {
        return "npm".to_string();
    }

    if let Ok(contents) = std::fs::read_to_string(path.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<Value>(&contents) {
            if let Some(pm) = json.get("packageManager").and_then(|v| v.as_str()) {
                let name = pm.split('@').next().unwrap_or(pm).trim();
                if matches!(name, "npm" | "yarn" | "pnpm") {
                    return name.to_string();
                }
            }
        }
    }

    "npm".to_string()
}

fn format_package_script_command(pm: &str, script: &str) -> String {
    if pm == "npm" {
        format!("{} run {}", pm, script)
    } else {
        format!("{} {}", pm, script)
    }
}

fn normalize_package_manager_command(command: &str, detected_pm: &str) -> String {
    let trimmed = command.trim();
    let mut parts = trimmed.split_whitespace();
    let Some(first) = parts.next() else {
        return command.to_string();
    };

    if matches!(first, "npm" | "yarn" | "pnpm") && first != detected_pm {
        let rest = parts.collect::<Vec<_>>().join(" ");
        if rest.is_empty() {
            detected_pm.to_string()
        } else {
            format!("{} {}", detected_pm, rest)
        }
    } else {
        command.to_string()
    }
}

#[allow(dead_code)]
fn is_install_step(command_template: &str) -> bool {
    let lower = command_template.to_lowercase();
    lower.contains("install") && !lower.contains("run install")
}

#[allow(dead_code)]
fn is_build_step(command_template: &str, step_name: &str) -> bool {
    let lower = command_template.to_lowercase();
    let name_lower = step_name.to_lowercase();
    lower.contains("run build") || lower.ends_with(" build") || name_lower.contains("build")
}

#[allow(dead_code)]
fn is_dev_step(command_template: &str, step_name: &str) -> bool {
    let lower = command_template.to_lowercase();
    let name_lower = step_name.to_lowercase();
    (lower.contains("run dev") || lower.ends_with(" dev") || name_lower.contains("dev server"))
        && !lower.contains("install")
}

/// Normalize package-manager script commands, respecting pnpm/yarn shorthand.
#[allow(dead_code)]
fn normalize_node_script_command(command: &str, detected_pm: &str, script: Option<&str>) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return if let Some(script) = script {
            format_package_script_command(detected_pm, script)
        } else {
            trimmed.to_string()
        };
    }

    let mut parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return trimmed.to_string();
    }

    if let Some(script) = script {
        if parts.len() >= 3
            && matches!(parts[0], "npm" | "yarn" | "pnpm")
            && parts[1] == "run"
            && parts[2] == script
        {
            parts[0] = detected_pm;
            return parts.join(" ");
        }

        if parts.len() >= 2 && matches!(parts[0], "npm" | "yarn" | "pnpm") && parts[1] == script {
            parts[0] = detected_pm;
            return parts.join(" ");
        }

        if !matches!(parts[0], "npm" | "yarn" | "pnpm") {
            return format_package_script_command(detected_pm, script);
        }
    } else if parts.len() >= 3 && matches!(parts[0], "npm" | "yarn" | "pnpm") && parts[1] == "run" {
        parts[0] = detected_pm;
        return parts.join(" ");
    } else if parts.len() >= 2 && matches!(parts[0], "npm" | "yarn" | "pnpm") {
        let known_cmds = [
            "run", "install", "ci", "add", "remove", "exec", "dlx", "create", "init", "test",
        ];
        if !known_cmds.contains(&parts[1]) {
            if detected_pm == "npm" {
                return format!("{} run {}", detected_pm, parts[1..].join(" "));
            }
            parts[0] = detected_pm;
            return parts.join(" ");
        }
    }

    normalize_package_manager_command(trimmed, detected_pm)
}

fn resolve_package_manager(project: &ProjectResponse) -> String {
    detect_package_manager_from_path(&project.path)
}

/// Kill spawned processes and their child tree (e.g. node under cmd on Windows).
async fn kill_children_async(children: &Arc<Mutex<Vec<Child>>>) {
    let mut child_handles: Vec<Child> = {
        let mut procs = children.lock().unwrap();
        std::mem::take(&mut *procs)
    };

    for mut child in child_handles.drain(..) {
        kill_process_tree(&mut child).await;
    }
}

async fn kill_process_tree(child: &mut Child) {
    if let Some(pid) = child.id() {
        if cfg!(target_os = "windows") {
            // cmd /C npm run dev spawns node as a child — taskkill /T terminates the tree
            let _ = Command::new("taskkill")
                .no_window()
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await;
        } else if cfg!(unix) {
            // Terminate child processes first (e.g. node under sh -c)
            let _ = Command::new("pkill")
                .args(["-TERM", "-P", &pid.to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await;
            let _ = child.start_kill();
        } else {
            let _ = child.start_kill();
        }
    } else {
        let _ = child.start_kill();
    }

    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), child.wait()).await;
}
