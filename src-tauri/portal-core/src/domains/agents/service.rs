//! `AgentRunService` — registry of [`AgentBackend`]s, a concurrency cap, a
//! pump task per run that turns a backend's raw [`AgentEvent`]s into the
//! fan-out + a late-attach ring buffer, and (P2) durable persistence: one
//! `agent_sessions` row per run, its message turns in `agent_messages`, and
//! its full raw event log in `agent_events` — see
//! `src-tauri/portal-core/src/migrations/m20261111_000048_create_agent_tables.rs`.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Statement,
};
use serde::Serialize;
use serde_json::json;
use tokio::sync::{mpsc, Mutex, Semaphore};
use ts_rs::TS;

use crate::database::DatabaseManager;
use crate::domains::threads::{AckStatus, CommandAck, Effect, Envelope, ThreadCommand, ThreadEffects, ThreadHub};
use crate::events::{EventSink, EventSinkExt};

use super::backend::{AgentBackend, BackendModel, RunControl, StartParams};
use super::claude_code::ClaudeCodeBackend;
use super::entities::{agent_event, agent_message, agent_session};
use super::events::AgentEvent;

/// Late-attach history depth per run — a live view, not the transcript (that
/// lives in the DB now, see `get_session_detail`).
const RING_CAPACITY: usize = 500;
/// How many backend processes may run at once.
const MAX_CONCURRENT_RUNS: usize = 10;
/// How long a finished run's entry (ring buffer included) stays in `runs`
/// after its process exits, so a `get_events` call already in flight still
/// sees the final events before the entry is freed.
const EVENT_REPLAY_GRACE: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct BackendInfo {
    pub id: String,
    pub display_name: String,
    pub available: bool,
    pub models: Vec<BackendModel>,
}

/// Row shape for `agents_list_sessions` — most recent first.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentSessionSummary {
    pub id: String,
    pub backend_id: String,
    pub model: Option<String>,
    pub cwd: String,
    /// `"running" | "exited" | "interrupted"` (the last when the event
    /// channel closed without a normal `Exited`, e.g. an app quit or crash
    /// mid-run — see the pump loop's channel-closed branch in `start_run`).
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub exit_code: Option<i32>,
    /// First line of the launching prompt, truncated — the sidebar's label.
    pub title: Option<String>,
    /// The task that launched this run, if any.
    pub task_id: Option<String>,
    /// The backend CLI's own session id, once known — `resume_session` needs
    /// this to be `Some` to resume-in-place.
    pub backend_session_id: Option<String>,
}

impl From<agent_session::Model> for AgentSessionSummary {
    fn from(m: agent_session::Model) -> Self {
        Self {
            id: m.id,
            backend_id: m.backend_id,
            model: m.model,
            cwd: m.cwd,
            status: m.status,
            started_at: m.started_at,
            ended_at: m.ended_at,
            exit_code: m.exit_code,
            title: m.title,
            task_id: m.task_id,
            backend_session_id: m.backend_session_id,
        }
    }
}

/// A persisted user/assistant turn, for the Conversation tab.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentMessageRow {
    pub id: String,
    // serde_json writes this as a JSON number and JS parses it as `number`;
    // ts-rs would default i64 to `bigint`, a type that never arrives.
    #[ts(type = "number")]
    pub seq: i64,
    pub role: String,
    pub text: String,
    pub parent_tool_use_id: Option<String>,
    pub created_at: String,
}

impl From<agent_message::Model> for AgentMessageRow {
    fn from(m: agent_message::Model) -> Self {
        Self {
            id: m.id,
            seq: m.seq,
            role: m.role,
            text: m.text,
            parent_tool_use_id: m.parent_tool_use_id,
            created_at: m.created_at,
        }
    }
}

/// One row of the raw event log, in order, for the Timeline tab.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentEventRow {
    pub id: String,
    #[ts(type = "number")]
    pub seq: i64,
    pub created_at: String,
    pub event: AgentEvent,
}

/// A live run's event paired with its ring-buffer seq — the `agents://event`
/// payload and `get_events`' snapshot both use this, so a late-attach client
/// can tell which persisted rows (`agent_events.seq`) it already has.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentEventSeq {
    #[ts(type = "number")]
    pub seq: i64,
    pub event: AgentEvent,
}

/// One row of the cross-session activity feed. `agent_events.seq` restarts at
/// 1 per session, so the feed pages on `cursor` (the table's insertion-order
/// rowid) instead.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentFeedEvent {
    #[ts(type = "number")]
    pub cursor: i64,
    pub session_id: String,
    pub created_at: String,
    pub event: AgentEvent,
}

/// One page of the feed. `next_cursor` is the last *raw* row's cursor when the
/// page came back full (rows that fail to parse are dropped from `events` but
/// still count), so paging never stops early on a short parsed page.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentFeedPage {
    pub events: Vec<AgentFeedEvent>,
    #[ts(type = "number | null")]
    pub next_cursor: Option<i64>,
}

#[derive(FromQueryResult)]
struct FeedRow {
    cursor: i64,
    session_id: String,
    created_at: String,
    event_json: String,
}

/// Streaming/diagnostic events that would drown the feed (stored as
/// `{"type":"<tag>",...}` — serde's internal tag is serialized first and
/// `serde_json::to_string` is compact, so `LIKE '{"type":"<tag>"%'` is exact).
const FEED_NOISE: [&str; 4] = ["text_delta", "thinking_delta", "raw", "usage"];

/// One (UTC day, model) cell of token/cost usage. Tokens exclude cache
/// traffic (`cache_tokens` = cache read + creation) so the chart isn't
/// dominated by re-read context.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentUsageCell {
    pub day: String,
    pub model: String,
    #[ts(type = "number")]
    pub tokens: i64,
    #[ts(type = "number")]
    pub cache_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentToolCount {
    pub name: String,
    #[ts(type = "number")]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentOutcomeCount {
    pub status: String,
    #[ts(type = "number")]
    pub count: i64,
}

/// Everything the Analytics section charts, for one backend or all.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentAnalytics {
    pub usage: Vec<AgentUsageCell>,
    pub tools: Vec<AgentToolCount>,
    pub outcomes: Vec<AgentOutcomeCount>,
    /// Mean per-process run time (`session_started` -> `exited`); `None` if none.
    pub avg_duration_secs: Option<f64>,
}

#[derive(FromQueryResult)]
struct UsageRow {
    session_id: String,
    kind: String,
    created_at: String,
    day: String,
    cost: Option<f64>,
    model_usage: Option<String>,
}

/// Full detail for one session — `get_session_detail`'s return shape.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct AgentSessionDetail {
    pub session: AgentSessionSummary,
    pub messages: Vec<AgentMessageRow>,
    pub events: Vec<AgentEventRow>,
}

struct RunEntry {
    control: Box<dyn RunControl>,
    ring: VecDeque<AgentEventSeq>,
    /// Monotonic per-run counter shared by `agent_messages.seq` and
    /// `agent_events.seq` — only needs to be strictly increasing within a
    /// run, not contiguous per-table, so one counter for both is enough.
    next_seq: i64,
    /// The process is gone; the entry only lingers for `EVENT_REPLAY_GRACE`.
    exited: bool,
}

pub struct AgentRunService {
    backends: Vec<Arc<dyn AgentBackend>>,
    runs: Mutex<HashMap<String, RunEntry>>,
    semaphore: Arc<Semaphore>,
    sink: Mutex<Option<Arc<dyn EventSink>>>,
    db: Arc<DatabaseManager>,
    /// Observes every pumped event; set once at startup via `attach_thread_hub`.
    thread_hub: std::sync::OnceLock<Arc<ThreadHub>>,
    /// Sessions a resume is starting right now (see `StartGuard`).
    starting: std::sync::Mutex<HashSet<String>>,
}

/// Marks a session as starting until dropped (after its `runs` entry exists, or on error).
struct StartGuard<'a>(&'a std::sync::Mutex<HashSet<String>>, String);

impl<'a> StartGuard<'a> {
    fn new(set: &'a std::sync::Mutex<HashSet<String>>, id: &str) -> Result<Self, String> {
        if !set.lock().unwrap_or_else(|e| e.into_inner()).insert(id.to_string()) {
            return Err("session is already starting".to_string());
        }
        Ok(Self(set, id.to_string()))
    }
}

impl Drop for StartGuard<'_> {
    fn drop(&mut self) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.1);
    }
}

