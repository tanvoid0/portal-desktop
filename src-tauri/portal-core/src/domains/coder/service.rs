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

use std::sync::{OnceLock, Weak};

use futures_util::StreamExt;

use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{oneshot, Mutex, Semaphore};

use crate::database::DatabaseManager;
use crate::domains::ai::chat_title::{
    fallback_title_from_message, is_placeholder_title, should_apply_generated_title,
    PLACEHOLDER_SESSION,
};
use crate::domains::ai::platform_config::{PlatformConfig, DESKTOP_CLIENT_ID};
use crate::domains::ai::platform_project;
use crate::domains::ai::services::AISettingsService;
use crate::domains::credentials::services::VaultSession;
use crate::domains::github::service::GitHubService;
use crate::domains::github::types::GitHubIssue;
use crate::domains::agents::AgentEvent;
use crate::domains::projects::move_lock;
use crate::domains::threads::{
    AckStatus, CommandAck, Effect, Envelope, PortalInput, ThreadCommand, ThreadEffects, ThreadHub,
    ThreadMeta, TurnOutcome, TurnTrigger,
};
use crate::events::{EventSink, EventSinkExt};

use super::agent_mode;
use super::diff;
use super::entities::{coder_file_change, coder_setting, coder_sub_agent, coder_thread};
use super::multitask::{fallback_prompt_for_task, parse_issue_url, DEFAULT_MAX_PARALLEL_SUBAGENTS};
use super::permissions::{self, Decision};
use super::platform_stream::{self, PlatformDone};
use super::tools;
use super::types::{
    ChangeStatus, ChatMessage, CoderAgentMode, CoderRunResult, CoderSubAgent, CoderSubAgentStatus,
    CoderThread, CoderThreadKind, CoderThreadSummary, FileChange, GitHubIssueRef,
    MultitaskCancelRequest, MultitaskCleanupRequest, MultitaskSpawnRequest, PermissionMode,
    PermissionRule, SpawnSubAgentTask,
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

/// `begin_run` hands a copy back; the run gives it to `end_run`.
#[derive(Clone)]
struct RunHandle {
    cancel: Arc<AtomicBool>,
    /// This run is the sub-agent's own work: its end changes the sub-agent's status.
    owned: bool,
}

/// How a run ended, as far as the sub-agent is concerned.
enum RunOutcome {
    /// Returned on a pending approval: the sub-agent is still working.
    Parked,
    Done,
    Failed(String),
}

/// Proof that the lifecycle lock is held (it is not reentrant: only entry points take it).
type Lifecycle<'a> = tokio::sync::MutexGuard<'a, ()>;

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
    session: Arc<VaultSession>,
    /// Set once at startup; used to stream loop-step events to whoever is watching.
    events: OnceLock<Arc<dyn EventSink>>,
    /// Set alongside `events`. The loop hands `Arc<Self>` to spawned runs, and a
    /// `&self` method cannot manufacture one — this is where it comes from.
    self_ref: OnceLock<Weak<Self>>,
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
    /// Every check-and-act on a sub-agent's status, its permit and its run holds this, and
    /// nothing else: never across `advance_platform`, `hub_command*` or a user wait.
    /// ponytail: one global lock; per-thread locks if it ever shows in a profile.
    lifecycle: Mutex<()>,
    /// A sub-agent's parallel-limit permit by child thread id: held while the sub-agent is live
    /// (a run parked on an approval keeps its slot), freed by `release_if_done`.
    child_permits: std::sync::Mutex<HashMap<String, tokio::sync::OwnedSemaphorePermit>>,
    /// Sub-agents the restart settle failed, for `report_restart_settle` to tell the hub.
    settled_on_load: std::sync::Mutex<Vec<String>>,
    /// The server-owned thread log; set once at startup. Unset, nothing is fed to it.
    hub: OnceLock<Arc<ThreadHub>>,
    /// Threads known to exist in the hub (created or confirmed once per process).
    hub_ready: Mutex<std::collections::HashSet<String>>,
    /// Threads whose latest client command came from a remote device. The effect task
    /// runs outside the daemon's task-local scope, so the mark travels here.
    remote_threads: std::sync::Mutex<std::collections::HashSet<String>>,
    /// A Retry/Edit/Approve waiting for its effect's prepare step (see `hub_command`).
    prep: std::sync::Mutex<HashMap<String, oneshot::Sender<Result<(), String>>>>,
}

