//! Coder agent state + the tool-calling loop.
//!
//! The loop is *resumable and stateless across calls*: all progress lives in
//! the thread transcript. A run advances until it either produces a final
//! assistant message or pauses on a tool call that needs human approval. On
//! approval we call [`CoderService::advance`] again with the granted call id
//! and it picks up exactly where it left off (a tool_call with no matching
//! `tool` response yet).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::sync::OnceLock;

use futures_util::StreamExt;

use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{oneshot, Mutex, Semaphore};

use crate::database::DatabaseManager;
use crate::domains::ai::chat_title::{
    fallback_title_from_message, is_placeholder_title, should_apply_generated_title,
    PLACEHOLDER_SESSION,
};
use crate::domains::ai::platform_config::{PlatformConfig, DESKTOP_CLIENT_ID};
use crate::domains::ai::services::AISettingsService;
use crate::domains::github::service::GitHubService;
use crate::domains::github::types::GitHubIssue;

use super::agent_mode;
use super::diff;
use super::entities::{coder_file_change, coder_setting, coder_sub_agent, coder_thread};
use super::multitask::{fallback_prompt_for_task, parse_issue_url, DEFAULT_MAX_PARALLEL_SUBAGENTS};
use super::permissions::{self, Decision};
use super::platform_stream::{self, PlatformDone};
use super::tools;
use super::types::{
    ChangeStatus, ChatMessage, CoderRunResult, CoderSubAgent, CoderSubAgentStatus, CoderThread,
    CoderThreadKind, CoderThreadSummary, FileChange, GitHubIssueRef, MultitaskCancelRequest,
    MultitaskCleanupRequest, MultitaskSpawnRequest, CoderAgentMode, PermissionMode, PermissionRule,
    SpawnSubAgentTask,
};
use super::worktree::{self, WorktreeSpec};

/// Persisted permission config (agent + permission modes + rules) stored as one JSON row.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SettingsBlob {
    #[serde(alias = "mode")]
    agent_mode: CoderAgentMode,
    #[serde(default)]
    permission_mode: PermissionMode,
    #[serde(default)]
    rules: Vec<PermissionRule>,
}

#[derive(Deserialize)]
struct LegacySettingsBlob {
    mode: Value,
    #[serde(default)]
    rules: Vec<PermissionRule>,
}

fn migrate_legacy_settings(mode: &Value, rules: Vec<PermissionRule>) -> SettingsBlob {
    match mode.as_str() {
        Some("review") => SettingsBlob {
            agent_mode: CoderAgentMode::Debug,
            permission_mode: PermissionMode::Review,
            rules,
        },
        Some("auto-accept-all") => SettingsBlob {
            agent_mode: CoderAgentMode::Auto,
            permission_mode: PermissionMode::AutoAcceptAll,
            rules,
        },
        Some("plan") => SettingsBlob {
            agent_mode: CoderAgentMode::Plan,
            permission_mode: PermissionMode::Review,
            rules,
        },
        Some("debug") => SettingsBlob {
            agent_mode: CoderAgentMode::Debug,
            permission_mode: PermissionMode::Review,
            rules,
        },
        Some("auto") => SettingsBlob {
            agent_mode: CoderAgentMode::Auto,
            permission_mode: PermissionMode::AutoAcceptAll,
            rules,
        },
        Some("ask") => SettingsBlob {
            agent_mode: CoderAgentMode::Ask,
            permission_mode: PermissionMode::Review,
            rules,
        },
        Some("multitask") => SettingsBlob {
            agent_mode: CoderAgentMode::Multitask,
            permission_mode: PermissionMode::AutoAcceptAll,
            rules,
        },
        _ => SettingsBlob {
            rules,
            ..SettingsBlob::default()
        },
    }
}

fn parse_settings_blob(data_json: &str) -> SettingsBlob {
    if let Ok(blob) = serde_json::from_str::<SettingsBlob>(data_json) {
        return blob;
    }
    if let Ok(legacy) = serde_json::from_str::<LegacySettingsBlob>(data_json) {
        return migrate_legacy_settings(&legacy.mode, legacy.rules);
    }
    SettingsBlob::default()
}

const SYSTEM_PROMPT: &str = concat!(
    "You are a coding agent operating directly in the user's native workspace via tools.\n",
    "Rules:\n",
    "- All paths are relative to the workspace root.\n",
    "- Explore before you change: use list_dir / read_file / search_files before editing.\n",
    "- For small changes use edit_file (exact-match replace). Use write_file only for new files or full rewrites; read first.\n",
    "- Prefer small, targeted changes. Do not touch files you were not asked to change.\n",
    "- Shell commands run in a background shell by default (fast). Pass terminal_id from list_terminals only when you need output in a specific interactive session tab.\n",
    "- When done, give a short summary of what you changed and why."
);

struct RunHandle {
    cancel: Arc<AtomicBool>,
}

struct PendingCommand {
    tx: oneshot::Sender<String>,
}

/// What the background agent task should run against agent-platform.
#[derive(Debug, Clone)]
pub enum AgentTurn {
    Send,
    Approve {
        call_id: String,
        approve: bool,
        edited_command: Option<String>,
    },
    Retry,
}

pub struct CoderService {
    db: Arc<DatabaseManager>,
    /// Set once at startup; used to stream loop-step events to the frontend.
    app_handle: OnceLock<AppHandle>,
    threads: Mutex<HashMap<String, CoderThread>>,
    sub_agents: Mutex<HashMap<String, CoderSubAgent>>,
    rules: Mutex<Vec<PermissionRule>>,
    agent_mode: Mutex<CoderAgentMode>,
    permission_mode: Mutex<PermissionMode>,
    /// Agent file edits awaiting or completed review (Cursor-style).
    changes: Mutex<Vec<FileChange>>,
    client: reqwest::Client,
    settings: AISettingsService,
    /// When set, the `delegate_task` tool is offered, targeting this team
    /// template on the platform. Configured via `CODER_TEAM_TEMPLATE_ID`.
    delegation_team_template_id: Option<i64>,
    /// In-flight agent loops keyed by thread id (for cancel + status).
    active_runs: Mutex<HashMap<String, RunHandle>>,
    /// Frontend-delegated run_command results keyed by `{thread_id}:{call_id}`.
    pending_commands: Mutex<HashMap<String, PendingCommand>>,
    /// Messages to prepend after an edit that rewinds the platform thread.
    edit_prefix: Mutex<HashMap<String, Vec<ChatMessage>>>,
    multitask_semaphore: Arc<Semaphore>,
}

impl CoderService {
    /// Build the service and hydrate threads + settings from the database.
    pub async fn new(db: Arc<DatabaseManager>) -> Self {
        let settings = AISettingsService::new();
        let delegation_team_template_id = std::env::var("CODER_TEAM_TEMPLATE_ID")
            .ok()
            .and_then(|v| v.trim().parse::<i64>().ok());
        let svc = Self {
            db,
            app_handle: OnceLock::new(),
            threads: Mutex::new(HashMap::new()),
            sub_agents: Mutex::new(HashMap::new()),
            rules: Mutex::new(Vec::new()),
            agent_mode: Mutex::new(CoderAgentMode::default()),
            permission_mode: Mutex::new(PermissionMode::default()),
            changes: Mutex::new(Vec::new()),
            client: reqwest::Client::new(),
            settings,
            delegation_team_template_id,
            active_runs: Mutex::new(HashMap::new()),
            pending_commands: Mutex::new(HashMap::new()),
            edit_prefix: Mutex::new(HashMap::new()),
            multitask_semaphore: Arc::new(Semaphore::new(DEFAULT_MAX_PARALLEL_SUBAGENTS)),
        };
        svc.load_from_db().await;
        svc
    }

    fn platform_config(&self) -> PlatformConfig {
        PlatformConfig::resolve(&self.settings)
    }

    /// Tool specs offered to the model, including platform delegation when
    /// configured.
    fn tool_specs_for_request(
        &self,
        thread_kind: CoderThreadKind,
        mode: CoderAgentMode,
    ) -> Vec<Value> {
        let mut specs: Vec<Value> = tools::tool_specs()
            .into_iter()
            .filter(|spec| {
                spec.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(Value::as_str)
                    .map(|name| agent_mode::includes_tool(mode, name))
                    .unwrap_or(false)
            })
            .collect();
        if matches!(thread_kind, CoderThreadKind::Coordinator)
            || mode == CoderAgentMode::Multitask
        {
            specs.extend(tools::multitask_tool_specs());
        }
        if self.delegation_team_template_id.is_some() && mode != CoderAgentMode::Ask {
            specs.extend(
                tools::platform_tool_specs()
                    .into_iter()
                    .filter(|spec| {
                        spec.get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(Value::as_str)
                            .map(|name| agent_mode::includes_tool(mode, name))
                            .unwrap_or(false)
                    }),
            );
        }
        specs
    }

