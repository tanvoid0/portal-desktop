use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{ActionMeta, ExecutionRepository};
use crate::domains::projects::pipelines::services::{ExecutionService, RunSpec};
use crate::domains::scripts::repositories::WorkflowLink;
use crate::entities::pipeline_execution::Model;
use crate::events::EventSink;
use crate::utils::shell_template;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteScriptRequest {
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub working_directory: Option<String>,
    /// Extra environment variables for the spawned process (e.g. CI=true for
    /// non-interactive Action runs). Merged onto the inherited env.
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Who asked for the run: "user" (default) or e.g. "mcp:<client>". Stored
    /// on the row so a stop can be told apart from a failure by whoever waits.
    #[serde(default)]
    pub triggered_by: Option<String>,
    /// Workflow grouping: every step of one run shares `workflow_run_id`;
    /// `workflow_plan` (`[{id,name,needs}]`, topo order) is stored per row.
    #[serde(default)]
    pub workflow_run_id: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub step_id: Option<String>,
    #[serde(default)]
    pub workflow_plan: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptExecutionInfo {
    pub id: String,
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub pid: Option<i32>,
    pub output: String,
    pub error: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub triggered_by: String,
    pub workflow_run_id: Option<String>,
    pub workflow_id: Option<String>,
    pub step_id: Option<String>,
    pub workflow_plan: Option<serde_json::Value>,
}

/// The run's log so far: every step's `logs`, one element per line, each
/// ending in `\n` (callers join with ''). A history row copied from
/// `script_executions` keeps its old output in the same place.
fn log_lines(row: &Model) -> Vec<String> {
    let steps: Vec<Value> = serde_json::from_str(&row.step_executions_json).unwrap_or_default();
    steps
        .iter()
        .filter_map(|s| s["logs"].as_array())
        .flatten()
        .filter_map(|l| l.as_str().map(|l| format!("{l}\n")))
        .collect()
}

impl From<Model> for ScriptExecutionInfo {
    fn from(e: Model) -> Self {
        let steps: Vec<Value> = serde_json::from_str(&e.step_executions_json).unwrap_or_default();
        let exit_code = steps
            .iter()
            .rev()
            .find_map(|s| s["exitCode"].as_i64())
            .map(|c| c as i32);
        // Live text comes from `get_live_output`; `output` appears once the run is over.
        let output = if matches!(e.status.as_str(), "pending" | "queued" | "running") {
            String::new()
        } else {
            log_lines(&e).concat()
        };
        // The runner records a Stop as `cancelled` with no error; the Action UI
        // and an MCP client read this string.
        let error = e
            .error
            .or_else(|| (e.status == "cancelled").then(|| "Stopped by user".to_string()));
        Self {
            id: e.id,
            block_id: e.block_id,
            command: e.command.unwrap_or_default(),
            parameters: serde_json::from_str(&e.variables_json).unwrap_or_default(),
            working_directory: e.working_directory,
            status: e.status,
            exit_code,
            pid: e.pid,
            output,
            error,
            started_at: e.started_at.to_rfc3339(),
            finished_at: e.finished_at.map(|d| d.to_rfc3339()),
            triggered_by: e.triggered_by,
            workflow_run_id: e.workflow_run_id,
            workflow_id: e.workflow_id,
            step_id: e.step_id,
            workflow_plan: e
                .workflow_plan_json
                .and_then(|j| serde_json::from_str(&j).ok()),
        }
    }
}

/// Action and terminal runs, as the `*_script_*` commands speak them, over the
/// shared pipeline runner: a run is a one-step `kind = 'action'` row in
/// `pipeline_executions` (see docs/development/UNIFIED_RUNNER_PLAN.md).
#[derive(Clone)]
pub struct ScriptExecutionService {
    repo: ExecutionRepository,
    exec: Arc<ExecutionService>,
}

impl ScriptExecutionService {
    /// `exec` must be the app's one `ExecutionService`: it owns the running
    /// processes, so a second instance could not Stop what this one started.
    pub fn new(db_manager: Arc<DatabaseManager>, exec: Arc<ExecutionService>) -> Self {
        Self {
            repo: ExecutionRepository::new(db_manager),
            exec,
        }
    }

    /// True if any action currently running has a working directory at or
    /// under `path`. Used by the project-relocation in-use check.
    pub fn running_under(&self, path: &str) -> bool {
        self.exec.running_actions_under(path)
    }

    /// The row of `id` if it is an action run; a pipeline's id is not ours.
    async fn action(&self, id: &str) -> Result<Option<Model>, String> {
        Ok(self.repo.get_by_id(id).await?.filter(|r| r.kind == "action"))
    }

    /// Run a command as a one-step run and return its id once it is queued.
    pub async fn execute_script(
        &self,
        request: ExecuteScriptRequest,
        events: Arc<dyn EventSink>,
    ) -> Result<String, String> {
        if request.command.trim().is_empty() {
            return Err("Command cannot be empty".to_string());
        }
        // No row for a folder that is not there; the caller hears why (as it did from the spawn).
        let cwd = request.working_directory.clone().unwrap_or_default();
        if !cwd.is_empty() && !Path::new(&cwd).is_dir() {
            return Err(format!("Working directory does not exist: {cwd}"));
        }
        // Shown and stored resolved (`${name}` / `$name` filled in, quoted for
        // the shell); the step keeps the template, which the runner renders
        // from `variables` — rendering the resolved text again would re-expand.
        let command = shell_template::render(&request.command, &request.parameters);
        let step_id = request.step_id.clone().unwrap_or_else(|| "run".to_string());
        self.exec
            .execute_steps(
                RunSpec {
                    cwd,
                    steps: vec![json!({
                        "id": step_id,
                        "name": command,
                        "dependsOn": [],
                        "config": { "command": request.command },
                    })],
                    variables: request.parameters,
                    env: request.env.unwrap_or_default(),
                    triggered_by: request.triggered_by.unwrap_or_else(|| "user".to_string()),
                    action: Some(ActionMeta {
                        command,
                        working_directory: request.working_directory,
                        block_id: request.block_id,
                        workflow_run_id: request.workflow_run_id,
                        workflow_id: request.workflow_id,
                        step_id: request.step_id,
                        workflow_plan_json: request.workflow_plan.map(|p| p.to_string()),
                    }),
                    ..Default::default()
                },
                events,
            )
            .await
    }

    /// Record a run that happened in the embedded interactive terminal (no process is spawned here).
    ///
    /// Also used for workflow steps that never spawn a local process (GitHub/n8n,
    /// no command, a spawn that threw): `link` ties the row to the run, and
    /// `status`/`error` override what the exit code would imply.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_interactive(
        &self,
        block_id: Option<String>,
        command: String,
        working_directory: Option<String>,
        exit_code: Option<i32>,
        started_at: Option<String>,
        link: WorkflowLink,
        status: Option<String>,
        error: Option<String>,
    ) -> Result<(), String> {
        let status = status.unwrap_or_else(|| {
            match exit_code {
                Some(0) => "success",
                Some(_) => "failed",
                None => "cancelled",
            }
            .to_string()
        });
        // Backdates started_at to when the terminal run began so duration is real.
        let started_at = started_at
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
            .map(|t| t.with_timezone(&chrono::Utc));
        self.exec
            .record_finished_action(
                ActionMeta {
                    command,
                    working_directory,
                    block_id,
                    workflow_run_id: link.run_id,
                    workflow_id: link.workflow_id,
                    step_id: link.step_id,
                    workflow_plan_json: link.plan_json,
                },
                started_at,
                status,
                exit_code,
                error,
            )
            .await?;
        Ok(())
    }

    /// Get execution details
    pub async fn get_execution(
        &self,
        execution_id: &str,
    ) -> Result<Option<ScriptExecutionInfo>, String> {
        Ok(self.action(execution_id).await?.map(Into::into))
    }

    /// Every step row of one workflow run, oldest first
    pub async fn get_executions_by_workflow_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<ScriptExecutionInfo>, String> {
        let rows = self.repo.get_by_workflow_run(run_id).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// The run's output so far, one element per line, each ending in `\n`.
    /// Read from the database, so it is the same after the run has finished.
    pub async fn get_live_output(&self, execution_id: &str) -> Vec<String> {
        match self.action(execution_id).await {
            Ok(Some(row)) => log_lines(&row),
            _ => Vec::new(),
        }
    }

    /// Stop a running execution and its whole process tree. The runner records
    /// `cancelled` at once: a Stop wins, even over a process that was exiting.
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        let row = self
            .action(execution_id)
            .await?
            .ok_or_else(|| format!("Execution not found: {execution_id}"))?;
        // The runner would flip a finished row to cancelled; refuse here.
        if !matches!(row.status.as_str(), "pending" | "queued" | "running") {
            return Err(format!("Run already finished ({})", row.status));
        }
        // Left over from an earlier Portal session — its process isn't ours to
        // kill (the pid may be reused), so close the row and say so.
        if row.status == "running" && !self.exec.is_running(execution_id) {
            // Re-read: the run may have finished since `row`, and must not get this stamped on it.
            let active = self.action(execution_id).await?.is_some_and(|r| {
                matches!(r.status.as_str(), "pending" | "queued" | "running")
            });
            if active {
                self.repo
                    .update_status(
                        execution_id,
                        "cancelled".to_string(),
                        Some("Not tracked by this Portal session — marked stopped".to_string()),
                    )
                    .await?;
            }
            return Ok(());
        }
        self.exec.cancel_execution(execution_id, None).await
    }

    /// Get executions for a specific block/script
    pub async fn get_executions_by_block(
        &self,
        block_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<ScriptExecutionInfo>, String> {
        let rows = self.repo.get_by_block(block_id, limit).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get all running executions
    pub async fn get_running_executions(&self) -> Result<Vec<ScriptExecutionInfo>, String> {
        let rows = self.repo.get_active_actions().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get recent executions (action runs only, not pipelines)
    pub async fn get_recent_executions(
        &self,
        limit: u64,
    ) -> Result<Vec<ScriptExecutionInfo>, String> {
        let rows = self.repo.get_recent(Some("action"), Some(limit)).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Kept for the `sync_script_executions` command: runs the last session
    /// left unfinished are closed at startup by `ExecutionService::reconcile_interrupted`.
    pub async fn sync_running_executions(&self) -> Result<(), String> {
        Ok(())
    }

    /// Delete an execution record
    pub async fn delete_execution(&self, execution_id: &str) -> Result<(), String> {
        // Only an action run is ours to delete; a pipeline's id is a no-op.
        if self.action(execution_id).await?.is_none() {
            return Ok(());
        }
        // First try to cancel if running
        let _ = self.cancel_execution(execution_id).await;
        // Then delete from database
        self.repo.delete(execution_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::pipeline_execution::{Column, Entity};
    use crate::migrations::m20261119_000051_unify_runs::copy_script_history;
    use sea_orm::{ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement};

    #[derive(Debug)]
    struct Quiet;
    impl EventSink for Quiet {
        fn emit_json(&self, _: &str, _: Value) -> bool {
            false
        }
    }

    async fn service() -> (ScriptExecutionService, Arc<DatabaseManager>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(
            DatabaseManager::new(dir.path().to_path_buf())
                .await
                .unwrap(),
        );
        let exec = Arc::new(ExecutionService::new(db.clone()));
        (ScriptExecutionService::new(db.clone(), exec), db, dir)
    }

    #[tokio::test]
    async fn record_interactive_stores_workflow_link_status_and_error() {
        let (svc, _db, _dir) = service().await;
        let link = WorkflowLink {
            run_id: Some("r1".into()),
            workflow_id: Some("w1".into()),
            step_id: Some("gh-deploy".into()),
            plan_json: Some("[]".into()),
        };
        svc.record_interactive(
            None,
            "gh-deploy".into(),
            None,
            None,
            None,
            link,
            Some("failed".into()),
            Some("deploy broke".into()),
        )
        .await
        .unwrap();
        let rows = svc.get_executions_by_workflow_run("r1").await.unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.step_id.as_deref(), Some("gh-deploy"));
        assert_eq!(r.workflow_id.as_deref(), Some("w1"));
        assert_eq!(r.status, "failed");
        assert_eq!(r.error.as_deref(), Some("deploy broke"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn script_facade_runs_on_pipeline_runner() {
        let (svc, db, dir) = service().await;
        let conn = db.get_connection();

        // History the old service wrote: still in script_executions after the
        // migrations ran, copied by the catch-up, readable under the same id.
        conn.execute(Statement::from_string(
            DbBackend::Sqlite,
            "INSERT INTO script_executions (id, command, parameters_json, working_directory, status, exit_code, output, started_at, finished_at, triggered_by)
             VALUES ('legacy', 'echo old', '{}', '/w', 'success', 0, 'one\ntwo\n', '2026-01-03T00:00:00+00:00', '2026-01-03T00:00:02+00:00', 'mcp:t')"
                .to_string(),
        ))
        .await
        .unwrap();
        copy_script_history(conn, true).await.unwrap();
        let old = svc.get_execution("legacy").await.unwrap().unwrap();
        assert_eq!(
            (old.command.as_str(), old.status.as_str(), old.output.as_str()),
            ("echo old", "success", "one\ntwo\n")
        );
        assert_eq!(old.triggered_by, "mcp:t");
        assert_eq!(old.exit_code, Some(0));

        // A new run goes through the pipeline runner.
        let request = ExecuteScriptRequest {
            block_id: None,
            command: "echo hi".into(),
            parameters: HashMap::new(),
            working_directory: Some(dir.path().display().to_string()),
            env: None,
            triggered_by: None,
            workflow_run_id: None,
            workflow_id: None,
            step_id: None,
            workflow_plan: None,
        };
        let id = svc.execute_script(request, Arc::new(Quiet)).await.unwrap();
        let mut finished = None;
        for _ in 0..400 {
            let run = svc.get_execution(&id).await.unwrap().unwrap();
            if matches!(run.status.as_str(), "success" | "failed") {
                finished = Some(run);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        let run = finished.expect("run did not finish");
        assert_eq!(run.status, "success", "{:?}", run.error);
        assert!(run.output.contains("hi"), "{:?}", run.output);
        assert!(svc.get_live_output(&id).await.join("").contains("hi\n"));

        let rows = Entity::find()
            .filter(Column::Id.eq(id.as_str()))
            .filter(Column::Kind.eq("action"))
            .all(conn)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        // And the one place a run can be Stopped has no run left to stop.
        let err = svc.cancel_execution(&id).await.unwrap_err();
        assert_eq!(err, "Run already finished (success)");
    }

    /// The run once `done` holds, polled every 50 ms for at most `secs`.
    async fn wait_for(
        svc: &ScriptExecutionService,
        id: &str,
        secs: u64,
        done: fn(&ScriptExecutionInfo) -> bool,
    ) -> ScriptExecutionInfo {
        for _ in 0..secs * 20 {
            let run = svc.get_execution(id).await.unwrap().unwrap();
            if done(&run) {
                return run;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        panic!("run {id} did not get there within {secs}s");
    }

    /// A Stop wins: the run ends `cancelled` with the Stop's message and no exit
    /// code, leaves the running list, and a second Stop has nothing to stop.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn script_facade_stop_wins() {
        let (svc, _db, dir) = service().await;
        let command = if cfg!(windows) {
            "ping -n 60 127.0.0.1 > nul"
        } else {
            "sleep 60"
        };
        let request = ExecuteScriptRequest {
            block_id: None,
            command: command.into(),
            parameters: HashMap::new(),
            working_directory: Some(dir.path().display().to_string()),
            env: None,
            triggered_by: None,
            workflow_run_id: None,
            workflow_id: None,
            step_id: None,
            workflow_plan: None,
        };
        let id = svc.execute_script(request, Arc::new(Quiet)).await.unwrap();
        // A pid means the process exists, so there is a tree to kill.
        wait_for(&svc, &id, 10, |r| r.status == "running" && r.pid.is_some()).await;
        svc.cancel_execution(&id).await.unwrap();
        let run = wait_for(&svc, &id, 15, |r| {
            matches!(r.status.as_str(), "success" | "failed" | "cancelled")
        })
        .await;
        assert_eq!(run.status, "cancelled", "{:?}", run.error);
        assert_eq!(run.exit_code, None);
        assert_eq!(run.error.as_deref(), Some("Stopped by user"));
        let running = svc.get_running_executions().await.unwrap();
        assert!(running.iter().all(|r| r.id != id), "{running:?}");
        let err = svc.cancel_execution(&id).await.unwrap_err();
        assert!(err.contains("already finished"), "{err}");
    }
}
