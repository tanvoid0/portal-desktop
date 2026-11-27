use crate::database::DatabaseManager;
use crate::domains::projects::entities::ProjectResponse;
use crate::domains::projects::move_lock;
pub use crate::domains::projects::pipelines::repositories::ActionMeta;
use crate::domains::projects::pipelines::repositories::{
    ExecutionRepository, NewRun, PipelineRepository,
};
use crate::domains::projects::pipelines::services::pipeline_file;
use crate::domains::projects::pipelines::services::pipeline_service::{
    is_release_key, RELEASE_ROW_REFUSAL,
};
use crate::domains::projects::pipelines::services::progress::{
    compute_progress, expected_durations, parse_progress_line, HistoryRun, ProgressStep,
    StepProgress,
};
use crate::domains::projects::pipelines::utils::dependency_resolver::resolve_execution_order;
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use crate::events::{EventSink, EventSinkExt};
use crate::log_warn;
use crate::process_ext::NoWindowExt;
use crate::utils::pnpm_workspace::{prepare_shell_command, warn_if_broken_pnpm_workspace};
use crate::utils::shell_template;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// How long a finished step's log readers may go without reading a line before
/// [`drain_log_readers`] stops waiting for them.
const READER_IDLE_GRACE: Duration = Duration::from_secs(3);
/// The most [`drain_log_readers`] waits in all, however busy the readers are.
const READER_DRAIN_CAP: Duration = Duration::from_secs(30);
/// A [`LogReader`] writes its pending lines to the DB when this many have
/// piled up, or when the oldest has waited [`LOG_FLUSH_EVERY`]. Each write
/// rewrites the whole step blob, so one write per line was the hot spot.
const LOG_FLUSH_LINES: usize = 64;
const LOG_FLUSH_EVERY: Duration = Duration::from_millis(250);

/// A non-long-running step with no explicit `timeout` is killed after this many
/// seconds, so a process that finishes its work but never exits (e.g.
/// svelte-check not releasing its `cmd` wrapper on Windows) cannot wedge the
/// pipeline forever.
/// ponytail: blanket 1h ceiling; set a step's `timeout` to raise it for a slow build.
const DEFAULT_STEP_TIMEOUT_SECS: u64 = 3600;

/// Read each `foreach: { items_file }` list for a workflow, relative to the
/// project root (blank lines and `#` comments ignored), keyed by the step's
/// action name — what [`pipeline_file::build_steps`] needs to fan a step out.
/// Inline `items` need no run context and are expanded by `build_steps` directly.
/// `strict` (a real run) fails on a list file that is missing, unreadable, empty or
/// repeats an item; when false (describe/preview) the step stays a single placeholder.
/// An empty fan-out would silently drop every `needs` pointing at the step.
fn resolve_foreach_items(
    project_path: &str,
    file: &pipeline_file::PipelineFile,
    workflow_id: &str,
    strict: bool,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut resolved: HashMap<String, Vec<String>> = HashMap::new();
    let Some(wf) = file.workflows.get(workflow_id) else {
        return Ok(resolved);
    };
    for step in &wf.steps {
        let Some(fe) = &step.foreach else { continue };
        if !fe.items.is_empty() {
            continue;
        }
        let Some(rel) = &fe.items_file else { continue };
        let path = Path::new(project_path).join(rel);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let items: Vec<String> = text
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(|l| l.to_string())
                    .collect();
                let problem = match pipeline_file::first_duplicate(&items) {
                    Some(dup) => Some(format!("repeats item '{dup}'")),
                    None => items.is_empty().then(|| "lists no items".to_string()),
                };
                match problem {
                    None => {
                        resolved.insert(step.action.clone(), items);
                    }
                    Some(p) if strict => {
                        return Err(format!("foreach items_file {}: {p}", path.display()))
                    }
                    Some(_) => {}
                }
            }
            Err(e) if strict => {
                return Err(format!("foreach items_file {}: {e}", path.display()))
            }
            Err(_) => {}
        }
    }
    Ok(resolved)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequestData {
    pub pipeline_id: String,
    pub variables: Option<HashMap<String, String>>,
    pub secrets: Option<HashMap<String, String>>,
    /// Who asked for the run, e.g. "mcp:<client>". Absent means "user".
    #[serde(default)]
    pub triggered_by: Option<String>,
}

/// What [`ExecutionService::execute_steps`] runs. `execute_pipeline` fills it
/// from a pipeline and its project; an Action run (one step, `action: Some`)
/// fills it from the request.
#[derive(Default)]
pub struct RunSpec {
    pub pipeline_id: Option<i32>,
    pub project_id: Option<i32>,
    /// Where the steps run: the project path, or an action's working directory
    /// (empty = the app's own). Also what the relocation in-use check compares.
    pub cwd: String,
    pub steps: Vec<Value>,
    /// `${NAME}` substitutions for the steps' commands, stored on the row.
    pub variables: HashMap<String, String>,
    /// Extra environment for every step, set as given (no `$` expansion).
    pub env: HashMap<String, String>,
    pub triggered_by: String,
    /// An action run has no deadline unless this is set. Pipeline steps ignore
    /// it: each gets its own `timeout`, or [`DEFAULT_STEP_TIMEOUT_SECS`].
    pub timeout: Option<Duration>,
    /// Rewrite `npm`/`yarn`/`pnpm` commands to this package manager; `None` leaves them alone.
    pub detected_pm: Option<String>,
    /// Median ms of each step's recent successful runs, by step id, for the ETA.
    pub expected: HashMap<String, i64>,
    /// `Some` makes this a `kind = 'action'` run: no `$ cmd` / `cwd:` header, no
    /// `[stderr] ` prefix (the log is exactly the process output), the pid is
    /// recorded, and a failure's error is [`action_failure_message`].
    pub action: Option<ActionMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StepExecutionState {
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
    /// Median duration (ms) of this step's recent successful runs, fixed at run start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expectedMs: Option<i64>,
    /// Latest `::portal-progress` line the step printed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<StepProgress>,
    #[serde(default)]
    longRunning: bool,
}

enum StepRunOutcome {
    Completed { exit_code: i32, success: bool },
    LongRunning,
}

struct RunningExecution {
    cancel_tx: watch::Sender<bool>,
    children: Arc<Mutex<Vec<Child>>>,
    /// The project path (or an action's working directory) this run executes
    /// against — for the relocation in-use check (`relocation::usage::running_under`).
    project_path: String,
    action: bool,
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

    /// True if any pipeline execution currently running is against a project
    /// at or under `path`. Used by the project-relocation in-use check.
    pub fn running_under(&self, path: &str) -> bool {
        self.running
            .lock()
            .unwrap()
            .values()
            .any(|r| !r.action && move_lock::is_under(&r.project_path, path))
    }

    /// True if any action run's working directory is inside `path`. One-way
    /// (`is_within`), so an action started from the home directory does not
    /// block every project move (board #431). Apart from [`Self::running_under`]
    /// so the in-use check reports a pipeline and a script as what they are.
    pub fn running_actions_under(&self, path: &str) -> bool {
        self.running
            .lock()
            .unwrap()
            .values()
            .any(|r| r.action && move_lock::is_within(&r.project_path, path))
    }

    /// Whether this process is driving `execution_id` right now.
    pub fn is_running(&self, execution_id: &str) -> bool {
        self.running.lock().unwrap().contains_key(execution_id)
    }

    /// Run a saved pipeline by id. Reachable over `/rpc`, so it refuses release
    /// rows: shipping is `local_only` (`release_ship`), and a row left over from
    /// a ship still has `RELEASE_DRY_RUN=0` baked into its steps.
    pub async fn execute_pipeline(
        &self,
        request: ExecutionRequestData,
        events: Arc<dyn EventSink>,
    ) -> Result<String, String> {
        self.run_pipeline_row(request, events, false).await
    }

    async fn run_pipeline_row(
        &self,
        request: ExecutionRequestData,
        events: Arc<dyn EventSink>,
        release: bool,
    ) -> Result<String, String> {
        let pipeline_id = request
            .pipeline_id
            .parse::<i32>()
            .map_err(|_| "Invalid pipeline ID".to_string())?;

        let pipeline = self
            .pipeline_repo
            .get_by_id(pipeline_id)
            .await?
            .ok_or_else(|| "Pipeline not found".to_string())?;
        if !release && is_release_key(pipeline.preset_key.as_deref()) {
            return Err(RELEASE_ROW_REFUSAL.to_string());
        }

        let project = self
            .project_repo
            .get_by_id(pipeline.project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        move_lock::ensure_unlocked(Path::new(&project.path))?;

        let steps: Vec<Value> = serde_json::from_str(&pipeline.steps_json).unwrap_or_default();
        reject_steps_after_long_running(&steps)?;

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

        let step_ids: Vec<String> = steps
            .iter()
            .filter_map(|s| s.get("id")?.as_str().map(String::from))
            .collect();
        let history: Vec<HistoryRun> = self
            .execution_repo
            .get_by_pipeline(pipeline_id, Some(20))
            .await?
            .iter()
            .map(|e| {
                serde_json::from_str::<Vec<StepExecutionState>>(&e.step_executions_json)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| (s.stepId, s.status, s.duration))
                    .collect()
            })
            .collect();
        let expected = expected_durations(&history, &step_ids);

        self.execute_steps(
            RunSpec {
                pipeline_id: Some(pipeline_id),
                project_id: Some(pipeline.project_id),
                cwd: project.path,
                steps,
                variables,
                triggered_by: request.triggered_by.unwrap_or_else(|| "user".to_string()),
                detected_pm: Some(detected_pm),
                expected,
                ..Default::default()
            },
            events,
        )
        .await
    }