    /// Execute a platform-delegated tool over HTTP. Currently `delegate_task`:
    /// start a multi-agent process, poll to completion, return its status.
    async fn execute_platform_tool(&self, tool: &str, args: &Value) -> Result<String, String> {
        match tool {
            "delegate_task" => {
                let team_id = self
                    .delegation_team_template_id
                    .ok_or("delegation not configured (set CODER_TEAM_TEMPLATE_ID)")?;
                let goal = args
                    .get("goal")
                    .and_then(Value::as_str)
                    .ok_or("missing arg: goal")?;

                let cfg = self.platform_config();
                let base = cfg.base_url.trim_end_matches('/');
                // Start the process.
                let start = self
                    .platform_post(
                        &format!("{base}/api/v1/processes"),
                        json!({
                            "goal": goal,
                            "auto_approve": true,
                            "team_template_id": team_id,
                            "client_id": "portal-coder",
                        }),
                    )
                    .await?;
                let process_id = start
                    .get("process_id")
                    .and_then(|v| {
                        v.as_i64()
                            .map(|n| n.to_string())
                            .or_else(|| v.as_str().map(str::to_string))
                    })
                    .ok_or_else(|| format!("no process_id in response: {start}"))?;

                // Poll until terminal or timeout (~90s).
                let url = format!("{base}/api/v1/processes/{process_id}");
                for _ in 0..60 {
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    let status = self.platform_get(&url).await?;
                    let state = status
                        .get("status")
                        .or_else(|| status.get("process").and_then(|p| p.get("status")))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_lowercase();
                    if state.contains("complete")
                        || state.contains("done")
                        || state.contains("failed")
                        || state.contains("error")
                        || state.contains("cancel")
                    {
                        let text = serde_json::to_string(&status).unwrap_or_default();
                        return Ok(truncate(&text, 8000));
                    }
                }
                Ok(format!(
                    "process {process_id} still running after timeout; poll {url}"
                ))
            }
            other => Err(format!("unknown platform tool: {other}")),
        }
    }

