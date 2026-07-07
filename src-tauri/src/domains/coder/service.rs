//! Coder agent state + the tool-calling loop.
//!
//! The loop is *resumable and stateless across calls*: all progress lives in
//! the thread transcript. A run advances until it either produces a final
//! assistant message or pauses on a tool call that needs human approval. On
//! approval we call [`CoderService::advance`] again with the granted call id
//! and it picks up exactly where it left off (a tool_call with no matching
//! `tool` response yet).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use std::sync::OnceLock;

use futures_util::StreamExt;

use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::database::DatabaseManager;

use super::diff;
use super::entities::{coder_file_change, coder_setting, coder_thread};
use super::permissions::{self, Decision};
use super::tools;
use super::types::{
    ChangeStatus, ChatMessage, CoderRunResult, CoderThread, FileChange, PendingApproval,
    PermissionMode, PermissionRule, ToolCall,
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

const MAX_ITERATIONS: usize = 25;

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
    base_url: String,
    master_key: Option<String>,
    default_model: Option<String>,
    /// When set, the `delegate_task` tool is offered, targeting this team
    /// template on the platform. Configured via `CODER_TEAM_TEMPLATE_ID`.
    delegation_team_template_id: Option<i64>,
}

impl CoderService {
    /// Build the service and hydrate threads + settings from the database.
    pub async fn new(db: Arc<DatabaseManager>) -> Self {
        let base_url = std::env::var("CODER_PLATFORM_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18410".to_string());
        let master_key = std::env::var("AGENT_PLATFORM_MASTER_KEY").ok().filter(|k| !k.is_empty());
        let default_model = std::env::var("CODER_MODEL").ok().filter(|m| !m.is_empty());
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
            base_url,
            master_key,
            default_model,
            delegation_team_template_id,
        };
        svc.load_from_db().await;
        svc
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

