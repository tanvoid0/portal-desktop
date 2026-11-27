//! `ThreadHub`: one tokio task (actor) per thread owns its state and is the only
//! writer. Producers `submit` commands; the actor decides, commits events + the
//! projection in one transaction, swaps its state, then emits frames. Streaming
//! deltas skip the database and are coalesced into `coder://thread-delta` frames.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};

use super::apply::apply;
use super::decide::decide;
use super::store;
use super::types::*;
use crate::database::DatabaseManager;
use crate::domains::agents::AgentEvent;
use crate::events::{EventSink, EventSinkExt};

/// One delta frame per this window, to protect the shared broadcast channel.
const DELTA_COALESCE: Duration = Duration::from_millis(40);
/// Minimum gap between reload attempts while the actor's state is untrusted.
const RELOAD_BACKOFF: Duration = Duration::from_secs(1);
const IDLE_EVICT: Duration = Duration::from_secs(10 * 60);
const EVENT_FRAME: &str = "coder://thread-event";
const DELTA_FRAME: &str = "coder://thread-delta";

enum Msg {
    Command(Envelope),
    Snapshot(oneshot::Sender<Option<ThreadSnapshot>>),
}

pub struct ThreadHub {
    db: Arc<DatabaseManager>,
    sink: Arc<dyn EventSink>,
    actors: Mutex<HashMap<String, mpsc::UnboundedSender<Msg>>>,
}

impl ThreadHub {
    pub fn new(db: Arc<DatabaseManager>, sink: Arc<dyn EventSink>) -> Arc<Self> {
        Arc::new(Self { db, sink, actors: Mutex::new(HashMap::new()) })
    }

    /// Sync and cheap (the agent pump calls it per event). An unknown thread is
    /// created by its actor from the persisted `agent_sessions` row.
    pub fn submit(self: &Arc<Self>, thread_id: &str, cmd: ThreadCommand) {
        let env = Envelope { cmd, actor: "system".into(), command_id: None };
        self.send(thread_id, Msg::Command(env));
    }

    pub async fn snapshot(self: &Arc<Self>, thread_id: &str) -> Result<Option<ThreadSnapshot>, String> {
        let live = self.actors.lock().unwrap_or_else(|e| e.into_inner()).contains_key(thread_id);
        if !live && !store::exists(self.db.get_connection(), thread_id).await? {
            return Ok(None);
        }
        let (tx, rx) = oneshot::channel();
        self.send(thread_id, Msg::Snapshot(tx));
        rx.await.map_err(|_| "threads: actor stopped".to_string())
    }

    pub async fn events_since(&self, thread_id: &str, seq: i64) -> Result<ThreadEventsPage, String> {
        store::events_since(self.db.get_connection(), thread_id, seq).await
    }

    pub async fn list(&self) -> Result<Vec<ThreadSummary>, String> {
        store::list(self.db.get_connection()).await
    }

