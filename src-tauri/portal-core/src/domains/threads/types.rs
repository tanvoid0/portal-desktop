//! Wire + state types of the thread log. Everything the hub emits or returns lives here.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::domains::agents::AgentEvent;
use crate::domains::coder::types::{ChatMessage, CoderSubAgent, PendingApproval};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
#[serde(default)]
pub struct ThreadMeta {
    /// `portal` or the agent backend id (`claude-code`, `codex`, `cursor`).
    pub provider: String,
    pub model: Option<String>,
    pub llm_provider: Option<String>,
    pub workspace_root: String,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    pub kind: String,
    pub title: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum ThreadStatus {
    #[default]
    Idle,
    Running,
    Waiting,
    Error,
}

impl ThreadStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum TurnTrigger {
    Send,
    Retry,
    Edit,
    Approve,
    Queue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum TurnOutcome {
    Completed,
    Stopped,
    Error,
    Exhausted,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum DequeueReason {
    Sent,
    Removed,
}

/// Persisted, one `seq` each. Section 2 of the design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum ThreadEvent {
    Created {
        provider: String,
        model: Option<String>,
        llm_provider: Option<String>,
        workspace_root: String,
        #[ts(type = "number | null")]
        project_id: Option<i64>,
        kind: String,
        title: String,
    },
    MessageAppended {
        message: ChatMessage,
    },
    TurnStarted {
        trigger: TurnTrigger,
    },
    TurnEnded {
        outcome: TurnOutcome,
        error: Option<String>,
        final_text: Option<String>,
    },
    ApprovalRequested {
        approval: PendingApproval,
    },
    ApprovalResolved {
        call_id: String,
        allow: bool,
        auto: bool,
    },
    UsageUpdated {
        total_cost_usd: Option<f64>,
        #[ts(type = "unknown")]
        model_usage: Option<Value>,
    },
    TodosUpdated {
        #[ts(type = "unknown")]
        items: Value,
    },
    SubagentsRunning {
        count: u32,
    },
    SessionLinked {
        backend_session_id: String,
    },
    ProcessExited {
        code: Option<i32>,
    },
    TitleSet {
        title: String,
    },
    ModelSet {
        model: Option<String>,
        llm_provider: Option<String>,
    },
    /// Edit, retry and platform reconcile: keep the first `keep` messages, then append `tail`.
    MessagesRewritten {
        keep: u32,
        tail: Vec<ChatMessage>,
    },
    StopRequested,
    HostRequested {
        call_id: String,
        /// `run_command` | `list_terminals` | `ask_user`.
        kind: String,
        #[ts(type = "unknown")]
        payload: Value,
    },
    HostResolved {
        call_id: String,
    },
    Enqueued {
        queue_id: String,
        text: String,
    },
    Dequeued {
        queue_id: String,
        reason: DequeueReason,
    },
    ChangeRecorded {
        change_id: String,
        path: String,
        status: String,
    },
    SubAgentUpdated {
        sub_agent: CoderSubAgent,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct HostRequest {
    pub call_id: String,
    pub kind: String,
    #[ts(type = "unknown")]
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct QueuedMessage {
    pub queue_id: String,
    pub text: String,
    /// Who sent it, so the drained turn runs with that sender's trust (local or remote).
    #[serde(default)]
    #[ts(skip)]
    pub actor: String,
}

/// An approval or host request that already has an answer, and who gave it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ResolvedCall {
    pub call_id: String,
    pub allow: bool,
    pub by: String,
}

/// How many `recently_resolved` entries a thread remembers.
pub const RECENTLY_RESOLVED: usize = 32;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
#[serde(default)]
pub struct ThreadUsage {
    pub total_cost_usd: Option<f64>,
    #[ts(type = "unknown")]
    pub model_usage: Option<Value>,
}

/// The fold of a thread's events. `messages` live in `thread_messages` (and in the
/// snapshot beside this), so they are not part of the serialized form.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
#[serde(default)]
pub struct ThreadState {
    pub meta: ThreadMeta,
    pub status: ThreadStatus,
    #[serde(skip)]
    #[ts(skip)]
    pub messages: Vec<ChatMessage>,
    pub approvals: Vec<PendingApproval>,
    pub usage: ThreadUsage,
    #[ts(type = "unknown")]
    pub todos: Option<Value>,
    pub subagents_running: u32,
    pub host_requests: Vec<HostRequest>,
    pub queue: Vec<QueuedMessage>,
    pub sub_agents: Vec<CoderSubAgent>,
    /// `StopRequested` until the turn actually ends.
    pub stopping: bool,
    /// Newest last, capped at `RECENTLY_RESOLVED`; the first-wins check reads it.
    pub recently_resolved: Vec<ResolvedCall>,
    pub error: Option<String>,
    pub backend_session_id: Option<String>,
    #[ts(type = "number")]
    pub last_seq: i64,
}

impl ThreadState {
    /// `meta.provider` is empty until `Created` is applied.
    pub fn created(&self) -> bool {
        !self.meta.provider.is_empty()
    }
    /// A turn is open while running or parked on an approval.
    pub fn turn_open(&self) -> bool {
        matches!(self.status, ThreadStatus::Running | ThreadStatus::Waiting)
    }
}

/// Who/when for `decide`, so it stays deterministic. `thinking` is the actor's
/// in-memory partial thinking, folded into a tool-call message like the TS reducer did.
#[derive(Debug, Clone)]
pub struct Ctx {
    pub now: String,
    pub actor: String,
    pub thinking: String,
    /// Fresh id for an `Enqueued` this command may produce.
    pub queue_id: String,
}

/// Portal-engine producer inputs (actor `system`).
#[derive(Debug, Clone)]
pub enum PortalInput {
    Reconcile { messages: Vec<ChatMessage>, title: Option<String> },
    Title { title: String },
    /// Live-only: the actor keeps it and emits a heartbeat frame, nothing is persisted.
    Heartbeat { waited_seconds: u32 },
    HostRequest { call_id: String, kind: String, payload: Value },
    Pending(PendingApproval),
    Failed { error: String },
    Finished { outcome: TurnOutcome, final_text: Option<String> },
}

#[derive(Debug, Clone)]
pub enum ThreadCommand {
    Create(ThreadMeta),
    // Client intents.
    Send { text: String },
    RemoveQueued { queue_id: String },
    EditMessage { index: usize, text: String },
    Retry,
    Stop,
    Approve {
        call_id: String,
        allow: bool,
        remember: bool,
        edited_pattern: Option<String>,
        edited_command: Option<String>,
    },
    ResolveHost { call_id: String, result: Value },
    SetModel { model: Option<String>, llm_provider: Option<String> },
    // Producer inputs.
    Provider(AgentEvent),
    Portal(PortalInput),
    ChangeRecorded { change_id: String, path: String, status: String },
    SubAgentUpdated { sub_agent: CoderSubAgent },
    /// The actor's own, after a turn completes with a queue behind it.
    DrainQueue,
}

/// What a committed command asks the provider side to do. Provider-neutral; the
/// registered `ThreadEffects` maps it onto the Coder service or an agent backend.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// `actor` is whoever sent the message (for a drained one, its original sender).
    StartTurn { trigger: TurnTrigger, text: String, actor: String },
    Cancel,
    ResolveApproval {
        call_id: String,
        allow: bool,
        remember: bool,
        edited_pattern: Option<String>,
        edited_command: Option<String>,
    },
    DeliverHostResult { call_id: String, result: Value },
}

/// Runs effects. Never awaited by the actor and never inside a transaction; results
/// re-enter as commands (`ThreadHub::submit` / `command`).
#[async_trait::async_trait]
pub trait ThreadEffects: Send + Sync {
    async fn run(&self, thread_id: &str, effect: Effect);
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub cmd: ThreadCommand,
    pub actor: String,
    pub command_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reject {
    AlreadyCreated,
    NotCreated,
    /// First wins: `by` (an actor) already answered with `allow`.
    AlreadyResolved { by: String, allow: bool },
    /// Unknown call, queue entry or message.
    NotFound,
    /// Edit/retry while a turn is open.
    Busy,
    /// Stop with nothing to stop.
    NotRunning,
    Invalid,
}

impl Reject {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyCreated => "already_created",
            Self::NotCreated => "not_created",
            Self::AlreadyResolved { .. } => "already_resolved",
            Self::NotFound => "not_found",
            Self::Busy => "busy",
            Self::NotRunning => "not_running",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub enum AckStatus {
    Accepted,
    Duplicate,
    Rejected,
}

/// The answer to a client command. `seq` is the thread's head after it (or the
/// original command's last seq for a duplicate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct CommandAck {
    pub status: AckStatus,
    #[ts(type = "number")]
    pub seq: i64,
    pub code: Option<String>,
    /// `already_resolved`: who answered first, and what they chose.
    pub resolved_by: Option<String>,
    pub allow: Option<bool>,
}

impl CommandAck {
    pub fn new(status: AckStatus, seq: i64) -> Self {
        Self { status, seq, code: None, resolved_by: None, allow: None }
    }
    pub fn rejected(r: &Reject, seq: i64) -> Self {
        let (resolved_by, allow) = match r {
            Reject::AlreadyResolved { by, allow } => (Some(by.clone()), Some(*allow)),
            _ => (None, None),
        };
        Self { code: Some(r.code().into()), resolved_by, allow, ..Self::new(AckStatus::Rejected, seq) }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadEventFrame {
    pub thread_id: String,
    /// `meta.provider` of the thread, so a client can route a frame without a lookup.
    pub provider: String,
    #[ts(type = "number")]
    pub seq: i64,
    pub at: String,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    pub event: ThreadEvent,
}

/// Live-only streaming chunk. `seq` is the last committed seq; `offset` the UTF-16 (JS string)
/// length of the partial this chunk extends (`text` or `thinking`,
/// never both in one frame).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadDeltaFrame {
    pub thread_id: String,
    #[ts(type = "number")]
    pub seq: i64,
    #[ts(type = "number")]
    pub offset: usize,
    pub text: String,
    pub thinking: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadSummary {
    pub id: String,
    pub provider: String,
    pub kind: String,
    pub title: String,
    pub workspace_root: String,
    #[ts(type = "number | null")]
    pub project_id: Option<i64>,
    pub model: Option<String>,
    pub llm_provider: Option<String>,
    pub status: String,
    #[ts(type = "number")]
    pub last_seq: i64,
    #[ts(type = "number")]
    pub message_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Answered by the actor, so the partial text/thinking of a turn in flight is included.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadSnapshot {
    pub thread: ThreadSummary,
    #[ts(type = "number")]
    pub seq: i64,
    pub messages: Vec<ChatMessage>,
    pub state: ThreadState,
    pub partial_text: String,
    pub partial_thinking: String,
    /// Live-only, from the last heartbeat of the turn in flight.
    pub waited_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadEventsPage {
    pub events: Vec<ThreadEventFrame>,
    #[ts(type = "number")]
    pub head_seq: i64,
    /// The client's cursor is ahead of the log or too far behind: re-snapshot.
    pub reset: bool,
}
