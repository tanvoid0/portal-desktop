//! Coder agent state + the tool-calling loop.
//!
//! The loop is *resumable and stateless across calls*: all progress lives in
//! the thread transcript. A run advances until it either produces a final
//! assistant message or pauses on a tool call that needs human approval. On
//! approval we call [`CoderService::advance`] again with the granted call id
//! and it picks up exactly where it left off (a tool_call with no matching
//! `tool` response yet).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::sync::OnceLock;

use futures_util::StreamExt;

use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::database::DatabaseManager;
use crate::domains::ai::chat_title::{
    fallback_title_from_message, is_placeholder_title, should_apply_generated_title,
    PLACEHOLDER_SESSION,
};
use crate::domains::ai::platform_config::{PlatformConfig, DESKTOP_CLIENT_ID};
use crate::domains::ai::services::AISettingsService;

use super::diff;
use super::entities::{coder_file_change, coder_setting, coder_thread};
use super::platform_stream::{self, PlatformDone};
use super::tools;
use super::types::{
    ChangeStatus, ChatMessage, CoderRunResult, CoderThread, FileChange, PendingApproval,
    PermissionMode, PermissionRule,
};

/// Persisted permission config (mode + rules) stored as one JSON row.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SettingsBlob {
    mode: PermissionMode,
    rules: Vec<PermissionRule>,
}

const SYSTEM_PROMPT: &str = concat!(
    "You are a coding agent operating directly in the user's native workspace via tools.\n",
    "Rules:\n",
    "- All paths are relative to the workspace root.\n",
    "- Explore before you change: use list_dir / read_file / search_files before editing.\n",
    "- For small changes use edit_file (exact-match replace). Use write_file only for new files or full rewrites; read first.\n",
    "- Prefer small, targeted changes. Do not touch files you were not asked to change.\n",
    "- When done, give a short summary of what you changed and why."
);