                let base = self.base_url.trim_end_matches('/');
                // Start the process.
                let start = self
                    .platform_post(
                        &format!("{base}/api/v1/processes"),
                        json!({ "goal": goal, "auto_approve": true, "team_template_id": team_id }),
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
        let mut req = self.client.post(url).json(&body);
        if let Some(key) = &self.master_key {
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
        let mut req = self.client.get(url);
        if let Some(key) = &self.master_key {
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
            title: "New session".into(),
            workspace_root,
            model: model.or_else(|| self.default_model.clone()),
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

    /// Append a user message and drive the loop until it pauses or finishes.
    pub async fn send(&self, thread_id: &str, message: String) -> Result<CoderRunResult, String> {
        {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(thread_id).ok_or("thread not found")?;
            if thread.messages.iter().filter(|m| m.role == "user").count() == 0 {
                thread.title = auto_title(&message);
            }
            thread.messages.push(ChatMessage::user(message));
        }
        self.persist_thread(thread_id).await;
        self.advance(thread_id, None).await
    }

    /// Resolve a pending approval, then continue the loop.
    ///
    /// `approve=false` records a rejection tool result so the model can adapt.
    /// `remember`/`edited_pattern` optionally persist an allow rule.
    pub async fn approve(
        &self,
        thread_id: &str,
        call_id: &str,
        approve: bool,
        remember: bool,
        edited_pattern: Option<String>,
    ) -> Result<CoderRunResult, String> {
        if !approve {
            let mut threads = self.threads.lock().await;
            let thread = threads.get_mut(thread_id).ok_or("thread not found")?;
            thread
                .messages
                .push(ChatMessage::tool_result(call_id, "user rejected this action"));
            drop(threads);
            self.persist_thread(thread_id).await;
            return self.advance(thread_id, None).await;
        }

        if remember {
            // Look up the pending call to derive tool + pattern.
            if let Some((tool, args)) = self.find_open_call(thread_id, call_id).await {
                let pattern = edited_pattern.unwrap_or_else(|| tools::suggested_rule(&tool, &args));
                self.add_rule(PermissionRule { tool, pattern, allow: true }).await;
            }
        }
        self.advance(thread_id, Some(call_id.to_string())).await
    }

    /// Core loop. `granted` is a one-shot approval for a specific call id.
    async fn advance(
        &self,
        thread_id: &str,
        mut granted: Option<String>,
    ) -> Result<CoderRunResult, String> {
        let mode = self.get_mode().await;
        let rules = self.list_rules().await;
        let mut llm_calls = 0usize;

        loop {
            // Snapshot current state.
            let (messages, model, root) = {
                let threads = self.threads.lock().await;
                let t = threads.get(thread_id).ok_or("thread not found")?;
                (t.messages.clone(), t.model.clone(), t.workspace_root.clone())
            };

            match open_calls(&messages) {
                // No unanswered tool calls: either finished or need the model.
                None => {
                    if let Some(last) = messages.last() {
                        if last.role == "assistant" && last.tool_calls.is_none() {
                            return Ok(self.finish(thread_id, last.content.clone(), false).await);
                        }
                    }
                    if llm_calls >= MAX_ITERATIONS {
                        return Ok(self.finish(thread_id, None, true).await);
                    }
                    llm_calls += 1;
                    let assistant = self.call_llm(thread_id, &messages, model.as_deref()).await?;
                    self.push_message(thread_id, assistant).await;
                }
                // There are tool calls awaiting execution.
                Some(calls) => {
                    for call in calls {
                        let args = tools::parse_args(&call).unwrap_or_else(|_| json!({}));
                        let one_shot = granted.as_deref() == Some(call.id.as_str());
                        let decision = if one_shot {
                            granted = None;
                            Decision::Allow
                        } else {
                            permissions::decide(mode, &rules, &call.function.name, &args)
                        };

                        match decision {
                            Decision::Allow => {
                                let name = call.function.name.clone();
                                // Snapshot before-content for change review.
                                let mutated = tools::mutated_path(&name, &args);
                                let before = mutated.as_ref().map(|p| tools::read_raw(&root, p));
                                let result = if tools::is_platform_tool(&name) {
                                    self.execute_platform_tool(&name, &args)
                                        .await
                                        .unwrap_or_else(|e| format!("tool error: {e}"))
                                } else {
                                    tools::execute(&root, &name, &args)
                                        .unwrap_or_else(|e| format!("tool error: {e}"))
                                };
                                self.push_message(thread_id, ChatMessage::tool_result(&call.id, result))
                                    .await;
                                if let Some(path) = mutated {
                                    self.record_change(thread_id, &name, &root, path, before.flatten())
                                        .await;
                                }
                            }
                            Decision::Deny(reason) => {
                                self.push_message(
                                    thread_id,
                                    ChatMessage::tool_result(&call.id, format!("denied: {reason}")),
                                )
                                .await;
                            }
                            Decision::Prompt => {
                                let messages = self.get_thread(thread_id).await.map(|t| t.messages).unwrap_or_default();
                                let pending = PendingApproval {
                                    call_id: call.id.clone(),
                                    tool: call.function.name.clone(),
                                    arguments: args.clone(),
                                    suggested_rule: tools::suggested_rule(&call.function.name, &args),
                                    summary: tools::summarize(&call.function.name, &args),
                                };
                                self.emit(
                                    "coder://pending",
                                    json!({ "thread_id": thread_id, "pending": pending }),
                                );
                                return Ok(CoderRunResult {
                                    thread_id: thread_id.to_string(),
                                    messages,
                                    pending: Some(pending),
                                    final_text: None,
                                    exhausted: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // ---- helpers -------------------------------------------------------

    async fn push_message(&self, thread_id: &str, msg: ChatMessage) {
        self.emit(
            "coder://message",
            json!({ "thread_id": thread_id, "message": msg }),
        );
        {
            let mut threads = self.threads.lock().await;
            if let Some(t) = threads.get_mut(thread_id) {
                t.messages.push(msg);
                t.updated_at = now_iso();
            }
        }
        self.persist_thread(thread_id).await;
    }

    async fn finish(&self, thread_id: &str, final_text: Option<String>, exhausted: bool) -> CoderRunResult {
        let messages = self.get_thread(thread_id).await.map(|t| t.messages).unwrap_or_default();
        self.emit(
            "coder://done",
            json!({ "thread_id": thread_id, "final_text": final_text, "exhausted": exhausted }),
        );
        CoderRunResult {
            thread_id: thread_id.to_string(),
            messages,
            pending: None,
            final_text,
            exhausted,
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

    /// One request to the platform's OpenAI-compatible `/v1/chat/completions`.
    async fn call_llm(
        &self,
        thread_id: &str,
        messages: &[ChatMessage],
        model: Option<&str>,
    ) -> Result<ChatMessage, String> {
        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
        let mut body = json!({
            "messages": messages,
            "tools": self.tool_specs_for_request(),
            "tool_choice": "auto",
            "stream": true,
        });
        if let Some(m) = model {
            body["model"] = json!(m);
        }

        let mut req = self.client.post(&url).json(&body);
        if let Some(key) = &self.master_key {
            req = req.bearer_auth(key);
        }

        let resp = req.send().await.map_err(|e| format!("platform request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("platform returned {status}: {text}"));
        }

        // Consume the SSE stream, accumulating content + tool-call deltas.
        let mut content = String::new();
        // Tool calls accumulate by index; id/name arrive first, arguments stream.
        let mut calls: Vec<PartialCall> = Vec::new();
        let mut buf: Vec<u8> = Vec::new();
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| format!("stream error: {e}"))?;
            buf.extend_from_slice(&bytes);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                let payload = match line.strip_prefix("data:") {
                    Some(p) => p.trim(),
                    None => continue,
                };
                if payload == "[DONE]" {
                    continue;
                }
                let Ok(value) = serde_json::from_str::<Value>(payload) else { continue };
                let Some(delta) = value
                    .get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("delta"))
                else {
                    continue;
                };

                if let Some(text) = delta.get("content").and_then(Value::as_str) {
                    if !text.is_empty() {
                        content.push_str(text);
                        self.emit(
                            "coder://token",
                            json!({ "thread_id": thread_id, "delta": text }),
                        );
                    }
                }

                if let Some(tcs) = delta.get("tool_calls").and_then(Value::as_array) {
                    for tc in tcs {
                        let idx = tc.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                        while calls.len() <= idx {
                            calls.push(PartialCall::default());
                        }
                        let slot = &mut calls[idx];
                        if let Some(id) = tc.get("id").and_then(Value::as_str) {
                            if !id.is_empty() {
                                slot.id = id.to_string();
                            }
                        }
                        if let Some(func) = tc.get("function") {
                            if let Some(name) = func.get("name").and_then(Value::as_str) {
                                if !name.is_empty() {
                                    slot.name = name.to_string();
                                }
                            }
                            if let Some(args) = func.get("arguments").and_then(Value::as_str) {
                                slot.arguments.push_str(args);
                            }
                        }
                    }
                }
            }
        }

        let tool_calls: Vec<ToolCall> = calls
            .into_iter()
            .filter(|c| !c.name.is_empty())
            .enumerate()
            .map(|(i, c)| ToolCall {
                id: if c.id.is_empty() { format!("call_{i}") } else { c.id },
                r#type: "function".into(),
                function: super::types::FunctionCall { name: c.name, arguments: c.arguments },
            })
            .collect();

        Ok(ChatMessage {
            role: "assistant".into(),
            content: if content.is_empty() { None } else { Some(content) },
            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
            tool_call_id: None,
        })
    }
}

/// Accumulator for a streamed tool call (fields arrive across SSE chunks).
#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    arguments: String,
}

/// Return tool calls in the last assistant turn that have no `tool` response
/// yet, or `None` if there are none pending.
fn open_calls(messages: &[ChatMessage]) -> Option<Vec<ToolCall>> {
    let answered: HashSet<&str> = messages
        .iter()
        .filter_map(|m| m.tool_call_id.as_deref())
        .collect();
    // Only the most recent assistant-with-tool_calls matters.
    for m in messages.iter().rev() {
        if m.role == "assistant" {
            if let Some(calls) = &m.tool_calls {
                let open: Vec<ToolCall> =
                    calls.iter().filter(|c| !answered.contains(c.id.as_str())).cloned().collect();
                return if open.is_empty() { None } else { Some(open) };
            }
            return None;
        }
    }
    None
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
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

fn auto_title(message: &str) -> String {
    let text: String = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.len() <= 48 {
        text
    } else {
        format!("{}...", &text[..45])
    }
}
