//! Wire + state types of the thread log. Everything the hub emits or returns lives here.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::domains::agents::AgentEvent;
use crate::domains::coder::types::{ChatMessage, PendingApproval};

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

/// Persisted, one `seq` each. Section 2 of the design; S1 carries the CLI subset.
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
}

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
}

#[derive(Debug, Clone)]
pub enum ThreadCommand {
    Create(ThreadMeta),
    Provider(AgentEvent),
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/threads/generated/")]
pub struct ThreadEventFrame {
    pub thread_id: String,
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