impl CoderService {
    /// Build the service and hydrate threads + settings from the database.
    pub async fn new(db: Arc<DatabaseManager>, session: Arc<VaultSession>) -> Self {
        let settings = AISettingsService::new();
        let delegation_team_template_id = std::env::var("CODER_TEAM_TEMPLATE_ID")
            .ok()
            .and_then(|v| v.trim().parse::<i64>().ok());
        let svc = Self {
            db,
            session,
            events: OnceLock::new(),
            self_ref: OnceLock::new(),
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
            lifecycle: Mutex::new(()),
            child_permits: Default::default(),
            settled_on_load: Default::default(),
            hub: OnceLock::new(),
            hub_ready: Mutex::new(Default::default()),
            remote_threads: Default::default(),
            prep: Default::default(),
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
        if matches!(thread_kind, CoderThreadKind::Coordinator) || mode == CoderAgentMode::Multitask
        {
            specs.extend(tools::multitask_tool_specs());
        }
        // Needs no team template, only a reachable platform — which every turn
        // already needs. Mode-filtered like the rest, and `is_read_only` lists
        // it, so Ask mode keeps it.
        specs.extend(tools::web_search_tool_specs().into_iter().filter(|spec| {
            spec.get("function")
                .and_then(|f| f.get("name"))
                .and_then(Value::as_str)
                .map(|name| agent_mode::includes_tool(mode, name))
                .unwrap_or(false)
        }));
        if self.delegation_team_template_id.is_some() && mode != CoderAgentMode::Ask {
            specs.extend(tools::platform_tool_specs().into_iter().filter(|spec| {
                spec.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(Value::as_str)
                    .map(|name| agent_mode::includes_tool(mode, name))
                    .unwrap_or(false)
            }));
        }
        specs
    }

    /// Execute a platform-delegated tool over HTTP: `delegate_task` starts a
    /// multi-agent process and polls it to completion; `web_search` runs a query
    /// through the platform's search route.
    async fn execute_platform_tool(&self, tool: &str, args: &Value) -> Result<String, String> {
        match tool {
            "web_search" => {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|q| !q.is_empty())
                    .ok_or("missing arg: query")?;
                // The route caps at 10 and never paginates.
                let limit = args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .unwrap_or(5)
                    .clamp(1, 10);

                let cfg = self.platform_config();
                let base = cfg.base_url.trim_end_matches('/');
                // reqwest's own encoder rather than a hand-rolled one: a query
                // with `&`, `#` or a space is the normal case here, not an edge.
                let url = reqwest::Url::parse_with_params(
                    &format!("{base}/api/v1/search"),
                    &[("q", query), ("limit", &limit.to_string())],
                )
                .map_err(|e| format!("bad search url: {e}"))?;
                let body = self.platform_get(url.as_str()).await?;
                Ok(format_search_results(&body))
            }
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
                let mut body = json!({
                    "goal": goal,
                    "auto_approve": true,
                    "team_template_id": team_id,
                    "client_id": "portal-coder",
                });
                if let Some(project_id) = self.resolve_project_id(base, &cfg).await? {
                    body["project_id"] = json!(project_id);
                }
                let start = self
                    .platform_post(&format!("{base}/api/v1/processes"), body)
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

    /// `Some(id)` only when the configured credential is a workspace token, for
    /// which `POST /api/v1/processes` requires one. See `ai::platform_project`.
    ///
    /// ponytail: re-resolved on every delegation. One extra GET in front of a
    /// process that then gets polled 60 times; cache it if that ever shows up.
    async fn resolve_project_id(
        &self,
        base: &str,
        cfg: &PlatformConfig,
    ) -> Result<Option<i64>, String> {
        if !platform_project::needs_project_id(cfg.api_token.as_deref()) {
            return Ok(None);
        }
        let url = format!("{base}/api/v1/projects/");
        let listed = self.platform_get(&url).await?;
        if let Some(id) = platform_project::first_project_id(&listed) {
            return Ok(Some(id));
        }
        let created = self
            .platform_post(&url, platform_project::create_body())
            .await?;
        created
            .get("id")
            .and_then(Value::as_i64)
            .map(Some)
            .ok_or_else(|| format!("project create returned no id: {created}"))
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
        } else if tool == "ask_user" {
            // The tool_call message already put the question card on screen.
            self.feed(
                local_thread_id,
                PortalInput::HostRequest { call_id: call_id.into(), kind: "ask_user".into(), payload: args.clone() },
            )
            .await;
            self.wait_user_answer(local_thread_id, call_id).await
        } else if tool == "list_terminals" {
            self.feed(
                local_thread_id,
                PortalInput::HostRequest { call_id: call_id.into(), kind: "list_terminals".into(), payload: json!({}) },
            )
            .await;
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

    /// Wire up event streaming. Takes the owning `Arc` because the loop spawns
    /// runs that outlive any `&self` borrow; called once at startup.
    pub fn init_events(self: &Arc<Self>, events: Arc<dyn EventSink>) {
        let _ = self.events.set(events);
        let _ = self.self_ref.set(Arc::downgrade(self));
    }

    pub fn attach_hub(&self, hub: Arc<ThreadHub>) {
        let _ = self.hub.set(hub);
    }

    /// Tell the hub which sub-agents the restart settle failed. After `legacy_import`, which is
    /// the only hub writer until then.
    pub async fn report_restart_settle(&self) {
        let ids = std::mem::take(&mut *self.settled_on_load.lock().unwrap_or_else(|e| e.into_inner()));
        for id in ids {
            let sa = self.sub_agents.lock().await.get(&id).cloned();
            if let Some(sa) = sa {
                self.feed_sub_agent(&sa).await;
            }
        }
    }

    /// The hub, once this thread exists in it: a thread with no `threads` row yet
    /// (made before the hub, or by a build that predates it) is created and seeded
    /// from the in-memory transcript. `None` while no hub is attached.
    async fn hub_for(&self, thread_id: &str) -> Option<Arc<ThreadHub>> {
        let hub = self.hub.get()?;
        let mut ready = self.hub_ready.lock().await;
        if !ready.contains(thread_id) {
            let t = self.get_thread(thread_id).await?;
            if hub.snapshot(thread_id).await.ok()?.is_none() {
                let meta = ThreadMeta {
                    provider: "portal".into(),
                    model: t.model,
                    llm_provider: t.llm_provider,
                    workspace_root: t.workspace_root,
                    project_id: t.project_id.map(i64::from),
                    kind: thread_kind_str(t.thread_kind).into(),
                    title: t.title,
                };
                let create = Envelope { cmd: ThreadCommand::Create(meta), actor: "system".into(), command_id: None };
                let ack = hub.command(thread_id, create).await.ok()?;
                if ack.status == AckStatus::Accepted {
                    // The system prompt sits in the transcript, so the hub's indexes match ours.
                    let reconcile = PortalInput::Reconcile { messages: t.messages, title: None };
                    hub.submit(thread_id, ThreadCommand::Portal(reconcile));
                } else if ack.code.as_deref() != Some("already_created") {
                    return None; // not ready: the next call tries again
                }
            }
            // Sub-agents from before the first feed, once per process (apply overwrites by id).
            for sa in self.sub_agents.lock().await.values().filter(|s| s.coordinator_thread_id == thread_id) {
                hub.submit(thread_id, ThreadCommand::SubAgentUpdated { sub_agent: sa.clone() });
            }
            ready.insert(thread_id.to_string());
        }
        Some(Arc::clone(hub))
    }

    /// Report what this engine just did (actor `system`). Never fails the run.
    async fn feed(&self, thread_id: &str, input: PortalInput) {
        self.feed_cmd(thread_id, ThreadCommand::Portal(input)).await;
    }

    async fn feed_cmd(&self, thread_id: &str, cmd: ThreadCommand) {
        if let Some(hub) = self.hub_for(thread_id).await {
            hub.submit(thread_id, cmd);
        }
    }

    /// A client command through the hub: dedup, first-wins and the queue live there;
    /// what it decides comes back as an `Effect` (see the `ThreadEffects` impl).
    ///
    /// Retry, Edit and Approve return the effect's prepare error, but the hub has already
    /// accepted the command and recorded its `command_id`: a client that retries a failed
    /// one must use a fresh id, or it gets `Duplicate` (a success) and nothing runs.
    pub async fn hub_command(
        &self,
        thread_id: &str,
        cmd: ThreadCommand,
        actor: String,
        command_id: Option<String>,
    ) -> Result<CommandAck, String> {
        // A sub-agent's first turn is the spawn task's; a user turn before it would be mistaken for it.
        if matches!(cmd, ThreadCommand::Send { .. } | ThreadCommand::Retry | ThreadCommand::EditMessage { .. })
            // The sub-agent decides; the thread kind only for a child whose row is not there yet
            // (the kind is user-changeable, so it cannot be what lets a Pending child through).
            && match self.sub_agent_of(thread_id).await {
                Some(s) => s.status == CoderSubAgentStatus::Pending,
                None => self.threads.lock().await.get(thread_id).is_some_and(|t| t.thread_kind == CoderThreadKind::SubAgent),
            }
        {
            return Err("sub-agent has not started yet".into());
        }
        let remote = crate::domains::credentials::services::REMOTE_CALLER.try_with(|_| ()).is_ok();
        self.hub_command_as(thread_id, cmd, actor, command_id, remote).await
    }

    /// `hub_command` for a caller whose remote mark is no longer a task-local (a spawned task).
    async fn hub_command_as(
        &self,
        thread_id: &str,
        cmd: ThreadCommand,
        actor: String,
        command_id: Option<String>,
        remote: bool,
    ) -> Result<CommandAck, String> {
        let hub = self.hub_for(thread_id).await.ok_or("thread not found")?;
        {
            let mut set = self.remote_threads.lock().unwrap_or_else(|e| e.into_inner());
            if remote {
                set.insert(thread_id.to_string());
            } else {
                set.remove(thread_id);
            }
        }
        // Retry, Edit and Approve change the transcript the caller re-reads right after:
        // wait (bounded) for the effect's prepare step, and hand its error back.
        let prepared = matches!(cmd, ThreadCommand::Retry | ThreadCommand::EditMessage { .. } | ThreadCommand::Approve { .. })
            .then(|| {
                let (tx, rx) = oneshot::channel();
                self.prep.lock().unwrap_or_else(|e| e.into_inner()).insert(thread_id.to_string(), tx);
                rx
            });
        let ack = hub.command(thread_id, Envelope { cmd, actor, command_id }).await;
        let Some(rx) = prepared else { return ack };
        if !matches!(&ack, Ok(a) if a.status == AckStatus::Accepted) {
            self.prep.lock().unwrap_or_else(|e| e.into_inner()).remove(thread_id);
            return ack;
        }
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(Err(e))) => Err(e),
            _ => ack,
        }
    }

    /// The hub's state of a thread; `None` while no hub is attached or the thread is unknown.
    async fn hub_state(&self, thread_id: &str) -> Option<crate::domains::threads::ThreadState> {
        let hub = self.hub_for(thread_id).await?;
        Some(hub.snapshot(thread_id).await.ok()??.state)
    }

    /// An approval only the legacy transcript knows (parked before the hub existed):
    /// nothing open in the hub, and the call is still there to answer.
    pub async fn is_legacy_approval(&self, thread_id: &str, call_id: &str) -> bool {
        let idle = !self.hub_state(thread_id).await.is_some_and(|s| s.turn_open());
        idle && self.find_open_call(thread_id, call_id).await.is_some()
    }

    fn prepared(&self, thread_id: &str, res: &Result<(), String>) {
        if let Some(tx) = self.prep.lock().unwrap_or_else(|e| e.into_inner()).remove(thread_id) {
            let _ = tx.send(res.clone());
        }
    }

    /// The legacy transcript is the truth: the hub diffs it against its own.
    async fn reconcile(&self, thread_id: &str) {
        if let Some(t) = self.get_thread(thread_id).await {
            self.feed(thread_id, PortalInput::Reconcile { messages: t.messages, title: Some(t.title) }).await;
        }
    }

    /// Register the run's cancel flag before anything can ask to stop it, so a Stop
    /// that lands before the run task starts still reaches it.
    async fn begin_run(&self, _g: &Lifecycle<'_>, thread_id: &str, owned: bool) -> RunHandle {
        let handle = RunHandle { cancel: Arc::new(AtomicBool::new(false)), owned };
        if let Some(old) = self.active_runs.lock().await.insert(thread_id.to_string(), handle.clone()) {
            old.cancel.store(true, Ordering::SeqCst);
        }
        handle
    }

    /// The one place a run ends. Drops the handle (unless a newer run replaced it), moves an
    /// owned run's sub-agent to its final status, then frees the permit if nothing holds it.
    /// Returns whether the run was stopped: its flag is set and it was still the current run
    /// (a newer `begin_run` sets the old flag too, which is not a stop).
    async fn end_run(&self, g: &Lifecycle<'_>, thread_id: &str, token: &RunHandle, outcome: RunOutcome) -> bool {
        let current = {
            let mut runs = self.active_runs.lock().await;
            let current = runs.get(thread_id).is_some_and(|h| Arc::ptr_eq(&h.cancel, &token.cancel));
            if current {
                runs.remove(thread_id);
            }
            current
        };
        let stopped = current && token.cancel.load(Ordering::SeqCst);
        if token.owned {
            let cancelled = Some("Cancelled by user.".to_string());
            match outcome {
                _ if stopped => self.sync_sub_agent_from_thread(g, thread_id, CoderSubAgentStatus::Cancelled, cancelled).await,
                RunOutcome::Parked => {}
                RunOutcome::Done => self.sync_sub_agent_from_thread(g, thread_id, CoderSubAgentStatus::Completed, None).await,
                RunOutcome::Failed(e) => {
                    self.sync_sub_agent_from_thread(g, thread_id, CoderSubAgentStatus::Failed, Some(e)).await
                }
            }
        }
        self.release_if_done(g, thread_id).await;
        stopped
    }

    async fn sub_agent_of(&self, child_thread_id: &str) -> Option<CoderSubAgent> {
        self.sub_agents.lock().await.values().find(|s| s.child_thread_id == child_thread_id).cloned()
    }

    /// Free the child's permit once its sub-agent is over (or it has none) and no run is left.
    async fn release_if_done(&self, _g: &Lifecycle<'_>, thread_id: &str) {
        let over = self.sub_agent_of(thread_id).await.is_none_or(|s| s.status.is_terminal());
        if over && !self.active_runs.lock().await.contains_key(thread_id) {
            self.child_permits.lock().unwrap_or_else(|e| e.into_inner()).remove(thread_id);
        }
    }

    /// End a live sub-agent by the user's hand: Cancelled at once, its own run flagged (it ends
    /// through `end_run`), or, with no run, its parked or not-yet-started turn closed. A plain run
    /// in the child is the user's and is left alone. False: no live sub-agent, nothing done.
    async fn cancel_sub_agent(&self, g: &Lifecycle<'_>, thread_id: &str) -> bool {
        if self.sub_agent_of(thread_id).await.is_none_or(|s| s.status.is_terminal()) {
            return false;
        }
        let run = self.active_runs.lock().await.get(thread_id).map(|h| (h.cancel.clone(), h.owned));
        self.sync_sub_agent_from_thread(g, thread_id, CoderSubAgentStatus::Cancelled, Some("Cancelled by user.".into())).await;
        match run {
            Some((cancel, true)) => {
                cancel.store(true, Ordering::SeqCst);
                self.cancel_pending_commands(thread_id).await;
                self.emit("coder://running", json!({ "thread_id": thread_id, "running": false }));
            }
            Some((_, false)) => {}
            None => self.feed(thread_id, PortalInput::Finished { outcome: TurnOutcome::Stopped, final_text: None }).await,
        }
        self.release_if_done(g, thread_id).await;
        true
    }

    /// The spawn task's permit step: Pending keeps the permit, anything else (cancelled while it
    /// waited for a slot) drops it. False: nothing may start.
    async fn adopt_permit(&self, sub_agent_id: &str, child_thread_id: &str, permit: tokio::sync::OwnedSemaphorePermit) -> bool {
        let _g = self.lifecycle.lock().await;
        let pending = self.sub_agents.lock().await.get(sub_agent_id).is_some_and(|s| s.status == CoderSubAgentStatus::Pending);
        if pending {
            self.child_permits.lock().unwrap_or_else(|e| e.into_inner()).insert(child_thread_id.to_string(), permit);
        }
        pending
    }

    /// Pending -> Failed, and only from Pending (compare-and-set under the lock).
    async fn fail_pending(&self, sub_agent_id: &str, error: String) {
        let g = self.lifecycle.lock().await;
        let sa = self.sub_agents.lock().await.get(sub_agent_id).cloned();
        let Some(sa) = sa.filter(|s| s.status == CoderSubAgentStatus::Pending) else {
            return;
        };
        self.sync_sub_agent_from_thread(&g, &sa.child_thread_id, CoderSubAgentStatus::Failed, Some(error)).await;
        self.release_if_done(&g, &sa.child_thread_id).await;
    }

    fn service_arc(&self) -> Result<Arc<Self>, String> {
        self.self_ref
            .get()
            .ok_or("coder service not initialised")?
            .upgrade()
            .ok_or_else(|| "coder service has been dropped".to_string())
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

        let payload = json!({
            "thread_id": thread_id,
            "call_id": call_id,
            "command": command,
            "terminal_id": terminal_id,
            "workspace_root": workspace_root,
        });
        self.feed(
            thread_id,
            PortalInput::HostRequest { call_id: call_id.into(), kind: "run_command".into(), payload: payload.clone() },
        )
        .await;
        self.emit("coder://run_command", payload);

        match tokio::time::timeout(std::time::Duration::from_secs(120), rx).await {
            Ok(Ok(result)) => result,
            _ => {
                self.pending_commands.lock().await.remove(&key);
                self.expire_host(thread_id, call_id).await;
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
                self.expire_host(thread_id, call_id).await;
                Err(())
            }
        }
    }

    /// Park an `ask_user` call until the card answers (or Stop cancels it).
    // ponytail: the platform drops a delegated call after 300s, so give up just
    // before that; waiting longer needs a platform-side change.
    async fn wait_user_answer(&self, thread_id: &str, call_id: &str) -> String {
        let key = Self::pending_key(thread_id, call_id);
        let (tx, rx) = oneshot::channel();
        self.pending_commands
            .lock()
            .await
            .insert(key.clone(), PendingCommand { tx });
        match tokio::time::timeout(std::time::Duration::from_secs(290), rx).await {
            Ok(Ok(answer)) => answer,
            _ => {
                self.pending_commands.lock().await.remove(&key);
                self.expire_host(thread_id, call_id).await;
                "Error: the user did not answer in time. Do not assume an answer; end your turn."
                    .into()
            }
        }
    }

    /// A host request nobody answered in time: resolve it in the hub too, so a late
    /// answer is rejected there instead of accepted and dropped. The resulting
    /// `DeliverHostResult` finds no waiter and does nothing.
    async fn expire_host(&self, thread_id: &str, call_id: &str) {
        let cmd = ThreadCommand::ResolveHost { call_id: call_id.into(), result: "timeout".into() };
        self.feed_cmd(thread_id, cmd).await;
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
        if let Some(events) = self.events.get() {
            events.emit(event, payload);
        }
    }

    /// The legacy `coder://change` event, plus the same fact in the thread log.
    async fn emit_change(&self, change: &FileChange) {
        self.emit("coder://change", json!({ "change": change }));
        let status = serde_json::to_value(change.status).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
        let cmd = ThreadCommand::ChangeRecorded { change_id: change.id.clone(), path: change.path.clone(), status };
        self.feed_cmd(&change.thread_id, cmd).await;
    }

    /// The legacy `coder://subagent-*` payload is the same sub-agent; the log keeps it on the coordinator.
    async fn feed_sub_agent(&self, sub_agent: &CoderSubAgent) {
        let cmd = ThreadCommand::SubAgentUpdated { sub_agent: sub_agent.clone() };
        self.feed_cmd(&sub_agent.coordinator_thread_id, cmd).await;
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
        self.settle_sub_agents_after_restart().await;
    }

    /// No run survives a restart: Pending and Running sub-agents end as failed, except a
    /// Running one with an open approval in its hub thread, which is genuinely parked.
    async fn settle_sub_agents_after_restart(&self) {
        let g = self.lifecycle.lock().await;
        let live: Vec<CoderSubAgent> = self.sub_agents.lock().await.values().filter(|s| !s.status.is_terminal()).cloned().collect();
        for sa in live {
            let parked = sa.status == CoderSubAgentStatus::Running
                && matches!(
                    crate::domains::threads::store::load(self.db.get_connection(), &sa.child_thread_id).await,
                    Ok(Some(st)) if st.turn_open() && !st.approvals.is_empty()
                );
            if parked {
                // ponytail: re-take a slot if one is free; past the limit a parked child runs unlimited once approved.
                if let Ok(permit) = Arc::clone(&self.multitask_semaphore).try_acquire_owned() {
                    self.child_permits.lock().unwrap_or_else(|e| e.into_inner()).insert(sa.child_thread_id.clone(), permit);
                }
            } else {
                let _ = self
                    .mark_sub_agent_status(&g, &sa.id, CoderSubAgentStatus::Failed, None, Some("interrupted by restart".into()))
                    .await;
                // No hub yet: `report_restart_settle` tells it.
                self.settled_on_load.lock().unwrap_or_else(|e| e.into_inner()).push(sa.id.clone());
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
        self.emit_change(&change).await;
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
            self.emit_change(&snapshot).await;
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
        self.emit_change(&snapshot).await;
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
        self.hub_for(&thread.id).await;
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

    /// Gate for caller-chosen workspace roots: the root must sit at or under a
    /// registered project, an existing thread's root or a sub-agent worktree,
    /// and never be a UNC path (board #864). Stops a paired device reading or
    /// running git anywhere on disk by naming an arbitrary `workspace_root`.
    pub async fn ensure_known_root(&self, root: &str) -> Result<(), String> {
        super::tools::reject_unc(root)?;
        let mut known: Vec<String> = crate::entities::project::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| format!("load projects: {e}"))?
            .into_iter()
            .map(|p| p.path)
            .collect();
        known.extend(self.threads.lock().await.values().map(|t| t.workspace_root.clone()));
        known.extend(self.sub_agents.lock().await.values().map(|s| s.workspace_root.clone()));
        if known.iter().any(|k| move_lock::is_within(root, k)) {
            Ok(())
        } else {
            Err(format!("Not a known project or workspace: {root}"))
        }
    }

    pub async fn get_thread(&self, id: &str) -> Option<CoderThread> {
        self.threads.lock().await.get(id).cloned()
    }

    /// True if any active run, pending `run_command`, or running sub-agent is
    /// working in a thread whose `workspace_root` is at or under `path`.
    /// Used by the project-relocation in-use check — a move must not start
    /// while the coder is mid-edit there.
    pub async fn uses_under(&self, path: &str) -> bool {
        let threads = self.threads.lock().await;
        let thread_under = |id: &str| {
            threads
                .get(id)
                .is_some_and(|t| move_lock::is_under(&t.workspace_root, path))
        };

        if self
            .active_runs
            .lock()
            .await
            .keys()
            .any(|id| thread_under(id))
        {
            return true;
        }
        if self
            .pending_commands
            .lock()
            .await
            .keys()
            .any(|key| thread_under(key.split_once(':').map_or(key.as_str(), |(t, _)| t)))
        {
            return true;
        }
        self.sub_agents.lock().await.values().any(|sa| {
            sa.status == CoderSubAgentStatus::Running
                && move_lock::is_under(&sa.workspace_root, path)
        })
    }

    pub async fn delete_thread(&self, id: &str) -> Result<bool, String> {
        {
            let g = self.lifecycle.lock().await;
            if self.active_runs.lock().await.contains_key(id) {
                return Err("cannot delete a thread that is currently running".to_string());
            }
            let busy = self.sub_agents.lock().await.values().any(|s| {
                s.coordinator_thread_id == id && !s.status.is_terminal()
            });
            if busy {
                return Err("cannot delete a thread whose sub-agents are still running".to_string());
            }
            // A child with a live sub-agent (parked or not started) ends it, freeing its permit.
            self.cancel_sub_agent(&g, id).await;
        }
        // Unknown first, so a late feed finds no thread to re-create in the hub.
        let removed = self.threads.lock().await.remove(id).is_some();
        self.hub_ready.lock().await.remove(id);
        if removed {
            let _ = coder_thread::Entity::delete_by_id(id.to_string())
                .exec(self.db.get_connection())
                .await;
            // `hub.delete` is only safe with no live run: stop it and wait (bounded) for it to leave.
            self.stop(id).await;
            for _ in 0..50 {
                if !self.active_runs.lock().await.contains_key(id) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            if let Some(hub) = self.hub.get() {
                let _ = hub.delete(id).await;
            }
        }
        Ok(removed)
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
        if let Some(t) = self.get_thread(id).await {
            self.feed_cmd(id, ThreadCommand::SetModel { model: t.model, llm_provider: t.llm_provider }).await;
        }
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
            self.feed_sub_agent(&sub_agent).await;
            self.emit(
                "coder://subagent-started",
                json!({
                    "coordinator_id": sub_agent.coordinator_thread_id,
                    "subagent": sub_agent,
                }),
            );

            let svc = Arc::clone(self);
            let remote = crate::domains::credentials::services::REMOTE_CALLER.try_with(|_| ()).is_ok();
            let prompt = fallback_prompt_for_task(&task);
            let child_thread_id = child_thread.id.clone();
            let sub_agent_id = sub_agent.id.clone();
            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let Ok(permit) = semaphore.acquire_owned().await else {
                    svc.fail_pending(&sub_agent_id, "parallel limiter unavailable".into()).await;
                    return;
                };
                if !svc.adopt_permit(&sub_agent_id, &child_thread_id, permit).await {
                    return;
                }
                // Through the hub, so the child's user message and turn are in the log. The
                // `subagent:<id>` actor is how its StartTurn effect knows this turn is the sub-agent's.
                let actor = format!("subagent:{sub_agent_id}");
                let sent = svc
                    .hub_command_as(&child_thread_id, ThreadCommand::Send { text: prompt }, actor, None, remote)
                    .await
                    .and_then(|ack| match ack.status {
                        AckStatus::Rejected => Err(ack.code.unwrap_or_else(|| "rejected".into())),
                        _ => Ok(()),
                    });
                if let Err(e) = sent {
                    svc.fail_pending(&sub_agent_id, e).await;
                }
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
            // A target that already ended is skipped inside.
            let g = self.lifecycle.lock().await;
            self.cancel_sub_agent(&g, &sub_agent.child_thread_id).await;
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
        move_lock::ensure_unlocked(&repo_root)?;
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
        GitHubService::new(Arc::clone(&self.db), Arc::clone(&self.session))
            .get_issue(&issue_ref.owner, &issue_ref.repo, issue_ref.number)
            .await
    }

    /// The only writer of a sub-agent's status. A final status is final: `Ok(false)` when refused.
    async fn mark_sub_agent_status(
        &self,
        _g: &Lifecycle<'_>,
        sub_agent_id: &str,
        status: CoderSubAgentStatus,
        summary: Option<String>,
        error: Option<String>,
    ) -> Result<bool, String> {
        let snapshot = {
            let mut sub_agents = self.sub_agents.lock().await;
            let sub_agent = sub_agents
                .get_mut(sub_agent_id)
                .ok_or("sub-agent not found")?;
            if sub_agent.status.is_terminal() {
                return Ok(false);
            }
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
        self.feed_sub_agent(&snapshot).await;
        self.emit(
            "coder://subagent-progress",
            json!({
                "coordinator_id": snapshot.coordinator_thread_id,
                "subagent": snapshot,
            }),
        );
        Ok(true)
    }

    async fn sync_sub_agent_from_thread(
        &self,
        g: &Lifecycle<'_>,
        child_thread_id: &str,
        status: CoderSubAgentStatus,
        error: Option<String>,
    ) {
        let Some(sub_agent) = self.sub_agent_of(child_thread_id).await else {
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

        let marked = self
            .mark_sub_agent_status(g, &sub_agent.id, status, summary, error)
            .await;

        if matches!(marked, Ok(true)) && status.is_terminal() {
            let finished = self.sub_agents.lock().await.get(&sub_agent.id).cloned();
            self.emit(
                "coder://subagent-finished",
                json!({
                    "coordinator_id": sub_agent.coordinator_thread_id,
                    "subagent": finished,
                }),
            );
            let subagents = self.list_sub_agents(&sub_agent.coordinator_thread_id).await;
            let all_done = subagents.iter().all(|item| item.status.is_terminal());
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
    /// streams via `coder://*` events. `token` is from `begin_run`.
    fn spawn_run(self: Arc<Self>, thread_id: String, turn: AgentTurn, token: RunHandle) {
        let svc = Arc::clone(&self);
        let tid = thread_id.clone();
        // tokio::spawn drops task-locals: carry the remote-caller mark into the run so a
        // remote-started agent cannot pop the desktop vault prompt. A run started by an
        // effect has no task-local either; `hub_command` recorded the mark per thread.
        let remote = crate::domains::credentials::services::REMOTE_CALLER.try_with(|_| ()).is_ok()
            || self.remote_threads.lock().unwrap_or_else(|e| e.into_inner()).contains(&thread_id);

        let run = async move {
            svc.emit(
                "coder://running",
                json!({ "thread_id": tid, "running": true }),
            );

            // Its own task, so a panic in the loop still reaches `end_run` instead of
            // stranding the run handle and the permit.
            let work = {
                let (svc, tid, cancel) = (Arc::clone(&svc), tid.clone(), token.cancel.clone());
                async move { svc.advance_platform(&tid, turn, Some(cancel)).await }
            };
            let step = if remote {
                tokio::spawn(crate::domains::credentials::services::REMOTE_CALLER.scope((), work))
            } else {
                tokio::spawn(work)
            };
            let result = step.await.unwrap_or_else(|e| Err(format!("run panicked: {e}")));

            let outcome = match &result {
                Ok(r) if r.pending.is_some() => RunOutcome::Parked,
                Ok(_) => RunOutcome::Done,
                Err(e) => RunOutcome::Failed(e.clone()),
            };
            let stopped = {
                let g = svc.lifecycle.lock().await;
                svc.end_run(&g, &tid, &token, outcome).await
            };
            svc.emit(
                "coder://running",
                json!({ "thread_id": tid, "running": false }),
            );

            svc.report_run_end(&tid, result, stopped).await;
        };
        if remote {
            tokio::spawn(crate::domains::credentials::services::REMOTE_CALLER.scope((), run));
        } else {
            tokio::spawn(run);
        }
    }

    /// What a run's exit tells the hub and the UI.
    async fn report_run_end(&self, tid: &str, result: Result<CoderRunResult, String>, stopped: bool) {
        match result {
            Err(e) => {
                eprintln!("coder: run failed for {tid}: {e}");
                self.reconcile(tid).await;
                // A stop that surfaces as an error is still a stop.
                let input = if stopped {
                    PortalInput::Finished { outcome: TurnOutcome::Stopped, final_text: None }
                } else {
                    PortalInput::Failed { error: e.clone() }
                };
                self.feed(tid, input).await;
                self.emit("coder://error", json!({ "thread_id": tid, "error": e }));
            }
            // A Stop that landed after the run parked on an approval ends the turn here: nothing
            // else will, and the UI hides Stop while it is stopping. Any other `Ok` already ended
            // its turn in `finish`; a second `Finished` would close the turn a queued message opened.
            Ok(r) if stopped && r.pending.is_some() => {
                self.feed(tid, PortalInput::Finished { outcome: TurnOutcome::Stopped, final_text: None }).await;
            }
            Ok(_) => {}
        }
    }

    /// Cancel an in-flight run for a thread, if any. Never touches a sub-agent's status:
    /// that is `cancel_sub_agent`'s, or `end_run`'s via the flag this sets.
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
            self.feed(thread_id, PortalInput::Title { title: fb.clone() }).await;
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
                                    // Each frame is a chunk, like the legacy token delta.
                                    self.feed_cmd(
                                        &thread_id_for_stream,
                                        ThreadCommand::Provider(AgentEvent::TextDelta {
                                            text: cleaned.clone(),
                                            parent_tool_use_id: None,
                                        }),
                                    )
                                    .await;
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
                            // Salvage aliased arg names here, before the permission
                            // check and before the call is persisted, so gating,
                            // approval text, change tracking and execution all see
                            // the same keys.
                            let args = tools::normalize_args(
                                name,
                                data.get("arguments").cloned().unwrap_or_else(|| json!({})),
                            );
                            if call_id.is_empty() {
                                stream_error =
                                    Some("platform tool_call missing call_id".to_string());
                            } else {
                                let reasoning = if pending_assistant_text.trim().is_empty() {
                                    None
                                } else {
                                    Some(std::mem::take(&mut pending_assistant_text))
                                };
                                // ponytail: the hub's tool-call message lacks `reasoning` until the
                                // done-time Reconcile; S4a carries it (live view only).
                                self.feed_cmd(
                                    &thread_id_for_stream,
                                    ThreadCommand::Provider(AgentEvent::ToolCall {
                                        id: call_id.into(),
                                        name: name.into(),
                                        input: args.clone(),
                                        parent_tool_use_id: None,
                                    }),
                                )
                                .await;
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
                                        // `is_error: false`: the result already carries its `Error: ` text.
                                        self.feed_cmd(
                                            &thread_id_for_stream,
                                            ThreadCommand::Provider(AgentEvent::ToolResult {
                                                tool_use_id: call_id.into(),
                                                content: result.clone(),
                                                is_error: false,
                                            }),
                                        )
                                        .await;
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
                            self.feed(
                                &thread_id_for_stream,
                                PortalInput::Heartbeat { waited_seconds: waited as u32 },
                            )
                            .await;
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

                // The turn stays open (Waiting) until Approve starts the next run: a
                // `Finished` here would clear the approval just requested.
                self.feed(thread_id, PortalInput::Pending(pending.clone())).await;
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
                t.messages =
                    super::types::with_message_timestamps(messages, &old_messages, Some(&updated));
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
                t.messages =
                    super::types::with_message_timestamps(messages, &old_messages, Some(&updated));
                if should_apply_generated_title(&t.title, fallback_title) {
                    t.title = done.title.clone();
                }
                t.updated_at = updated;
            }
        }
        self.persist_thread(local_id).await;
        self.reconcile(local_id).await;
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
        let outcome = match (cancelled, exhausted) {
            (true, _) => TurnOutcome::Stopped,
            (_, true) => TurnOutcome::Exhausted,
            _ => TurnOutcome::Completed,
        };
        // A stopped run never synced: mid-turn tool messages must not outlive it in the hub.
        self.reconcile(thread_id).await;
        self.feed(thread_id, PortalInput::Finished { outcome, final_text: final_text.clone() }).await;
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
            self.feed(thread_id, PortalInput::Title { title: title.into() }).await;
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

impl CoderService {
    /// The hub already holds the user message (and any rewrite); bring this transcript
    /// to the same point and start the run. The run is registered first, so a Stop
    /// during the prepare step still reaches it.
    async fn start_turn(&self, thread_id: &str, trigger: TurnTrigger, text: String, actor: &str) -> Result<(), String> {
        let svc = self.service_arc()?;
        let token = {
            let g = self.lifecycle.lock().await;
            let sa = self.sub_agent_of(thread_id).await;
            // Only the spawn task's own Send, tagged with the sub-agent's id, starts the sub-agent.
            // Every other turn (a user's, a queued message) is a plain thread turn.
            match sa.filter(|s| actor == format!("subagent:{}", s.id)) {
                Some(s) if s.status == CoderSubAgentStatus::Pending => {
                    let _ = self
                        .mark_sub_agent_status(&g, &s.id, CoderSubAgentStatus::Running, Some("Starting agent run.".into()), None)
                        .await;
                    self.begin_run(&g, thread_id, true).await
                }
                // Cancelled before its turn started: close the turn the Send opened.
                Some(_) => {
                    self.feed(thread_id, PortalInput::Finished { outcome: TurnOutcome::Stopped, final_text: None }).await;
                    return Ok(());
                }
                None => self.begin_run(&g, thread_id, false).await,
            }
        };
        let prepared = async {
            Ok::<_, String>(match trigger {
                TurnTrigger::Retry => {
                    self.prepare_retry(thread_id).await?;
                    AgentTurn::Retry
                }
                TurnTrigger::Edit => {
                    // The hub rewrote to `[..index, edited]` and nothing has run since.
                    let hub = self.hub.get().ok_or("thread hub not attached")?;
                    let snap = hub.snapshot(thread_id).await?.ok_or("thread not found")?;
                    let index = snap.messages.len().checked_sub(1).ok_or("thread has no messages")?;
                    self.prepare_edit_message(thread_id, index, text).await?;
                    AgentTurn::Send
                }
                // `Approve` is `resume_approve`'s, never a `StartTurn`.
                _ => {
                    svc.prepare_send(thread_id, text).await?;
                    AgentTurn::Send
                }
            })
        }
        .await;
        self.prepared(thread_id, &prepared.as_ref().map(|_| ()).map_err(String::clone));
        match prepared {
            Ok(turn) => {
                CoderService::spawn_run(svc, thread_id.to_string(), turn, token);
                Ok(())
            }
            Err(e) => {
                let g = self.lifecycle.lock().await;
                self.end_run(&g, thread_id, &token, RunOutcome::Failed(e.clone())).await;
                Err(e)
            }
        }
    }

    /// Record `remember`, then continue the loop with the user's answer. Also the
    /// fallback for an approval the hub never saw (one parked before it existed).
    pub async fn resume_approve(
        &self,
        thread_id: &str,
        call_id: String,
        approve: bool,
        remember: bool,
        edited_pattern: Option<String>,
        edited_command: Option<String>,
    ) -> Result<(), String> {
        self.approve_run(thread_id, call_id, approve, remember, edited_pattern, edited_command, false).await
    }

    /// `hub_effect`: the approval came through the hub, whose turn a Cancel may have closed
    /// since (the effects are unordered): then nothing runs.
    async fn approve_run(
        &self,
        thread_id: &str,
        call_id: String,
        approve: bool,
        remember: bool,
        edited_pattern: Option<String>,
        edited_command: Option<String>,
        hub_effect: bool,
    ) -> Result<(), String> {
        let svc = self.service_arc()?;
        let token = {
            let g = self.lifecycle.lock().await;
            // The Cancel's `Finished` is in the actor's mailbox before this snapshot request.
            if hub_effect && self.hub_state(thread_id).await.is_some_and(|s| !s.turn_open()) {
                self.prepared(thread_id, &Err("turn was stopped".into()));
                return Ok(());
            }
            // A Running sub-agent's parked turn resumes as its own work; anything else is the user's own turn.
            let owned = self.sub_agent_of(thread_id).await.is_some_and(|s| s.status == CoderSubAgentStatus::Running);
            self.begin_run(&g, thread_id, owned).await
        };
        let res = self
            .prepare_approve(thread_id, &call_id, approve, remember, edited_pattern, None, None)
            .await;
        self.prepared(thread_id, &res);
        if let Err(e) = res {
            let g = self.lifecycle.lock().await;
            self.end_run(&g, thread_id, &token, RunOutcome::Failed(e.clone())).await;
            return Err(e);
        }
        let turn = AgentTurn::Approve { call_id, approve, edited_command };
        CoderService::spawn_run(svc, thread_id.to_string(), turn, token);
        Ok(())
    }
}

#[async_trait::async_trait]
impl ThreadEffects for CoderService {
    async fn run(&self, thread_id: &str, effect: Effect) {
        let started = match effect {
            Effect::StartTurn { trigger, text, actor } => {
                // A queued message runs with its own sender's mark, not the last command's:
                // a phone's message must not pop the desktop vault prompt, nor the reverse.
                if trigger == TurnTrigger::Queue {
                    let mut set = self.remote_threads.lock().unwrap_or_else(|e| e.into_inner());
                    if actor == "local" {
                        set.remove(thread_id);
                    } else {
                        set.insert(thread_id.to_string());
                    }
                }
                self.start_turn(thread_id, trigger, text, &actor).await
            }
            Effect::ResolveApproval { call_id, allow, remember, edited_pattern, edited_command } => {
                self.approve_run(thread_id, call_id, allow, remember, edited_pattern, edited_command, true).await
            }
            // Idempotent: a repeated Stop lands here again, and `Finished` ends nothing twice.
            Effect::Cancel => {
                let g = self.lifecycle.lock().await;
                // The sub-agent's own run (or its parked / not-yet-started turn) ends the
                // sub-agent; a plain run, or a thread with none, is a plain stop.
                let own = match self.active_runs.lock().await.get(thread_id).map(|h| h.owned) {
                    Some(owned) => owned,
                    None => self.sub_agent_of(thread_id).await.is_some_and(|s| !s.status.is_terminal()),
                };
                if !(own && self.cancel_sub_agent(&g, thread_id).await) && !self.stop(thread_id).await {
                    // No run in flight (parked on an approval): nobody else will end the turn.
                    let done = PortalInput::Finished { outcome: TurnOutcome::Stopped, final_text: None };
                    self.feed(thread_id, done).await;
                }
                Ok(())
            }
            Effect::DeliverHostResult { call_id, result } => {
                let text = match result {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                self.submit_command_result(thread_id.to_string(), call_id, text).await
            }
        };
        if let Err(e) = started {
            eprintln!("coder: effect for {thread_id} failed: {e}");
            // What the legacy run exit did, so its UI recovers too.
            self.reconcile(thread_id).await;
            self.feed(thread_id, PortalInput::Failed { error: e.clone() }).await;
            self.emit("coder://running", json!({ "thread_id": thread_id, "running": false }));
            self.emit("coder://error", json!({ "thread_id": thread_id, "error": e }));
        }
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
        obj.insert(
            "agent_mode".into(),
            json!(agent_mode::mode_as_str(agent_mode)),
        );
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
    move_lock::ensure_unlocked(path)?;
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

/// Render `GET /api/v1/search` for a model to read.
///
/// The route answers `configured: false` with an empty `results` list when no
/// `SEARCH_API_KEY` is set on the platform — never a 503. That has to read as
/// "search is switched off", not as "the web has nothing", or the model will
/// conclude the thing it searched for does not exist.
fn format_search_results(body: &Value) -> String {
    let query = body.get("query").and_then(Value::as_str).unwrap_or("");

    if body.get("configured").and_then(Value::as_bool) == Some(false) {
        return "Web search is not configured on the agent-platform (no SEARCH_API_KEY). \
                This is not a result — treat the web as unavailable, not as empty."
            .to_string();
    }

    let results = body.get("results").and_then(Value::as_array);
    let Some(results) = results.filter(|r| !r.is_empty()) else {
        return format!("No results for `{query}`.");
    };

    let mut out = format!("Results for `{query}`:\n");
    for (i, r) in results.iter().enumerate() {
        let field = |k: &str| {
            r.get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string()
        };
        out.push_str(&format!(
            "\n{}. {}\n   {}\n",
            i + 1,
            field("title"),
            field("url")
        ));
        let snippet = field("snippet");
        if !snippet.is_empty() {
            out.push_str(&format!("   {snippet}\n"));
        }
    }
    if let Some(total) = body.get("total_estimate").and_then(Value::as_i64) {
        out.push_str(&format!(
            "\n({total} estimated matches; showing the top ones.)"
        ));
    }
    out
}

#[cfg(test)]
mod search_format_tests {
    use super::format_search_results;
    use serde_json::json;

    /// The whole reason this function exists: the platform answers an
    /// unconfigured search with `configured: false` and `results: []`, which is
    /// byte-for-byte what "nothing matched" looks like. They must not read the
    /// same to the model.
    #[test]
    fn unconfigured_does_not_read_as_no_results() {
        let unconfigured = format_search_results(
            &json!({ "query": "axum sse", "configured": false, "results": [] }),
        );
        let empty = format_search_results(
            &json!({ "query": "axum sse", "configured": true, "results": [] }),
        );

        assert!(unconfigured.contains("not configured"));
        assert!(!unconfigured.contains("No results"));
        assert!(empty.contains("No results"));
        assert_ne!(unconfigured, empty);
    }

    #[test]
    fn results_render_title_url_and_snippet() {
        let out = format_search_results(&json!({
            "query": "axum sse",
            "configured": true,
            "total_estimate": 42,
            "results": [
                { "title": "Axum SSE", "url": "https://example.test/sse", "snippet": "Server-sent events." },
                { "title": "No snippet", "url": "https://example.test/2", "snippet": "" }
            ]
        }));

        assert!(out.contains("1. Axum SSE"));
        assert!(out.contains("https://example.test/sse"));
        assert!(out.contains("Server-sent events."));
        assert!(out.contains("2. No snippet"));
        assert!(out.contains("42 estimated matches"));
    }
}

#[cfg(test)]
mod hub_tests {
    use super::*;
    use crate::domains::coder::types::PendingApproval;
    use crate::domains::threads::{store, ThreadStatus};

    #[derive(Debug)]
    struct Null;
    impl EventSink for Null {
        fn emit_json(&self, _: &str, _: Value) -> bool {
            true
        }
    }

    async fn setup() -> (Arc<CoderService>, Arc<ThreadHub>, Arc<DatabaseManager>, String, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(DatabaseManager::new(dir.path().to_path_buf()).await.unwrap());
        let hub = ThreadHub::new(db.clone(), Arc::new(Null));
        let svc = Arc::new(CoderService::new(db.clone(), Arc::new(VaultSession::default())).await);
        svc.init_events(Arc::new(Null));
        svc.attach_hub(hub.clone());
        let id = svc.create_thread("/w".into(), None, None, None, None).await.id;
        (svc, hub, db, id, dir)
    }

    async fn send(svc: &CoderService, id: &str, text: &str) {
        let ack = svc.hub_command(id, ThreadCommand::Send { text: text.into() }, "local".into(), None).await.unwrap();
        assert_eq!(ack.status, AckStatus::Accepted);
    }

    /// Roles, text and tool links; timestamps differ by design (each side stamps its own).
    fn bare(messages: Vec<ChatMessage>) -> Vec<Value> {
        messages
            .into_iter()
            .map(|m| serde_json::to_value(ChatMessage { timestamp: None, ..m }).unwrap())
            .collect()
    }

    /// After a scripted Portal turn the hub's `thread_messages` equal `messages_json`.
    #[tokio::test]
    async fn parity_after_scripted_turn() {
        let (svc, hub, db, id, _dir) = setup().await;
        // What the Send effect does once the hub has appended the user message.
        send(&svc, &id, "hi").await;
        svc.prepare_send(&id, "hi".into()).await.unwrap();
        // The run: streamed text, a tool call, its result, then `done` and `finish`.
        let args = json!({ "path": "a.txt" });
        let text = AgentEvent::TextDelta { text: "let me look".into(), parent_tool_use_id: None };
        let call = AgentEvent::ToolCall { id: "c1".into(), name: "read_file".into(), input: args.clone(), parent_tool_use_id: None };
        let result = AgentEvent::ToolResult { tool_use_id: "c1".into(), content: "text".into(), is_error: false };
        for ev in [text, call, result] {
            svc.feed_cmd(&id, ThreadCommand::Provider(ev)).await;
        }
        let mut last = ChatMessage::user("");
        last.role = "assistant".into();
        last.content = Some("done".into());
        let messages = vec![
            ChatMessage::system(SYSTEM_PROMPT),
            ChatMessage::user("hi"),
            ChatMessage::assistant_tool_call("c1", "read_file", &args, Some("let me look".into())),
            ChatMessage::tool_result("c1", "text"),
            last,
        ];
        let done = PlatformDone { title: "Hi".into(), messages, pending: None, final_text: Some("done".into()), context_usage: None, llm_usage: None };
        svc.sync_thread_from_platform(&id, 1, &done, "hi").await;
        svc.finish(&id, Some("done".into()), false, false).await;
        hub.snapshot(&id).await.unwrap(); // the actor has drained everything above

        let conn = db.get_connection();
        let state = store::load(conn, &id).await.unwrap().unwrap();
        let row = coder_thread::Entity::find_by_id(id.clone()).one(conn).await.unwrap().unwrap();
        let legacy: Vec<ChatMessage> = serde_json::from_str(&row.messages_json).unwrap();
        assert_eq!(legacy.len(), 5);
        assert_eq!(bare(state.messages), bare(legacy));
        assert_eq!(state.status, crate::domains::threads::ThreadStatus::Idle);
    }

    /// A stopped run never syncs from the platform: the tool messages the hub took
    /// mid-turn must not outlive it, or an Edit by index lands on the wrong message.
    #[tokio::test]
    async fn parity_after_stopped_turn_with_tool_call() {
        let (svc, hub, db, id, _dir) = setup().await;
        send(&svc, &id, "hi").await;
        svc.prepare_send(&id, "hi".into()).await.unwrap();
        let call = AgentEvent::ToolCall { id: "c1".into(), name: "read_file".into(), input: json!({}), parent_tool_use_id: None };
        let result = AgentEvent::ToolResult { tool_use_id: "c1".into(), content: "x".into(), is_error: false };
        for ev in [call, result] {
            svc.feed_cmd(&id, ThreadCommand::Provider(ev)).await;
        }
        svc.finish(&id, None, false, true).await;
        hub.snapshot(&id).await.unwrap();

        let conn = db.get_connection();
        let state = store::load(conn, &id).await.unwrap().unwrap();
        let row = coder_thread::Entity::find_by_id(id.clone()).one(conn).await.unwrap().unwrap();
        let legacy: Vec<ChatMessage> = serde_json::from_str(&row.messages_json).unwrap();
        assert_eq!(bare(state.messages), bare(legacy));
        assert_eq!(state.status, ThreadStatus::Idle);
    }

    /// A drained message sets the thread's remote mark from its own sender.
    #[tokio::test]
    async fn queued_turn_takes_its_senders_remote_mark() {
        let (svc, _hub, _db, id, _dir) = setup().await;
        for (actor, remote) in [("device:phone", true), ("local", false)] {
            let fx = Effect::StartTurn { trigger: TurnTrigger::Queue, text: "x".into(), actor: actor.into() };
            svc.run(&id, fx).await;
            assert_eq!(svc.remote_threads.lock().unwrap().contains(&id), remote, "{actor}");
        }
    }

    /// `coder_send` is a plain Send: queued behind an open turn, started when it completes.
    #[tokio::test]
    async fn send_during_a_turn_queues_and_drains_after_it() {
        let (svc, hub, _db, id, _dir) = setup().await;
        send(&svc, &id, "first").await;
        send(&svc, &id, "second").await;
        let s = hub.snapshot(&id).await.unwrap().unwrap();
        assert_eq!(s.state.queue.len(), 1);
        assert_eq!(s.messages.last().and_then(|m| m.content.as_deref()), Some("first"));

        svc.finish(&id, None, false, false).await;
        hub.snapshot(&id).await.unwrap(); // the queue drain lands behind this one
        let s = hub.snapshot(&id).await.unwrap().unwrap();
        assert!(s.state.queue.is_empty());
        assert_eq!(s.messages.last().and_then(|m| m.content.as_deref()), Some("second"));
    }

    /// A run that returns on a pending approval leaves the turn Waiting, and Stop with no
    /// run in flight ends it by itself; a repeated Stop changes nothing.
    #[tokio::test]
    async fn pending_keeps_turn_open_and_cancel_ends_it() {
        let (svc, hub, _db, id, _dir) = setup().await;
        send(&svc, &id, "hi").await;
        let pending = PendingApproval {
            call_id: "c1".into(),
            tool: "run_command".into(),
            arguments: json!({ "command": "ls" }),
            suggested_rule: String::new(),
            summary: "ls".into(),
        };
        svc.feed(&id, PortalInput::Pending(pending)).await;
        let snap = hub.snapshot(&id).await.unwrap().unwrap();
        assert_eq!(snap.state.status, ThreadStatus::Waiting);
        assert_eq!(snap.state.approvals.len(), 1);

        svc.hub_command(&id, ThreadCommand::Stop, "local".into(), None).await.unwrap();
        for _ in 0..2 {
            svc.run(&id, Effect::Cancel).await;
            let s = hub.snapshot(&id).await.unwrap().unwrap().state;
            assert_eq!(s.status, ThreadStatus::Idle);
            assert!(!s.stopping && s.approvals.is_empty());
        }
    }

    /// #1845: a live run blocks the delete; once it ends the thread goes.
    #[tokio::test]
    async fn delete_thread_refuses_while_running() {
        let (svc, _hub, _db, id, _dir) = setup().await;
        let g = svc.lifecycle.lock().await;
        let token = svc.begin_run(&g, &id, false).await;
        drop(g);
        assert!(svc.delete_thread(&id).await.unwrap_err().contains("currently running"));
        assert!(svc.get_thread(&id).await.is_some());
        let g = svc.lifecycle.lock().await;
        svc.end_run(&g, &id, &token, RunOutcome::Done).await;
        drop(g);
        assert_eq!(svc.delete_thread(&id).await, Ok(true));
    }

    fn test_sub_agent(coordinator: &str, child: &str) -> CoderSubAgent {
        CoderSubAgent {
            id: "sa1".into(),
            coordinator_thread_id: coordinator.into(),
            child_thread_id: child.into(),
            title: "t".into(),
            workspace_root: "/w".into(),
            branch: "b".into(),
            status: CoderSubAgentStatus::Running,
            github_owner: None,
            github_repo: None,
            github_issue_number: None,
            github_issue_url: None,
            result_summary: None,
            error: None,
            created_at: now_iso(),
            updated_at: now_iso(),
        }
    }

    /// #1845: a coordinator with a live sub-agent cannot be deleted.
    #[tokio::test]
    async fn delete_coordinator_refuses_with_live_sub_agents() {
        let (svc, _hub, _db, id, _dir) = setup().await;
        svc.upsert_sub_agent(test_sub_agent(&id, "c")).await;
        assert!(svc.delete_thread(&id).await.unwrap_err().contains("sub-agents"));
        let g = svc.lifecycle.lock().await;
        svc.mark_sub_agent_status(&g, "sa1", CoderSubAgentStatus::Cancelled, None, None).await.unwrap();
        drop(g);
        assert_eq!(svc.delete_thread(&id).await, Ok(true));
    }

    /// #2005: sub-agent updates land in the hub's state.
    #[tokio::test]
    async fn sub_agent_updates_reach_the_hub() {
        let (svc, hub, _db, id, _dir) = setup().await;
        let sa = test_sub_agent(&id, "c");
        svc.feed_sub_agent(&sa).await;
        let s = hub.snapshot(&id).await.unwrap().unwrap();
        assert_eq!(s.state.sub_agents.len(), 1);
        assert_eq!(s.state.sub_agents[0].status, CoderSubAgentStatus::Running);
    }

    // ---- sub-agent lifecycle -------------------------------------------

    /// A coordinator thread already in the hub, so children's sub-agents land under it.
    async fn coordinator(svc: &CoderService) -> String {
        let id = svc.create_thread("/w".into(), None, None, None, None).await.id;
        svc.hub_for(&id).await.unwrap();
        id
    }

    /// A child thread plus its sub-agent `sa_id` in `status`.
    async fn child(svc: &CoderService, coordinator: &str, sa_id: &str, status: CoderSubAgentStatus) -> String {
        // A platform thread id and a workspace that does not exist: a run that reaches the loop fails in
        // `validate_workspace_root` with no HTTP call (the service reads the developer's real platform settings).
        let missing = std::env::temp_dir().join(format!("portal-test-missing-{}", uuid::Uuid::new_v4()));
        let c = svc.create_thread(missing.to_string_lossy().into(), None, None, Some(CoderThreadKind::SubAgent), None).await.id;
        svc.threads.lock().await.get_mut(&c).unwrap().platform_thread_id = Some(1);
        svc.upsert_sub_agent(CoderSubAgent { id: sa_id.into(), status, ..test_sub_agent(coordinator, &c) }).await;
        c
    }

    /// A turn the hub has open in `c`, as `actor` sent it. Bypasses the client gate, like the spawn task.
    async fn open_turn(svc: &CoderService, c: &str, actor: &str) {
        let ack = svc.hub_command_as(c, ThreadCommand::Send { text: "go".into() }, actor.into(), None, false).await.unwrap();
        assert_eq!(ack.status, AckStatus::Accepted);
    }

    async fn turn_open(hub: &Arc<ThreadHub>, c: &str) -> bool {
        hub.snapshot(c).await.unwrap().unwrap().state.turn_open()
    }

    async fn status_of(svc: &CoderService, sa_id: &str) -> CoderSubAgentStatus {
        svc.sub_agents.lock().await[sa_id].status
    }

    /// Take a parallel-limit permit for `c`, as the spawn task does.
    async fn hold_permit(svc: &CoderService, c: &str) {
        let p = Arc::clone(&svc.multitask_semaphore).acquire_owned().await.unwrap();
        svc.child_permits.lock().unwrap().insert(c.into(), p);
    }

    fn permits_free(svc: &CoderService) -> bool {
        svc.multitask_semaphore.available_permits() == DEFAULT_MAX_PARALLEL_SUBAGENTS
    }

    /// `Some(owned)` of the run registered for `c`.
    async fn run_owned(svc: &CoderService, c: &str) -> Option<bool> {
        svc.active_runs.lock().await.get(c).map(|h| h.owned)
    }

    /// Wait for a spawned run to leave (it fails fast: there is no platform to talk to).
    async fn settle(svc: &CoderService, c: &str) {
        for _ in 0..200 {
            if run_owned(svc, c).await.is_none() {
                drop(svc.lifecycle.lock().await); // `end_run` finishes its status and permit under this
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        panic!("run did not end");
    }

    fn start(trigger: TurnTrigger, actor: &str) -> Effect {
        Effect::StartTurn { trigger, text: "x".into(), actor: actor.into() }
    }

    /// Transition #2: the sub-agent's own StartTurn takes Pending to Running with an owned run;
    /// on a Cancelled one it runs nothing and closes the turn the Send opened.
    #[tokio::test]
    async fn system_start_turn_runs_a_pending_sub_agent_only() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        hold_permit(&svc, &c).await;
        open_turn(&svc, &c, "subagent:sa1").await;
        svc.run(&c, start(TurnTrigger::Send, "subagent:sa1")).await;
        {
            let _freeze = svc.lifecycle.lock().await; // the run's end waits for this
            assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Running);
            assert_eq!(run_owned(&svc, &c).await, Some(true));
        }
        settle(&svc, &c).await;
        assert!(status_of(&svc, "sa1").await.is_terminal());
        assert!(permits_free(&svc));

        let c = child(&svc, &coord, "sa2", CoderSubAgentStatus::Cancelled).await;
        open_turn(&svc, &c, "subagent:sa2").await;
        svc.run(&c, start(TurnTrigger::Send, "subagent:sa2")).await;
        assert_eq!(run_owned(&svc, &c).await, None);
        assert!(!turn_open(&hub, &c).await);
        assert_eq!(status_of(&svc, "sa2").await, CoderSubAgentStatus::Cancelled);
    }

    /// A user's turn in a finished child is plain (no status change, no permit), and so is a
    /// queued message that drains with actor `system` (amendment A3: only `subagent:<id>` counts).
    #[tokio::test]
    async fn user_and_system_turns_in_a_finished_child_are_plain() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Completed).await;
        for actor in ["local", "system", "subagent:other"] {
            svc.run(&c, start(TurnTrigger::Queue, actor)).await;
            {
                let _freeze = svc.lifecycle.lock().await;
                assert_eq!(run_owned(&svc, &c).await, Some(false), "{actor}");
                assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed);
                assert!(permits_free(&svc));
            }
            settle(&svc, &c).await;
            assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed, "{actor}");
        }
    }

    /// Transitions #3 and the parked case: an owned run that parks keeps Running and its permit
    /// (#2057); one that finishes is Completed and frees the permit.
    #[tokio::test]
    async fn owned_run_end_parks_or_completes() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let g = svc.lifecycle.lock().await;
        let t = svc.begin_run(&g, &c, true).await;
        assert!(!svc.end_run(&g, &c, &t, RunOutcome::Parked).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Running);
        assert!(!permits_free(&svc));
        let t = svc.begin_run(&g, &c, true).await;
        svc.end_run(&g, &c, &t, RunOutcome::Done).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed);
        assert!(permits_free(&svc));
    }

    /// Transition #4: an owned run that failed to prepare fails the sub-agent and frees the permit;
    /// a plain run's prepare failure leaves the sub-agent alone.
    #[tokio::test]
    async fn prepare_failure_fails_only_an_owned_run() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let g = svc.lifecycle.lock().await;
        let t = svc.begin_run(&g, &c, false).await;
        svc.end_run(&g, &c, &t, RunOutcome::Failed("boom".into())).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Running);
        assert!(!permits_free(&svc));
        let t = svc.begin_run(&g, &c, true).await;
        svc.end_run(&g, &c, &t, RunOutcome::Failed("boom".into())).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Failed);
        assert_eq!(svc.sub_agents.lock().await["sa1"].error.as_deref(), Some("boom"));
        assert!(permits_free(&svc));
    }

    /// Transition #7 is compare-and-set from Pending only (A7).
    #[tokio::test]
    async fn fail_pending_only_from_pending() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        svc.fail_pending("sa1", "late".into()).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Running);
        child(&svc, &coord, "sa2", CoderSubAgentStatus::Pending).await;
        svc.fail_pending("sa2", "early".into()).await;
        assert_eq!(status_of(&svc, "sa2").await, CoderSubAgentStatus::Failed);
    }

