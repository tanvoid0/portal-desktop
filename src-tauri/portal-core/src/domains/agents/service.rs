//! `AgentRunService` — registry of [`AgentBackend`]s, a concurrency cap, a
//! pump task per run that turns a backend's raw [`AgentEvent`]s into the
//! fan-out + a late-attach ring buffer, and (P2) durable persistence: one
//! `agent_sessions` row per run, its message turns in `agent_messages`, and
//! its full raw event log in `agent_events` — see
//! `src-tauri/portal-core/src/migrations/m20261111_000048_create_agent_tables.rs`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use serde_json::json;
use tokio::sync::{mpsc, Mutex, Semaphore};
use ts_rs::TS;

use crate::database::DatabaseManager;
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
    ring: VecDeque<AgentEvent>,
    /// Monotonic per-run counter shared by `agent_messages.seq` and
    /// `agent_events.seq` — only needs to be strictly increasing within a
    /// run, not contiguous per-table, so one counter for both is enough.
    next_seq: i64,
}

pub struct AgentRunService {
    backends: Vec<Arc<dyn AgentBackend>>,
    runs: Mutex<HashMap<String, RunEntry>>,
    semaphore: Arc<Semaphore>,
    sink: Mutex<Option<Arc<dyn EventSink>>>,
    db: Arc<DatabaseManager>,
}

impl AgentRunService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            backends: vec![Arc::new(ClaudeCodeBackend)],
            runs: Mutex::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS)),
            sink: Mutex::new(None),
            db,
        }
    }

    pub async fn init_events(&self, sink: Arc<dyn EventSink>) {
        *self.sink.lock().await = Some(sink);
    }

    async fn emit(&self, run_id: &str, event: &AgentEvent) {
        if let Some(sink) = self.sink.lock().await.as_ref() {
            sink.emit(
                "agents://event",
                json!({ "run_id": run_id, "event": event }),
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
    /// conversation streams via `agents://event` (`{run_id, event}`).
    pub async fn start_run(
        self: &Arc<Self>,
        backend_id: &str,
        params: StartParams,
    ) -> Result<String, String> {
        let backend = self
            .find_backend(backend_id)
            .ok_or_else(|| format!("unknown backend '{backend_id}'"))?;
        // Held for the run's lifetime via the pump task below, so a run that
        // never gets cancelled still frees its slot once the process exits.
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|e| e.to_string())?;

        let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
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
            },
        );

        self.persist_session_start(&run_id, backend_id, &params)
            .await;
        if has_prompt {
            self.persist_message(&run_id, 0, "user", &params.prompt, None)
                .await;
        }

        let svc = Arc::clone(self);
        let rid = run_id.clone();
        tokio::spawn(async move {
            loop {
                let Some(event) = rx.recv().await else {
                    // The channel closed without an `Exited` event ever
                    // coming through (process crash, app quit mid-run) — the
                    // session would otherwise sit at status "running" forever
                    // with no `ended_at`, since nothing else reconciles it.
                    svc.persist_session_end(&rid, None, "interrupted").await;
                    break;
                };
                svc.emit(&rid, &event).await;
                // `code` here means "this event IS Exited"; its inner
                // `Option<i32>` is the process exit code itself.
                let exit_code: Option<Option<i32>> = match &event {
                    AgentEvent::Exited { code } => Some(*code),
                    _ => None,
                };
                let exited = exit_code.is_some();
                let mut runs = svc.runs.lock().await;
                let seq = runs.get_mut(&rid).map(|entry| {
                    let seq = entry.next_seq;
                    entry.next_seq += 1;
                    if entry.ring.len() >= RING_CAPACITY {
                        entry.ring.pop_front();
                    }
                    entry.ring.push_back(event.clone());
                    seq
                });
                drop(runs);

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
                    svc.persist_event(&rid, seq.unwrap_or(0), &event).await;
                }
                if let AgentEvent::Message {
                    role,
                    text,
                    parent_tool_use_id,
                } = &event
                {
                    svc.persist_message(
                        &rid,
                        seq.unwrap_or(0),
                        role,
                        text,
                        parent_tool_use_id.clone(),
                    )
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

            // Grace period for a client mid-poll of `get_events` to still
            // see the final events, then free the entry so `runs` doesn't
            // grow for the life of the process.
            tokio::time::sleep(EVENT_REPLAY_GRACE).await;
            svc.runs.lock().await.remove(&rid);
        });

        Ok(run_id)
    }

    // ponytail: one `Mutex<HashMap<..>>` for every run's control handle —
    // each call below just forwards a quick stdin write/HTTP-decision, so
    // holding it for the await is cheap. Split per-run if a slow backend ever
    // makes that untrue.
    pub async fn send_user(&self, run_id: &str, text: String) -> Result<(), String> {
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

    pub async fn decide_permission(
        &self,
        run_id: &str,
        request_id: String,
        allow: bool,
    ) -> Result<(), String> {
        let runs = self.runs.lock().await;
        let entry = runs.get(run_id).ok_or("run not found")?;
        entry.control.decide_permission(request_id, allow).await
    }

    pub async fn interrupt(&self, run_id: &str) -> Result<(), String> {
        let runs = self.runs.lock().await;
        let entry = runs.get(run_id).ok_or("run not found")?;
        entry.control.interrupt().await
    }

    pub async fn cancel(&self, run_id: &str) -> Result<(), String> {
        let runs = self.runs.lock().await;
        let entry = runs.get(run_id).ok_or("run not found")?;
        entry.control.kill().await
    }

    /// Snapshot of this run's event ring, for a UI that attaches late.
    pub async fn get_events(&self, run_id: &str) -> Vec<AgentEvent> {
        self.runs
            .lock()
            .await
            .get(run_id)
            .map(|e| e.ring.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Run ids with a live backend process right now, regardless of which
    /// page/tab started them. `/agents/run` has no persisted run id of its
    /// own (unlike coder's thread list) — a reload calls this on mount to
    /// find a run it should reattach to instead of showing a blank launcher
    /// while the backend sits alive (QA-127). Cheap and non-sensitive (just
    /// ids), so `agents` rather than `agents.history`.
    pub async fn list_running(&self) -> Vec<String> {
        self.runs.lock().await.keys().cloned().collect()
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
                .filter_map(|row| {
                    match serde_json::from_str::<AgentEvent>(&row.event_json) {
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
                    }
                })
                .collect(),
        }))
    }

    async fn persist_session_start(&self, run_id: &str, backend_id: &str, params: &StartParams) {
        let am = agent_session::ActiveModel {
            id: Set(run_id.to_string()),
            backend_id: Set(backend_id.to_string()),
            model: Set(params.model.clone()),
            cwd: Set(params.cwd.clone()),
            status: Set("running".to_string()),
            started_at: Set(now_iso()),
            ended_at: Set(None),
            exit_code: Set(None),
        };
        if let Err(e) = agent_session::Entity::insert(am)
            .exec(self.db.get_connection())
            .await
        {
            eprintln!("agents: persist session start failed: {e}");
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

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