    /// Retry a failed run in place: rerun its non-`success` steps — the failed node
    /// and everything after it — on the same execution row, keeping the steps that
    /// already succeeded. Reloads the pipeline's current steps and reuses the run's
    /// stored variables, so a release retry keeps `RELEASE_DRY_RUN` (a rehearsal
    /// stays a rehearsal, a ship stays a ship). The command is `local_only`: this
    /// can re-trigger a production ship.
    pub async fn retry_execution(
        &self,
        execution_id: &str,
        events: Arc<dyn EventSink>,
    ) -> Result<Value, String> {
        // The driver writes `failed` a moment before it frees its run slot, so an
        // immediate Retry gives a failed run's driver up to 2s to leave.
        let mut waits = 0;
        let exec = loop {
            // Slot before row: a row read first could predate the driver leaving.
            let running = self.is_running(execution_id);
            let exec = self
                .execution_repo
                .get_by_id(execution_id)
                .await?
                .ok_or_else(|| "Execution not found".to_string())?;
            if !running {
                break exec;
            }
            if exec.status != "failed" || waits == 100 {
                return Err("This run is still going; stop it before retrying.".to_string());
            }
            waits += 1;
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        if exec.status != "failed" {
            return Err(format!(
                "Only a failed run can be retried (this one is {}).",
                exec.status
            ));
        }
        let pipeline_id = exec
            .pipeline_id
            .ok_or_else(|| "This run has no pipeline to retry".to_string())?;
        let pipeline = self
            .pipeline_repo
            .get_by_id(pipeline_id)
            .await?
            .ok_or_else(|| "The pipeline for this run no longer exists".to_string())?;
        let project = self
            .project_repo
            .get_by_id(pipeline.project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        move_lock::ensure_unlocked(Path::new(&project.path))?;

        let mut steps: Vec<Value> = serde_json::from_str(&pipeline.steps_json).unwrap_or_default();
        reject_steps_after_long_running(&steps)?;
        let variables: HashMap<String, String> =
            serde_json::from_str(&exec.variables_json).unwrap_or_default();
        let detected_pm = resolve_package_manager(&project);

        // Re-apply this run's variables onto each step's `env`, overwriting any value
        // the shared pipeline row still carries from a later run of the same workflow
        // id. This keeps the retry's own `RELEASE_DRY_RUN` (a ship retry stays a ship,
        // a rehearsal stays a rehearsal) instead of whatever ran last. Mirrors the
        // baking in `execute_file_workflow`, but authoritative — overwrite, not fill.
        for step in &mut steps {
            if let Some(env) = step["config"]["env"].as_object_mut() {
                for (k, v) in &variables {
                    env.insert(k.clone(), json!(v));
                }
            }
        }

        // The ETA bars read the same history a first run would.
        let step_ids: Vec<String> = steps
            .iter()
            .filter_map(|s| s.get("id")?.as_str().map(String::from))
            .collect();
        let history: Vec<HistoryRun> = self
            .execution_repo
            .get_by_pipeline(pipeline_id, Some(20))
            .await?
            .iter()
            .map(|e| {
                serde_json::from_str::<Vec<StepExecutionState>>(&e.step_executions_json)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| (s.stepId, s.status, s.duration))
                    .collect()
            })
            .collect();
        let expected = expected_durations(&history, &step_ids);

        // Rebuild the step states from the pipeline as it is now, carrying over any
        // step that already succeeded (by id); everything else restarts `pending`.
        // Building from the current steps also absorbs steps added or removed since
        // the failed run.
        let prev: HashMap<String, StepExecutionState> =
            serde_json::from_str::<Vec<StepExecutionState>>(&exec.step_executions_json)
                .unwrap_or_default()
                .into_iter()
                .map(|s| (s.stepId.clone(), s))
                .collect();
        let mut states = build_initial_step_executions(&steps, &expected);
        for st in &mut states {
            match prev.get(&st.stepId) {
                // A step that already succeeded is carried over as-is (and skipped by
                // `run_execution`); anything else restarts `pending` with its retry
                // count bumped. A step new to the pipeline since the failure stays 0.
                Some(p) if p.status == "success" => *st = p.clone(),
                Some(p) => st.retryCount = p.retryCount + 1,
                None => {}
            }
        }
        let states_json = serde_json::to_string(&states)
            .map_err(|e| format!("Failed to serialize steps: {}", e))?;

        // Reserve the run slot before the writes below, so two concurrent retries of
        // the same run cannot both drive it (which would, for a release, ship twice).
        // The `is_running` wait at the top is only a courtesy; this is the real guard.
        let Some((cancel_rx, children)) =
            self.reserve_run(execution_id, project.path.clone(), false)
        else {
            return Err("This run is still going; stop it before retrying.".to_string());
        };

        if let Err(e) = self
            .execution_repo
            .update_step_executions(execution_id, states_json)
            .await
        {
            self.running.lock().unwrap().remove(execution_id);
            return Err(e);
        }
        if let Err(e) = self.execution_repo.reopen(execution_id).await {
            self.running.lock().unwrap().remove(execution_id);
            return Err(e);
        }

        self.drive_run(
            execution_id.to_string(),
            RunSpec {
                pipeline_id: Some(pipeline_id),
                project_id: Some(pipeline.project_id),
                cwd: project.path,
                steps,
                variables,
                triggered_by: exec.triggered_by.clone(),
                detected_pm: Some(detected_pm),
                expected,
                ..Default::default()
            },
            events,
            cancel_rx,
            children,
        );

        self.get_execution(execution_id)
            .await?
            .ok_or_else(|| "Execution not found after retry".to_string())
    }

    /// The ETA for an action's step: the median of its own recent successful runs
    /// (see [`ExecutionRepository::get_action_history`]), for `RunSpec::expected`.
    /// An action is one step whatever a workflow names it, so every one-step run in
    /// the history answers for `step_id`; a run of any other length is not an action's.
    /// Best effort: a history that cannot be read is no history, never a failed run.
    pub async fn action_expected(&self, meta: &ActionMeta, step_id: &str) -> HashMap<String, i64> {
        let rows = self
            .execution_repo
            .get_action_history(
                meta.block_id.as_deref(),
                &meta.command,
                meta.working_directory.as_deref(),
                20,
            )
            .await
            .unwrap_or_default();
        let history: Vec<HistoryRun> = rows
            .iter()
            .filter_map(|e| {
                let steps =
                    serde_json::from_str::<Vec<StepExecutionState>>(&e.step_executions_json)
                        .ok()?;
                let [s] = steps.as_slice() else { return None };
                // A one-step run lasts as long as its step: a row copied from
                // `script_executions` may carry no step `duration`, only its own span.
                let span = e.finished_at.map(|f| (f - e.started_at).num_milliseconds());
                Some(vec![(
                    step_id.to_string(),
                    s.status.clone(),
                    s.duration.or(span),
                )])
            })
            .collect();
        expected_durations(&history, &[step_id.to_string()])
    }

    /// Store `spec` as a run, register it so Stop can find it, and drive it on a
    /// background task. Returns the execution id as soon as the run is queued.
    /// Pipelines and Actions both come through here.
    pub async fn execute_steps(
        &self,
        spec: RunSpec,
        events: Arc<dyn EventSink>,
    ) -> Result<String, String> {
        // `execute_pipeline` checks its project; an action's folder is checked
        // here, so no caller can start one inside a project being moved.
        if spec.action.is_some() && !spec.cwd.is_empty() {
            move_lock::ensure_unlocked_within(Path::new(&spec.cwd))?;
        }
        let execution_id = Uuid::new_v4().to_string();
        let step_executions = build_initial_step_executions(&spec.steps, &spec.expected);
        let step_executions_json = serde_json::to_string(&step_executions)
            .map_err(|e| format!("Failed to serialize steps: {}", e))?;
        let variables_json = serde_json::to_string(&spec.variables)
            .map_err(|e| format!("Failed to serialize variables: {}", e))?;

        self.execution_repo
            .insert(NewRun {
                id: execution_id.clone(),
                pipeline_id: spec.pipeline_id,
                project_id: spec.project_id,
                status: "pending".to_string(),
                triggered_by: spec.triggered_by.clone(),
                step_executions_json,
                variables_json,
                action: spec.action.clone(),
                ..Default::default()
            })
            .await?;

        let Some((cancel_rx, children)) =
            self.reserve_run(&execution_id, spec.cwd.clone(), spec.action.is_some())
        else {
            // A freshly minted id cannot already be running.
            return Err("Execution already running".to_string());
        };

        if let Err(e) = self
            .execution_repo
            .update_status(&execution_id, "queued".to_string(), None)
            .await
        {
            // Nothing will drive this run: unregister it and close the row.
            self.running.lock().unwrap().remove(&execution_id);
            let _ = self
                .execution_repo
                .update_status(&execution_id, "failed".to_string(), Some(e.clone()))
                .await;
            return Err(e);
        }

        self.drive_run(execution_id.clone(), spec, events, cancel_rx, children);
        Ok(execution_id)
    }

    /// Reserve the run slot for `execution_id` so exactly one task drives it, and
    /// hand back the cancel receiver and child-process handle. `None` when a task is
    /// already driving this id — the check-and-insert is atomic under the lock, so it
    /// is the real guard against two drivers of one run (a fresh run mints a unique
    /// id and never collides; a retry reuses the id and could).
    fn reserve_run(
        &self,
        execution_id: &str,
        project_path: String,
        action: bool,
    ) -> Option<(watch::Receiver<bool>, Arc<Mutex<Vec<Child>>>)> {
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let children: Arc<Mutex<Vec<Child>>> = Arc::new(Mutex::new(Vec::new()));
        let mut running = self.running.lock().unwrap();
        if running.contains_key(execution_id) {
            return None;
        }
        running.insert(
            execution_id.to_string(),
            RunningExecution {
                cancel_tx,
                children: Arc::clone(&children),
                project_path,
                action,
            },
        );
        Some((cancel_rx, children))
    }

    /// Drive a reserved `queued` run on a background task; `run_execution` marks it
    /// `running`. Shared by a first run ([`Self::execute_steps`]) and a retry
    /// ([`Self::retry_execution`]); both reserve the slot with [`Self::reserve_run`]
    /// first.
    fn drive_run(
        &self,
        execution_id: String,
        spec: RunSpec,
        events: Arc<dyn EventSink>,
        cancel_rx: watch::Receiver<bool>,
        children: Arc<Mutex<Vec<Child>>>,
    ) {
        let service = self.clone();
        let exec_id = execution_id;
        let events_handle = events.clone();

        tokio::spawn(async move {
            let result = service
                .run_execution(
                    exec_id.clone(),
                    spec,
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
    }

    async fn run_execution(
        &self,
        execution_id: String,
        spec: RunSpec,
        children: Arc<Mutex<Vec<Child>>>,
        mut cancel_rx: watch::Receiver<bool>,
        events: Arc<dyn EventSink>,
    ) -> Result<(), String> {
        let is_action = spec.action.is_some();
        let RunSpec {
            steps,
            cwd,
            variables,
            env,
            timeout,
            detected_pm,
            ..
        } = spec;

        self.execution_repo
            .update_status(&execution_id, "running".to_string(), None)
            .await?;

        if let Ok(Some(exec)) = self.get_execution(&execution_id).await {
            self.emit_execution_update(&events, exec);
        }

        // Steps already `success` on this row — which a retry preserves — are
        // skipped below, so a retry reruns only the failed node and its downstream.
        // Empty on a first run, where every step is still `pending`.
        let already_done: std::collections::HashSet<String> = self
            .execution_repo
            .get_by_id(&execution_id)
            .await?
            .map(|e| {
                serde_json::from_str::<Vec<StepExecutionState>>(&e.step_executions_json)
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.status == "success")
            .map(|s| s.stepId)
            .collect();

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

                if already_done.contains(&step_id) {
                    continue;
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
                // An action has no per-step timeout: the run's, or none at all.
                let step_timeout = if is_action {
                    timeout
                } else {
                    Some(Duration::from_secs(resolve_step_timeout(step)))
                };

                // Values are quoted for the shell (a folder named `a & calc`
                // stays one argument), in one pass.
                let mut command = shell_template::render(&command_template, &variables);

                // Honor the resolved command from the Actions catalog / FE.
                // Only normalize the package-manager binary name — do not replace
                // build/dev commands with project metadata heuristics.
                if let Some(pm) = &detected_pm {
                    command = normalize_package_manager_command(&command, pm);
                }

                // Optional per-step `cwd` (relative to the project) and `env`, from
                // `.portal/pipeline.yml`. Absent for steps built by the UI.
                let step_dir = match config.get("cwd").and_then(|v| v.as_str()) {
                    Some(rel) => Path::new(&cwd).join(rel).to_string_lossy().into_owned(),
                    None => cwd.clone(),
                };
                let mut step_env: HashMap<String, String> = config
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|o| {
                        o.iter()
                            .filter_map(|(k, v)| {
                                v.as_str()
                                    .map(|s| (k.clone(), shell_template::expand_raw(s, &variables)))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                step_env.extend(env.clone());

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
                        step_timeout,
                        is_action,
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
                            let error = if is_action {
                                let logs = self
                                    .get_step_logs(&execution_id, &step_id)
                                    .await
                                    .unwrap_or_default();
                                let output: String =
                                    logs.iter().map(|l| format!("{l}\n")).collect();
                                action_failure_message(Some(exit_code), &output)
                            } else {
                                format!("Step '{}' failed with exit code {}", step_name, exit_code)
                            };
                            self.execution_repo
                                .update_status(&execution_id, "failed".to_string(), Some(error))
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
        timeout: Option<Duration>,
        action: bool,
        children: Arc<Mutex<Vec<Child>>>,
        cancel_rx: &mut watch::Receiver<bool>,
        events: &Arc<dyn EventSink>,
    ) -> Result<StepRunOutcome, String> {
        // An action may have no working directory: it then runs in the app's own.
        let exec_command = if working_directory.is_empty() {
            command.to_string()
        } else {
            prepare_shell_command(command, working_directory)
        };

        // An action's log is exactly what the process printed: no header, no warnings.
        if !action {
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
        }

        let pm_prefix = exec_command.split_whitespace().next();
        if !action
            && matches!(pm_prefix, Some("npm" | "yarn" | "pnpm"))
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

        if !action && pm_prefix == Some("pnpm") {
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

        let mut cmd = shell_template::shell_command(&exec_command);
        if !working_directory.is_empty() {
            cmd.current_dir(working_directory);
        }
        cmd.envs(env);
        // A GUI parent has no usable stdin; children (flutter, gradle) that read
        // it exit non-zero. Give them an empty one, like a shell's </dev/null.
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);
        // Own process group, so Stop can kill everything the shell started.
        #[cfg(unix)]
        cmd.process_group(0);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        if let (true, Some(pid)) = (action, child.id()) {
            let _ = self
                .execution_repo
                .update_pid(execution_id, pid as i32)
                .await;
        }

        {
            let mut procs = children.lock().unwrap();
            procs.push(child);
        }

        let lines_read = Arc::new(AtomicUsize::new(0));
        let mut readers = Vec::new();
        let reader = |stream| LogReader {
            repo: self.execution_repo.clone(),
            execution_id: execution_id.to_string(),
            step_id: step_id.to_string(),
            stream,
            events: events.clone(),
            lines_read: Arc::clone(&lines_read),
            raw: action,
        };
        if let Some(stdout) = stdout {
            readers.push(reader("stdout").spawn(stdout));
        }
        if let Some(stderr) = stderr {
            readers.push(reader("stderr").spawn(stderr));
        }

        if long_running {
            return Ok(StepRunOutcome::LongRunning);
        }

        let deadline = timeout.map(|t| (std::time::Instant::now() + t, t));

        let exit_status = loop {
            if *cancel_rx.borrow() {
                kill_children_async(&children).await;
                return Err("Execution cancelled".to_string());
            }

            // A command that runs its work but never exits (e.g. svelte-check not
            // releasing its cmd wrapper on Windows) would otherwise spin here
            // forever; the deadline kills its process tree and fails the step.
            if let Some((at, limit)) = deadline {
                if std::time::Instant::now() >= at {
                    kill_children_async(&children).await;
                    return Err(format!("Step timed out after {}s", limit.as_secs()));
                }
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

        drain_log_readers(readers, &lines_read, cancel_rx).await;
        // A cancel that lands while the readers drain must not be overwritten
        // by this step's "success".
        if *cancel_rx.borrow() {
            return Err("Execution cancelled".to_string());
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
            // A Stop that landed first is final: a late success/failed must not undo it.
            if step.status == "cancelled" && status != "cancelled" {
                return;
            }
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
        modify_steps(&self.execution_repo, execution_id, |steps| {
            if let Some(step) = steps.iter_mut().find(|s| s.stepId == step_id) {
                update(step);
            }
            true
        })
        .await
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
        events.emit("pipeline-execution-update", tick_payload(execution));
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

    pub(super) fn execution_to_json(e: &crate::entities::pipeline_execution::Model) -> Value {
        let steps: Vec<ProgressStep> =
            serde_json::from_str::<Vec<StepExecutionState>>(&e.step_executions_json)
                .unwrap_or_default()
                .into_iter()
                .map(|s| ProgressStep {
                    started_at: chrono::DateTime::parse_from_rfc3339(&s.startedAt)
                        .ok()
                        .map(|t| t.with_timezone(&Utc)),
                    step_id: s.stepId,
                    name: s.stepName,
                    status: s.status,
                    long_running: s.longRunning,
                    expected_ms: s.expectedMs,
                    progress: s.progress,
                    duration_ms: s.duration,
                })
                .collect();
        let mut item = json!({
            "progress": compute_progress(&steps, &e.status, Utc::now()),
            "id": e.id,
            "pipelineId": e.pipeline_id.map(|i| i.to_string()),
            "projectId": e.project_id.map(|i| i.to_string()),
            "status": e.status,
            "startedAt": e.started_at.to_rfc3339(),
            "finishedAt": e.finished_at.map(|d| d.to_rfc3339()),
            "triggeredBy": e.triggered_by,
            "stepExecutions": serde_json::from_str::<Value>(&e.step_executions_json).unwrap_or(json!([])),
            "variables": serde_json::from_str::<Value>(&e.variables_json).unwrap_or(json!({})),
            "error": e.error,
            "kind": e.kind,
        });
        if e.kind == "action" {
            item["command"] = json!(e.command);
            item["workingDirectory"] = json!(e.working_directory);
            item["blockId"] = json!(e.block_id);
            item["workflowRunId"] = json!(e.workflow_run_id);
            item["workflowId"] = json!(e.workflow_id);
            item["stepId"] = json!(e.step_id);
            item["workflowPlan"] = json!(e
                .workflow_plan_json
                .as_deref()
                .and_then(|j| serde_json::from_str::<Value>(j).ok()));
        }
        item
    }

    /// Store a run that already happened elsewhere (the embedded terminal, a
    /// workflow step that spawns nothing) as a finished action row: one
    /// finished step, no output. `started_at` backdates it so the duration is real.
    pub async fn record_finished_action(
        &self,
        action: ActionMeta,
        started_at: Option<chrono::DateTime<Utc>>,
        status: String,
        exit_code: Option<i32>,
        error: Option<String>,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let started = started_at.unwrap_or_else(Utc::now);
        let finished = Utc::now();
        let step = StepExecutionState {
            id: id.clone(),
            stepId: action.step_id.clone().unwrap_or_else(|| "run".to_string()),
            stepName: action.command.clone(),
            status: status.clone(),
            startedAt: started.to_rfc3339(),
            finishedAt: Some(finished.to_rfc3339()),
            output: String::new(),
            error: error.clone(),
            exitCode: exit_code,
            duration: Some((finished - started).num_milliseconds()),
            retryCount: 0,
            logs: Vec::new(),
            expectedMs: None,
            progress: None,
            longRunning: false,
        };
        self.execution_repo
            .insert(NewRun {
                id: id.clone(),
                status,
                triggered_by: "user".to_string(),
                step_executions_json: serde_json::to_string(&[step])
                    .map_err(|e| format!("Failed to serialize steps: {}", e))?,
                variables_json: "{}".to_string(),
                started_at: Some(started),
                finished_at: Some(finished),
                error,
                action: Some(action),
                ..Default::default()
            })
            .await?;
        Ok(id)
    }

    async fn enrich_execution_list_item(
        &self,
        e: crate::entities::pipeline_execution::Model,
    ) -> Result<Value, String> {
        let mut item = Self::execution_to_json(&e);

        if let Some(pipeline) = match e.pipeline_id {
            Some(id) => self.pipeline_repo.get_by_id(id).await?,
            None => None,
        } {
            if let Some(obj) = item.as_object_mut() {
                obj.insert("pipelineName".to_string(), json!(pipeline.name));
            }
        }

        if let Some(project) = match e.project_id {
            Some(id) => self.project_repo.get_by_id(id).await?,
            None => None,
        } {
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
    /// file stays the source of truth) and handed to the same runner as
    /// `execute_pipeline`.
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
        move_lock::ensure_unlocked(Path::new(&project.path))?;
        let file = pipeline_file::load(&project.path)?
            .ok_or_else(|| "No .portal/pipeline.yml in this project".to_string())?;

        if let Some(bump) = file.release.as_ref().and_then(|r| r.bump.clone()) {
            variables.entry("BUMP".to_string()).or_insert(bump);
        }

        // Resolve `foreach: { items_file }` lists now — read natively, no shell.
        // A missing list file is a hard error on a real run (strict).
        let resolved = resolve_foreach_items(&project.path, &file, workflow_id, true)?;

        let mut steps = pipeline_file::build_steps(&file, workflow_id, &resolved)?;
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
        let pipeline_id = match self
            .pipeline_repo
            .get_by_preset_key(project_id, &key)
            .await?
        {
            Some(p) => {
                self.pipeline_repo
                    .update(
                        p.id,
                        Some(name),
                        None,
                        Some(steps_json),
                        // Reset with the steps: nothing outside the file may
                        // feed a ship.
                        Some("[]".to_string()),
                        Some("[]".to_string()),
                        Some("{}".to_string()),
                        None,
                        None,
                        None,
                    )
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

        self.run_pipeline_row(
            ExecutionRequestData {
                pipeline_id: pipeline_id.to_string(),
                variables: Some(variables),
                secrets: None,
                triggered_by: None,
            },
            events,
            true,
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
            // Resolve `items_file` fan-outs so the plan shows one node per app
            // (the launcher sources its app subset from these). Lenient: a missing
            // list file leaves that step a single placeholder rather than failing.
            let resolved = resolve_foreach_items(&project.path, &file, id, false)?;
            let steps = pipeline_file::build_steps(&file, id, &resolved)?;
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
            if is_release_key(p.preset_key.as_deref()) {
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
        // A finished run has nothing left to stop: its status and times stay as they are.
        if let Some(run) = self.execution_repo.get_by_id(execution_id).await? {
            if matches!(run.status.as_str(), "success" | "failed" | "cancelled") {
                return Ok(());
            }
        }
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

    /// Startup only: any execution still unfinished lost its driver when the app quit.
    pub async fn reconcile_interrupted(&self) -> Result<usize, String> {
        let stale = self.execution_repo.get_non_terminal().await?;
        for e in &stale {
            self.finalize_running_steps(&e.id, "failed").await?;
            self.execution_repo
                .update_status(
                    &e.id,
                    "failed".to_string(),
                    Some("Interrupted by app restart".to_string()),
                )
                .await?;
        }
        Ok(stale.len())
    }

    async fn finalize_running_steps(&self, execution_id: &str, status: &str) -> Result<(), String> {
        modify_steps(&self.execution_repo, execution_id, |steps| {
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
            changed
        })
        .await
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

/// True at most once a second per execution, so a chatty step cannot flood the UI.
/// A line skipped here still rides the next update (step finish, or a later line).
fn progress_emit_due(execution_id: &str) -> bool {
    static LAST: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);
    let mut guard = LAST.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    map.retain(|_, t| t.elapsed() < Duration::from_secs(60));
    match map.get(execution_id) {
        Some(t) if t.elapsed() < Duration::from_secs(1) => false,
        _ => {
            map.insert(execution_id.to_string(), Instant::now());
            true
        }
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
    if let Some(p) = parse_progress_line(line) {
        record_progress(repo, execution_id, step_id, p, events).await;
        return;
    }
    ExecutionService::emit_step_log(events, execution_id, step_id, line, stream);
    write_step_logs(repo, execution_id, step_id, &[format_log_line(line, stream)]).await;
}

/// Store a `::portal-progress` line as the step's progress and nudge the UI
/// (at most once a second).
async fn record_progress(
    repo: &ExecutionRepository,
    execution_id: &str,
    step_id: &str,
    p: StepProgress,
    events: &Arc<dyn EventSink>,
) {
    let _ = modify_steps(repo, execution_id, |steps| {
        if let Some(step) = steps.iter_mut().find(|s| s.stepId == step_id) {
            step.progress = Some(p);
        }
        true
    })
    .await;
    if progress_emit_due(execution_id) {
        if let Ok(Some(e)) = repo.get_by_id(execution_id).await {
            events.emit("pipeline-execution-update", tick_payload(ExecutionService::execution_to_json(&e)));
        }
    }
}

fn format_log_line(line: &str, stream: &str) -> String {
    if stream == "stderr" {
        format!("[stderr] {}", line)
    } else {
        line.to_string()
    }
}

/// Append formatted lines to the step's `logs` and `output` in one blob write.
async fn write_step_logs(
    repo: &ExecutionRepository,
    execution_id: &str,
    step_id: &str,
    lines: &[String],
) {
    if lines.is_empty() {
        return;
    }
    let _ = modify_steps(repo, execution_id, |steps| {
        if let Some(step) = steps.iter_mut().find(|s| s.stepId == step_id) {
            for l in lines {
                step.logs.push(l.clone());
                step.output.push_str(l);
                step.output.push('\n');
            }
            cap_step_logs(step);
        }
        true
    })
    .await;
}

/// Most log lines kept per step; a dev server printing for hours must not
/// grow the blob that every batch rewrites.
const MAX_STEP_LOG_LINES: usize = 2000;
/// Log lines a progress tick carries for an action run (the UI fetches the rest).
const TICK_LOG_TAIL: usize = 200;

/// Ring buffer: drop the oldest lines past the cap and rebuild `output` to match.
fn cap_step_logs(step: &mut StepExecutionState) {
    if step.logs.len() > MAX_STEP_LOG_LINES {
        step.logs.drain(..step.logs.len() - MAX_STEP_LOG_LINES);
        step.output = step.logs.iter().map(|l| format!("{l}\n")).collect();
    }
}

/// Trim an action run's per-step logs to the tail (and drop `output`) before
/// emitting a progress tick. Pipelines keep full ticks: their monitor reads them.
fn tick_payload(mut exec: Value) -> Value {
    if exec["kind"] == "action" {
        if let Some(steps) = exec["stepExecutions"].as_array_mut() {
            for s in steps {
                if let Some(logs) = s["logs"].as_array_mut() {
                    logs.drain(..logs.len().saturating_sub(TICK_LOG_TAIL));
                }
                s["output"] = json!("");
            }
        }
    }
    exec
}

/// Copies one of a step's output pipes into its log, line by line. Lines are
/// emitted live but written to the DB in batches ([`LOG_FLUSH_LINES`] /
/// [`LOG_FLUSH_EVERY`]); the last batch is written at EOF, which
/// [`drain_log_readers`] waits for before the step is finalized.
/// stdout and stderr batch independently, so stored logs may interleave the two
/// streams within one 250 ms batch (live events stay ordered).
/// ponytail: a cancel aborts the reader, losing up to one unflushed batch.
struct LogReader {
    repo: ExecutionRepository,
    execution_id: String,
    step_id: String,
    stream: &'static str,
    events: Arc<dyn EventSink>,
    /// Shared by the step's readers; [`drain_log_readers`] watches it.
    lines_read: Arc<AtomicUsize>,
    /// Store lines as read, without the `[stderr] ` prefix (an action's log).
    raw: bool,
}

/// Reads up to and including the next LF or CR, so a progress bar redrawn
/// with a bare CR (pnpm, flutter, curl) streams live. A CRLF pair counts as
/// one terminator (`after_cr` remembers a trailing CR across chunks).
/// Returns the segment's length in `buf`; 0 means EOF. Cancel-safe.
async fn read_segment<R: AsyncRead + Unpin>(
    r: &mut BufReader<R>,
    buf: &mut Vec<u8>,
    after_cr: &mut bool,
) -> std::io::Result<usize> {
    loop {
        let chunk = r.fill_buf().await?;
        if chunk.is_empty() {
            return Ok(buf.len());
        }
        if std::mem::take(after_cr) && chunk[0] == b'\n' {
            r.consume(1);
            continue;
        }
        match chunk.iter().position(|&b| b == b'\r' || b == b'\n') {
            Some(i) => {
                *after_cr = chunk[i] == b'\r';
                buf.extend_from_slice(&chunk[..=i]);
                r.consume(i + 1);
                return Ok(buf.len());
            }
            None => {
                let l = chunk.len();
                buf.extend_from_slice(chunk);
                r.consume(l);
            }
        }
    }
}

impl LogReader {
    fn spawn<R: AsyncRead + Unpin + Send + 'static>(self, pipe: R) -> JoinHandle<()> {
        tokio::spawn(async move {
            // Bytes, not `lines()`: one non-UTF-8 byte (a tool writing in the
            // console code page) would end `lines()`, and with nobody reading
            // the pipe the child blocks on its next write once the pipe fills.
            let mut pipe = BufReader::new(pipe);
            let mut buf = Vec::new();
            let mut after_cr = false;
            let mut pending: Vec<String> = Vec::new();
            let mut flush_at: Option<tokio::time::Instant> = None;
            loop {
                // read_segment is cancel-safe: a partial line stays in `buf`.
                let n = tokio::select! {
                    r = read_segment(&mut pipe, &mut buf, &mut after_cr) => r.unwrap_or(0),
                    _ = async move {
                        match flush_at {
                            Some(t) => tokio::time::sleep_until(t).await,
                            None => std::future::pending().await,
                        }
                    } => {
                        write_step_logs(&self.repo, &self.execution_id, &self.step_id, &pending).await;
                        pending.clear();
                        flush_at = None;
                        continue;
                    }
                };
                if n == 0 {
                    break;
                }
                self.lines_read.fetch_add(1, Ordering::Relaxed);
                let line = String::from_utf8_lossy(&buf);
                let line = line.trim_end_matches(['\n', '\r']);
                if let Some(p) = parse_progress_line(line) {
                    record_progress(&self.repo, &self.execution_id, &self.step_id, p, &self.events)
                        .await;
                } else {
                    ExecutionService::emit_step_log(
                        &self.events,
                        &self.execution_id,
                        &self.step_id,
                        line,
                        self.stream,
                    );
                    pending.push(if self.raw {
                        line.to_string()
                    } else {
                        format_log_line(line, self.stream)
                    });
                    flush_at.get_or_insert_with(|| tokio::time::Instant::now() + LOG_FLUSH_EVERY);
                    if pending.len() >= LOG_FLUSH_LINES {
                        write_step_logs(&self.repo, &self.execution_id, &self.step_id, &pending)
                            .await;
                        pending.clear();
                        flush_at = None;
                    }
                }
                buf.clear();
            }
            write_step_logs(&self.repo, &self.execution_id, &self.step_id, &pending).await;
        })
    }
}

/// Wait for a finished step's readers to reach EOF, so its log is complete
/// before the step is finalized. A grandchild that outlives the step can keep
/// the pipe open indefinitely (a daemonized helper, or a tool like MSVC's
/// vctip.exe that link.exe leaves behind), so the step stops waiting once the
/// readers go [`READER_IDLE_GRACE`] without reading a line, or after
/// [`READER_DRAIN_CAP`] in all (a grandchild that keeps writing). Those readers
/// are detached, not aborted: aborting would drop the pipe and give a
/// backgrounded child (`cmd &`, `nohup`) EPIPE on its next write, and could cut
/// a line still being saved. They end on their own at EOF. Only cancel aborts.
async fn drain_log_readers(
    readers: Vec<JoinHandle<()>>,
    lines_read: &AtomicUsize,
    cancel_rx: &mut watch::Receiver<bool>,
) {
    let aborts: Vec<_> = readers.iter().map(|r| r.abort_handle()).collect();
    let all = async {
        for r in readers {
            let _ = r.await;
        }
    };
    tokio::pin!(all);
    let cap = tokio::time::sleep(READER_DRAIN_CAP);
    tokio::pin!(cap);
    let mut seen = lines_read.load(Ordering::Relaxed);
    let mut cancelled = false;
    loop {
        tokio::select! {
            _ = &mut all => return,
            _ = &mut cap => break,
            changed = cancel_rx.changed() => {
                if changed.is_err() || *cancel_rx.borrow() {
                    cancelled = true;
                    break;
                }
            }
            _ = tokio::time::sleep(READER_IDLE_GRACE) => {
                let now = lines_read.load(Ordering::Relaxed);
                if now == seen {
                    break;
                }
                seen = now;
            }
        }
    }
    log_warn!(
        "ExecutionService",
        "Step exited but its output pipe is still open; no longer waiting on its log readers"
    );
    if cancelled {
        for a in aborts {
            a.abort();
        }
    }
    // Otherwise dropping `all` (on return) detaches the readers.
}

/// The error of a failed action run: the exit code, then the last few lines the
/// process printed (`output` is its log, one line per `\n`).
pub(crate) fn action_failure_message(exit_code: Option<i32>, output: &str) -> String {
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let tail = lines[lines.len().saturating_sub(8)..].join("\n");

    match (exit_code, tail.is_empty()) {
        (Some(code), false) => format!("Exit code {code}\n{tail}"),
        (Some(code), true) => format!("Exit code {code}"),
        (None, false) => tail,
        (None, true) => "Process failed with no output".to_string(),
    }
}

/// Refuse a run in which any step comes after a `longRunning` one. A
/// long-running step (a dev server) never finishes: the run parks on it until
/// cancelled, so nothing ordered after it would ever start.
fn reject_steps_after_long_running(steps: &[Value]) -> Result<(), String> {
    let order = resolve_execution_order(steps)?.concat();
    let is_long_running = |id: &String| {
        steps.iter().any(|s| {
            s["id"].as_str() == Some(id.as_str())
                && s["config"]["longRunning"].as_bool() == Some(true)
        })
    };
    if let Some(at) = order.iter().position(is_long_running) {
        if let Some(next) = order.get(at + 1) {
            let server = &order[at];
            return Err(format!(
                "Step '{next}' comes after '{server}', which is longRunning: it never \
                 finishes, so '{next}' could never start. longRunning is for servers; \
                 drop it from '{server}', or make it the last step by giving it a \
                 dependsOn on every other step."
            ));
        }
    }
    Ok(())
}

/// Held across every read-modify-write of `pipeline_executions.step_executions_json`.
/// The run loop and a step's stdout/stderr readers all rewrite the whole blob;
/// unserialized, a reader's stale write undid `finalize_step` (a finished step
/// stuck at "running") and `set_step_running` (the next one shown "pending").
/// ponytail: one lock for every execution's blob; key it by execution id if
/// concurrent runs ever contend on it.
static STEPS_BLOB: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Load the execution's step states, let `f` change them, and write them back
/// when `f` returns true — all under [`STEPS_BLOB`].
async fn modify_steps(
    repo: &ExecutionRepository,
    execution_id: &str,
    f: impl FnOnce(&mut Vec<StepExecutionState>) -> bool,
) -> Result<(), String> {
    let _guard = STEPS_BLOB.lock().await;
    let execution = repo
        .get_by_id(execution_id)
        .await?
        .ok_or_else(|| "Execution not found".to_string())?;
    let mut steps: Vec<StepExecutionState> =
        serde_json::from_str(&execution.step_executions_json).unwrap_or_default();
    if f(&mut steps) {
        let json = serde_json::to_string(&steps)
            .map_err(|e| format!("Failed to serialize step executions: {}", e))?;
        repo.update_step_executions(execution_id, json).await?;
    }
    Ok(())
}

fn build_initial_step_executions(
    steps: &[Value],
    expected: &HashMap<String, i64>,
) -> Vec<StepExecutionState> {
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
                expectedMs: expected.get(id).copied(),
                progress: None,
                longRunning: step["config"]["longRunning"].as_bool() == Some(true),
            })
        })
        .collect()
}

/// Seconds before a non-long-running step is force-killed: its own `timeout`
/// (a positive value) if set, else [`DEFAULT_STEP_TIMEOUT_SECS`]. `timeout` is a
/// top-level step field in seconds (see the `PipelineStep` type); 0 or absent
/// means "use the default".
fn resolve_step_timeout(step: &Value) -> u64 {
    step.get("timeout")
        .and_then(|v| v.as_u64())
        .filter(|s| *s > 0)
        .unwrap_or(DEFAULT_STEP_TIMEOUT_SECS)
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
        // cmd /C npm run dev spawns node as a child — taskkill /T terminates the tree
        #[cfg(windows)]
        let _ = Command::new("taskkill")
            .no_window()
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;
        // Spawned with process_group(0), so the group id is the shell's pid.
        // TERM first, so a dev server can clean up after itself; KILL whatever
        // is still in the group after ~2 s (or left behind by a shell that
        // already exited).
        #[cfg(unix)]
        {
            signal_group("-TERM", pid).await;
            let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
            signal_group("-KILL", pid).await;
        }
    }
    let _ = child.start_kill();

    let _ = tokio::time::timeout(tokio::time::Duration::from_secs(5), child.wait()).await;
}

/// Send `signal` (e.g. `-TERM`) to the process group led by `pid`.
#[cfg(unix)]
async fn signal_group(signal: &str, pid: u32) {
    let _ = Command::new("kill")
        .args([signal, "--", &format!("-{pid}")])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn step_logs_ring_buffer_and_action_tick_tail() {
        let mut step = build_initial_step_executions(&[json!({ "id": "s" })], &HashMap::new())
            .remove(0);
        step.logs = (0..MAX_STEP_LOG_LINES + 5).map(|i| i.to_string()).collect();
        cap_step_logs(&mut step);
        assert_eq!(step.logs.len(), MAX_STEP_LOG_LINES);
        assert_eq!(step.logs[0], "5");
        assert!(step.output.starts_with("5\n"));

        let tick = |kind| {
            tick_payload(json!({ "kind": kind, "stepExecutions": [{
                "logs": step.logs, "output": "x" }] }))
        };
        assert_eq!(tick("action")["stepExecutions"][0]["logs"].as_array().unwrap().len(), TICK_LOG_TAIL);
        assert_eq!(tick("pipeline")["stepExecutions"][0]["logs"].as_array().unwrap().len(), MAX_STEP_LOG_LINES);
    }

    #[test]
    fn step_timeout_honors_explicit_value_else_defaults() {
        // Explicit positive timeout wins.
        assert_eq!(resolve_step_timeout(&json!({ "timeout": 30 })), 30);
        // Absent or non-positive falls back to the default, so a hung step is
        // still killed rather than running forever.
        assert_eq!(resolve_step_timeout(&json!({})), DEFAULT_STEP_TIMEOUT_SECS);
        assert_eq!(
            resolve_step_timeout(&json!({ "timeout": 0 })),
            DEFAULT_STEP_TIMEOUT_SECS
        );
    }

    #[test]
    fn action_failure_message_keeps_the_code_and_the_last_eight_lines() {
        let out: String = (1..=10).map(|i| format!("line {i}\n\n")).collect();
        let tail: Vec<String> = (3..=10).map(|i| format!("line {i}")).collect();
        assert_eq!(
            action_failure_message(Some(2), &out),
            format!("Exit code 2\n{}", tail.join("\n"))
        );
        assert_eq!(action_failure_message(Some(1), " \n"), "Exit code 1");
        assert_eq!(action_failure_message(None, "boom\n"), "boom");
        assert_eq!(
            action_failure_message(None, ""),
            "Process failed with no output"
        );
    }

    #[derive(Debug)]
    struct Quiet;
    impl EventSink for Quiet {
        fn emit_json(&self, _: &str, _: Value) -> bool {
            false
        }
    }

    #[test]
    fn refuses_steps_after_a_long_running_one() {
        let server = json!({ "id": "build", "dependsOn": [], "config": { "longRunning": true } });
        let needs = json!({ "id": "publish", "dependsOn": ["build"] });
        let err = reject_steps_after_long_running(&[server.clone(), needs]).unwrap_err();
        assert!(err.contains("'publish' comes after 'build'"), "{err}");
        // Ordered after it without needing it: still never starts.
        let after = json!({ "id": "e2e", "dependsOn": [] });
        assert!(reject_steps_after_long_running(&[server.clone(), after.clone()]).is_err());
        // Last in the order is fine.
        assert!(reject_steps_after_long_running(&[after, server]).is_ok());
    }

    /// A pipe whose write end outlives the step (a grandchild holding it) must
    /// not hold the run: the drain gives up once the reader goes idle.
    #[tokio::test]
    async fn drain_abandons_a_pipe_that_never_closes() {
        let (svc, _dir) = service_with_execution().await;
        let (mut held_open, pipe) = tokio::io::duplex(64);
        tokio::io::AsyncWriteExt::write_all(&mut held_open, b"one\ntwo\n")
            .await
            .unwrap();
        let lines_read = Arc::new(AtomicUsize::new(0));
        let reader = LogReader {
            repo: svc.execution_repo.clone(),
            execution_id: "e".into(),
            step_id: "a".into(),
            stream: "stdout",
            events: Arc::new(Quiet),
            lines_read: Arc::clone(&lines_read),
            raw: false,
        }
        .spawn(pipe);
        let (_cancel_tx, mut cancel_rx) = watch::channel(false);
        let drained = tokio::time::timeout(
            // Only has to be finite: without the idle give-up this never ends.
            READER_IDLE_GRACE * 20,
            drain_log_readers(vec![reader], &lines_read, &mut cancel_rx),
        )
        .await;
        assert!(drained.is_ok(), "drain waited on a pipe that never closes");
        expect_logs(&svc, &["one", "two"]).await;
        // Detached, not aborted: the read end is still open, so a background
        // child writing later gets no EPIPE, and its output is still logged.
        tokio::io::AsyncWriteExt::write_all(&mut held_open, b"late\n")
            .await
            .expect("reader was aborted: write end got a broken pipe");
        expect_logs(&svc, &["one", "two", "late"]).await;
    }

    /// Step "a"'s log reaches `want`. Waits rather than sleeping a fixed time:
    /// the batched write has no deadline on a loaded machine.
    async fn expect_logs(svc: &ExecutionService, want: &[&str]) {
        let mut got = Vec::new();
        for _ in 0..600 {
            got = svc.get_step_logs("e", "a").await.unwrap();
            if got == want {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert_eq!(got, want);
    }

    /// A service over a scratch database holding execution "e" with pending
    /// steps "a" and "b".
    async fn service_with_execution() -> (ExecutionService, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            DatabaseManager::new(dir.path().to_path_buf())
                .await
                .unwrap(),
        );
        let svc = ExecutionService::new(db);
        let project = svc
            .project_repo
            .create(
                "p".into(),
                None,
                dir.path().display().to_string(),
                vec![],
                vec![],
                vec![],
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let pipeline = svc
            .pipeline_repo
            .create(
                "p".into(),
                None,
                project.id,
                "[]".into(),
                "[]".into(),
                "[]".into(),
                "{}".into(),
                true,
                None,
                None,
            )
            .await
            .unwrap();
        let steps = build_initial_step_executions(&[json!({ "id": "a" }), json!({ "id": "b" })], &HashMap::new());
        svc.execution_repo
            .create(
                "e".into(),
                Some(pipeline.id),
                Some(project.id),
                "running".into(),
                "user".into(),
                serde_json::to_string(&steps).unwrap(),
                "{}".into(),
            )
            .await
            .unwrap();
        (svc, dir)
    }

    /// Two log readers append while the run loop finalizes step `a` and starts
    /// `b` — the shape of a step whose output is still draining when it exits.
    /// Every line and both status writes must survive.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_log_appends_and_finalize_lose_nothing() {
        let (svc, _dir) = service_with_execution().await;
        svc.set_step_running("e", "a").await.unwrap();

        const LINES: usize = 60;
        let events: Arc<dyn EventSink> = Arc::new(Quiet);
        let readers: Vec<_> = ["stdout", "stderr"]
            .into_iter()
            .map(|stream| {
                let (repo, events) = (svc.execution_repo.clone(), events.clone());
                tokio::spawn(async move {
                    for i in 0..LINES {
                        append_step_log(&repo, "e", "a", &format!("{i}"), stream, &events).await;
                    }
                })
            })
            .collect();
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        svc.finalize_step("e", "a", "success", Some(0), None)
            .await
            .unwrap();
        svc.set_step_running("e", "b").await.unwrap();
        for r in readers {
            r.await.unwrap();
        }

        let exec = svc.execution_repo.get_by_id("e").await.unwrap().unwrap();
        let steps: Vec<StepExecutionState> =
            serde_json::from_str(&exec.step_executions_json).unwrap();
        assert_eq!(steps[0].status, "success");
        assert_eq!(steps[1].status, "running");
        assert_eq!(steps[0].logs.len(), 2 * LINES);
    }

    async fn finished(
        svc: &ExecutionService,
        id: &str,
    ) -> crate::entities::pipeline_execution::Model {
        for _ in 0..400 {
            let e = svc.execution_repo.get_by_id(id).await.unwrap().unwrap();
            if matches!(e.status.as_str(), "success" | "failed" | "cancelled") {
                return e;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("run {id} did not finish");
    }

    /// `execute_pipeline` is on `/rpc`; a `release:` row must not be its way
    /// around `local_only` `release_ship`.
    #[tokio::test]
    async fn execute_pipeline_refuses_a_release_row() {
        let (svc, _dir) = service_with_execution().await;
        let seed = svc.execution_repo.get_by_id("e").await.unwrap().unwrap();
        let pipeline_id = seed.pipeline_id.unwrap();
        svc.pipeline_repo
            .update(
                pipeline_id,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(Some("release:ship".into())),
                None,
            )
            .await
            .unwrap();
        let request = ExecutionRequestData {
            pipeline_id: pipeline_id.to_string(),
            variables: None,
            secrets: None,
            triggered_by: None,
        };
        let refused = svc.execute_pipeline(request, Arc::new(Quiet)).await;
        assert_eq!(refused.unwrap_err(), RELEASE_ROW_REFUSAL);
    }

    /// One runner, two log formats: a pipeline keeps its `$ cmd` / `cwd:` header
    /// and `[stderr] ` prefix; an action's log is exactly the process output.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn pipeline_and_action_runs_share_the_runner_but_not_the_log_format() {
        let (svc, dir) = service_with_execution().await;
        let cwd = dir.path().display().to_string();
        let seed = svc.execution_repo.get_by_id("e").await.unwrap().unwrap();
        let pipeline_id = seed.pipeline_id.unwrap();
        let step = |command: &str| json!({ "id": "s", "name": "s", "dependsOn": [], "config": { "command": command } });

        let command = "echo out&& echo err>&2";
        let steps = json!([step(command)]).to_string();
        svc.pipeline_repo
            .update(
                pipeline_id,
                None,
                None,
                Some(steps),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let request = ExecutionRequestData {
            pipeline_id: pipeline_id.to_string(),
            variables: None,
            secrets: None,
            triggered_by: Some("mcp:t".into()),
        };
        let id = svc
            .execute_pipeline(request, Arc::new(Quiet))
            .await
            .unwrap();
        let run = finished(&svc, &id).await;
        assert_eq!(run.status, "success");
        assert_eq!(run.triggered_by, "mcp:t");
        assert_eq!(run.kind, "pipeline");
        assert_eq!(run.pid, None);
        let logs = svc.get_step_logs(&id, "s").await.unwrap();
        assert_eq!(logs[..2], [format!("$ {command}"), format!("cwd: {cwd}")]);
        assert!(logs.contains(&"out".to_string()), "{logs:?}");
        assert!(logs.contains(&"[stderr] err".to_string()), "{logs:?}");
        let json = svc.get_execution(&id).await.unwrap().unwrap();
        assert_eq!(json["kind"], "pipeline");
        assert!(json.get("command").is_none());

        // An action: `env` reaches the process as given, `variables` only fill `${}`.
        let (ok, bad) = if cfg!(windows) {
            ("echo hi&& echo %PORTAL_T%", "echo oops>&2&& exit 3")
        } else {
            ("echo hi&& echo $PORTAL_T", "echo oops>&2&& exit 3")
        };
        let run_action = |command: &str| {
            let spec = RunSpec {
                cwd: cwd.clone(),
                steps: vec![step(command)],
                variables: HashMap::from([("B".to_string(), "x".to_string())]),
                env: HashMap::from([("PORTAL_T".to_string(), "a$B".to_string())]),
                triggered_by: "mcp:t".into(),
                action: Some(ActionMeta {
                    command: command.into(),
                    working_directory: Some(cwd.clone()),
                    step_id: Some("s".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            svc.execute_steps(spec, Arc::new(Quiet))
        };
        let id = run_action(ok).await.unwrap();
        let run = finished(&svc, &id).await;
        assert_eq!(run.status, "success");
        assert_eq!(run.kind, "action");
        assert!(run.pid.is_some(), "the action's pid is recorded");
        assert_eq!(svc.get_step_logs(&id, "s").await.unwrap(), ["hi", "a$B"]);
        let json = svc.get_execution(&id).await.unwrap().unwrap();
        assert_eq!(json["kind"], "action");
        assert_eq!(json["command"], ok);
        assert_eq!(json["stepId"], "s");
        assert!(json["pipelineId"].is_null() && json["workflowPlan"].is_null());

        let id = run_action(bad).await.unwrap();
        let run = finished(&svc, &id).await;
        assert_eq!(run.status, "failed");
        assert_eq!(run.error.as_deref(), Some("Exit code 3\noops"));
        assert_eq!(svc.get_step_logs(&id, "s").await.unwrap(), ["oops"]);
    }

    /// Retry reruns the failed node and everything after it, on the same run, while
    /// steps that already succeeded are left alone (not rerun).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn retry_reruns_the_failed_step_and_after_keeping_successes() {
        let (svc, dir) = service_with_execution().await;
        let path = dir.path().to_path_buf();
        let pipeline_id = svc
            .execution_repo
            .get_by_id("e")
            .await
            .unwrap()
            .unwrap()
            .pipeline_id
            .unwrap();

        // s1 succeeds (appends a line); s2 fails until a marker file exists, so it
        // fails on the first run and passes on the retry; s3 (after s2) writes a file.
        let (s1, s2, s3) = if cfg!(windows) {
            (
                "echo x>> count.txt",
                "if exist marker (exit 0) else (type nul>marker& exit 1)",
                "echo done> done.txt",
            )
        } else {
            (
                "echo x >> count.txt",
                "if [ -f marker ]; then exit 0; else : > marker; exit 1; fi",
                "echo done > done.txt",
            )
        };
        let step = |id: &str, needs: Value, command: &str| {
            json!({ "id": id, "name": id, "dependsOn": needs, "config": { "command": command } })
        };
        let steps = json!([
            step("s1", json!([]), s1),
            step("s2", json!(["s1"]), s2),
            step("s3", json!(["s2"]), s3),
        ])
        .to_string();
        svc.pipeline_repo
            .update(
                pipeline_id,
                None,
                None,
                Some(steps),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let request = || ExecutionRequestData {
            pipeline_id: pipeline_id.to_string(),
            variables: None,
            secrets: None,
            triggered_by: None,
        };
        let count_lines = || {
            std::fs::read_to_string(path.join("count.txt"))
                .unwrap()
                .lines()
                .count()
        };

        let id = svc.execute_pipeline(request(), Arc::new(Quiet)).await.unwrap();
        let run = finished(&svc, &id).await;
        assert_eq!(run.status, "failed");
        assert!(!path.join("done.txt").exists(), "s3 must not run after s2 fails");
        assert_eq!(count_lines(), 1);

        // Same run, retried: s1 is skipped, s2 now passes, s3 runs. The driver
        // writes `failed` before it frees its slot; hold the slot a while so the
        // retry always lands inside that gap.
        while svc.is_running(&id) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(svc.reserve_run(&id, String::new(), false).is_some());
        let (held, held_id) = (svc.clone(), id.clone());
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            held.running.lock().unwrap().remove(&held_id);
        });
        svc.retry_execution(&id, Arc::new(Quiet)).await.unwrap();
        let run = finished(&svc, &id).await;
        assert_eq!(run.status, "success");
        assert_eq!(run.error, None, "the old failure's error is cleared");
        assert_eq!(count_lines(), 1, "the succeeded step s1 is not rerun");
        assert!(path.join("done.txt").exists(), "s3 runs on the retry");

        let steps: Vec<StepExecutionState> =
            serde_json::from_str(&run.step_executions_json).unwrap();
        let status = |id: &str| {
            steps
                .iter()
                .find(|s| s.stepId == id)
                .map(|s| s.status.clone())
                .unwrap()
        };
        assert_eq!(
            (status("s1"), status("s2"), status("s3")),
            ("success".into(), "success".into(), "success".into())
        );
    }

    /// A retry runs with the run's own variables, not whatever env the shared
    /// pipeline row still carries from a later run of the same workflow — so a
    /// rehearsal retried after a ship stays a rehearsal (the release-safety property).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn retry_reapplies_the_runs_variables_over_the_pipelines_baked_env() {
        let (svc, dir) = service_with_execution().await;
        let path = dir.path().to_path_buf();
        let seed = svc.execution_repo.get_by_id("e").await.unwrap().unwrap();
        let pipeline_id = seed.pipeline_id.unwrap();
        let project_id = seed.project_id.unwrap();

        // The shared pipeline row carries a step whose env was baked MODE=ship by an
        // earlier run; the step writes $MODE to a file.
        let cmd = if cfg!(windows) {
            "echo %MODE%>mode.txt"
        } else {
            "printf '%s' \"$MODE\" > mode.txt"
        };
        let steps = json!([{
            "id": "s", "name": "s", "dependsOn": [],
            "config": { "command": cmd, "env": { "MODE": "ship" } }
        }])
        .to_string();
        svc.pipeline_repo
            .update(
                pipeline_id,
                None,
                None,
                Some(steps),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // A failed run of that pipeline whose OWN variables say MODE=rehearse.
        svc.execution_repo
            .insert(NewRun {
                id: "r".into(),
                pipeline_id: Some(pipeline_id),
                project_id: Some(project_id),
                status: "failed".into(),
                triggered_by: "user".into(),
                step_executions_json: json!([{
                    "id": "x", "stepId": "s", "stepName": "s", "status": "failed",
                    "startedAt": "", "output": "", "retryCount": 0, "logs": []
                }])
                .to_string(),
                variables_json: json!({ "MODE": "rehearse" }).to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        svc.retry_execution("r", Arc::new(Quiet)).await.unwrap();
        let run = finished(&svc, "r").await;
        assert_eq!(run.status, "success");
        assert_eq!(
            std::fs::read_to_string(path.join("mode.txt")).unwrap().trim(),
            "rehearse",
            "the retry must use the run's own MODE, not the pipeline's baked env"
        );
    }

    /// An action's ETA history is its block's successful runs (plus the block-less
    /// ones of the same command and folder, which is how a built-in block's old
    /// history was copied), else those of the same command in the same folder (no
    /// folder matches no folder). It is one step whatever its id; failed runs do
    /// not use up the window, and a row copied from `script_executions` with no
    /// step `duration` counts for its own span.
    #[tokio::test]
    async fn action_expected_is_the_median_of_the_blocks_or_the_commands_own_runs() {
        let (svc, _dir) = service_with_execution().await;
        let meta = |block: Option<&str>, command: &str, cwd: Option<&str>| ActionMeta {
            command: command.into(),
            working_directory: cwd.map(Into::into),
            block_id: block.map(Into::into),
            ..Default::default()
        };
        // An action's step, named `m.step_id` (default "run") as `record_finished_action`
        // does, that lasted `ms` (`None`: no `duration` written).
        let step = |m: &ActionMeta, status: &str, ms: Option<i64>| {
            json!({
                "id": "i", "stepId": m.step_id.as_deref().unwrap_or("run"), "stepName": m.command,
                "status": status, "startedAt": "", "output": "", "retryCount": 0, "logs": [],
                "duration": ms
            })
        };
        // A one-step action row.
        let row = |m: ActionMeta, status: &str, ms: Option<i64>| NewRun {
            id: Uuid::new_v4().to_string(),
            status: status.into(),
            triggered_by: "user".into(),
            step_executions_json: json!([step(&m, status, ms)]).to_string(),
            variables_json: "{}".into(),
            action: Some(m),
            ..Default::default()
        };
        // The ETA for `m`'s step (`m.step_id`, default "run"), as `execute_script` asks for it.
        async fn eta(svc: &ExecutionService, m: ActionMeta) -> Option<i64> {
            let step_id = m.step_id.clone().unwrap_or_else(|| "run".to_string());
            svc.action_expected(&m, &step_id)
                .await
                .get(&step_id)
                .copied()
        }
        let repo = &svc.execution_repo;

        // Block "b": runs of 1 s, 3 s and 5 s, then twenty newer failures that must
        // not push them out of the window. The command asked for does not matter.
        for ms in [1000, 3000, 5000] {
            repo.insert(row(meta(Some("b"), "x", None), "success", Some(ms)))
                .await
                .unwrap();
        }
        for _ in 0..20 {
            repo.insert(row(meta(Some("b"), "x", None), "failed", Some(9999)))
                .await
                .unwrap();
        }
        assert_eq!(eta(&svc, meta(Some("b"), "other", None)).await, Some(3000));
        assert_eq!(eta(&svc, meta(Some("none"), "x", None)).await, None);

        // No block: the same command in the same folder, and NULL matches NULL.
        for (cwd, ms) in [(Some("/w"), 7000), (Some("/w"), 9000), (None, 1000)] {
            repo.insert(row(meta(None, "c", cwd), "success", Some(ms)))
                .await
                .unwrap();
        }
        assert_eq!(eta(&svc, meta(None, "c", Some("/w"))).await, Some(8000));
        assert_eq!(eta(&svc, meta(None, "c", None)).await, Some(1000));
        assert_eq!(eta(&svc, meta(None, "c", Some("/elsewhere"))).await, None);

        // A copied row without a step `duration` counts for the run's own 10 s; a
        // row whose steps are unreadable counts for nothing and fails nothing.
        let t0 = Utc::now() - chrono::Duration::seconds(60);
        repo.insert(NewRun {
            started_at: Some(t0),
            finished_at: Some(t0 + chrono::Duration::seconds(10)),
            ..row(meta(None, "old", None), "success", None)
        })
        .await
        .unwrap();
        repo.insert(NewRun {
            step_executions_json: "garbage".into(),
            ..row(meta(None, "old", None), "success", None)
        })
        .await
        .unwrap();
        assert_eq!(eta(&svc, meta(None, "old", None)).await, Some(10_000));

        // An action is one step whatever a workflow names it: block "b"'s runs above
        // were recorded as "run" and answer for "build"; runs recorded as "build"
        // answer for "run".
        let build = |m: ActionMeta| ActionMeta {
            step_id: Some("build".into()),
            ..m
        };
        assert_eq!(
            eta(&svc, build(meta(Some("b"), "x", None))).await,
            Some(3000)
        );
        for ms in [2000, 4000] {
            repo.insert(row(build(meta(Some("w"), "x", None)), "success", Some(ms)))
                .await
                .unwrap();
        }
        assert_eq!(eta(&svc, meta(Some("w"), "x", None)).await, Some(3000));
        // A run of several steps is not an action's: it answers for nothing.
        let m = meta(None, "multi", None);
        let s = step(&m, "success", Some(500));
        repo.insert(NewRun {
            step_executions_json: json!([s.clone(), s]).to_string(),
            ..row(m, "success", Some(500))
        })
        .await
        .unwrap();
        assert_eq!(eta(&svc, meta(None, "multi", None)).await, None);

        // A built-in block's old history was copied with a NULL block: for a run that
        // has a block, the same command in the same folder counts; another folder,
        // another command and another block's run do not.
        for (block, command, cwd, ms) in [
            (None, "npm test", Some("/w"), 6000),
            (None, "npm test", Some("/other"), 90_000),
            (None, "npm run test", Some("/w"), 90_000),
            (Some("not-t"), "npm test", Some("/w"), 90_000),
        ] {
            repo.insert(row(meta(block, command, cwd), "success", Some(ms)))
                .await
                .unwrap();
        }
        assert_eq!(
            eta(&svc, meta(Some("t"), "npm test", Some("/w"))).await,
            Some(6000)
        );
    }

    #[tokio::test]
    async fn read_segment_splits_on_cr_lf_and_crlf() {
        let mut pipe = BufReader::new(&b"10%\r50%\rdone\r\nnext\n\nlast"[..]);
        let (mut buf, mut after_cr) = (Vec::new(), false);
        let mut got = Vec::new();
        while read_segment(&mut pipe, &mut buf, &mut after_cr).await.unwrap() > 0 {
            got.push(String::from_utf8(std::mem::take(&mut buf)).unwrap());
        }
        assert_eq!(got, ["10%\r", "50%\r", "done\r", "next\n", "\n", "last"]);
    }
}

#[cfg(test)]
mod foreach_tests {
    use super::*;

    #[test]
    fn empty_or_duplicated_items_file_fails_a_real_run() {
        let file = pipeline_file::parse(
            "actions: { a: { run: x } }\nworkflows: { w: { steps: [ { action: a, foreach: { items_file: l.txt } } ] } }",
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        for list in ["# nothing\n\n", "gym\ngym\n"] {
            std::fs::write(dir.path().join("l.txt"), list).unwrap();
            assert!(resolve_foreach_items(root, &file, "w", true).is_err());
            assert!(resolve_foreach_items(root, &file, "w", false)
                .unwrap()
                .is_empty());
        }
    }
}