struct RunHandle {
    cancel: Arc<AtomicBool>,
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
    rules: Mutex<Vec<PermissionRule>>,
    mode: Mutex<PermissionMode>,
    /// Agent file edits awaiting or completed review (Cursor-style).
    changes: Mutex<Vec<FileChange>>,
    client: reqwest::Client,
    settings: AISettingsService,
    /// When set, the `delegate_task` tool is offered, targeting this team
    /// template on the platform. Configured via `CODER_TEAM_TEMPLATE_ID`.
    delegation_team_template_id: Option<i64>,
    /// In-flight agent loops keyed by thread id (for cancel + status).
    active_runs: Mutex<HashMap<String, RunHandle>>,
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
            rules: Mutex::new(Vec::new()),
            mode: Mutex::new(PermissionMode::default()),
            changes: Mutex::new(Vec::new()),
            client: reqwest::Client::new(),
            settings,
            delegation_team_template_id,
            active_runs: Mutex::new(HashMap::new()),
        };
        svc.load_from_db().await;
        svc
    }

    fn platform_config(&self) -> PlatformConfig {
        PlatformConfig::resolve(&self.settings)
    }

    /// Tool specs offered to the model, including platform delegation when
    /// configured.
    fn tool_specs_for_request(&self) -> Vec<Value> {
        let mut specs = tools::tool_specs();
        if self.delegation_team_template_id.is_some() {
            specs.extend(tools::platform_tool_specs());
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
                    .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(str::to_string)))
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
                Ok(format!("process {process_id} still running after timeout; poll {url}"))
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
        let resp = req.send().await.map_err(|e| format!("platform POST failed: {e}"))?;
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
        let resp = req.send().await.map_err(|e| format!("platform GET failed: {e}"))?;
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
    ) -> Result<(), String> {
        let mutated = tools::mutated_path(tool, args);
        let before = mutated
            .as_ref()
            .and_then(|path| tools::read_raw(workspace_root, path));

        let result = match tools::execute(workspace_root, tool, args) {
            Ok(output) => output,
            Err(e) => format!("Error: {e}"),
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
        Ok(())
    }

    /// Wire the Tauri handle so loop steps can be streamed to the UI.
    pub fn init_app_handle(&self, handle: AppHandle) {
        let _ = self.app_handle.set(handle);
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
                            model: row.model,
                            platform_thread_id: row.platform_thread_id,
                            messages,
                            created_at: row.created_at,
                            updated_at: row.updated_at,
                        },
                    );
                }
            }
            Err(e) => eprintln!("coder: load threads failed: {e}"),
        }

        if let Ok(Some(row)) = coder_setting::Entity::find_by_id("default").one(conn).await {
            if let Ok(blob) = serde_json::from_str::<SettingsBlob>(&row.data_json) {
                *self.mode.lock().await = blob.mode;
                *self.rules.lock().await = blob.rules;
            }
        }

        if let Ok(rows) = coder_file_change::Entity::find().all(conn).await {
            let mut changes = self.changes.lock().await;
            for row in rows {
                if let Ok(fc) = serde_json::from_str::<FileChange>(&row.data_json) {
                    changes.push(fc);
                }
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
    pub async fn set_hunk(&self, change_id: &str, hunk_index: usize, accepted: bool) -> Result<(), String> {
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
            let c = changes.iter().find(|c| c.id == change_id).ok_or("change not found")?;
            (c.thread_id.clone(), c.path.clone(), c.before.clone())
        };
        let root = self.workspace_root_for(&thread_id).await.ok_or("thread not found")?;
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
            let c = changes.iter_mut().find(|c| c.id == change_id).ok_or("change not found")?;
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
            model: Set(thread.model),
            platform_thread_id: Set(thread.platform_thread_id),
            messages_json: Set(messages_json),
            created_at: Set(thread.created_at),
            updated_at: Set(thread.updated_at),
        };
        let res = coder_thread::Entity::insert(am)
            .on_conflict(
                OnConflict::column(coder_thread::Column::Id)
                    .update_columns([
                        coder_thread::Column::Title,
                        coder_thread::Column::WorkspaceRoot,
                        coder_thread::Column::Model,
                        coder_thread::Column::PlatformThreadId,
                        coder_thread::Column::MessagesJson,
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
            mode: *self.mode.lock().await,
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

    pub async fn create_thread(&self, workspace_root: String, model: Option<String>) -> CoderThread {
        let now = now_iso();
        let thread = CoderThread {
            id: uuid::Uuid::new_v4().to_string(),
            title: PLACEHOLDER_SESSION.into(),
            workspace_root,
            model: model.or_else(|| self.platform_config().default_model.clone()),
            platform_thread_id: None,
            messages: vec![ChatMessage::system(SYSTEM_PROMPT)],
            created_at: now.clone(),
            updated_at: now,
        };
        self.threads.lock().await.insert(thread.id.clone(), thread.clone());
        self.persist_thread(&thread.id).await;
        thread
    }

    pub async fn list_threads(&self) -> Vec<CoderThread> {
        let mut v: Vec<_> = self.threads.lock().await.values().cloned().collect();
        v.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        v
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

    /// Change the model for an existing thread (mid-session switching).
    pub async fn update_thread_model(&self, id: &str, model: Option<String>) -> Result<(), String> {
        {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(id).ok_or("thread not found")?;
            thread.model = model;
            thread.updated_at = now_iso();
        }
        self.persist_thread(id).await;
        Ok(())
    }

    // ---- permission config --------------------------------------------

    pub async fn set_mode(&self, mode: PermissionMode) {
        *self.mode.lock().await = mode;
        self.persist_settings().await;
    }
    pub async fn get_mode(&self) -> PermissionMode {
        *self.mode.lock().await
    }
    pub async fn list_rules(&self) -> Vec<PermissionRule> {
        self.rules.lock().await.clone()
    }
    pub async fn add_rule(&self, rule: PermissionRule) {
        {
            let mut rules = self.rules.lock().await;
            if rules.iter().any(|r| r.tool == rule.tool && r.pattern == rule.pattern && r.allow == rule.allow) {
                return;
            }
            rules.push(rule);
        }
        self.persist_settings().await;
    }
    pub async fn remove_rule(&self, tool: &str, pattern: &str) {
        self.rules.lock().await.retain(|r| !(r.tool == tool && r.pattern == pattern));
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
                svc.emit(
                    "coder://error",
                    json!({ "thread_id": tid, "error": e }),
                );
            }
        });
    }

    /// Cancel an in-flight run for a thread, if any.
    pub async fn stop(&self, thread_id: &str) -> bool {
        let handle = self.active_runs.lock().await.get(thread_id).map(|h| RunHandle {
            cancel: Arc::clone(&h.cancel),
        });
        if let Some(h) = handle {
            h.cancel.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
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
    pub async fn prepare_send(self: &Arc<Self>, thread_id: &str, message: String) -> Result<(), String> {
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
    ) -> Result<(), String> {
        if remember && approve {
            if let Some((tool, args)) = self.find_open_call(thread_id, call_id).await {
                let pattern = edited_pattern.unwrap_or_else(|| tools::suggested_rule(&tool, &args));
                self.add_rule(PermissionRule {
                    tool,
                    pattern,
                    allow: true,
                })
                .await;
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
    ) -> Result<(), String> {
        self.prepare_approve(thread_id, call_id, approve, remember, edited_pattern)
            .await
    }

    /// Run one turn via agent-platform `POST /api/v1/coder/chat/stream`, `/retry`, or `/approve`.
    async fn advance_platform(
        &self,
        thread_id: &str,
        turn: AgentTurn,
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

        let platform_thread_id = self.ensure_platform_thread(thread_id).await?;
        let (model, workspace_root, send_message, fallback_title) = {
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
                t.workspace_root.clone(),
                send_message,
                fallback,
            )
        };

        validate_workspace_root(&workspace_root)?;

        let (allow_commands, auto_approve) = self.permission_flags().await;
        let cfg = self.platform_config();
        let base = cfg.base_url.trim_end_matches('/');

        let mut req = match turn {
            AgentTurn::Send => {
                let url = format!("{base}/api/v1/coder/chat/stream");
                let mut body = json!({
                    "message": send_message,
                    "thread_id": platform_thread_id,
                    "workspace_root": workspace_root,
                    "allow_commands": allow_commands,
                    "auto_approve_commands": auto_approve,
                    "delegate_tools": true,
                });
                if let Some(m) = &model {
                    body["model"] = json!(m);
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
                });
                if let Some(m) = &model {
                    body["model"] = json!(m);
                }
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
                });
                if let Some(m) = &model {
                    body["model"] = json!(m);
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
            let Some(chunk) = chunk else { break };
            if is_cancelled() {
                break;
            }
            let bytes = chunk.map_err(|e| format!("platform stream read error: {e}"))?;
            buf.extend_from_slice(&bytes);

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
                            if !content.is_empty() {
                                self.emit(
                                    "coder://token",
                                    json!({ "thread_id": thread_id_for_stream, "delta": content }),
                                );
                            }
                        }
                    }
                    "tool_call" => {
                        if is_cancelled() {
                            break 'stream;
                        }
                        let call_id = data
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        let name = data.get("name").and_then(Value::as_str).unwrap_or("");
                        let args = data
                            .get("arguments")
                            .cloned()
                            .unwrap_or_else(|| json!({}));
                        if call_id.is_empty() {
                            stream_error =
                                Some("platform tool_call missing call_id".to_string());
                        } else if let Err(e) = self
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
                            stream_error = Some(e);
                        }
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
        }

        if is_cancelled() {
            return Ok(self.finish(thread_id, None, false, true).await);
        }

        let done = if let Some(done) = done {
            done
        } else if let Some(err) = stream_error {
            return Err(err);
        } else {
            return Err("platform stream ended without done event".into());
        };

        self.sync_thread_from_platform(thread_id, platform_thread_id, &done, &fallback_title)
            .await;

        if let Some(pending) = done.pending.clone() {
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

        Ok(self.finish(thread_id, done.final_text, false, false).await)
    }

    async fn permission_flags(&self) -> (bool, bool) {
        match self.get_mode().await {
            PermissionMode::AutoAcceptAll => (true, true),
            PermissionMode::Review => (true, false),
            PermissionMode::Plan => (false, false),
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
        let title = data
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(PLACEHOLDER_SESSION)
            .to_string();
        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(local_id) {
                t.messages = messages;
                t.title = title;
                t.updated_at = now_iso();
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
        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(local_id) {
                t.platform_thread_id = Some(platform_id);
                t.messages = done.messages.clone();
                if should_apply_generated_title(&t.title, fallback_title) {
                    t.title = done.title.clone();
                }
                t.updated_at = now_iso();
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