impl AgentRunService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            backends: vec![Arc::new(ClaudeCodeBackend), Arc::new(super::codex::CodexBackend), Arc::new(super::cursor::CursorBackend)],
            runs: Mutex::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS)),
            sink: Mutex::new(None),
            db,
            thread_hub: std::sync::OnceLock::new(),
            starting: std::sync::Mutex::new(HashSet::new()),
        }
    }

    pub fn attach_thread_hub(&self, hub: Arc<ThreadHub>) {
        let _ = self.thread_hub.set(hub);
    }

    /// A command through the thread hub (dedup, first-wins, queue).
    async fn command(&self, run_id: &str, cmd: ThreadCommand, actor: &str, command_id: Option<String>) -> Result<(), String> {
        let hub = self.thread_hub.get().ok_or("threads: hub not attached")?;
        accepted(hub.command(run_id, Envelope { cmd, actor: actor.into(), command_id }).await?)
    }

    async fn is_live(&self, run_id: &str) -> bool {
        self.runs.lock().await.get(run_id).is_some_and(|e| !e.exited)
    }

    pub async fn init_events(&self, sink: Arc<dyn EventSink>) {
        *self.sink.lock().await = Some(sink);
    }

    async fn emit(&self, run_id: &str, seq: i64, event: &AgentEvent) {
        if let Some(sink) = self.sink.lock().await.as_ref() {
            sink.emit(
                "agents://event",
                json!({ "run_id": run_id, "seq": seq, "event": event }),
            );
        }
    }

    pub async fn list_backends(&self) -> Vec<BackendInfo> {
        let mut out = Vec::with_capacity(self.backends.len());
        for b in &self.backends {
            out.push(BackendInfo {
                id: b.id().to_string(),
                display_name: b.display_name().to_string(),
                available: b.detect().await,
                models: b.list_models().await,
            });
        }
        out
    }

    fn find_backend(&self, id: &str) -> Option<Arc<dyn AgentBackend>> {
        self.backends.iter().find(|b| b.id() == id).cloned()
    }

    /// Start a run. Returns immediately once the backend process is up; the
    /// conversation streams via `agents://event` (`{run_id, seq, event}`).
    pub async fn start_run(
        self: &Arc<Self>,
        backend_id: &str,
        params: StartParams,
        actor: &str,
        command_id: Option<String>,
    ) -> Result<String, String> {
        // A Codex run with no prompt opens a turn nothing ever closes.
        if params.prompt.trim().is_empty() {
            return Err("prompt is empty".into());
        }
        let backend = self
            .find_backend(backend_id)
            .ok_or_else(|| format!("unknown backend '{backend_id}'"))?;
        // Held for the run's lifetime via the pump task below, so a run that
        // never gets cancelled still frees its slot once the process exits.
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;

        let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
        let control = backend.start(params.clone(), tx).await?;
        let run_id = uuid::Uuid::new_v4().to_string();

        // The initial prompt is the run's first user turn — persisted at
        // seq 0 so the Conversation tab has it even though the backend
        // never echoes it back as an event.
        let has_prompt = !params.prompt.trim().is_empty();
        self.runs.lock().await.insert(
            run_id.clone(),
            RunEntry {
                control,
                ring: VecDeque::new(),
                next_seq: if has_prompt { 1 } else { 0 },
                exited: false,
            },
        );

        self.persist_session_start(&run_id, backend_id, &params)
            .await;
        if has_prompt {
            self.persist_message(&run_id, 0, "user", &params.prompt, None)
                .await;
            // Awaited, after the session row exists: the thread is created before the id is
            // returned. The command_id only tags this frame: a retried start is a new run,
            // so it cannot dedup.
            self.record_user(&run_id, &params.prompt, actor, command_id).await;
        }

        self.spawn_pump(run_id.clone(), rx, permit);
        Ok(run_id)
    }

    /// The persisted row of a session that can be resumed, and its backend session id.
    async fn resumable(&self, session_id: &str) -> Result<(agent_session::Model, String), String> {
        let row = agent_session::Entity::find_by_id(session_id.to_string())
            .one(self.db.get_connection())
            .await
            .map_err(|e| format!("agents: load session for resume failed: {e}"))?
            .ok_or("session not found")?;

        let backend_session_id = row
            .backend_session_id
            .clone()
            .ok_or("this session never reached the backend — nothing to resume")?;
        // The live `runs` map is the source of truth, not just the persisted
        // `status` — covers the DB lagging reality during the race right
        // after a run starts or exits (see `spawn_pump`'s own entry removal).
        if row.status == "running" || self.is_live(session_id).await {
            return Err("session is already running".to_string());
        }
        Ok((row, backend_session_id))
    }

    /// Resumes a prior session **in place** — same `agent_sessions` row,
    /// same id reused as the new run's id, same backend conversation (the
    /// backend's own `--resume <backend_session_id>`), never a new row.
    /// Confirmed decision for the IDE-shaped session view (#546): "Resume"
    /// continues the thread rather than forking a new one.
    /// `by` = (actor, command_id) of a client's turn, recorded in the thread log once the
    /// process is up; `None` when the log already has it (the `StartTurn` effect).
    pub async fn resume_session(
        self: &Arc<Self>,
        session_id: &str,
        text: String,
        by: Option<(&str, Option<String>)>,
    ) -> Result<String, String> {
        if text.trim().is_empty() {
            return Err("prompt is empty".into());
        }
        let conn = self.db.get_connection();
        // Two clients resuming at once would start two CLIs on one session.
        let _starting = StartGuard::new(&self.starting, session_id)?;
        let (row, backend_session_id) = self.resumable(session_id).await?;

        let backend = self
            .find_backend(&row.backend_id)
            .ok_or_else(|| format!("unknown backend '{}'", row.backend_id))?;
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;

        let params = StartParams {
            cwd: row.cwd.clone(),
            model: row.model.clone(),
            prompt: text.clone(),
            resume_session_id: Some(backend_session_id),
            task_id: row.task_id.clone(),
            permission_mode: "manual".to_string(),
            // Same as `agents_start_run` post-#546: a real `claude` session,
            // not an isolated one — resuming should see the same governance
            // the original run did.
            isolate_settings: false,
        };
        let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
        let control = backend.start(params, tx).await?;

        let seq = self.next_persisted_seq(session_id).await;
        self.runs.lock().await.insert(
            session_id.to_string(),
            RunEntry {
                control,
                ring: VecDeque::new(),
                next_seq: seq + 1,
                exited: false,
            },
        );

        let mut am: agent_session::ActiveModel = row.into();
        am.status = Set("running".to_string());
        am.ended_at = Set(None);
        if let Err(e) = am.update(conn).await {
            eprintln!("agents: persist resume start failed: {e}");
        }
        self.persist_message(session_id, seq, "user", &text, None)
            .await;
        // Also when the thread still reads as mid-turn: the message just appends.
        if let Some((actor, command_id)) = by {
            self.record_user(session_id, &text, actor, command_id).await;
        }

        self.spawn_pump(session_id.to_string(), rx, permit);
        Ok(session_id.to_string())
    }

    /// One more than the highest seq already persisted for this session,
    /// across both tables (they share one counter — see `RunEntry::next_seq`)
    /// — where a resumed run's own counter picks back up.
    async fn next_persisted_seq(&self, session_id: &str) -> i64 {
        let conn = self.db.get_connection();
        let max_event = agent_event::Entity::find()
            .filter(agent_event::Column::SessionId.eq(session_id))
            .order_by_desc(agent_event::Column::Seq)
            .one(conn)
            .await
            .ok()
            .flatten()
            .map(|r| r.seq);
        let max_message = agent_message::Entity::find()
            .filter(agent_message::Column::SessionId.eq(session_id))
            .order_by_desc(agent_message::Column::Seq)
            .one(conn)
            .await
            .ok()
            .flatten()
            .map(|r| r.seq);
        [max_event, max_message]
            .into_iter()
            .flatten()
            .max()
            .unwrap_or(0)
            + 1
    }

    /// The event-pump task shared by `start_run` and `resume_session`: turns
    /// the backend's raw `AgentEvent`s into the ring buffer + fan-out +
    /// persistence, and frees `permit` once the process exits.
    fn spawn_pump(
        self: &Arc<Self>,
        run_id: String,
        mut rx: mpsc::UnboundedReceiver<AgentEvent>,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        let svc = Arc::clone(self);
        let rid = run_id;
        tokio::spawn(async move {
            loop {
                let Some(event) = rx.recv().await else {
                    // The channel closed without an `Exited` event ever
                    // coming through (process crash, app quit mid-run) — the
                    // session would otherwise sit at status "running" forever
                    // with no `ended_at`, since nothing else reconciles it.
                    svc.persist_session_end(&rid, None, "interrupted").await;
                    // So a thread left mid-turn does not wait on a process that is gone.
                    if let Some(hub) = svc.thread_hub.get() {
                        hub.submit(&rid, ThreadCommand::Provider(AgentEvent::Exited { code: None }));
                    }
                    // The UI keeps a crashed run as running until it hears this.
                    let seq = svc.runs.lock().await.get_mut(&rid).map(|e| {
                        e.next_seq += 1;
                        e.next_seq - 1
                    });
                    if let Some(seq) = seq {
                        svc.emit(&rid, seq, &AgentEvent::Exited { code: None }).await;
                    }
                    break;
                };
                if let AgentEvent::SessionStarted { session_id } = &event {
                    svc.persist_backend_session_id(&rid, session_id).await;
                }
                // `code` here means "this event IS Exited"; its inner
                // `Option<i32>` is the process exit code itself.
                let exit_code: Option<Option<i32>> = match &event {
                    AgentEvent::Exited { code } => Some(*code),
                    _ => None,
                };
                let exited = exit_code.is_some();
                let mut runs = svc.runs.lock().await;
                let seq = runs
                    .get_mut(&rid)
                    .map(|entry| {
                        let seq = entry.next_seq;
                        entry.next_seq += 1;
                        if entry.ring.len() >= RING_CAPACITY {
                            entry.ring.pop_front();
                        }
                        entry.ring.push_back(AgentEventSeq {
                            seq,
                            event: event.clone(),
                        });
                        seq
                    })
                    .unwrap_or(0);
                drop(runs);
                svc.emit(&rid, seq, &event).await;
                if let Some(hub) = svc.thread_hub.get() {
                    hub.submit(&rid, ThreadCommand::Provider(event.clone()));
                }
                if exited {
                    // Before the persists below: a send in that window gets an error
                    // ("already running", until the session end commits; the UI keeps the
                    // draft) instead of writing to a dead stdin.
                    if let Some(e) = svc.runs.lock().await.get_mut(&rid) {
                        e.exited = true;
                    }
                }

                // `TextDelta`/`ThinkingDelta` are superseded by the `Message`
                // event that follows once the turn finishes — persisting
                // every chunk would be one awaited INSERT per delta on a
                // busy run, for no row the Conversation/Timeline tabs ever
                // need (see `eventTimeline.ts`'s `foldTimelineEvents`).
                let is_delta = matches!(
                    event,
                    AgentEvent::TextDelta { .. } | AgentEvent::ThinkingDelta { .. }
                );
                if !is_delta {
                    svc.persist_event(&rid, seq, &event).await;
                }
                if let AgentEvent::Message {
                    role,
                    text,
                    parent_tool_use_id,
                } = &event
                {
                    svc.persist_message(&rid, seq, role, text, parent_tool_use_id.clone())
                        .await;
                }

                // `rx` otherwise never closes on its own: `runs` still holds
                // this run's `RunControl`, and so a live `events_tx` clone,
                // for as long as the entry stays in the map. Breaking here
                // instead of waiting for channel-close is what releases the
                // concurrency permit below the moment the run actually ends.
                if exited {
                    svc.persist_session_end(&rid, exit_code.flatten(), "exited")
                        .await;
                    break;
                }
            }
            drop(permit);
            if let Some(e) = svc.runs.lock().await.get_mut(&rid) {
                e.exited = true;
            }

            // Grace period for a client mid-poll of `get_events` to still
            // see the final events, then free the entry so `runs` doesn't
            // grow for the life of the process.
            tokio::time::sleep(EVENT_REPLAY_GRACE).await;
            // Not a newer run that resumed this session in the meantime.
            let mut runs = svc.runs.lock().await;
            if runs.get(&rid).is_some_and(|e| e.exited) {
                runs.remove(&rid);
            }
        });
    }

    /// A user turn from a client. A live run takes a hub `Send`: queued behind an open turn,
    /// else the `StartTurn` effect writes it to the process (the log already has it). A
    /// finished run is resumed, so start errors reach the caller; that turn is recorded in
    /// the thread log as the backend's user message. Returns the run id.
    pub async fn send_user(self: &Arc<Self>, run_id: &str, text: String, actor: &str, command_id: Option<String>) -> Result<String, String> {
        if !self.is_live(run_id).await {
            return self.resume_session(run_id, text, Some((actor, command_id))).await;
        }
        self.command(run_id, ThreadCommand::Send { text }, actor, command_id).await?;
        Ok(run_id.to_string())
    }

    /// Drop a queued (not yet started) message.
    pub async fn remove_queued(&self, run_id: &str, queue_id: String, actor: &str, command_id: Option<String>) -> Result<(), String> {
        self.command(run_id, ThreadCommand::RemoveQueued { queue_id }, actor, command_id).await
    }

    /// The backend never echoes the user's turn, so the log is told here.
    async fn record_user(&self, run_id: &str, text: &str, actor: &str, command_id: Option<String>) {
        let msg = AgentEvent::Message { role: "user".into(), text: text.to_string(), parent_tool_use_id: None };
        if let Err(e) = self.command(run_id, ThreadCommand::Provider(msg), actor, command_id).await {
            eprintln!("agents: thread log missed a user turn: {e}");
        }
    }

    /// The hub records `ApprovalResolved{by: actor}` first and rejects a second answer.
    pub async fn decide_permission(&self, run_id: &str, request_id: String, allow: bool, actor: &str, command_id: Option<String>) -> Result<(), String> {
        let approve = ThreadCommand::Approve { call_id: request_id, allow, remember: false, edited_pattern: None, edited_command: None };
        self.command(run_id, approve, actor, command_id).await
    }

    /// Stop the turn, not the process. Nothing to stop is not an error.
    pub async fn interrupt(&self, run_id: &str, actor: &str, command_id: Option<String>) -> Result<(), String> {
        match self.command(run_id, ThreadCommand::Stop, actor, command_id).await {
            Err(e) if e.contains("not_running") => Ok(()),
            r => r,
        }
    }

    // ponytail: one `Mutex<HashMap<..>>` for every run's control handle —
    // each call below just forwards a quick stdin write/HTTP-decision, so
    // holding it for the await is cheap. Split per-run if a slow backend ever
    // makes that untrue.
    /// The turn is already in the log (the `StartTurn` effect); this only delivers it.
    async fn send_live(&self, run_id: &str, text: String) -> Result<(), String> {
        let seq = {
            let mut runs = self.runs.lock().await;
            let entry = runs.get_mut(run_id).ok_or("run not found")?;
            let seq = entry.next_seq;
            entry.next_seq += 1;
            entry.control.send_user(text.clone()).await?;
            seq
        };
        self.persist_message(run_id, seq, "user", &text, None).await;
        Ok(())
    }

    pub async fn cancel(&self, run_id: &str) -> Result<(), String> {
        let runs = self.runs.lock().await;
        let entry = runs.get(run_id).ok_or("run not found")?;
        entry.control.kill().await
    }

    /// Snapshot of this run's event ring, for a UI that attaches late.
    pub async fn get_events(&self, run_id: &str) -> Vec<AgentEventSeq> {
        self.runs
            .lock()
            .await
            .get(run_id)
            .map(|e| e.ring.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Run ids with a live backend process right now, regardless of which
    /// page/tab started them. The agent tab (`/ai/coder/<backend>`) has no persisted run id of its
    /// own (unlike coder's thread list) — a reload calls this on mount to
    /// find a run it should reattach to instead of showing a blank launcher
    /// while the backend sits alive (QA-127). Cheap and non-sensitive (just
    /// ids), so `agents` rather than `agents.history`.
    pub async fn list_running(&self) -> Vec<String> {
        self.runs.lock().await.iter().filter(|(_, e)| !e.exited).map(|(k, _)| k.clone()).collect()
    }

    /// Flips every `agent_sessions` row still `status = "running"` to
    /// `"interrupted"` — called once at startup. `runs` always starts empty
    /// on a fresh process (see `new`), so any row still `"running"` from a
    /// previous process is orphaned: there is no live backend left to ever
    /// resolve it, and nothing else will. Returns how many rows it touched.
    pub async fn reconcile_stale_sessions(&self) -> Result<usize, String> {
        let conn = self.db.get_connection();
        let stale = agent_session::Entity::find()
            .filter(agent_session::Column::Status.eq("running"))
            .all(conn)
            .await
            .map_err(|e| format!("agents: load stale sessions failed: {e}"))?;
        let now = now_iso();
        for row in &stale {
            let mut am: agent_session::ActiveModel = row.clone().into();
            am.status = Set("interrupted".to_string());
            am.ended_at = Set(Some(now.clone()));
            am.update(conn)
                .await
                .map_err(|e| format!("agents: reconcile stale session failed: {e}"))?;
            // The thread log's own startup reconcile (`threads::legacy_import`, S3c) ends any
            // turn left open; submitting here would race its direct commits.
        }
        Ok(stale.len())
    }

    /// Every persisted session, most recent first.
    pub async fn list_sessions(&self) -> Result<Vec<AgentSessionSummary>, String> {
        let rows = agent_session::Entity::find()
            .order_by_desc(agent_session::Column::StartedAt)
            .all(self.db.get_connection())
            .await
            .map_err(|e| format!("agents: list sessions failed: {e}"))?;
        Ok(rows.into_iter().map(AgentSessionSummary::from).collect())
    }

    /// Cross-session activity feed, newest first, without streaming noise.
    /// `after_seq` = only newer than that cursor, `before_seq` = only older
    /// (the next page back); `limit` is clamped to 1..=200; `backend_id`
    /// restricts to that backend's sessions in SQL, before the LIMIT.
    // Cursor = rowid: VACUUM can renumber it, and deleting the newest session's
    // events lets SQLite reuse numbers. Fine for a feed; not a durable key.
    pub async fn list_events(
        &self,
        after_seq: Option<i64>,
        before_seq: Option<i64>,
        limit: Option<u64>,
        backend_id: Option<String>,
    ) -> Result<AgentFeedPage, String> {
        let limit = limit.unwrap_or(50).clamp(1, 200);
        let mut q = agent_event::Entity::find()
            .select_only()
            .column_as(Expr::cust("rowid"), "cursor")
            .columns([
                agent_event::Column::SessionId,
                agent_event::Column::CreatedAt,
                agent_event::Column::EventJson,
            ])
            .order_by_desc(Expr::cust("rowid"))
            .limit(limit);
        if let Some(b) = backend_id {
            q = q.filter(Expr::cust_with_values(
                "session_id IN (SELECT id FROM agent_sessions WHERE backend_id = ?)",
                [b],
            ));
        }
        for tag in FEED_NOISE {
            q = q.filter(Expr::cust(format!(
                "event_json NOT LIKE '{{\"type\":\"{tag}\"%'"
            )));
        }
        if let Some(n) = after_seq {
            q = q.filter(Expr::cust_with_values("rowid > ?", [n]));
        }
        if let Some(n) = before_seq {
            q = q.filter(Expr::cust_with_values("rowid < ?", [n]));
        }
        let rows = q
            .into_model::<FeedRow>()
            .all(self.db.get_connection())
            .await
            .map_err(|e| format!("agents: list events failed: {e}"))?;
        let next_cursor = rows
            .last()
            .filter(|_| rows.len() as u64 == limit)
            .map(|r| r.cursor);
        let events = rows
            .into_iter()
            .filter_map(
                |r| match serde_json::from_str::<AgentEvent>(&r.event_json) {
                    Ok(event) => Some(AgentFeedEvent {
                        cursor: r.cursor,
                        session_id: r.session_id,
                        created_at: r.created_at,
                        event,
                    }),
                    Err(e) => {
                        eprintln!("agents: skipping unparsable event row: {e}");
                        None
                    }
                },
            )
            .collect();
        Ok(AgentFeedPage {
            events,
            next_cursor,
        })
    }

    /// Usage per day/model, tool-call counts and session outcomes, read from
    /// `agent_events.event_json` via `json_extract`; `backend_id` filters in SQL.
    /// `Usage` events carry the run's *cumulative* totals, so each becomes a
    /// per-session delta; a resumed process counts from zero again, so a
    /// `session_started`/`exited` boundary resets the baseline (a counter
    /// drop is the fallback). Run duration is per process too
    /// (`session_started` -> `exited`), so a resume's idle gap isn't counted.
    pub async fn analytics(&self, backend_id: Option<String>) -> Result<AgentAnalytics, String> {
        let conn = self.db.get_connection();
        let err = |e: sea_orm::DbErr| format!("agents: analytics failed: {e}");
        // Every query takes the optional backend twice: `(? IS NULL OR … = ?)`.
        let stmt = |sql: String| {
            let b = || sea_orm::Value::String(backend_id.clone().map(Box::new));
            Statement::from_sql_and_values(DbBackend::Sqlite, sql, [b(), b()])
        };
        const EV: &str =
            "(? IS NULL OR session_id IN (SELECT id FROM agent_sessions WHERE backend_id = ?))";
        const SS: &str = "(? IS NULL OR backend_id = ?)";

        let rows = UsageRow::find_by_statement(stmt(format!(
            "SELECT session_id, json_extract(event_json, '$.type') AS kind, created_at, \
             substr(created_at, 1, 10) AS day, \
             json_extract(event_json, '$.total_cost_usd') AS cost, \
             json_extract(event_json, '$.model_usage') AS model_usage \
             FROM agent_events WHERE (event_json LIKE '{{\"type\":\"usage\"%' \
             OR event_json LIKE '{{\"type\":\"session_started\"%' \
             OR event_json LIKE '{{\"type\":\"exited\"%') AND {EV} \
             ORDER BY rowid"
        )))
        .all(conn)
        .await
        .map_err(err)?;
        // (day, model) -> (tokens, cache, cost); `prev` is per (session, model).
        let mut cells: std::collections::BTreeMap<(String, String), (i64, i64, f64)> =
            Default::default();
        let mut prev: HashMap<(String, String), (i64, i64, f64)> = HashMap::new();
        // Sessions where any usage event carries per-model `costUSD`; only the rest
        // may fall back to the session total (deciding per event flips mid-session).
        let has_model_cost: std::collections::HashSet<&str> = rows
            .iter()
            .filter(|r| {
                r.kind == "usage"
                    && r.model_usage
                        .as_deref()
                        .is_some_and(|j| j.contains("\"costUSD\""))
            })
            .map(|r| r.session_id.as_str())
            .collect();
        let mut started: HashMap<&str, chrono::DateTime<chrono::FixedOffset>> = HashMap::new();
        let mut run_secs: Vec<f64> = Vec::new();
        for r in &rows {
            if r.kind != "usage" {
                prev.retain(|(s, _), _| s != &r.session_id);
                let at = chrono::DateTime::parse_from_rfc3339(&r.created_at).ok();
                if r.kind == "exited" {
                    // A second `exited` (EOF + stop) finds no start and is ignored.
                    if let (Some(a), Some(t)) = (started.remove(r.session_id.as_str()), at) {
                        run_secs.push((t - a).num_milliseconds() as f64 / 1000.0);
                    }
                } else if let Some(t) = at {
                    started.entry(r.session_id.as_str()).or_insert(t);
                }
                continue;
            }
            let usage = r
                .model_usage
                .as_deref()
                .and_then(|j| serde_json::from_str::<serde_json::Value>(j).ok());
            let Some(models) = usage.as_ref().and_then(|u| u.as_object()) else {
                continue;
            };
            let n = |m: &serde_json::Value, k: &str| m.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
            for (model, m) in models {
                let cost = m.get("costUSD").and_then(|v| v.as_f64()).or(
                    if models.len() == 1 && !has_model_cost.contains(r.session_id.as_str()) {
                        r.cost
                    } else {
                        None
                    },
                );
                let cur = (
                    n(m, "inputTokens") + n(m, "outputTokens"),
                    n(m, "cacheReadInputTokens") + n(m, "cacheCreationInputTokens"),
                    cost.unwrap_or(0.0),
                );
                let (pt, pc, pu) = prev
                    .insert((r.session_id.clone(), model.clone()), cur)
                    .unwrap_or_default();
                let c = cells.entry((r.day.clone(), model.clone())).or_default();
                c.0 += if cur.0 >= pt { cur.0 - pt } else { cur.0 };
                c.1 += if cur.1 >= pc { cur.1 - pc } else { cur.1 };
                c.2 += if cur.2 >= pu { cur.2 - pu } else { cur.2 };
            }
        }
        let usage = cells
            .into_iter()
            .map(
                |((day, model), (tokens, cache_tokens, cost_usd))| AgentUsageCell {
                    day,
                    model,
                    tokens,
                    cache_tokens,
                    cost_usd,
                },
            )
            .collect();

        #[derive(FromQueryResult)]
        struct Tool {
            name: String,
            count: i64,
        }
        let tools = Tool::find_by_statement(stmt(format!(
            "SELECT json_extract(event_json, '$.name') AS name, COUNT(*) AS count \
             FROM agent_events WHERE event_json LIKE '{{\"type\":\"tool_call\"%' \
             AND json_extract(event_json, '$.name') IS NOT NULL AND {EV} \
             GROUP BY name ORDER BY count DESC, name LIMIT 15"
        )))
        .all(conn)
        .await
        .map_err(err)?
        .into_iter()
        .map(|t| AgentToolCount {
            name: t.name,
            count: t.count,
        })
        .collect();

        #[derive(FromQueryResult)]
        struct Outcome {
            status: String,
            count: i64,
        }
        // A non-zero exit on a finished session is a failure, whatever its status says.
        let outcomes = Outcome::find_by_statement(stmt(format!(
            "SELECT CASE WHEN status = 'exited' AND COALESCE(exit_code, 0) != 0 THEN 'failed' \
             ELSE status END AS status, COUNT(*) AS count \
             FROM agent_sessions WHERE {SS} GROUP BY 1 ORDER BY count DESC"
        )))
        .all(conn)
        .await
        .map_err(err)?
        .into_iter()
        .map(|o| AgentOutcomeCount {
            status: o.status,
            count: o.count,
        })
        .collect();

        let avg_duration_secs =
            (!run_secs.is_empty()).then(|| run_secs.iter().sum::<f64>() / run_secs.len() as f64);

        Ok(AgentAnalytics {
            usage,
            tools,
            outcomes,
            avg_duration_secs,
        })
    }

    /// Deletes a persisted session and its transcript/event rows (FK cascade
    /// on `agent_sessions.id`, same shape as `coder_file_changes`). Refuses a
    /// session that's live or persisted as `"running"` — same "the only
    /// thing that can end a run is the run itself" rule `resume_session`'s
    /// guard enforces.
    pub async fn delete_session(&self, session_id: &str) -> Result<(), String> {
        if self.runs.lock().await.contains_key(session_id) {
            return Err("cannot delete a session that is currently running".to_string());
        }
        let conn = self.db.get_connection();
        let row = agent_session::Entity::find_by_id(session_id.to_string())
            .one(conn)
            .await
            .map_err(|e| format!("agents: load session for delete failed: {e}"))?
            .ok_or("session not found")?;
        if row.status == "running" {
            return Err("cannot delete a session that is currently running".to_string());
        }
        // Thread first: if this fails the session row survives, so a retry can reach it.
        if let Some(hub) = self.thread_hub.get() {
            hub.delete(session_id).await?;
        }
        agent_session::Entity::delete_by_id(session_id.to_string())
            .exec(conn)
            .await
            .map_err(|e| format!("agents: delete session failed: {e}"))?;
        Ok(())
    }

    /// One session's full persisted transcript + event log, in order.
    /// `Ok(None)` means "no such session"; `Err` is a real DB failure — kept
    /// distinct so the frontend's `PageError` only fires on the latter.
    pub async fn get_session_detail(
        &self,
        session_id: &str,
    ) -> Result<Option<AgentSessionDetail>, String> {
        let conn = self.db.get_connection();
        let session = match agent_session::Entity::find_by_id(session_id.to_string())
            .one(conn)
            .await
            .map_err(|e| format!("agents: get session failed: {e}"))?
        {
            Some(s) => s,
            None => return Ok(None),
        };

        let messages = agent_message::Entity::find()
            .filter(agent_message::Column::SessionId.eq(session_id))
            .order_by_asc(agent_message::Column::Seq)
            .all(conn)
            .await
            .map_err(|e| format!("agents: load messages failed: {e}"))?;

        let events = agent_event::Entity::find()
            .filter(agent_event::Column::SessionId.eq(session_id))
            .order_by_asc(agent_event::Column::Seq)
            .all(conn)
            .await
            .map_err(|e| format!("agents: load events failed: {e}"))?;

        Ok(Some(AgentSessionDetail {
            session: session.into(),
            messages: messages.into_iter().map(AgentMessageRow::from).collect(),
            events: events
                .into_iter()
                .filter_map(
                    |row| match serde_json::from_str::<AgentEvent>(&row.event_json) {
                        Ok(event) => Some(AgentEventRow {
                            id: row.id,
                            seq: row.seq,
                            created_at: row.created_at,
                            event,
                        }),
                        Err(e) => {
                            eprintln!("agents: skipping unparsable event row: {e}");
                            None
                        }
                    },
                )
                .collect(),
        }))
    }

    async fn persist_session_start(&self, run_id: &str, backend_id: &str, params: &StartParams) {
        // Sidebar label — first non-empty line of the launching prompt, short
        // enough to sit in a nav row. `None` when the prompt itself is empty
        // (the launcher's resume-with-no-text case).
        let title = params
            .prompt
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(|l| truncate_title(l, 80));
        let am = agent_session::ActiveModel {
            id: Set(run_id.to_string()),
            backend_id: Set(backend_id.to_string()),
            model: Set(params.model.clone()),
            cwd: Set(params.cwd.clone()),
            status: Set("running".to_string()),
            started_at: Set(now_iso()),
            ended_at: Set(None),
            exit_code: Set(None),
            title: Set(title),
            task_id: Set(params.task_id.clone()),
            backend_session_id: Set(None),
        };
        if let Err(e) = agent_session::Entity::insert(am)
            .exec(self.db.get_connection())
            .await
        {
            eprintln!("agents: persist session start failed: {e}");
        }
    }

    /// Backfills the backend's own session id once the CLI reports it via
    /// `SessionStarted` — `resume_session` requires this to be `Some`.
    async fn persist_backend_session_id(&self, run_id: &str, backend_session_id: &str) {
        let conn = self.db.get_connection();
        let existing = match agent_session::Entity::find_by_id(run_id.to_string())
            .one(conn)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return,
            Err(e) => {
                eprintln!("agents: load session for backend id failed: {e}");
                return;
            }
        };
        let mut am: agent_session::ActiveModel = existing.into();
        am.backend_session_id = Set(Some(backend_session_id.to_string()));
        if let Err(e) = am.update(conn).await {
            eprintln!("agents: persist backend session id failed: {e}");
        }
    }

    /// `status` is `"exited"` for a normal process exit or `"interrupted"`
    /// when the event channel closed without one (crash, app quit mid-run) —
    /// either way the row needs a terminal status + `ended_at`, or it sits at
    /// `"running"` forever with nothing left to reconcile it.
    async fn persist_session_end(&self, run_id: &str, code: Option<i32>, status: &str) {
        let conn = self.db.get_connection();
        let existing = match agent_session::Entity::find_by_id(run_id.to_string())
            .one(conn)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return,
            Err(e) => {
                eprintln!("agents: load session for end failed: {e}");
                return;
            }
        };
        let mut am: agent_session::ActiveModel = existing.into();
        am.status = Set(status.to_string());
        am.ended_at = Set(Some(now_iso()));
        am.exit_code = Set(code);
        if let Err(e) = am.update(conn).await {
            eprintln!("agents: persist session end failed: {e}");
        }
    }

    async fn persist_message(
        &self,
        run_id: &str,
        seq: i64,
        role: &str,
        text: &str,
        parent_tool_use_id: Option<String>,
    ) {
        let am = agent_message::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            session_id: Set(run_id.to_string()),
            seq: Set(seq),
            role: Set(role.to_string()),
            text: Set(text.to_string()),
            parent_tool_use_id: Set(parent_tool_use_id),
            created_at: Set(now_iso()),
        };
        if let Err(e) = agent_message::Entity::insert(am)
            .exec(self.db.get_connection())
            .await
        {
            eprintln!("agents: persist message failed: {e}");
        }
    }

    async fn persist_event(&self, run_id: &str, seq: i64, event: &AgentEvent) {
        let event_json = serde_json::to_string(event).unwrap_or_else(|_| "{}".into());
        let am = agent_event::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            session_id: Set(run_id.to_string()),
            seq: Set(seq),
            event_json: Set(event_json),
            created_at: Set(now_iso()),
        };
        if let Err(e) = agent_event::Entity::insert(am)
            .exec(self.db.get_connection())
            .await
        {
            eprintln!("agents: persist event failed: {e}");
        }
    }
}

