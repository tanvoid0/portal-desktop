//! `AgentRunService` — registry of [`AgentBackend`]s, a concurrency cap, and
//! one pump task per run that turns a backend's raw [`AgentEvent`]s into the
//! fan-out + a late-attach ring buffer. In-memory only (no DB) — that's P2.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use tokio::sync::{mpsc, Mutex, Semaphore};
use ts_rs::TS;

use crate::events::{EventSink, EventSinkExt};

use super::backend::{AgentBackend, BackendModel, RunControl, StartParams};
use super::claude_code::ClaudeCodeBackend;
use super::events::AgentEvent;

/// Late-attach history depth per run — a live view, not a transcript (that's
/// P2's job, backed by the database).
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

struct RunEntry {
    control: Box<dyn RunControl>,
    ring: VecDeque<AgentEvent>,
}

pub struct AgentRunService {
    backends: Vec<Arc<dyn AgentBackend>>,
    runs: Mutex<HashMap<String, RunEntry>>,
    semaphore: Arc<Semaphore>,
    sink: Mutex<Option<Arc<dyn EventSink>>>,
}

impl Default for AgentRunService {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRunService {
    pub fn new() -> Self {
        Self {
            backends: vec![Arc::new(ClaudeCodeBackend)],
            runs: Mutex::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS)),
            sink: Mutex::new(None),
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
        let control = backend.start(params, tx).await?;
        let run_id = uuid::Uuid::new_v4().to_string();

        self.runs.lock().await.insert(
            run_id.clone(),
            RunEntry {
                control,
                ring: VecDeque::new(),
            },
        );

        let svc = Arc::clone(self);
        let rid = run_id.clone();
        tokio::spawn(async move {
            loop {
                let Some(event) = rx.recv().await else { break };
                svc.emit(&rid, &event).await;
                let exited = matches!(event, AgentEvent::Exited { .. });
                let mut runs = svc.runs.lock().await;
                if let Some(entry) = runs.get_mut(&rid) {
                    if entry.ring.len() >= RING_CAPACITY {
                        entry.ring.pop_front();
                    }
                    entry.ring.push_back(event);
                }
                drop(runs);
                // `rx` otherwise never closes on its own: `runs` still holds
                // this run's `RunControl`, and so a live `events_tx` clone,
                // for as long as the entry stays in the map. Breaking here
                // instead of waiting for channel-close is what releases the
                // concurrency permit below the moment the run actually ends.
                if exited {
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
        let runs = self.runs.lock().await;
        let entry = runs.get(run_id).ok_or("run not found")?;
        entry.control.send_user(text).await
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
}