    /// Transitions #5/#6 (A1, A5): cancelling a running owned run marks Cancelled at once, flags the
    /// run, keeps the permit until `end_run`, and the run's later `Ok` does not make it Completed.
    #[tokio::test]
    async fn cancel_of_an_owned_run_waits_for_its_end() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let g = svc.lifecycle.lock().await;
        let t = svc.begin_run(&g, &c, true).await;
        assert!(svc.cancel_sub_agent(&g, &c).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(t.cancel.load(Ordering::SeqCst));
        assert!(!permits_free(&svc));
        assert!(svc.end_run(&g, &c, &t, RunOutcome::Done).await, "a flagged current run is a stop");
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(permits_free(&svc));
        assert!(!svc.cancel_sub_agent(&g, &c).await, "already final");
    }

    /// A5: cancelling the sub-agent leaves a plain run in its child running.
    #[tokio::test]
    async fn cancel_leaves_a_plain_run_alone() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        let g = svc.lifecycle.lock().await;
        let t = svc.begin_run(&g, &c, false).await;
        assert!(svc.cancel_sub_agent(&g, &c).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(!t.cancel.load(Ordering::SeqCst));
        assert_eq!(svc.active_runs.lock().await.len(), 1);
    }

    /// Transition #6 on a parked sub-agent: Cancelled, the turn closed, the permit freed.
    #[tokio::test]
    async fn cancel_of_a_parked_sub_agent_closes_the_turn() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        open_turn(&svc, &c, "subagent:sa1").await;
        let pending = PendingApproval {
            call_id: "c1".into(),
            tool: "run_command".into(),
            arguments: json!({}),
            suggested_rule: String::new(),
            summary: "ls".into(),
        };
        svc.feed(&c, PortalInput::Pending(pending)).await;
        assert!(turn_open(&hub, &c).await);
        let g = svc.lifecycle.lock().await;
        assert!(svc.cancel_sub_agent(&g, &c).await);
        drop(g);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(!turn_open(&hub, &c).await);
        assert!(permits_free(&svc));
    }

    /// #2058: a sub-agent cancelled while Pending drops the permit the spawn task then wins.
    #[tokio::test]
    async fn permit_won_after_a_pending_cancel_is_dropped() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        {
            let g = svc.lifecycle.lock().await;
            assert!(svc.cancel_sub_agent(&g, &c).await);
        }
        let permit = Arc::clone(&svc.multitask_semaphore).acquire_owned().await.unwrap();
        assert!(!svc.adopt_permit("sa1", &c, permit).await);
        assert!(permits_free(&svc));
        // And a Pending one keeps it.
        let c2 = child(&svc, &coord, "sa2", CoderSubAgentStatus::Pending).await;
        let permit = Arc::clone(&svc.multitask_semaphore).acquire_owned().await.unwrap();
        assert!(svc.adopt_permit("sa2", &c2, permit).await);
        assert!(!permits_free(&svc));
    }

    /// `multitask_cancel` skips a target that already ended: no stop, no `Finished` fed to its thread.
    #[tokio::test]
    async fn multitask_cancel_skips_a_final_target() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Completed).await;
        open_turn(&svc, &c, "local").await; // the user's own turn in the finished child
        let req = MultitaskCancelRequest { coordinator_thread_id: coord.clone(), sub_agent_id: Some("sa1".into()) };
        svc.multitask_cancel(req).await.unwrap();
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed);
        assert!(turn_open(&hub, &c).await);
    }

    fn approval() -> Effect {
        Effect::ResolveApproval { call_id: "c1".into(), allow: true, remember: false, edited_pattern: None, edited_command: None }
    }

    /// An approval in a Cancelled child is not refused: the user's own turn runs plain, the sub-agent
    /// stays Cancelled, and the waiting `prepared()` channel is answered (no 10 s hang).
    #[tokio::test]
    async fn approval_in_a_cancelled_child_runs_plain() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Cancelled).await;
        let (tx, mut rx) = oneshot::channel();
        svc.prep.lock().unwrap().insert(c.clone(), tx);
        svc.resume_approve(&c, "c1".into(), true, false, None, None).await.unwrap();
        assert_eq!(rx.try_recv(), Ok(Ok(())));
        {
            let _freeze = svc.lifecycle.lock().await;
            assert_eq!(run_owned(&svc, &c).await, Some(false));
        }
        settle(&svc, &c).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
    }

    /// A2: an approval whose turn a Cancel already closed runs nothing and answers `prepared`;
    /// one in an open turn of a Running sub-agent resumes it as its own (owned) run.
    #[tokio::test]
    async fn approval_after_cancel_runs_nothing() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let (tx, mut rx) = oneshot::channel();
        svc.prep.lock().unwrap().insert(c.clone(), tx);
        svc.run(&c, approval()).await; // no open turn
        assert_eq!(rx.try_recv(), Ok(Err("turn was stopped".to_string())));
        assert_eq!(run_owned(&svc, &c).await, None);

        open_turn(&svc, &c, "subagent:sa1").await;
        svc.run(&c, approval()).await;
        {
            let _freeze = svc.lifecycle.lock().await;
            assert_eq!(run_owned(&svc, &c).await, Some(true));
        }
        settle(&svc, &c).await;
        assert!(status_of(&svc, "sa1").await.is_terminal());
        assert!(permits_free(&svc));
    }

    /// `Effect::Cancel` on a plain run in a finished child stops that run and leaves the sub-agent be.
    #[tokio::test]
    async fn cancel_effect_on_a_plain_run_leaves_the_sub_agent() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Completed).await;
        let t = {
            let g = svc.lifecycle.lock().await;
            svc.begin_run(&g, &c, false).await
        };
        svc.run(&c, Effect::Cancel).await;
        assert!(t.cancel.load(Ordering::SeqCst));
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed);
        // On the sub-agent's own run it cancels the sub-agent.
        let c2 = child(&svc, &coord, "sa2", CoderSubAgentStatus::Running).await;
        let t2 = {
            let g = svc.lifecycle.lock().await;
            svc.begin_run(&g, &c2, true).await
        };
        svc.run(&c2, Effect::Cancel).await;
        assert!(t2.cancel.load(Ordering::SeqCst));
        assert_eq!(status_of(&svc, "sa2").await, CoderSubAgentStatus::Cancelled);
    }

    /// A7: a client Send, Retry or Edit into a child that has not started (Pending, or no sub-agent
    /// row yet) is refused before the hub sees it; a finished child takes them.
    #[tokio::test]
    async fn client_turns_into_a_pending_child_are_refused() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let pending = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        let orphan = svc.create_thread("/w".into(), None, None, Some(CoderThreadKind::SubAgent), None).await.id;
        for c in [&pending, &orphan] {
            for cmd in [
                ThreadCommand::Send { text: "x".into() },
                ThreadCommand::Retry,
                ThreadCommand::EditMessage { index: 0, text: "x".into() },
            ] {
                let err = svc.hub_command(c, cmd, "local".into(), None).await.unwrap_err();
                assert!(err.contains("not started"), "{err}");
            }
        }
        let done = child(&svc, &coord, "sa2", CoderSubAgentStatus::Completed).await;
        send(&svc, &done, "hello").await;
    }

    /// #1845 (kept): deleting a parked child cancels its sub-agent and frees the permit.
    #[tokio::test]
    async fn delete_of_a_parked_child_frees_its_permit() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        park(&svc, &c).await;
        assert!(turn_open(&hub, &c).await);
        assert_eq!(svc.delete_thread(&c).await, Ok(true));
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(permits_free(&svc));
        assert!(!hub.snapshot(&c).await.unwrap().is_some_and(|s| s.state.turn_open()));
    }

    /// Restart settle: Pending and not-parked Running fail at load; the hub hears of it only from
    /// `report_restart_settle`.
    #[tokio::test]
    async fn restart_settles_and_reports_only_when_asked() {
        let (svc, hub, db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        child(&svc, &coord, "p", CoderSubAgentStatus::Pending).await;
        child(&svc, &coord, "r", CoderSubAgentStatus::Running).await;
        child(&svc, &coord, "d", CoderSubAgentStatus::Completed).await;
        let svc2 = Arc::new(CoderService::new(db.clone(), Arc::new(VaultSession::default())).await);
        svc2.init_events(Arc::new(Null));
        svc2.attach_hub(hub.clone());
        assert_eq!(status_of(&svc2, "p").await, CoderSubAgentStatus::Failed);
        assert_eq!(status_of(&svc2, "r").await, CoderSubAgentStatus::Failed);
        assert_eq!(status_of(&svc2, "d").await, CoderSubAgentStatus::Completed);
        assert!(hub.snapshot(&coord).await.unwrap().unwrap().state.sub_agents.is_empty());
        svc2.report_restart_settle().await;
        let reported = hub.snapshot(&coord).await.unwrap().unwrap().state.sub_agents;
        let failed = reported.iter().filter(|s| s.status == CoderSubAgentStatus::Failed).count();
        assert_eq!(failed, 2);
    }

    /// A8: the architect's interleaving. R1 (owned) finishes with a user Send queued; the drained
    /// StartTurn begins R2 before R1's `end_run`. The sub-agent is Completed, R1 is not a stop (it feeds
    /// no `Finished`), the hub's turn stays open for R2, and the permit frees only when R2 ends.
    #[tokio::test]
    async fn a_drained_turn_replacing_the_owned_run_is_not_a_stop() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let r1 = {
            let g = svc.lifecycle.lock().await;
            svc.begin_run(&g, &c, true).await
        };
        send(&svc, &c, "first").await;
        send(&svc, &c, "queued").await;
        svc.finish(&c, None, false, false).await; // R1's loop is done: the hub drains "queued"
        assert!(turn_open(&hub, &c).await);
        let g = svc.lifecycle.lock().await;
        let r2 = svc.begin_run(&g, &c, false).await; // what the drained StartTurn does
        assert!(r1.cancel.load(Ordering::SeqCst), "replaced: its flag is set");
        let stopped = svc.end_run(&g, &c, &r1, RunOutcome::Done).await;
        assert!(!stopped, "a replaced run is not a stop, so spawn_run feeds no Finished");
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Completed);
        assert!(!permits_free(&svc));
        svc.end_run(&g, &c, &r2, RunOutcome::Done).await;
        assert!(permits_free(&svc));
        drop(g);
        assert!(turn_open(&hub, &c).await);
    }

    /// A turn parked on an approval in `c`.
    async fn park(svc: &CoderService, c: &str) {
        open_turn(svc, c, "local").await;
        let pending = PendingApproval {
            call_id: "c1".into(),
            tool: "run_command".into(),
            arguments: json!({}),
            suggested_rule: String::new(),
            summary: "ls".into(),
        };
        svc.feed(c, PortalInput::Pending(pending)).await;
    }

    /// Restart settle (#8): a Running sub-agent parked on an approval stays Running and re-takes a free
    /// permit; past the limit it stays Running without one.
    #[tokio::test]
    async fn restart_keeps_a_parked_sub_agent_and_takes_free_permits() {
        let (svc, hub, db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        for i in 0..=DEFAULT_MAX_PARALLEL_SUBAGENTS {
            let c = child(&svc, &coord, &format!("p{i}"), CoderSubAgentStatus::Running).await;
            park(&svc, &c).await;
            assert!(turn_open(&hub, &c).await);
        }
        let svc2 = Arc::new(CoderService::new(db.clone(), Arc::new(VaultSession::default())).await);
        for i in 0..=DEFAULT_MAX_PARALLEL_SUBAGENTS {
            assert_eq!(status_of(&svc2, &format!("p{i}")).await, CoderSubAgentStatus::Running);
        }
        assert_eq!(svc2.child_permits.lock().unwrap().len(), DEFAULT_MAX_PARALLEL_SUBAGENTS);
        assert_eq!(svc2.multitask_semaphore.available_permits(), 0);
    }

    /// `Effect::Cancel` on a Pending child whose spawn Send opened the turn but whose StartTurn has not
    /// run: the turn closes, the sub-agent is Cancelled, the permit is freed, and the late system
    /// StartTurn runs nothing.
    #[tokio::test]
    async fn cancel_before_the_start_turn_runs_leaves_nothing_to_start() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        hold_permit(&svc, &c).await;
        open_turn(&svc, &c, "subagent:sa1").await;
        svc.run(&c, Effect::Cancel).await;
        assert!(!turn_open(&hub, &c).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(permits_free(&svc));
        svc.run(&c, start(TurnTrigger::Send, "subagent:sa1")).await;
        assert_eq!(run_owned(&svc, &c).await, None);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
    }

    /// An owned run that ends `Ok` without parking, with its flag set after `finish` already reported
    /// the turn and the hub drained a queued user turn, must not feed a second `Finished(Stopped)`:
    /// that would close the user's new turn.
    #[tokio::test]
    async fn a_late_stop_flag_does_not_close_the_next_turn() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        send(&svc, &c, "first").await;
        send(&svc, &c, "queued").await;
        let result = svc.finish(&c, None, false, false).await; // the hub drains "queued"
        assert!(turn_open(&hub, &c).await);
        svc.report_run_end(&c, Ok(result), true).await;
        assert!(turn_open(&hub, &c).await);
    }

    /// The gate follows the sub-agent, not the thread kind: a Pending child switched to Coordinator
    /// (`coder_set_thread_kind` is unchecked) still refuses a client Send, Retry and Edit.
    #[tokio::test]
    async fn the_pending_gate_survives_a_kind_change() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        svc.set_thread_kind(&c, CoderThreadKind::Coordinator).await.unwrap();
        for cmd in [
            ThreadCommand::Send { text: "x".into() },
            ThreadCommand::Retry,
            ThreadCommand::EditMessage { index: 0, text: "x".into() },
        ] {
            let err = svc.hub_command(&c, cmd, "local".into(), None).await.unwrap_err();
            assert!(err.contains("not started"), "{err}");
        }
    }

    /// #7 reports like any other end: a sub-agent failed from Pending emits `subagent-finished`.
    #[tokio::test]
    async fn fail_pending_reports_the_finish() {
        #[derive(Debug, Default)]
        struct Recorder(std::sync::Mutex<Vec<String>>);
        impl EventSink for Recorder {
            fn emit_json(&self, event: &str, _: Value) -> bool {
                self.0.lock().unwrap().push(event.into());
                true
            }
        }
        let (_svc, hub, db, _id, _dir) = setup().await;
        let sink = Arc::new(Recorder::default());
        let svc = Arc::new(CoderService::new(db, Arc::new(VaultSession::default())).await);
        svc.init_events(sink.clone());
        svc.attach_hub(hub);
        let coord = coordinator(&svc).await;
        child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        svc.fail_pending("sa1", "boom".into()).await;
        assert!(sink.0.lock().unwrap().iter().any(|e| e == "coder://subagent-finished"));
    }

    /// `multitask_cancel` on live targets: an owned run is flagged (its permit frees when it ends), a
    /// parked sub-agent's turn is closed and its permit freed at once.
    #[tokio::test]
    async fn multitask_cancel_cancels_live_targets() {
        let (svc, hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let running = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &running).await;
        let t = {
            let g = svc.lifecycle.lock().await;
            svc.begin_run(&g, &running, true).await
        };
        let parked = child(&svc, &coord, "sa2", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &parked).await;
        park(&svc, &parked).await;
        let req = MultitaskCancelRequest { coordinator_thread_id: coord.clone(), sub_agent_id: None };
        svc.multitask_cancel(req).await.unwrap();
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert_eq!(status_of(&svc, "sa2").await, CoderSubAgentStatus::Cancelled);
        assert!(t.cancel.load(Ordering::SeqCst));
        assert!(!turn_open(&hub, &parked).await);
        assert_eq!(svc.child_permits.lock().unwrap().len(), 1, "only the run's permit is left");
        let g = svc.lifecycle.lock().await;
        svc.end_run(&g, &running, &t, RunOutcome::Done).await;
        assert!(permits_free(&svc));
    }

    /// A prepare failure on the sub-agent's own StartTurn (a Retry with no user message) goes through
    /// `end_run`: the sub-agent is Failed, the permit released, no run left behind.
    #[tokio::test]
    async fn start_turn_prepare_failure_fails_the_sub_agent() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Pending).await;
        hold_permit(&svc, &c).await;
        svc.run(&c, start(TurnTrigger::Retry, "subagent:sa1")).await;
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Failed);
        assert_eq!(run_owned(&svc, &c).await, None);
        assert!(permits_free(&svc));
    }

    /// A1 through `stop()` (the legacy `coder_stop` fallback, not `cancel_sub_agent`): the flag alone
    /// ends an owned run's sub-agent as Cancelled, not Completed, and frees the permit.
    #[tokio::test]
    async fn stop_flag_ends_an_owned_run_as_cancelled() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        let c = child(&svc, &coord, "sa1", CoderSubAgentStatus::Running).await;
        hold_permit(&svc, &c).await;
        let g = svc.lifecycle.lock().await;
        let t = svc.begin_run(&g, &c, true).await;
        drop(g);
        assert!(svc.stop(&c).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Running);
        let g = svc.lifecycle.lock().await;
        assert!(svc.end_run(&g, &c, &t, RunOutcome::Done).await);
        assert_eq!(status_of(&svc, "sa1").await, CoderSubAgentStatus::Cancelled);
        assert!(permits_free(&svc));
    }

    /// The hub approval effect in a finished child, in the user's own parked turn: a plain run, the
    /// sub-agent untouched, and `prepared` answered.
    #[tokio::test]
    async fn hub_approval_in_a_finished_child_runs_plain() {
        let (svc, _hub, _db, _id, _dir) = setup().await;
        let coord = coordinator(&svc).await;
        for (sa, status) in [("sa1", CoderSubAgentStatus::Completed), ("sa2", CoderSubAgentStatus::Cancelled)] {
            let c = child(&svc, &coord, sa, status).await;
            park(&svc, &c).await;
            let (tx, mut rx) = oneshot::channel();
            svc.prep.lock().unwrap().insert(c.clone(), tx);
            svc.run(&c, approval()).await;
            assert_eq!(rx.try_recv(), Ok(Ok(())));
            {
                let _freeze = svc.lifecycle.lock().await;
                assert_eq!(run_owned(&svc, &c).await, Some(false));
            }
            settle(&svc, &c).await;
            assert_eq!(status_of(&svc, sa).await, status);
        }
    }
}