    async fn platform_post(&self, url: &str, body: Value) -> Result<Value, String> {
        let cfg = self.platform_config();
        let mut req = self
            .client
            .post(url)
            .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
            .json(&body);
        if let Some(key) = &cfg.api_token {
            req = req.bearer_auth(key);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("platform POST failed: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("platform {status}: {text}"));
        }
        serde_json::from_str(&text).map_err(|e| format!("bad platform JSON: {e}"))
    }

    async fn platform_get(&self, url: &str) -> Result<Value, String> {
        let cfg = self.platform_config();
        let mut req = self
            .client
            .get(url)
            .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID);
        if let Some(key) = &cfg.api_token {
            req = req.bearer_auth(key);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("platform GET failed: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("platform {status}: {text}"));
        }
        serde_json::from_str(&text).map_err(|e| format!("bad platform JSON: {e}"))
    }

    async fn execute_delegated_tool(
        &self,
        local_thread_id: &str,
        platform_thread_id: i64,
        workspace_root: &str,
        call_id: &str,
        tool: &str,
        args: &Value,
    ) -> Result<String, String> {
        let mutated = tools::mutated_path(tool, args);
        let before = mutated
            .as_ref()
            .and_then(|path| tools::read_raw(workspace_root, path));

        let result = if tool == "run_command" {
            self.execute_run_command(local_thread_id, call_id, workspace_root, args)
                .await
        } else if tool == "spawn_parallel_tasks" {
            let request = serde_json::from_value::<MultitaskSpawnRequest>(json!({
                "coordinatorThreadId": local_thread_id,
                "baseRef": args.get("base_ref").cloned().unwrap_or(Value::Null),
                "tasks": args.get("tasks").cloned().unwrap_or_else(|| json!([])),
                "issueUrls": args.get("issue_urls").cloned().unwrap_or(Value::Null),
            }))
            .map_err(|e| format!("invalid multitask request: {e}"))?;
            let started = self.service_arc()?.multitask_spawn(request).await?;
            serde_json::to_string(&started).unwrap_or_else(|_| "[]".to_string())
        } else if tool == "list_terminals" {
            self.emit(
                "coder://list_terminals",
                json!({ "thread_id": local_thread_id, "call_id": call_id }),
            );
            match self.wait_terminal_list(local_thread_id, call_id).await {
                Ok(list) => list,
                Err(_) => "[]".to_string(),
            }
        } else {
            match tools::execute(workspace_root, tool, args) {
                Ok(output) => output,
                Err(e) => format!("Error: {e}"),
            }
        };

        if !result.starts_with("Error: ") {
            if let Some(path) = mutated {
                self.record_change(local_thread_id, tool, workspace_root, path, before)
                    .await;
            }
        }

        let cfg = self.platform_config();
        let base = cfg.base_url.trim_end_matches('/');
        let body = json!({
            "thread_id": platform_thread_id,
            "call_id": call_id,
            "result": result,
        });
        self.platform_post(&format!("{base}/api/v1/coder/chat/tool-result"), body)
            .await?;
        Ok(result)
    }

    /// Wire the Tauri handle so loop steps can be streamed to the UI.
    pub fn init_app_handle(&self, handle: AppHandle) {
        let _ = self.app_handle.set(handle);
    }

    fn service_arc(&self) -> Result<Arc<Self>, String> {
        let handle = self.app_handle.get().ok_or("app handle not initialized")?;
        let state: tauri::State<'_, Arc<CoderService>> = handle.state();
        Ok(state.inner().clone())
    }

    fn pending_key(thread_id: &str, call_id: &str) -> String {
        format!("{thread_id}:{call_id}")
    }

    /// Run a command in a background shell by default; delegate to the UI
    /// terminal only when the agent passes an explicit `terminal_id`.
    async fn execute_run_command(
        &self,
        thread_id: &str,
        call_id: &str,
        workspace_root: &str,
        args: &Value,
    ) -> String {
        let terminal_id = args
            .get("terminal_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let delegate_to_ui = matches!(terminal_id, Some(id) if id != "new");

        if delegate_to_ui {
            return self
                .execute_run_command_via_terminal(thread_id, call_id, workspace_root, args)
                .await;
        }

        let workspace = workspace_root.to_string();
        let args = args.clone();
        tokio::task::spawn_blocking(move || tools::execute(&workspace, "run_command", &args))
            .await
            .unwrap_or_else(|e| Err(format!("command task failed: {e}")))
            .unwrap_or_else(|e| format!("Error: {e}"))
    }

    /// Delegate run_command to the frontend terminal stack (interactive session).
    async fn execute_run_command_via_terminal(
        &self,
        thread_id: &str,
        call_id: &str,
        workspace_root: &str,
        args: &Value,
    ) -> String {
        let command = args.get("command").and_then(Value::as_str).unwrap_or("");
        let terminal_id = args.get("terminal_id").and_then(Value::as_str);

        let key = Self::pending_key(thread_id, call_id);
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_commands.lock().await;
            pending.insert(key.clone(), PendingCommand { tx });
        }

        self.emit(
            "coder://run_command",
            json!({
                "thread_id": thread_id,
                "call_id": call_id,
                "command": command,
                "terminal_id": terminal_id,
                "workspace_root": workspace_root,
            }),
        );

        match tokio::time::timeout(std::time::Duration::from_secs(120), rx).await {
            Ok(Ok(result)) => result,
            _ => {
                self.pending_commands.lock().await.remove(&key);
                match tools::execute(workspace_root, "run_command", args) {
                    Ok(output) => output,
                    Err(e) => format!("Error: {e}"),
                }
            }
        }
    }

    async fn wait_terminal_list(&self, thread_id: &str, call_id: &str) -> Result<String, ()> {
        let key = Self::pending_key(thread_id, call_id);
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_commands.lock().await;
            pending.insert(key.clone(), PendingCommand { tx });
        }

        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(result)) => Ok(result),
            _ => {
                self.pending_commands.lock().await.remove(&key);
                Err(())
            }
        }
    }

    /// Frontend submits the captured output for a delegated run_command.
    pub async fn submit_command_result(
        &self,
        thread_id: String,
        call_id: String,
        result: String,
    ) -> Result<(), String> {
        let key = Self::pending_key(&thread_id, &call_id);
        let tx = self
            .pending_commands
            .lock()
            .await
            .remove(&key)
            .map(|p| p.tx);
        if let Some(tx) = tx {
            let _ = tx.send(result);
        }
        Ok(())
    }

    /// Frontend responds to list_terminals with JSON array text.
    pub async fn submit_terminal_list(
        &self,
        thread_id: String,
        call_id: String,
        list_json: String,
    ) -> Result<(), String> {
        self.submit_command_result(thread_id, call_id, list_json)
            .await
    }

    fn emit(&self, event: &str, payload: Value) {
        if let Some(h) = self.app_handle.get() {
            let _ = h.emit(event, payload);
        }
    }

    async fn load_from_db(&self) {
        let conn = self.db.get_connection();

        match coder_thread::Entity::find().all(conn).await {
            Ok(rows) => {
                let mut threads = self.threads.lock().await;
                for row in rows {
                    let messages: Vec<ChatMessage> =
                        serde_json::from_str(&row.messages_json).unwrap_or_default();
                    threads.insert(
                        row.id.clone(),
                        CoderThread {
                            id: row.id,
                            title: row.title,
                            workspace_root: row.workspace_root,
                            project_id: row.project_id,
                            model: row.model,
                            platform_thread_id: row.platform_thread_id,
                            llm_provider: row.llm_provider,
                            messages,
                            thread_kind: parse_thread_kind(&row.thread_kind),
                            created_at: row.created_at,
                            updated_at: row.updated_at,
                        },
                    );
                }
            }
            Err(e) => eprintln!("coder: load threads failed: {e}"),
        }

        if let Ok(Some(row)) = coder_setting::Entity::find_by_id("default").one(conn).await {
            let blob = parse_settings_blob(&row.data_json);
            *self.agent_mode.lock().await = blob.agent_mode;
            *self.permission_mode.lock().await = blob.permission_mode;
            *self.rules.lock().await = blob.rules;
        }

        if let Ok(rows) = coder_file_change::Entity::find().all(conn).await {
            let mut changes = self.changes.lock().await;
            for row in rows {
                if let Ok(fc) = serde_json::from_str::<FileChange>(&row.data_json) {
                    changes.push(fc);
                }
            }
        }

        if let Ok(rows) = coder_sub_agent::Entity::find().all(conn).await {
            let mut sub_agents = self.sub_agents.lock().await;
            for row in rows {
                sub_agents.insert(row.id.clone(), self.sub_agent_from_row(row));
            }
        }
    }

    async fn persist_change(&self, change: &FileChange) {
        let data_json = serde_json::to_string(change).unwrap_or_else(|_| "{}".into());
        let status = serde_json::to_value(change.status)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "pending".into());
        let am = coder_file_change::ActiveModel {
            id: Set(change.id.clone()),
            thread_id: Set(change.thread_id.clone()),
            path: Set(change.path.clone()),
            status: Set(status),
            data_json: Set(data_json),
            created_at: Set(change.created_at.clone()),
        };
        let res = coder_file_change::Entity::insert(am)
            .on_conflict(
                OnConflict::column(coder_file_change::Column::Id)
                    .update_columns([
                        coder_file_change::Column::Status,
                        coder_file_change::Column::DataJson,
                    ])
                    .to_owned(),
            )
            .exec(self.db.get_connection())
            .await;
        if let Err(e) = res {
            eprintln!("coder: persist change failed: {e}");
        }
    }

    /// Record a file edit for review. The edit is already on disk; we snapshot
    /// `before`, read the resulting content, and compute reviewable hunks.
    async fn record_change(
        &self,
        thread_id: &str,
        tool: &str,
        root: &str,
        path: String,
        before: Option<String>,
    ) {
        let after = tools::read_raw(root, &path).unwrap_or_default();
        let before_str = before.clone().unwrap_or_default();
        if after == before_str {
            return; // tool failed or no-op; nothing to review
        }
        let change = FileChange {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.to_string(),
            path,
            tool: tool.to_string(),
            hunks: diff::compute_hunks(&before_str, &after),
            before: before_str,
            original_after: after,
            created: before.is_none(),
            status: ChangeStatus::Pending,
            created_at: now_iso(),
        };
        self.changes.lock().await.push(change.clone());
        self.persist_change(&change).await;
        self.emit("coder://change", json!({ "change": change }));
    }

    // ---- change review (public) ---------------------------------------

    pub async fn list_changes(&self, thread_id: Option<&str>) -> Vec<FileChange> {
        let changes = self.changes.lock().await;
        changes
            .iter()
            .filter(|c| thread_id.map(|t| c.thread_id == t).unwrap_or(true))
            .cloned()
            .collect()
    }

    /// Rewrite the file on disk from `before` + the current hunk decisions and
    /// persist the change.
    async fn rewrite_from_hunks(&self, change: &FileChange, root: &str) -> Result<(), String> {
        let content = diff::rebuild(&change.before, &change.hunks);
        tools::write_raw(root, &change.path, &content)
    }

    async fn workspace_root_for(&self, thread_id: &str) -> Option<String> {
        self.get_thread(thread_id).await.map(|t| t.workspace_root)
    }

    /// Accept the whole change: all hunks kept, disk = original_after.
    pub async fn accept_change(&self, change_id: &str) -> Result<(), String> {
        self.update_change(change_id, |c| {
            for h in &mut c.hunks {
                h.accepted = true;
            }
            c.status = ChangeStatus::Accepted;
        })
        .await
    }

    /// Reject the whole change: disk restored to `before`.
    pub async fn reject_change(&self, change_id: &str) -> Result<(), String> {
        self.update_change(change_id, |c| {
            for h in &mut c.hunks {
                h.accepted = false;
            }
            c.status = ChangeStatus::Rejected;
        })
        .await
    }

    /// Toggle a single hunk and rewrite the file.
    pub async fn set_hunk(
        &self,
        change_id: &str,
        hunk_index: usize,
        accepted: bool,
    ) -> Result<(), String> {
        self.update_change(change_id, |c| {
            if let Some(h) = c.hunks.iter_mut().find(|h| h.index == hunk_index) {
                h.accepted = accepted;
            }
            // Any per-hunk touch means it's under active review, not final.
            c.status = ChangeStatus::Pending;
        })
        .await
    }

    /// Replace the file content wholesale (user's manual edit) and recompute
    /// the change against the original `before`.
    pub async fn modify_change(&self, change_id: &str, new_content: String) -> Result<(), String> {
        let (thread_id, path, before) = {
            let changes = self.changes.lock().await;
            let c = changes
                .iter()
                .find(|c| c.id == change_id)
                .ok_or("change not found")?;
            (c.thread_id.clone(), c.path.clone(), c.before.clone())
        };
        let root = self
            .workspace_root_for(&thread_id)
            .await
            .ok_or("thread not found")?;
        tools::write_raw(&root, &path, &new_content)?;
        let mut changes = self.changes.lock().await;
        if let Some(c) = changes.iter_mut().find(|c| c.id == change_id) {
            c.hunks = diff::compute_hunks(&before, &new_content);
            c.original_after = new_content;
            c.status = ChangeStatus::Pending;
            let snapshot = c.clone();
            drop(changes);
            self.persist_change(&snapshot).await;
            self.emit("coder://change", json!({ "change": snapshot }));
        }
        Ok(())
    }

    /// Shared helper: mutate a change in memory, rewrite disk, persist, emit.
    async fn update_change(
        &self,
        change_id: &str,
        mutate: impl FnOnce(&mut FileChange),
    ) -> Result<(), String> {
        let (snapshot, root) = {
            let mut changes = self.changes.lock().await;
            let c = changes
                .iter_mut()
                .find(|c| c.id == change_id)
                .ok_or("change not found")?;
            mutate(c);
            let snapshot = c.clone();
            drop(changes);
            let root = self
                .workspace_root_for(&snapshot.thread_id)
                .await
                .ok_or("thread not found")?;
            (snapshot, root)
        };
        self.rewrite_from_hunks(&snapshot, &root).await?;
        self.persist_change(&snapshot).await;
        self.emit("coder://change", json!({ "change": snapshot }));
        Ok(())
    }

    async fn persist_thread(&self, thread_id: &str) {
        let thread = match self.get_thread(thread_id).await {
            Some(t) => t,
            None => return,
        };
        let messages_json = serde_json::to_string(&thread.messages).unwrap_or_else(|_| "[]".into());
        let am = coder_thread::ActiveModel {
            id: Set(thread.id.clone()),
            title: Set(thread.title),
            workspace_root: Set(thread.workspace_root),
            project_id: Set(thread.project_id),
            model: Set(thread.model),
            platform_thread_id: Set(thread.platform_thread_id),
            llm_provider: Set(thread.llm_provider.clone()),
            messages_json: Set(messages_json),
            thread_kind: Set(thread_kind_str(thread.thread_kind).to_string()),
            created_at: Set(thread.created_at),
            updated_at: Set(thread.updated_at),
        };
        let res = coder_thread::Entity::insert(am)
            .on_conflict(
                OnConflict::column(coder_thread::Column::Id)
                    .update_columns([
                        coder_thread::Column::Title,
                        coder_thread::Column::WorkspaceRoot,
                        coder_thread::Column::ProjectId,
                        coder_thread::Column::Model,
                        coder_thread::Column::PlatformThreadId,
                        coder_thread::Column::LlmProvider,
                        coder_thread::Column::MessagesJson,
                        coder_thread::Column::ThreadKind,
                        coder_thread::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(self.db.get_connection())
            .await;
        if let Err(e) = res {
            eprintln!("coder: persist thread failed: {e}");
        }
    }

    async fn persist_settings(&self) {
        let blob = SettingsBlob {
            agent_mode: *self.agent_mode.lock().await,
            permission_mode: *self.permission_mode.lock().await,
            rules: self.rules.lock().await.clone(),
        };
        let data_json = serde_json::to_string(&blob).unwrap_or_else(|_| "{}".into());
        let am = coder_setting::ActiveModel {
            id: Set("default".to_string()),
            data_json: Set(data_json),
        };
        let res = coder_setting::Entity::insert(am)
            .on_conflict(
                OnConflict::column(coder_setting::Column::Id)
                    .update_column(coder_setting::Column::DataJson)
                    .to_owned(),
            )
            .exec(self.db.get_connection())
            .await;
        if let Err(e) = res {
            eprintln!("coder: persist settings failed: {e}");
        }
    }

    // ---- thread CRUD ---------------------------------------------------

    pub async fn create_thread(
        &self,
        workspace_root: String,
        model: Option<String>,
        llm_provider: Option<String>,
        thread_kind: Option<CoderThreadKind>,
        project_id: Option<i32>,
    ) -> CoderThread {
        self.create_thread_with_kind(
            workspace_root,
            model,
            llm_provider,
            thread_kind.unwrap_or(CoderThreadKind::Session),
            project_id,
        )
        .await
    }

    async fn create_thread_with_kind(
        &self,
        workspace_root: String,
        model: Option<String>,
        llm_provider: Option<String>,
        thread_kind: CoderThreadKind,
        project_id: Option<i32>,
    ) -> CoderThread {
        let now = now_iso();
        let thread = CoderThread {
            id: uuid::Uuid::new_v4().to_string(),
            title: PLACEHOLDER_SESSION.into(),
            workspace_root,
            project_id,
            model: model.or_else(|| self.platform_config().default_model.clone()),
            platform_thread_id: None,
            llm_provider,
            messages: vec![ChatMessage::system(SYSTEM_PROMPT)],
            thread_kind,
            created_at: now.clone(),
            updated_at: now,
        };
        self.threads
            .lock()
            .await
            .insert(thread.id.clone(), thread.clone());
        self.persist_thread(&thread.id).await;
        thread
    }

    pub async fn list_threads(&self) -> Vec<CoderThread> {
        let mut v: Vec<_> = self.threads.lock().await.values().cloned().collect();
        v.retain(|t| !matches!(t.thread_kind, CoderThreadKind::SubAgent));
        v.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        v
    }

    /// Sidebar-friendly thread list without serializing full transcripts.
    pub async fn list_thread_summaries(&self) -> Vec<CoderThreadSummary> {
        use std::collections::HashSet;

        let running_ids: HashSet<String> = self.active_runs.lock().await.keys().cloned().collect();
        let threads = self.threads.lock().await;
        let mut summaries: Vec<CoderThreadSummary> = threads
            .values()
            .filter(|t| !matches!(t.thread_kind, CoderThreadKind::SubAgent))
            .map(|t| {
                let message_count = t
                    .messages
                    .iter()
                    .filter(|m| m.role == "user" || m.role == "assistant")
                    .count();
                CoderThreadSummary {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    workspace_root: t.workspace_root.clone(),
                    project_id: t.project_id,
                    model: t.model.clone(),
                    platform_thread_id: t.platform_thread_id,
                    llm_provider: t.llm_provider.clone(),
                    created_at: t.created_at.clone(),
                    updated_at: t.updated_at.clone(),
                    thread_kind: t.thread_kind,
                    message_count,
                    is_running: running_ids.contains(&t.id),
                }
            })
            .collect();
        drop(threads);

        for id in &running_ids {
            if summaries.iter().any(|s| &s.id == id) {
                continue;
            }
            summaries.push(CoderThreadSummary {
                id: id.clone(),
                title: "Running session".into(),
                workspace_root: String::new(),
                project_id: None,
                model: None,
                platform_thread_id: None,
                llm_provider: None,
                created_at: String::new(),
                updated_at: now_iso(),
                thread_kind: CoderThreadKind::Session,
                message_count: 0,
                is_running: true,
            });
        }

        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    pub async fn get_thread(&self, id: &str) -> Option<CoderThread> {
        self.threads.lock().await.get(id).cloned()
    }

    pub async fn delete_thread(&self, id: &str) -> bool {
        let removed = self.threads.lock().await.remove(id).is_some();
        if removed {
            let _ = coder_thread::Entity::delete_by_id(id.to_string())
                .exec(self.db.get_connection())
                .await;
        }
        removed
    }

    /// Change the model / backend provider for an existing thread (mid-session switching).
    pub async fn update_thread_model(
        &self,
        id: &str,
        model: Option<String>,
        llm_provider: Option<String>,
    ) -> Result<(), String> {
        {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(id).ok_or("thread not found")?;
            if model.is_some() {
                thread.model = model;
            }
            if llm_provider.is_some() {
                thread.llm_provider = llm_provider;
            }
            thread.updated_at = now_iso();
        }
        self.persist_thread(id).await;
        Ok(())
    }

    pub async fn set_thread_kind(
        &self,
        id: &str,
        thread_kind: CoderThreadKind,
    ) -> Result<(), String> {
        {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(id).ok_or("thread not found")?;
            thread.thread_kind = thread_kind;
            thread.updated_at = now_iso();
        }
        self.persist_thread(id).await;
        Ok(())
    }

    fn sub_agent_from_row(&self, row: coder_sub_agent::Model) -> CoderSubAgent {
        CoderSubAgent {
            id: row.id,
            coordinator_thread_id: row.coordinator_thread_id,
            child_thread_id: row.child_thread_id,
            title: row.title,
            workspace_root: row.workspace_root,
            branch: row.branch,
            status: parse_sub_agent_status(&row.status),
            github_owner: row.github_owner,
            github_repo: row.github_repo,
            github_issue_number: row.github_issue_number,
            github_issue_url: row.github_issue_url,
            result_summary: row.result_summary,
            error: row.error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    async fn persist_sub_agent(&self, sub_agent: &CoderSubAgent) {
        let am = coder_sub_agent::ActiveModel {
            id: Set(sub_agent.id.clone()),
            coordinator_thread_id: Set(sub_agent.coordinator_thread_id.clone()),
            child_thread_id: Set(sub_agent.child_thread_id.clone()),
            title: Set(sub_agent.title.clone()),
            workspace_root: Set(sub_agent.workspace_root.clone()),
            branch: Set(sub_agent.branch.clone()),
            status: Set(sub_agent_status_str(sub_agent.status).to_string()),
            github_owner: Set(sub_agent.github_owner.clone()),
            github_repo: Set(sub_agent.github_repo.clone()),
            github_issue_number: Set(sub_agent.github_issue_number),
            github_issue_url: Set(sub_agent.github_issue_url.clone()),
            result_summary: Set(sub_agent.result_summary.clone()),
            error: Set(sub_agent.error.clone()),
            created_at: Set(sub_agent.created_at.clone()),
            updated_at: Set(sub_agent.updated_at.clone()),
        };
        let res = coder_sub_agent::Entity::insert(am)
            .on_conflict(
                OnConflict::column(coder_sub_agent::Column::Id)
                    .update_columns([
                        coder_sub_agent::Column::Title,
                        coder_sub_agent::Column::WorkspaceRoot,
                        coder_sub_agent::Column::Branch,
                        coder_sub_agent::Column::Status,
                        coder_sub_agent::Column::GithubOwner,
                        coder_sub_agent::Column::GithubRepo,
                        coder_sub_agent::Column::GithubIssueNumber,
                        coder_sub_agent::Column::GithubIssueUrl,
                        coder_sub_agent::Column::ResultSummary,
                        coder_sub_agent::Column::Error,
                        coder_sub_agent::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(self.db.get_connection())
            .await;
        if let Err(e) = res {
            eprintln!("coder: persist sub-agent failed: {e}");
        }
    }

    async fn upsert_sub_agent(&self, sub_agent: CoderSubAgent) {
        self.sub_agents
            .lock()
            .await
            .insert(sub_agent.id.clone(), sub_agent.clone());
        self.persist_sub_agent(&sub_agent).await;
    }

    pub async fn list_sub_agents(&self, coordinator_thread_id: &str) -> Vec<CoderSubAgent> {
        let mut items: Vec<_> = self
            .sub_agents
            .lock()
            .await
            .values()
            .filter(|s| s.coordinator_thread_id == coordinator_thread_id)
            .cloned()
            .collect();
        items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        items
    }

    pub async fn multitask_spawn(
        self: &Arc<Self>,
        request: MultitaskSpawnRequest,
    ) -> Result<Vec<CoderSubAgent>, String> {
        let coordinator = self
            .get_thread(&request.coordinator_thread_id)
            .await
            .ok_or("coordinator thread not found")?;
        if !matches!(coordinator.thread_kind, CoderThreadKind::Coordinator) {
            return Err("multitask spawn requires a coordinator thread".into());
        }

        let workspace_root = PathBuf::from(coordinator.workspace_root.clone());
        validate_workspace_root(&coordinator.workspace_root)?;
        if !worktree::is_git_repo(&workspace_root) {
            return Err("multitask requires a git workspace".into());
        }

        let base_ref = request
            .base_ref
            .clone()
            .unwrap_or_else(|| "HEAD".to_string())
            .trim()
            .to_string();

        let tasks = self.expand_multitask_tasks(&request).await?;
        if tasks.is_empty() {
            return Err("add at least one task or issue URL".into());
        }

        let root_dir = worktree::worktree_root(&workspace_root, &request.coordinator_thread_id);
        std::fs::create_dir_all(&root_dir).map_err(|e| format!("create worktree root: {e}"))?;

        let repo_dirty = worktree::has_uncommitted_changes(&workspace_root);
        let semaphore = Arc::clone(&self.multitask_semaphore);
        let mut started = Vec::new();

        for (index, task) in tasks.into_iter().enumerate() {
            let slug = build_sub_agent_slug(index, &task.title);
            let branch = format!(
                "portal/{}/{}",
                short_id(&request.coordinator_thread_id),
                slug
            );
            let agent_root = root_dir.join(&slug);
            worktree::create_worktree(
                &workspace_root,
                &WorktreeSpec {
                    path: agent_root.clone(),
                    branch: branch.clone(),
                    base_ref: base_ref.clone(),
                },
            )?;

            let issue = resolve_task_issue_ref(&task);
            let child_thread = self
                .create_thread_with_kind(
                    agent_root.to_string_lossy().to_string(),
                    coordinator.model.clone(),
                    coordinator.llm_provider.clone(),
                    CoderThreadKind::SubAgent,
                    coordinator.project_id,
                )
                .await;
            {
                let mut threads = self.threads.lock().await;
                if let Some(thread) = threads.get_mut(&child_thread.id) {
                    thread.title = task.title.clone();
                    thread.updated_at = now_iso();
                }
            }
            self.persist_thread(&child_thread.id).await;

            let now = now_iso();
            let sub_agent = CoderSubAgent {
                id: uuid::Uuid::new_v4().to_string(),
                coordinator_thread_id: request.coordinator_thread_id.clone(),
                child_thread_id: child_thread.id.clone(),
                title: task.title.clone(),
                workspace_root: agent_root.to_string_lossy().to_string(),
                branch: branch.clone(),
                status: CoderSubAgentStatus::Pending,
                github_owner: issue.as_ref().map(|i| i.owner.clone()),
                github_repo: issue.as_ref().map(|i| i.repo.clone()),
                github_issue_number: issue.as_ref().map(|i| i.number),
                github_issue_url: issue.and_then(|i| i.url),
                result_summary: Some(if repo_dirty {
                    "Spawned from committed HEAD because the main workspace has uncommitted changes.".into()
                } else {
                    format!("Spawned from {base_ref}.")
                }),
                error: None,
                created_at: now.clone(),
                updated_at: now,
            };
            self.upsert_sub_agent(sub_agent.clone()).await;
            self.emit(
                "coder://subagent-started",
                json!({
                    "coordinator_id": sub_agent.coordinator_thread_id,
                    "subagent": sub_agent,
                }),
            );

            let svc = Arc::clone(self);
            let prompt = fallback_prompt_for_task(&task);
            let child_thread_id = child_thread.id.clone();
            let sub_agent_id = sub_agent.id.clone();
            let semaphore = Arc::clone(&semaphore);
            tauri::async_runtime::spawn(async move {
                let Ok(_permit) = semaphore.acquire_owned().await else {
                    let _ = svc
                        .mark_sub_agent_status(
                            &sub_agent_id,
                            CoderSubAgentStatus::Failed,
                            None,
                            Some("parallel limiter unavailable".into()),
                        )
                        .await;
                    return;
                };
                let _ = svc
                    .mark_sub_agent_status(
                        &sub_agent_id,
                        CoderSubAgentStatus::Running,
                        Some("Starting agent run.".into()),
                        None,
                    )
                    .await;
                if let Err(e) = svc.prepare_send(&child_thread_id, prompt).await {
                    let _ = svc
                        .mark_sub_agent_status(
                            &sub_agent_id,
                            CoderSubAgentStatus::Failed,
                            None,
                            Some(e),
                        )
                        .await;
                    return;
                }
                CoderService::spawn_run(Arc::clone(&svc), child_thread_id, AgentTurn::Send);
            });
            started.push(sub_agent);
        }

        Ok(started)
    }

    pub async fn multitask_cancel(
        &self,
        request: MultitaskCancelRequest,
    ) -> Result<Vec<CoderSubAgent>, String> {
        let targets = self.cancel_targets(&request).await?;
        for sub_agent in &targets {
            self.stop(&sub_agent.child_thread_id).await;
            self.mark_sub_agent_status(
                &sub_agent.id,
                CoderSubAgentStatus::Cancelled,
                None,
                Some("Cancelled by user.".into()),
            )
            .await?;
        }
        Ok(self.list_sub_agents(&request.coordinator_thread_id).await)
    }

    pub async fn multitask_cleanup(
        &self,
        request: MultitaskCleanupRequest,
    ) -> Result<Vec<CoderSubAgent>, String> {
        let coordinator = self
            .get_thread(&request.coordinator_thread_id)
            .await
            .ok_or("coordinator thread not found")?;
        let repo_root = PathBuf::from(coordinator.workspace_root.clone());
        let targets = if request.sub_agent_ids.is_empty() {
            self.list_sub_agents(&request.coordinator_thread_id).await
        } else {
            let wanted: std::collections::HashSet<_> =
                request.sub_agent_ids.iter().cloned().collect();
            self.list_sub_agents(&request.coordinator_thread_id)
                .await
                .into_iter()
                .filter(|s| wanted.contains(&s.id))
                .collect()
        };
        for sub_agent in &targets {
            worktree::remove_worktree(
                &repo_root,
                Path::new(&sub_agent.workspace_root),
                request.force,
            )?;
        }
        Ok(targets)
    }

    async fn cancel_targets(
        &self,
        request: &MultitaskCancelRequest,
    ) -> Result<Vec<CoderSubAgent>, String> {
        let sub_agents = self.list_sub_agents(&request.coordinator_thread_id).await;
        if let Some(id) = &request.sub_agent_id {
            let sub_agent = sub_agents
                .into_iter()
                .find(|s| &s.id == id)
                .ok_or("sub-agent not found")?;
            Ok(vec![sub_agent])
        } else {
            Ok(sub_agents)
        }
    }

    async fn expand_multitask_tasks(
        &self,
        request: &MultitaskSpawnRequest,
    ) -> Result<Vec<SpawnSubAgentTask>, String> {
        let mut tasks = request.tasks.clone();
        if let Some(issue_urls) = &request.issue_urls {
            for url in issue_urls {
                tasks.push(SpawnSubAgentTask {
                    title: String::new(),
                    prompt: String::new(),
                    github_issue: None,
                    github_issue_url: Some(url.clone()),
                });
            }
        }

        let mut expanded = Vec::new();
        for task in tasks {
            expanded.push(self.hydrate_task(task).await?);
        }
        Ok(expanded)
    }

    async fn hydrate_task(&self, mut task: SpawnSubAgentTask) -> Result<SpawnSubAgentTask, String> {
        if task.github_issue.is_none() {
            if let Some(url) = task.github_issue_url.as_deref() {
                task.github_issue = parse_issue_url(url);
            }
        }
        if let Some(issue_ref) = &task.github_issue {
            let issue = self.fetch_github_issue(issue_ref).await?;
            if task.title.trim().is_empty() {
                task.title = format!("Issue #{} - {}", issue.number, issue.title);
            }
            if task.prompt.trim().is_empty() {
                let issue_url = issue_ref
                    .url
                    .clone()
                    .unwrap_or_else(|| issue.html_url.clone());
                task.prompt = build_issue_prompt(&issue_url, &issue);
            }
        }
        if task.title.trim().is_empty() {
            return Err("multitask task title is required".into());
        }
        Ok(task)
    }

    async fn fetch_github_issue(&self, issue_ref: &GitHubIssueRef) -> Result<GitHubIssue, String> {
        GitHubService::new(Arc::clone(&self.db))
            .get_issue(&issue_ref.owner, &issue_ref.repo, issue_ref.number)
            .await
    }

    async fn mark_sub_agent_status(
        &self,
        sub_agent_id: &str,
        status: CoderSubAgentStatus,
        summary: Option<String>,
        error: Option<String>,
    ) -> Result<(), String> {
        let snapshot = {
            let mut sub_agents = self.sub_agents.lock().await;
            let sub_agent = sub_agents
                .get_mut(sub_agent_id)
                .ok_or("sub-agent not found")?;
            sub_agent.status = status;
            sub_agent.updated_at = now_iso();
            if let Some(summary) = summary {
                sub_agent.result_summary = Some(summary);
            }
            if error.is_some() {
                sub_agent.error = error;
            }
            sub_agent.clone()
        };
        self.persist_sub_agent(&snapshot).await;
        self.emit(
            "coder://subagent-progress",
            json!({
                "coordinator_id": snapshot.coordinator_thread_id,
                "subagent": snapshot,
            }),
        );
        Ok(())
    }

    async fn sync_sub_agent_from_thread(
        &self,
        child_thread_id: &str,
        status: CoderSubAgentStatus,
        error: Option<String>,
    ) {
        let sub_agent = {
            let sub_agents = self.sub_agents.lock().await;
            sub_agents
                .values()
                .find(|s| s.child_thread_id == child_thread_id)
                .cloned()
        };
        let Some(sub_agent) = sub_agent else {
            return;
        };

        let summary = self.get_thread(child_thread_id).await.and_then(|thread| {
            thread
                .messages
                .iter()
                .rev()
                .find(|m| m.role == "assistant" && m.tool_calls.is_none())
                .and_then(|m| m.content.clone())
        });

        let _ = self
            .mark_sub_agent_status(&sub_agent.id, status, summary, error)
            .await;

        if matches!(
            status,
            CoderSubAgentStatus::Completed
                | CoderSubAgentStatus::Failed
                | CoderSubAgentStatus::Cancelled
        ) {
            let finished = self.sub_agents.lock().await.get(&sub_agent.id).cloned();
            self.emit(
                "coder://subagent-finished",
                json!({
                    "coordinator_id": sub_agent.coordinator_thread_id,
                    "subagent": finished,
                }),
            );
            let subagents = self.list_sub_agents(&sub_agent.coordinator_thread_id).await;
            let all_done = subagents.iter().all(|item| {
                matches!(
                    item.status,
                    CoderSubAgentStatus::Completed
                        | CoderSubAgentStatus::Failed
                        | CoderSubAgentStatus::Cancelled
                )
            });
            if all_done {
                self.emit(
                    "coder://multitask-complete",
                    json!({
                        "coordinator_id": sub_agent.coordinator_thread_id,
                        "subagents": subagents,
                    }),
                );
            }
        }
    }

    // ---- permission config --------------------------------------------

    pub async fn set_agent_mode(&self, mode: CoderAgentMode) {
        *self.agent_mode.lock().await = mode;
        self.persist_settings().await;
    }
    pub async fn get_agent_mode(&self) -> CoderAgentMode {
        *self.agent_mode.lock().await
    }
    pub async fn set_permission_mode(&self, mode: PermissionMode) {
        *self.permission_mode.lock().await = mode;
        self.persist_settings().await;
    }
    pub async fn get_permission_mode(&self) -> PermissionMode {
        *self.permission_mode.lock().await
    }

    /// Back-compat aliases used by existing Tauri commands.
    pub async fn set_mode(&self, mode: CoderAgentMode) {
        self.set_agent_mode(mode).await;
    }
    pub async fn get_mode(&self) -> CoderAgentMode {
        self.get_agent_mode().await
    }
    pub async fn list_rules(&self) -> Vec<PermissionRule> {
        self.rules.lock().await.clone()
    }
    pub async fn add_rule(&self, rule: PermissionRule) {
        {
            let mut rules = self.rules.lock().await;
            if rules
                .iter()
                .any(|r| r.tool == rule.tool && r.pattern == rule.pattern && r.allow == rule.allow)
            {
                return;
            }
            rules.push(rule);
        }
        self.persist_settings().await;
    }
    pub async fn remove_rule(&self, tool: &str, pattern: &str) {
        self.rules
            .lock()
            .await
            .retain(|r| !(r.tool == tool && r.pattern == pattern));
        self.persist_settings().await;
    }

    // ---- run / resume --------------------------------------------------

    /// Spawn the agent loop in the background. Returns immediately; progress
    /// streams via `coder://*` events.
    pub fn spawn_run(self: Arc<Self>, thread_id: String, turn: AgentTurn) {
        let cancel = Arc::new(AtomicBool::new(false));
        let handle = RunHandle {
            cancel: cancel.clone(),
        };
        let svc = Arc::clone(&self);
        let tid = thread_id.clone();

        tauri::async_runtime::spawn(async move {
            {
                let mut runs = svc.active_runs.lock().await;
                if let Some(old) = runs.insert(tid.clone(), handle) {
                    old.cancel.store(true, Ordering::SeqCst);
                }
            }
            svc.emit(
                "coder://running",
                json!({ "thread_id": tid, "running": true }),
            );
            svc.sync_sub_agent_from_thread(&tid, CoderSubAgentStatus::Running, None)
                .await;

            let result = svc.advance_platform(&tid, turn, Some(cancel)).await;

            {
                let mut runs = svc.active_runs.lock().await;
                runs.remove(&tid);
            }
            svc.emit(
                "coder://running",
                json!({ "thread_id": tid, "running": false }),
            );

            if let Err(e) = result {
                eprintln!("coder: run failed for {tid}: {e}");
                svc.sync_sub_agent_from_thread(&tid, CoderSubAgentStatus::Failed, Some(e.clone()))
                    .await;
                svc.emit("coder://error", json!({ "thread_id": tid, "error": e }));
            } else {
                let cancelled = {
                    let sub_agents = svc.sub_agents.lock().await;
                    sub_agents
                        .values()
                        .find(|s| s.child_thread_id == tid)
                        .map(|s| matches!(s.status, CoderSubAgentStatus::Cancelled))
                        .unwrap_or(false)
                };
                if !cancelled {
                    svc.sync_sub_agent_from_thread(&tid, CoderSubAgentStatus::Completed, None)
                        .await;
                }
            }
        });
    }

    /// Cancel an in-flight run for a thread, if any.
    pub async fn stop(&self, thread_id: &str) -> bool {
        let had_run = if let Some(h) = self.active_runs.lock().await.get(thread_id) {
            h.cancel.store(true, Ordering::SeqCst);
            true
        } else {
            false
        };
        let had_pending = self.cancel_pending_commands(thread_id).await;
        if had_run || had_pending {
            self.emit(
                "coder://running",
                json!({ "thread_id": thread_id, "running": false }),
            );
            self.sync_sub_agent_from_thread(
                thread_id,
                CoderSubAgentStatus::Cancelled,
                Some("Cancelled by user.".into()),
            )
            .await;
        }
        had_run || had_pending
    }

    /// Fail any frontend-delegated commands still waiting on a result.
    async fn cancel_pending_commands(&self, thread_id: &str) -> bool {
        let prefix = format!("{thread_id}:");
        let keys: Vec<String> = {
            let pending = self.pending_commands.lock().await;
            pending
                .keys()
                .filter(|k| k.starts_with(&prefix))
                .cloned()
                .collect()
        };
        let mut cancelled = false;
        for key in keys {
            if let Some(cmd) = self.pending_commands.lock().await.remove(&key) {
                let _ = cmd.tx.send("Error: cancelled".to_string());
                cancelled = true;
            }
        }
        cancelled
    }

    pub async fn list_running(&self) -> Vec<String> {
        self.active_runs.lock().await.keys().cloned().collect()
    }

    /// Fetch estimated context usage for a thread from agent-platform.
    pub async fn get_context_usage(
        &self,
        thread_id: &str,
    ) -> Result<Option<crate::domains::ai::context_usage::ContextUsage>, String> {
        use crate::domains::ai::context_usage::parse_context_usage;

        let platform_id = self
            .get_thread(thread_id)
            .await
            .and_then(|t| t.platform_thread_id)
            .ok_or_else(|| "Thread has no platform id yet".to_string())?;

        let cfg = self.platform_config();
        let base = cfg.base_url.trim_end_matches('/');
        let url = format!("{base}/api/v1/coder/chat/context-usage?thread_id={platform_id}");
        let value = self.platform_get(&url).await?;
        Ok(parse_context_usage(&value))
    }

    /// Edit a user message, truncate the transcript from that point, and reset
    /// the platform thread so the agent restarts from the edited turn.
    pub async fn prepare_edit_message(
        &self,
        thread_id: &str,
        message_index: usize,
        new_content: String,
    ) -> Result<(), String> {
        let trimmed = new_content.trim().to_string();
        if trimmed.is_empty() {
            return Err("message cannot be empty".into());
        }

        let prefix = {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(thread_id).ok_or("thread not found")?;
            if message_index >= thread.messages.len() {
                return Err("message index out of range".into());
            }
            let msg = &thread.messages[message_index];
            if msg.role != "user" {
                return Err("only user messages can be edited".into());
            }

            let prefix = if message_index > 0 {
                thread.messages[..message_index].to_vec()
            } else {
                Vec::new()
            };

            thread.messages.truncate(message_index + 1);
            if let Some(m) = thread.messages.get_mut(message_index) {
                m.content = Some(trimmed);
            }
            thread.platform_thread_id = None;
            thread.updated_at = now_iso();
            prefix
        };

        if !prefix.is_empty() {
            self.edit_prefix
                .lock()
                .await
                .insert(thread_id.to_string(), prefix);
        } else {
            self.edit_prefix.lock().await.remove(thread_id);
        }

        self.persist_thread(thread_id).await;
        Ok(())
    }

    /// Resume the agent loop from the current thread state (e.g. after a failed
    /// LLM request). Truncates any partial assistant/tool tail after the last user
    /// message so the platform retry does not duplicate the user turn.
    pub async fn prepare_retry(&self, thread_id: &str) -> Result<(), String> {
        {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(thread_id).ok_or("thread not found")?;
            let last_user = thread.messages.iter().rposition(|m| {
                m.role == "user" && m.content.as_deref().unwrap_or("").trim().len() > 0
            });
            let Some(idx) = last_user else {
                return Err("no user message to retry".into());
            };
            thread.messages.truncate(idx + 1);
            thread.updated_at = now_iso();
        }
        self.persist_thread(thread_id).await;
        Ok(())
    }

    /// Append a user message and drive the loop until it pauses or finishes.
    pub async fn prepare_send(
        self: &Arc<Self>,
        thread_id: &str,
        message: String,
    ) -> Result<(), String> {
        let fallback_title = {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(thread_id).ok_or("thread not found")?;
            let is_first = thread.messages.iter().filter(|m| m.role == "user").count() == 0;
            let mut fallback = None;
            if is_first && is_placeholder_title(&thread.title) {
                let fb = fallback_title_from_message(&message, PLACEHOLDER_SESSION);
                thread.title = fb.clone();
                fallback = Some(fb);
            }
            let user_msg = ChatMessage::user(message.clone());
            thread.messages.push(user_msg.clone());
            self.emit(
                "coder://message",
                json!({ "thread_id": thread_id, "message": user_msg }),
            );
            fallback
        };
        self.persist_thread(thread_id).await;
        if let Some(fb) = fallback_title {
            self.emit(
                "coder://title",
                json!({ "thread_id": thread_id, "title": fb }),
            );
        }
        Ok(())
    }

    /// Resolve a pending approval, then continue the loop.
    pub async fn prepare_approve(
        &self,
        thread_id: &str,
        call_id: &str,
        approve: bool,
        remember: bool,
        edited_pattern: Option<String>,
        remember_tool: Option<String>,
        remember_args: Option<Value>,
    ) -> Result<(), String> {
        if remember && approve {
            let from_messages = self.find_open_call(thread_id, call_id).await;
            let (tool, args) = if let Some(found) = from_messages {
                found
            } else if let (Some(tool), Some(args)) = (remember_tool, remember_args) {
                (tool, args)
            } else {
                (String::new(), json!({}))
            };
            if !tool.is_empty() {
                let pattern = edited_pattern.unwrap_or_else(|| tools::suggested_rule(&tool, &args));
                if !pattern.is_empty() {
                    self.add_rule(PermissionRule {
                        tool,
                        pattern,
                        allow: true,
                    })
                    .await;
                }
            }
        }
        let _ = (thread_id, call_id, approve);
        Ok(())
    }

    /// Legacy synchronous entry points — kept for compatibility, now spawn.
    pub async fn retry(&self, thread_id: &str) -> Result<(), String> {
        self.prepare_retry(thread_id).await?;
        Ok(())
    }

    pub async fn approve(
        &self,
        thread_id: &str,
        call_id: &str,
        approve: bool,
        remember: bool,
        edited_pattern: Option<String>,
        remember_tool: Option<String>,
        remember_args: Option<Value>,
    ) -> Result<(), String> {
        self.prepare_approve(
            thread_id,
            call_id,
            approve,
            remember,
            edited_pattern,
            remember_tool,
            remember_args,
        )
        .await
    }

    /// Run one turn via agent-platform `POST /api/v1/coder/chat/stream`, `/retry`, or `/approve`.
    async fn advance_platform(
        &self,
        thread_id: &str,
        mut turn: AgentTurn,
        cancel: Option<Arc<AtomicBool>>,
    ) -> Result<CoderRunResult, String> {
        let is_cancelled = || {
            cancel
                .as_ref()
                .map(|c| c.load(Ordering::SeqCst))
                .unwrap_or(false)
        };

        if is_cancelled() {
            return Ok(self.finish(thread_id, None, false, true).await);
        }

        loop {
            let platform_thread_id = self.ensure_platform_thread(thread_id).await?;
            let (model, llm_provider, workspace_root, send_message, fallback_title, thread_kind) = {
                let threads = self.threads.lock().await;
                let t = threads.get(thread_id).ok_or("thread not found")?;
                let send_message = match &turn {
                    AgentTurn::Send => t
                        .messages
                        .iter()
                        .rev()
                        .find(|m| m.role == "user")
                        .and_then(|m| m.content.clone())
                        .ok_or("no user message to send")?,
                    AgentTurn::Approve { .. } => String::new(),
                    AgentTurn::Retry => String::new(),
                };
                let fallback = t
                    .messages
                    .iter()
                    .rev()
                    .find(|m| m.role == "user")
                    .and_then(|m| m.content.as_deref())
                    .map(|m| fallback_title_from_message(m, PLACEHOLDER_SESSION))
                    .unwrap_or_else(|| PLACEHOLDER_SESSION.to_string());
                (
                    t.model.clone(),
                    t.llm_provider.clone(),
                    t.workspace_root.clone(),
                    send_message,
                    fallback,
                    t.thread_kind,
                )
            };
            let agent_mode = self.get_agent_mode().await;
            let permission_mode = self.get_permission_mode().await;
            let tool_specs = self.tool_specs_for_request(thread_kind, agent_mode);

            validate_workspace_root(&workspace_root)?;

            let (allow_commands, auto_approve) =
                agent_mode::permission_flags(agent_mode, permission_mode);
            let cfg = self.platform_config();
            let base = cfg.base_url.trim_end_matches('/');

            let req = match turn {
                AgentTurn::Send => {
                    let url = format!("{base}/api/v1/coder/chat/stream");
                    let mut body = json!({
                        "message": send_message,
                        "thread_id": platform_thread_id,
                        "workspace_root": workspace_root,
                        "allow_commands": allow_commands,
                        "auto_approve_commands": auto_approve,
                        "delegate_tools": true,
                        "tools": tool_specs.clone(),
                    });
                    attach_agent_mode_fields(&mut body, agent_mode);
                    if let Some(m) = &model {
                        body["model"] = json!(m);
                    }
                    attach_llm_provider(&mut body, &llm_provider);
                    let mut r = self
                        .client
                        .post(&url)
                        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
                        .json(&body);
                    if let Some(key) = &cfg.api_token {
                        r = r.bearer_auth(key);
                    }
                    r
                }
                AgentTurn::Approve {
                    call_id,
                    approve,
                    edited_command,
                } => {
                    let url = format!("{base}/api/v1/coder/chat/approve");
                    let mut body = json!({
                        "thread_id": platform_thread_id,
                        "call_id": call_id,
                        "approve": approve,
                        "allow_commands": allow_commands,
                        "auto_approve_commands": auto_approve,
                        "delegate_tools": true,
                        "tools": tool_specs.clone(),
                    });
                    attach_agent_mode_fields(&mut body, agent_mode);
                    if let Some(m) = &model {
                        body["model"] = json!(m);
                    }
                    attach_llm_provider(&mut body, &llm_provider);
                    if let Some(cmd) = edited_command {
                        body["edited_command"] = json!(cmd);
                    }
                    let mut r = self
                        .client
                        .post(&url)
                        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
                        .json(&body);
                    if let Some(key) = &cfg.api_token {
                        r = r.bearer_auth(key);
                    }
                    r
                }
                AgentTurn::Retry => {
                    let url = format!("{base}/api/v1/coder/chat/retry");
                    let mut body = json!({
                        "thread_id": platform_thread_id,
                        "workspace_root": workspace_root,
                        "allow_commands": allow_commands,
                        "auto_approve_commands": auto_approve,
                        "delegate_tools": true,
                        "tools": tool_specs.clone(),
                    });
                    attach_agent_mode_fields(&mut body, agent_mode);
                    if let Some(m) = &model {
                        body["model"] = json!(m);
                    }
                    attach_llm_provider(&mut body, &llm_provider);
                    let mut r = self
                        .client
                        .post(&url)
                        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
                        .json(&body);
                    if let Some(key) = &cfg.api_token {
                        r = r.bearer_auth(key);
                    }
                    r
                }
            };

            let resp = req.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                format!(
                    "Can't reach agent-platform at {}. Is it running? Configure under AI → Providers.",
                    cfg.base_url
                )
            } else {
                format!("platform coder request failed: {e}")
            }
        })?;

            let thread_id_for_stream = thread_id.to_string();
            let fallback_for_title = fallback_title.clone();
            let mut done: Option<PlatformDone> = None;
            let mut stream_error: Option<String> = None;
            // Assistant reasoning streamed before a tool_call; attached to the
            // synthetic tool-call message so it stays visible while the tool runs.
            let mut pending_assistant_text = String::new();
            let platform_thread_id_for_tools = platform_thread_id;
            let workspace_root_for_tools = workspace_root.clone();
            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("platform stream {status}: {text}"));
            }

            let mut buf: Vec<u8> = Vec::new();
            let mut stream = resp.bytes_stream();
            'stream: loop {
                if is_cancelled() {
                    break;
                }
                let chunk = tokio::select! {
                    biased;
                    _ = async {
                        while !is_cancelled() {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                    } => break,
                    c = stream.next() => c,
                };
                let mut eof = false;
                match chunk {
                    Some(chunk) => {
                        if is_cancelled() {
                            break;
                        }
                        let bytes =
                            chunk.map_err(|e| format!("platform stream read error: {e}"))?;
                        buf.extend_from_slice(&bytes);
                    }
                    // Stream closed. A trailing SSE event (e.g. `done`) may not be
                    // terminated by a blank line before the connection closes, which
                    // would leave it unparsed in the buffer. Terminate the buffer so
                    // the final event still flushes instead of being silently dropped.
                    None => {
                        if !buf.is_empty() {
                            buf.extend_from_slice(b"\n\n");
                        }
                        eof = true;
                    }
                }

                for (event, data) in platform_stream::drain_sse_events(&mut buf) {
                    if is_cancelled() {
                        break 'stream;
                    }
                    match event.as_str() {
                        "title" => {
                            if let Some(title) = data.get("title").and_then(Value::as_str) {
                                self.apply_generated_title(
                                    &thread_id_for_stream,
                                    title,
                                    &fallback_for_title,
                                )
                                .await;
                            }
                        }
                        "assistant" => {
                            if let Some(content) = data.get("content").and_then(Value::as_str) {
                                let cleaned = platform_stream::strip_leaked_tool_syntax(content);
                                if !cleaned.is_empty() {
                                    pending_assistant_text = cleaned.clone();
                                    self.emit(
                                    "coder://token",
                                    json!({ "thread_id": thread_id_for_stream, "delta": cleaned }),
                                );
                                }
                            }
                        }
                        "tool_call" => {
                            if is_cancelled() {
                                break 'stream;
                            }
                            let call_id = data.get("call_id").and_then(Value::as_str).unwrap_or("");
                            let name = data.get("name").and_then(Value::as_str).unwrap_or("");
                            let args = data.get("arguments").cloned().unwrap_or_else(|| json!({}));
                            if call_id.is_empty() {
                                stream_error =
                                    Some("platform tool_call missing call_id".to_string());
                            } else {
                                let reasoning = if pending_assistant_text.trim().is_empty() {
                                    None
                                } else {
                                    Some(std::mem::take(&mut pending_assistant_text))
                                };
                                self.emit(
                                    "coder://message",
                                    json!({
                                        "thread_id": thread_id_for_stream,
                                        "message": ChatMessage::assistant_tool_call(call_id, name, &args, reasoning),
                                    }),
                                );
                                match self
                                    .execute_delegated_tool(
                                        &thread_id_for_stream,
                                        platform_thread_id_for_tools,
                                        &workspace_root_for_tools,
                                        call_id,
                                        name,
                                        &args,
                                    )
                                    .await
                                {
                                    Ok(result) => {
                                        self.emit(
                                            "coder://message",
                                            json!({
                                                "thread_id": thread_id_for_stream,
                                                "message": ChatMessage::tool_result(call_id, result),
                                            }),
                                        );
                                    }
                                    Err(e) => stream_error = Some(e),
                                }
                            }
                        }
                        "heartbeat" => {
                            let waited = data
                                .get("waited_seconds")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0);
                            self.emit(
                                "coder://heartbeat",
                                json!({ "thread_id": thread_id_for_stream, "waited_seconds": waited }),
                            );
                        }
                        "error" => {
                            let detail = data
                                .get("detail")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown platform error")
                                .to_string();
                            stream_error = Some(detail);
                        }
                        "done" => {
                            let parsed = platform_stream::done_from_event(&data);
                            if let Some(ctx) = &parsed.context_usage {
                                self.emit(
                                    "coder://context-usage",
                                    json!({
                                        "thread_id": thread_id_for_stream,
                                        "context_usage": ctx,
                                        "llm_usage": parsed.llm_usage,
                                    }),
                                );
                            }
                            done = Some(parsed);
                        }
                        _ => {}
                    }
                }

                if eof {
                    break;
                }
            }

            if is_cancelled() {
                return Ok(self.finish(thread_id, None, false, true).await);
            }

            // A platform `error` event (e.g. the LLM proxy rejecting a base model
            // that can't do chat/tool-calling) is often followed by a `done` event
            // carrying only the user turn. Surface the error first so the user sees
            // it and gets a Retry, instead of a silent no-reply run.
            if let Some(err) = stream_error {
                return Err(err);
            }
            let done = match done {
                Some(done) => done,
                None => return Err("platform stream ended without done event".into()),
            };

            if let Some(tool) = platform_stream::find_leaked_tool_call(&done.messages) {
                return Err(format!(
                    "The model wrote `<function={tool}>` as text instead of running a tool. \
                     Choose a model with tool/function calling support in AI → Providers, then retry."
                ));
            }

            self.sync_thread_from_platform(thread_id, platform_thread_id, &done, &fallback_title)
                .await;

            if let Some(pending) = done.pending.clone() {
                let agent_mode = self.get_agent_mode().await;
                let permission_mode = self.get_permission_mode().await;
                let rules = self.list_rules().await;
                match permissions::decide(
                    agent_mode,
                    permission_mode,
                    &rules,
                    &pending.tool,
                    &pending.arguments,
                ) {
                    Decision::Allow => {
                        turn = AgentTurn::Approve {
                            call_id: pending.call_id.clone(),
                            approve: true,
                            edited_command: None,
                        };
                        continue;
                    }
                    Decision::Deny(reason) => {
                        eprintln!("coder: auto-rejecting {} ({reason})", pending.call_id);
                        turn = AgentTurn::Approve {
                            call_id: pending.call_id.clone(),
                            approve: false,
                            edited_command: None,
                        };
                        continue;
                    }
                    Decision::Prompt => {}
                }

                self.emit(
                    "coder://pending",
                    json!({ "thread_id": thread_id, "pending": pending }),
                );
                let messages = self
                    .get_thread(thread_id)
                    .await
                    .map(|t| t.messages)
                    .unwrap_or_default();
                return Ok(CoderRunResult {
                    thread_id: thread_id.to_string(),
                    messages,
                    pending: Some(pending),
                    final_text: None,
                    exhausted: false,
                });
            }

            return Ok(self.finish(thread_id, done.final_text, false, false).await);
        }
    }

    async fn ensure_platform_thread(&self, local_id: &str) -> Result<i64, String> {
        if let Some(id) = self
            .get_thread(local_id)
            .await
            .and_then(|t| t.platform_thread_id)
        {
            return Ok(id);
        }

        let (title, workspace_root) = {
            let threads = self.threads.lock().await;
            let t = threads.get(local_id).ok_or("thread not found")?;
            (t.title.clone(), t.workspace_root.clone())
        };

        let cfg = self.platform_config();
        let base = cfg.base_url.trim_end_matches('/');
        let body = json!({
            "title": title,
            "workspace_root": workspace_root,
        });
        let created = self
            .platform_post(&format!("{base}/api/v1/coder/chat/threads"), body)
            .await?;
        let platform_id = created
            .get("thread_id")
            .and_then(Value::as_i64)
            .ok_or_else(|| format!("platform thread create missing thread_id: {created}"))?;

        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(local_id) {
                t.platform_thread_id = Some(platform_id);
                t.updated_at = now_iso();
            }
        }
        self.persist_thread(local_id).await;
        Ok(platform_id)
    }

    async fn sync_from_platform(&self, local_id: &str) -> Result<(), String> {
        let platform_id = self
            .get_thread(local_id)
            .await
            .and_then(|t| t.platform_thread_id)
            .ok_or("thread not linked to platform")?;
        let cfg = self.platform_config();
        let base = cfg.base_url.trim_end_matches('/');
        let data = self
            .platform_get(&format!(
                "{base}/api/v1/coder/chat/thread?thread_id={platform_id}"
            ))
            .await?;
        let messages: Vec<ChatMessage> = data
            .get("messages")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| serde_json::from_value(m.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
        let mut messages = messages;
        agent_mode::normalize_platform_user_messages(&mut messages);
        let title = data
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(PLACEHOLDER_SESSION)
            .to_string();
        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(local_id) {
                let old_messages = t.messages.clone();
                let updated = now_iso();
                t.messages = super::types::with_message_timestamps(messages, &old_messages, Some(&updated));
                t.title = title;
                t.updated_at = updated;
            }
        }
        self.persist_thread(local_id).await;
        Ok(())
    }

    async fn sync_thread_from_platform(
        &self,
        local_id: &str,
        platform_id: i64,
        done: &platform_stream::PlatformDone,
        fallback_title: &str,
    ) {
        let edit_prefix = self.edit_prefix.lock().await.remove(local_id);
        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(local_id) {
                let old_messages = t.messages.clone();
                let updated = now_iso();
                t.platform_thread_id = Some(platform_id);
                let mut messages = if let Some(prefix) = edit_prefix {
                    let mut merged = prefix;
                    merged.extend(done.messages.clone());
                    merged
                } else {
                    done.messages.clone()
                };
                agent_mode::normalize_platform_user_messages(&mut messages);
                platform_stream::sanitize_platform_messages(&mut messages);
                t.messages = super::types::with_message_timestamps(
                    messages,
                    &old_messages,
                    Some(&updated),
                );
                if should_apply_generated_title(&t.title, fallback_title) {
                    t.title = done.title.clone();
                }
                t.updated_at = updated;
            }
        }
        self.persist_thread(local_id).await;
    }

    async fn finish(
        &self,
        thread_id: &str,
        final_text: Option<String>,
        exhausted: bool,
        cancelled: bool,
    ) -> CoderRunResult {
        let (messages, title) = match self.get_thread(thread_id).await {
            Some(t) => (t.messages, t.title),
            None => (Vec::new(), PLACEHOLDER_SESSION.to_string()),
        };
        self.emit(
            "coder://done",
            json!({
                "thread_id": thread_id,
                "final_text": final_text,
                "exhausted": exhausted,
                "cancelled": cancelled,
                "title": title,
            }),
        );
        CoderRunResult {
            thread_id: thread_id.to_string(),
            messages,
            pending: None,
            final_text,
            exhausted,
        }
    }

    async fn apply_generated_title(&self, thread_id: &str, title: &str, fallback: &str) {
        let should_emit = {
            let mut threads = self.threads.lock().await;
            let Some(thread) = threads.get_mut(thread_id) else {
                return;
            };
            if !should_apply_generated_title(&thread.title, fallback) {
                return;
            }
            if thread.title == title {
                return;
            }
            thread.title = title.to_string();
            true
        };
        if should_emit {
            self.persist_thread(thread_id).await;
            self.emit(
                "coder://title",
                json!({ "thread_id": thread_id, "title": title }),
            );
        }
    }

    async fn find_open_call(&self, thread_id: &str, call_id: &str) -> Option<(String, Value)> {
        let messages = self.get_thread(thread_id).await?.messages;
        for m in messages.iter().rev() {
            if let Some(calls) = &m.tool_calls {
                if let Some(c) = calls.iter().find(|c| c.id == call_id) {
                    let args = tools::parse_args(c).unwrap_or_else(|_| json!({}));
                    return Some((c.function.name.clone(), args));
                }
            }
        }
        None
    }
}