    fn send(self: &Arc<Self>, id: &str, msg: Msg) {
        // Held across the send so an idle actor deciding to exit (`try_evict`) cannot
        // race a message into a channel nobody reads any more.
        let mut map = self.actors.lock().unwrap_or_else(|e| e.into_inner());
        let msg = match map.get(id) {
            Some(tx) => match tx.send(msg) {
                Ok(()) => return,
                Err(mpsc::error::SendError(m)) => m, // actor died; replace it
            },
            None => msg,
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = tx.send(msg);
        map.insert(id.to_string(), tx);
        tokio::spawn(actor_loop(Arc::clone(self), id.to_string(), rx));
    }

    fn remove(&self, id: &str) {
        self.actors.lock().unwrap_or_else(|e| e.into_inner()).remove(id);
    }

    /// Drop a thread for good (its session was deleted): forget the actor's sender
    /// and let the `ON DELETE CASCADE` take the events and messages. That does not
    /// stop an actor mid-queue (a queued commit could recreate the row), so it is only
    /// safe for a thread with no live run, which is all `delete_session` allows.
    pub async fn delete(&self, thread_id: &str) -> Result<(), String> {
        self.remove(thread_id);
        store::delete(self.db.get_connection(), thread_id).await
    }

    /// An actor idle past `IDLE_EVICT` leaves the map unless a message just landed.
    fn try_evict(&self, id: &str, rx: &mpsc::UnboundedReceiver<Msg>) -> bool {
        let mut map = self.actors.lock().unwrap_or_else(|e| e.into_inner());
        let idle = rx.is_empty();
        if idle {
            map.remove(id);
        }
        idle
    }
}

struct Pending {
    thinking: bool,
    text: String,
    offset: usize,
    since: Instant,
}

struct Actor {
    id: String,
    db: Arc<DatabaseManager>,
    sink: Arc<dyn EventSink>,
    state: ThreadState,
    /// `state` mirrors the database. False after a failed (re)load: commands are
    /// dropped until a reload, retried at most once per `RELOAD_BACKOFF`.
    trusted: bool,
    last_load: Option<Instant>,
    /// Streaming text of the turn in flight; in memory only, never persisted.
    partial_text: String,
    partial_thinking: String,
    /// UTF-16 length of each partial, the unit JS clients count in (delta `offset`).
    text_units: usize,
    thinking_units: usize,
    pending: Option<Pending>,
}

async fn actor_loop(hub: Arc<ThreadHub>, id: String, mut rx: mpsc::UnboundedReceiver<Msg>) {
    let mut a = Actor {
        id: id.clone(),
        db: Arc::clone(&hub.db),
        sink: Arc::clone(&hub.sink),
        state: ThreadState::default(),
        trusted: false,
        last_load: None,
        partial_text: String::new(),
        partial_thinking: String::new(),
        text_units: 0,
        thinking_units: 0,
        pending: None,
    };
    a.ensure_trusted().await;
    loop {
        let wait = match &a.pending {
            Some(p) => DELTA_COALESCE.saturating_sub(p.since.elapsed()),
            None => IDLE_EVICT,
        };
        match tokio::time::timeout(wait, rx.recv()).await {
            Ok(Some(Msg::Command(env))) => a.command(env).await,
            Ok(Some(Msg::Snapshot(reply))) => {
                a.flush_delta(); // so the client's partial matches the next frame's offset
                let _ = reply.send(a.snapshot().await);
            }
            Ok(None) => break,
            Err(_) if a.pending.is_some() => {}
            Err(_) if hub.try_evict(&id, &rx) => break,
            Err(_) => {}
        }
        if a.pending.as_ref().is_some_and(|p| p.since.elapsed() >= DELTA_COALESCE) {
            a.flush_delta();
        }
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl Actor {
    /// Reload the state from the database if it is untrusted (rate-limited).
    async fn ensure_trusted(&mut self) -> bool {
        if self.trusted {
            return true;
        }
        if self.last_load.is_some_and(|t| t.elapsed() < RELOAD_BACKOFF) {
            return false;
        }
        self.last_load = Some(Instant::now());
        match store::load(self.db.get_connection(), &self.id).await {
            Ok(s) => {
                self.state = s.unwrap_or_default();
                self.trusted = true;
            }
            Err(e) => eprintln!("{e}"),
        }
        self.trusted
    }

    /// An untrusted actor drops the command (deltas silently); nothing ends the actor.
    async fn command(&mut self, env: Envelope) {
        if !self.ensure_trusted().await {
            return;
        }
        if !self.state.created() && !matches!(env.cmd, ThreadCommand::Create(_)) {
            let meta = match store::session_meta(self.db.get_connection(), &self.id).await {
                Ok(Some(m)) => m,
                Ok(None) => return eprintln!("threads: no session row for {}, dropping event", self.id),
                Err(e) => return eprintln!("{e}"),
            };
            let create = Envelope { cmd: ThreadCommand::Create(meta), actor: "system".into(), command_id: None };
            self.run(create).await;
            if !self.trusted {
                return;
            }
        }
        let delta = match &env.cmd {
            ThreadCommand::Provider(AgentEvent::TextDelta { text, parent_tool_use_id: None }) => Some((false, text.clone())),
            ThreadCommand::Provider(AgentEvent::ThinkingDelta { text, parent_tool_use_id: None }) => Some((true, text.clone())),
            _ => None,
        };
        self.run(env).await;
        if let (true, Some((thinking, text))) = (self.trusted, delta) {
            self.push_delta(thinking, text);
        }
    }

    /// decide -> apply in place -> one transaction -> emit. A failed commit leaves the
    /// state untrusted until it is reloaded.
    async fn run(&mut self, env: Envelope) {
        // Only a tool call folds the streamed thinking into its message.
        let thinking = match &env.cmd {
            ThreadCommand::Provider(AgentEvent::ToolCall { .. }) => self.partial_thinking.clone(),
            _ => String::new(),
        };
        let ctx = Ctx { now: now_iso(), actor: env.actor.clone(), thinking };
        let events = match decide(&self.state, &env.cmd, &ctx) {
            Ok(e) => e,
            Err(r) => return eprintln!("threads: {} rejected command: {r:?}", self.id),
        };
        if events.is_empty() {
            return;
        }
        self.flush_delta(); // pending deltas precede this commit's frames
        let base = self.state.last_seq;
        let prev_messages = self.state.messages.len();
        let frames: Vec<ThreadEventFrame> = events
            .into_iter()
            .enumerate()
            .map(|(i, event)| {
                apply(&mut self.state, &event);
                ThreadEventFrame {
                    thread_id: self.id.clone(),
                    seq: base + 1 + i as i64,
                    at: ctx.now.clone(),
                    actor: ctx.actor.clone(),
                    command_id: env.command_id.clone(),
                    event,
                }
            })
            .collect();
        self.state.last_seq = base + frames.len() as i64;
        if let Err(e) = store::commit(self.db.get_connection(), &self.id, prev_messages, &self.state, &frames).await {
            eprintln!("{e}");
            // The transaction rolled back, so the database still holds the pre-command
            // state; the in-memory one is ahead of it until reloaded.
            self.trusted = false;
            self.last_load = None;
            self.ensure_trusted().await;
            return;
        }
        // The turn's streamed text is superseded once the assistant's message lands or
        // the turn ends; a user message or tool result mid-turn leaves it alone.
        let clears = |e: &ThreadEvent| match e {
            ThreadEvent::MessageAppended { message } => message.role == "assistant",
            ThreadEvent::TurnEnded { .. } | ThreadEvent::ProcessExited { .. } => true,
            _ => false,
        };
        if frames.iter().any(|f| clears(&f.event)) {
            self.partial_text.clear();
            self.partial_thinking.clear();
            self.text_units = 0;
            self.thinking_units = 0;
        }
        for f in &frames {
            self.sink.emit(EVENT_FRAME, f);
        }
    }

    fn push_delta(&mut self, thinking: bool, text: String) {
        // One kind per frame, so `offset` is unambiguous.
        if self.pending.as_ref().is_some_and(|p| p.thinking != thinking) {
            self.flush_delta();
        }
        let (buf, units) = if thinking {
            (&mut self.partial_thinking, &mut self.thinking_units)
        } else {
            (&mut self.partial_text, &mut self.text_units)
        };
        let offset = *units;
        buf.push_str(&text);
        *units += text.encode_utf16().count();
        self.pending
            .get_or_insert_with(|| Pending { thinking, text: String::new(), offset, since: Instant::now() })
            .text
            .push_str(&text);
    }

    fn flush_delta(&mut self) {
        let Some(p) = self.pending.take() else { return };
        let (text, thinking) = if p.thinking { (String::new(), p.text) } else { (p.text, String::new()) };
        self.sink.emit(
            DELTA_FRAME,
            ThreadDeltaFrame { thread_id: self.id.clone(), seq: self.state.last_seq, offset: p.offset, text, thinking },
        );
    }

    async fn snapshot(&mut self) -> Option<ThreadSnapshot> {
        if !self.ensure_trusted().await || !self.state.created() {
            return None;
        }
        let thread = store::summary(self.db.get_connection(), &self.id).await.ok().flatten()?;
        let mut state = self.state.clone();
        let messages = std::mem::take(&mut state.messages);
        Some(ThreadSnapshot {
            thread,
            seq: state.last_seq,
            messages,
            state,
            partial_text: self.partial_text.clone(),
            partial_thinking: self.partial_thinking.clone(),
        })
    }
}