/// A rejected command is an error for the caller; accepted and duplicate both mean "done".
fn accepted(ack: CommandAck) -> Result<(), String> {
    if ack.status != AckStatus::Rejected {
        return Ok(());
    }
    let code = ack.code.unwrap_or_default();
    Err(match ack.resolved_by {
        Some(by) => format!("{code}: already answered by {by}"),
        None => code,
    })
}

/// What a committed thread command asks of a CLI run. Runs outside the actor and
/// never inside a transaction; a failure re-enters the log as an event.
#[async_trait::async_trait]
impl ThreadEffects for Arc<AgentRunService> {
    async fn run(&self, thread_id: &str, effect: Effect) {
        let starts_turn = matches!(effect, Effect::StartTurn { .. });
        let result = match effect {
            // The turn is already in the log; deliver it, resuming a finished run.
            Effect::StartTurn { text, .. } if self.is_live(thread_id).await => self.send_live(thread_id, text).await,
            // A Stop landing while this is still starting the process finds no run and is
            // lost; a second Stop works once the entry is inserted.
            Effect::StartTurn { text, .. } => self.resume_session(thread_id, text, None).await.map(|_| ()),
            Effect::ResolveApproval { call_id, allow, .. } => match self.runs.lock().await.get(thread_id) {
                Some(e) => e.control.decide_permission(call_id, allow).await,
                None => Err("run not found".into()),
            },
            // Fire-and-forget and repeatable (a lost one is retried by a second Stop): a
            // run that is gone has nothing left to stop.
            Effect::Cancel => match self.runs.lock().await.get(thread_id).filter(|e| !e.exited) {
                Some(e) => e.control.interrupt().await,
                None => Ok(()),
            },
            // CLI backends make no host requests.
            Effect::DeliverHostResult { .. } => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("agents: effect for {thread_id} failed: {e}");
            // Else the turn the log opened would wait on a process that never started.
            if let (Some(hub), true) = (self.thread_hub.get(), starts_turn) {
                hub.submit(thread_id, ThreadCommand::Provider(AgentEvent::TurnDone { final_text: Some(e), is_error: true }));
            }
        }
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// First `max` chars of `s`, `…`-suffixed if it was longer — char-counted,
/// not byte-counted, so it never splits a multi-byte char.
fn truncate_title(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max).collect();
        t.push('…');
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No-op `RunControl` — these tests exercise `resume_session`'s guards,
    /// never a real backend process.
    struct FakeControl;
    #[async_trait::async_trait]
    impl RunControl for FakeControl {
        async fn send_user(&self, _text: String) -> Result<(), String> {
            Ok(())
        }
        async fn decide_permission(&self, _request_id: String, _allow: bool) -> Result<(), String> {
            Ok(())
        }
        async fn interrupt(&self) -> Result<(), String> {
            Ok(())
        }
        async fn kill(&self) -> Result<(), String> {
            Ok(())
        }
    }

    async fn test_service() -> (Arc<AgentRunService>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = DatabaseManager::new(dir.path().to_path_buf())
            .await
            .expect("db init");
        (Arc::new(AgentRunService::new(Arc::new(db))), dir)
    }

    /// The risky bit of resume-in-place: the live `runs` map must win over a
    /// persisted `status` that hasn't caught up yet (the race right after a
    /// run starts/exits — see `spawn_pump`'s own delayed entry removal), not
    /// just a row that's still literally `"running"`.
    #[tokio::test]
    async fn resume_refuses_a_session_already_live_even_if_its_row_says_exited() {
        let (svc, _dir) = test_service().await;
        let conn = svc.db.get_connection();
        let am = agent_session::ActiveModel {
            id: Set("sess-1".to_string()),
            backend_id: Set("claude-code".to_string()),
            model: Set(None),
            cwd: Set(".".to_string()),
            status: Set("exited".to_string()),
            started_at: Set(now_iso()),
            ended_at: Set(Some(now_iso())),
            exit_code: Set(Some(0)),
            title: Set(None),
            task_id: Set(None),
            backend_session_id: Set(Some("backend-sess-1".to_string())),
        };
        agent_session::Entity::insert(am)
            .exec(conn)
            .await
            .expect("insert");

        svc.runs.lock().await.insert(
            "sess-1".to_string(),
            RunEntry {
                control: Box::new(FakeControl),
                ring: VecDeque::new(),
                next_seq: 1,
                exited: false,
            },
        );

        let err = svc
            .resume_session("sess-1", "go".to_string(), None)
            .await
            .unwrap_err();
        assert!(err.contains("already running"), "got: {err}");
    }

    /// The straightforward case the row alone already covers.
    #[tokio::test]
    async fn resume_refuses_a_session_whose_row_says_running() {
        let (svc, _dir) = test_service().await;
        let conn = svc.db.get_connection();
        let am = agent_session::ActiveModel {
            id: Set("sess-2".to_string()),
            backend_id: Set("claude-code".to_string()),
            model: Set(None),
            cwd: Set(".".to_string()),
            status: Set("running".to_string()),
            started_at: Set(now_iso()),
            ended_at: Set(None),
            exit_code: Set(None),
            title: Set(None),
            task_id: Set(None),
            backend_session_id: Set(Some("backend-sess-2".to_string())),
        };
        agent_session::Entity::insert(am)
            .exec(conn)
            .await
            .expect("insert");

        let err = svc
            .resume_session("sess-2", "go".to_string(), None)
            .await
            .unwrap_err();
        assert!(err.contains("already running"), "got: {err}");
    }

    #[tokio::test]
    async fn resume_refuses_a_session_with_no_backend_session_id() {
        let (svc, _dir) = test_service().await;
        let conn = svc.db.get_connection();
        let am = agent_session::ActiveModel {
            id: Set("sess-3".to_string()),
            backend_id: Set("claude-code".to_string()),
            model: Set(None),
            cwd: Set(".".to_string()),
            status: Set("exited".to_string()),
            started_at: Set(now_iso()),
            ended_at: Set(Some(now_iso())),
            exit_code: Set(Some(0)),
            title: Set(None),
            task_id: Set(None),
            backend_session_id: Set(None),
        };
        agent_session::Entity::insert(am)
            .exec(conn)
            .await
            .expect("insert");

        let err = svc
            .resume_session("sess-3", "go".to_string(), None)
            .await
            .unwrap_err();
        assert!(err.contains("never reached the backend"), "got: {err}");
    }

    // ---- S3b: client commands go through the thread hub ----

    #[derive(Default)]
    struct Calls {
        sent: Vec<String>,
        decided: Vec<(String, bool)>,
        interrupts: usize,
        fail_send: bool,
    }
    struct RecControl(Arc<std::sync::Mutex<Calls>>);
    #[async_trait::async_trait]
    impl RunControl for RecControl {
        async fn send_user(&self, text: String) -> Result<(), String> {
            let mut c = self.0.lock().unwrap();
            if c.fail_send {
                return Err("a Cursor turn is still running".into());
            }
            c.sent.push(text);
            Ok(())
        }
        async fn decide_permission(&self, id: String, allow: bool) -> Result<(), String> {
            self.0.lock().unwrap().decided.push((id, allow));
            Ok(())
        }
        async fn interrupt(&self) -> Result<(), String> {
            self.0.lock().unwrap().interrupts += 1;
            Ok(())
        }
        async fn kill(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct NullSink;
    impl EventSink for NullSink {
        fn emit_json(&self, _event: &str, _payload: serde_json::Value) -> bool {
            true
        }
    }

    /// A service with a hub, CLI effects registered, and one live run `run-1`.
    async fn hub_service() -> (Arc<AgentRunService>, Arc<ThreadHub>, Arc<std::sync::Mutex<Calls>>, tempfile::TempDir) {
        let (svc, dir) = test_service().await;
        seed_session(&svc, "run-1").await;
        let hub = ThreadHub::new(svc.db.clone(), Arc::new(NullSink));
        svc.attach_thread_hub(hub.clone());
        hub.register_cli_effects(Arc::new(svc.clone()));
        let calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        svc.runs.lock().await.insert(
            "run-1".into(),
            RunEntry { control: Box::new(RecControl(calls.clone())), ring: VecDeque::new(), next_seq: 1, exited: false },
        );
        (svc, hub, calls, dir)
    }

    fn feed(hub: &Arc<ThreadHub>, events: Vec<AgentEvent>) {
        for e in events {
            hub.submit("run-1", ThreadCommand::Provider(e));
        }
    }

    /// Effects run in their own tasks; wait for one's side effect.
    async fn until(mut done: impl FnMut() -> bool) {
        for _ in 0..200 {
            if done() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("timed out");
    }

    fn user(text: &str) -> AgentEvent {
        AgentEvent::Message { role: "user".into(), text: text.into(), parent_tool_use_id: None }
    }

    /// A recorded run replayed through the hub, with client commands in the middle: the
    /// approval is recorded once with its answerer, the backend's echo and a second answer
    /// are dropped, a mid-turn message is queued and reaches the backend after the turn.
    #[tokio::test]
    async fn replayed_run_with_client_commands_snapshots_as_expected() {
        use crate::domains::threads::ThreadEvent;
        let (svc, hub, calls, _dir) = hub_service().await;
        feed(
            &hub,
            vec![
                user("go"),
                AgentEvent::SessionStarted { session_id: "sess".into() },
                AgentEvent::PermissionRequest { request_id: "r1".into(), tool_name: "Bash".into(), tool_input: json!({"command": "ls"}), suggestions: vec![] },
            ],
        );
        svc.send_user("run-1", "next".into(), "local", None).await.unwrap();
        assert!(calls.lock().unwrap().sent.is_empty(), "mid-turn sends queue behind the turn");

        svc.decide_permission("run-1", "r1".into(), true, "device:phone", Some("c1".into())).await.unwrap();
        until(|| calls.lock().unwrap().decided == [("r1".to_string(), true)]).await;
        let again = svc.decide_permission("run-1", "r1".into(), false, "local", None).await.unwrap_err();
        assert!(again.contains("already_resolved") && again.contains("device:phone"), "got: {again}");

        feed(
            &hub,
            vec![
                AgentEvent::PermissionResolved { request_id: "r1".into(), allowed: true }, // the backend's echo
                AgentEvent::Message { role: "assistant".into(), text: "ok".into(), parent_tool_use_id: None },
                AgentEvent::TurnDone { final_text: Some("ok".into()), is_error: false },
            ],
        );

        until(|| calls.lock().unwrap().sent == ["next"]).await;
        let snap = hub.snapshot("run-1").await.unwrap().unwrap();
        let page = hub.events_since("run-1", 0).await.unwrap();
        let resolved: Vec<_> = page.events.iter().filter(|f| matches!(f.event, ThreadEvent::ApprovalResolved { .. })).collect();
        assert_eq!(resolved.len(), 1, "the echo is dropped as already resolved");
        assert_eq!((resolved[0].actor.as_str(), resolved[0].command_id.as_deref()), ("device:phone", Some("c1")));
        assert!(snap.state.queue.is_empty() && snap.state.approvals.is_empty());
        let texts: Vec<_> = snap.messages.iter().map(|m| m.content.clone().unwrap_or_default()).collect();
        assert_eq!(texts, ["go", "ok", "next"]);
        assert_eq!(snap.seq, page.head_seq);
    }

    /// Between turns a live run's Send reaches the process once, with one user message in
    /// the log; during a turn it queues and `RemoveQueued` drops it (unknown ids reject).
    #[tokio::test]
    async fn live_send_goes_once_and_queued_can_be_removed() {
        let (svc, hub, calls, _dir) = hub_service().await;
        svc.send_user("run-1", "a".into(), "local", None).await.unwrap();
        svc.send_user("run-1", "b".into(), "local", None).await.unwrap();
        let snap = hub.snapshot("run-1").await.unwrap().unwrap();
        assert_eq!(snap.state.queue.len(), 1);
        let texts: Vec<_> = snap.messages.iter().map(|m| m.content.clone().unwrap_or_default()).collect();
        assert_eq!(texts, ["a"], "one user message, not two");

        svc.remove_queued("run-1", "nope".into(), "local", None).await.unwrap_err();
        svc.remove_queued("run-1", snap.state.queue[0].queue_id.clone(), "local", None).await.unwrap();
        feed(&hub, vec![AgentEvent::TurnDone { final_text: None, is_error: false }]);
        let snap = hub.snapshot("run-1").await.unwrap().unwrap();
        assert!(snap.state.queue.is_empty());
        let texts: Vec<_> = snap.messages.iter().map(|m| m.content.clone().unwrap_or_default()).collect();
        assert_eq!(texts, ["a"], "the removed message never started");
        let _ = calls;
    }

    /// An empty prompt would open a turn nothing closes (Codex emits no TurnDone for it).
    #[tokio::test]
    async fn empty_prompt_is_refused() {
        let (svc, _hub, _calls, _dir) = hub_service().await;
        let err = svc.resume_session("run-1", "  ".into(), None).await.unwrap_err();
        assert!(err.contains("empty"), "{err}");
        let params = StartParams {
            cwd: ".".into(),
            model: None,
            prompt: "".into(),
            resume_session_id: None,
            task_id: None,
            permission_mode: "manual".into(),
            isolate_settings: false,
        };
        let err = svc.start_run("claude-code", params, "local", None).await.unwrap_err();
        assert!(err.contains("empty"), "{err}");
    }

    /// A run whose channel closes with no `Exited` is announced as exited once, and a run
    /// that already exited is not listed as running.
    #[tokio::test]
    async fn crashed_run_emits_exited_and_leaves_running_list() {
        #[derive(Debug)]
        struct Rec(Arc<std::sync::Mutex<Vec<serde_json::Value>>>);
        impl EventSink for Rec {
            fn emit_json(&self, _e: &str, payload: serde_json::Value) -> bool {
                self.0.lock().unwrap().push(payload);
                true
            }
        }
        let (svc, _hub, _calls, _dir) = hub_service().await;
        let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        svc.init_events(Arc::new(Rec(seen.clone()))).await;
        assert_eq!(svc.list_running().await, ["run-1"]);
        let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
        let permit = Arc::clone(&svc.semaphore).acquire_owned().await.unwrap();
        svc.spawn_pump("run-1".into(), rx, permit);
        drop(tx);
        until(|| seen.lock().unwrap().len() == 1).await;
        assert_eq!(seen.lock().unwrap()[0]["event"]["type"], "exited");
        until(|| svc.runs.try_lock().is_ok_and(|r| r["run-1"].exited)).await;
        assert!(svc.list_running().await.is_empty());
    }

    /// A send the backend refuses surfaces as a failed turn in the log (the effect runs after
    /// the ack), not a stuck one; a second resume of a session already starting is refused.
    #[tokio::test]
    async fn failed_send_closes_its_turn_and_resume_is_single_flight() {
        use crate::domains::threads::ThreadStatus;
        let (svc, hub, calls, _dir) = hub_service().await;
        feed(&hub, vec![user("go"), AgentEvent::TurnDone { final_text: None, is_error: false }]);
        calls.lock().unwrap().fail_send = true;
        svc.send_user("run-1", "x".into(), "local", None).await.unwrap();
        for _ in 0..200 {
            let s = hub.snapshot("run-1").await.unwrap().unwrap();
            if s.state.status == ThreadStatus::Error {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let snap = hub.snapshot("run-1").await.unwrap().unwrap();
        assert_eq!(snap.state.status, ThreadStatus::Error, "the failed turn is closed");
        assert!(calls.lock().unwrap().sent.is_empty());

        svc.starting.lock().unwrap().insert("sess-9".into());
        let err = svc.resume_session("sess-9", "go".into(), None).await.unwrap_err();
        assert!(err.contains("already starting"), "got: {err}");
    }

    /// A Stop while stopping fires the interrupt again; one with nothing to stop is fine.
    #[tokio::test]
    async fn interrupt_is_idempotent() {
        let (svc, hub, calls, _dir) = hub_service().await;
        feed(&hub, vec![user("go")]);
        svc.interrupt("run-1", "local", None).await.unwrap();
        svc.interrupt("run-1", "local", None).await.unwrap();
        until(|| calls.lock().unwrap().interrupts == 2).await;
        feed(&hub, vec![AgentEvent::TurnDone { final_text: None, is_error: false }]);
        svc.interrupt("run-1", "local", None).await.unwrap(); // idle: nothing to stop
        let page = hub.events_since("run-1", 0).await.unwrap();
        let stops = page.events.iter().filter(|f| matches!(f.event, crate::domains::threads::ThreadEvent::StopRequested)).count();
        assert_eq!(stops, 1);
        assert_eq!(calls.lock().unwrap().interrupts, 2);
    }

    /// Cross-session feed: newest first, noise dropped, `after_seq`/`before_seq`
    /// page on insertion order even though per-session `seq` collides.
    #[tokio::test]
    async fn list_events_pages_across_sessions_and_skips_noise() {
        let (svc, _dir) = test_service().await;
        for id in ["a", "b"] {
            let am = agent_session::ActiveModel {
                id: Set(id.to_string()),
                backend_id: Set(if id == "a" { "claude-code" } else { "other" }.to_string()),
                model: Set(None),
                cwd: Set(".".to_string()),
                status: Set("exited".to_string()),
                started_at: Set(now_iso()),
                ended_at: Set(None),
                exit_code: Set(None),
                title: Set(None),
                task_id: Set(None),
                backend_session_id: Set(None),
            };
            agent_session::Entity::insert(am)
                .exec(svc.db.get_connection())
                .await
                .expect("insert session");
        }
        let exit = AgentEvent::Exited { code: Some(0) };
        let noise = AgentEvent::TextDelta {
            text: "x".into(),
            parent_tool_use_id: None,
        };
        // Both sessions reuse seq 1/2; the delta must not show up.
        svc.persist_event("a", 1, &exit).await;
        svc.persist_event("a", 2, &noise).await;
        svc.persist_event("b", 1, &exit).await;
        svc.persist_event("b", 2, &exit).await;

        let all = svc
            .list_events(None, None, None, None)
            .await
            .unwrap()
            .events;
        assert_eq!(all.len(), 3);
        assert!(all[0].cursor > all[1].cursor && all[1].cursor > all[2].cursor);
        assert_eq!(all[2].session_id, "a");

        let newer = svc
            .list_events(Some(all[2].cursor), None, None, None)
            .await
            .unwrap()
            .events;
        assert_eq!(newer.len(), 2);
        let older = svc
            .list_events(None, Some(all[0].cursor), Some(1), None)
            .await
            .unwrap();
        assert_eq!(older.events.len(), 1);
        assert_eq!(older.events[0].cursor, all[1].cursor);
        // Full page -> raw cursor to continue from; short page -> none.
        assert_eq!(older.next_cursor, Some(all[1].cursor));
        assert_eq!(
            svc.list_events(None, None, None, None)
                .await
                .unwrap()
                .next_cursor,
            None
        );

        // Backend filter applies before LIMIT: only "b" (backend "other") rows.
        let only_b = svc
            .list_events(None, None, Some(2), Some("other".into()))
            .await
            .unwrap();
        assert_eq!(only_b.events.len(), 2);
        assert!(only_b.events.iter().all(|e| e.session_id == "b"));
    }

    /// Usage deltas, tool counts and outcomes, seeded across two backends.
    #[tokio::test]
    async fn analytics_aggregates_per_backend() {
        let (svc, _dir) = test_service().await;
        for (id, backend, code) in [("a", "claude-code", 0), ("b", "other", 1)] {
            let am = agent_session::ActiveModel {
                id: Set(id.to_string()),
                backend_id: Set(backend.to_string()),
                model: Set(None),
                cwd: Set(".".to_string()),
                status: Set("exited".to_string()),
                started_at: Set("2026-11-01T10:00:00+00:00".to_string()),
                ended_at: Set(Some("2026-11-01T10:01:00+00:00".to_string())),
                exit_code: Set(Some(code)),
                title: Set(None),
                task_id: Set(None),
                backend_session_id: Set(None),
            };
            agent_session::Entity::insert(am)
                .exec(svc.db.get_connection())
                .await
                .expect("insert session");
        }
        let usage = |out: i64, cost: f64| AgentEvent::Usage {
            total_cost_usd: Some(cost),
            model_usage: Some(
                json!({"opus": {"inputTokens": 10, "outputTokens": out, "costUSD": cost}}),
            ),
        };
        let tool = |n: &str| AgentEvent::ToolCall {
            id: "t".into(),
            name: n.into(),
            input: json!({}),
            parent_tool_use_id: None,
        };
        // Session a reports cumulative totals: 30 then 60 tokens -> 60 summed, not 90.
        svc.persist_event("a", 1, &usage(20, 1.0)).await;
        svc.persist_event("a", 2, &usage(50, 2.5)).await;
        svc.persist_event("a", 3, &tool("Bash")).await;
        svc.persist_event("a", 4, &tool("Bash")).await;
        svc.persist_event("b", 1, &usage(5, 0.5)).await;
        svc.persist_event("b", 2, &tool("Read")).await;

        let all = svc.analytics(None).await.unwrap();
        assert_eq!(all.usage.len(), 1);
        assert_eq!(all.usage[0].tokens, 75); // a: 60, b: 15
        assert!((all.usage[0].cost_usd - 3.0).abs() < 1e-9);
        assert_eq!(
            (all.tools[0].name.as_str(), all.tools[0].count),
            ("Bash", 2)
        );
        assert!(all
            .outcomes
            .iter()
            .any(|o| o.status == "failed" && o.count == 1));

        let only_b = svc.analytics(Some("other".into())).await.unwrap();
        assert_eq!(only_b.usage[0].tokens, 15);
        assert_eq!(only_b.tools.len(), 1);
        assert_eq!(only_b.outcomes.len(), 1);
    }

    async fn seed_session(svc: &AgentRunService, id: &str) {
        let am = agent_session::ActiveModel {
            id: Set(id.to_string()),
            backend_id: Set("claude-code".to_string()),
            model: Set(None),
            cwd: Set(".".to_string()),
            status: Set("exited".to_string()),
            started_at: Set("2026-11-01T10:00:00+00:00".to_string()),
            ended_at: Set(None),
            exit_code: Set(Some(0)),
            title: Set(None),
            task_id: Set(None),
            backend_session_id: Set(None),
        };
        agent_session::Entity::insert(am)
            .exec(svc.db.get_connection())
            .await
            .expect("insert session");
    }

    /// A resume reuses the session row and restarts the counters at 0, so its first
    /// total can exceed the old one; the boundary must stop that reading as a delta.
    /// Duration is per process: the idle gap between the runs is not counted.
    #[tokio::test]
    async fn analytics_resumed_session_counts_each_process_from_zero() {
        let (svc, _dir) = test_service().await;
        seed_session(&svc, "r").await;
        let usage = |out: i64, cost: f64| AgentEvent::Usage {
            total_cost_usd: Some(cost),
            model_usage: Some(json!({"opus": {"outputTokens": out, "costUSD": cost}})),
        };
        let started = AgentEvent::SessionStarted {
            session_id: "x".into(),
        };
        let exited = AgentEvent::Exited { code: Some(0) };
        // (event, created_at): run 1 lasts 60s, run 2 starts an hour later and lasts 120s.
        let evs = [
            (&started, "2026-11-01T10:00:00+00:00"),
            (&usage(1000, 0.10), "2026-11-01T10:00:30+00:00"),
            (&exited, "2026-11-01T10:01:00+00:00"),
            (&started, "2026-11-01T11:01:00+00:00"),
            (&usage(1200, 0.30), "2026-11-01T11:02:00+00:00"),
            (&exited, "2026-11-01T11:03:00+00:00"),
        ];
        for (i, (e, at)) in evs.into_iter().enumerate() {
            let am = agent_event::ActiveModel {
                id: Set(format!("e{i}")),
                session_id: Set("r".to_string()),
                seq: Set(i as i64),
                event_json: Set(serde_json::to_string(e).unwrap()),
                created_at: Set(at.to_string()),
            };
            agent_event::Entity::insert(am)
                .exec(svc.db.get_connection())
                .await
                .expect("insert event");
        }
        let a = svc.analytics(None).await.unwrap();
        assert_eq!(a.usage[0].tokens, 2200);
        assert!((a.usage[0].cost_usd - 0.40).abs() < 1e-9);
        assert!((a.avg_duration_secs.unwrap() - 90.0).abs() < 1e-6);
    }

    /// A lone model without `costUSD` may use the session total, but only when no
    /// usage event of the session has per-model cost (else it flips mid-session).
    #[tokio::test]
    async fn analytics_cost_fallback_is_per_session_not_per_event() {
        let (svc, _dir) = test_service().await;
        seed_session(&svc, "f").await;
        let ev = |m: serde_json::Value, cost: f64| AgentEvent::Usage {
            total_cost_usd: Some(cost),
            model_usage: Some(json!({ "opus": m })),
        };
        svc.persist_event("f", 1, &ev(json!({"outputTokens": 10}), 0.5))
            .await;
        svc.persist_event(
            "f",
            2,
            &ev(json!({"outputTokens": 20, "costUSD": 1.0}), 1.0),
        )
        .await;
        let a = svc.analytics(None).await.unwrap();
        // Event 1 has no per-model cost and the session does elsewhere: 0 then 1.0.
        assert!((a.usage[0].cost_usd - 1.0).abs() < 1e-9);
    }
}