fn attach_llm_provider(body: &mut Value, llm_provider: &Option<String>) {
    if let Some(p) = llm_provider
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("provider".into(), json!(p));
        }
    }
}

fn attach_agent_mode_fields(body: &mut Value, agent_mode: CoderAgentMode) {
    if let Some(obj) = body.as_object_mut() {
        obj.insert(
            "mode_instruction".into(),
            json!(agent_mode::mode_instruction(agent_mode)),
        );
        obj.insert("agent_mode".into(), json!(agent_mode::mode_as_str(agent_mode)));
    }
}

fn thread_kind_str(kind: CoderThreadKind) -> &'static str {
    match kind {
        CoderThreadKind::Session => "session",
        CoderThreadKind::Coordinator => "coordinator",
        CoderThreadKind::SubAgent => "sub-agent",
    }
}

fn parse_thread_kind(value: &str) -> CoderThreadKind {
    match value {
        "coordinator" => CoderThreadKind::Coordinator,
        "sub-agent" => CoderThreadKind::SubAgent,
        _ => CoderThreadKind::Session,
    }
}

fn sub_agent_status_str(status: CoderSubAgentStatus) -> &'static str {
    match status {
        CoderSubAgentStatus::Pending => "pending",
        CoderSubAgentStatus::Running => "running",
        CoderSubAgentStatus::Completed => "completed",
        CoderSubAgentStatus::Failed => "failed",
        CoderSubAgentStatus::Cancelled => "cancelled",
    }
}

fn parse_sub_agent_status(value: &str) -> CoderSubAgentStatus {
    match value {
        "running" => CoderSubAgentStatus::Running,
        "completed" => CoderSubAgentStatus::Completed,
        "failed" => CoderSubAgentStatus::Failed,
        "cancelled" => CoderSubAgentStatus::Cancelled,
        _ => CoderSubAgentStatus::Pending,
    }
}

fn short_id(value: &str) -> String {
    value.chars().take(8).collect()
}

fn build_sub_agent_slug(index: usize, title: &str) -> String {
    let slug = worktree::sanitize_slug(title);
    if slug.is_empty() {
        format!("task-{}", index + 1)
    } else {
        slug
    }
}

fn resolve_task_issue_ref(task: &SpawnSubAgentTask) -> Option<GitHubIssueRef> {
    task.github_issue
        .clone()
        .or_else(|| task.github_issue_url.as_deref().and_then(parse_issue_url))
}

fn build_issue_prompt(issue_url: &str, issue: &GitHubIssue) -> String {
    let body = issue.body.clone().unwrap_or_default();
    format!(
        "Complete GitHub issue {}.\n\nTitle: {}\n\nBody:\n{}\n\nWork in the isolated worktree for this task, run focused verification, and summarize the result.",
        issue_url,
        issue.title,
        body.trim()
    )
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn validate_workspace_root(root: &str) -> Result<(), String> {
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return Err("Set a workspace folder first.".into());
    }
    let path = Path::new(trimmed);
    if !path.is_dir() {
        return Err(format!(
            "Workspace folder does not exist or is not a directory: {trimmed}"
        ));
    }
    Ok(())
}

/// Cap a tool result so a huge platform payload can't blow the context budget.
fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [truncated {} bytes]", &text[..end], text.len() - end)
}
