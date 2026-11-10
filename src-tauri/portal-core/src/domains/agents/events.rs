//! Normalized event stream every [`super::backend::AgentBackend`] emits.
//!
//! A backend's own wire format (Claude Code's stream-json, Codex's later)
//! never leaves `claude_code.rs` / its sibling adapters — everything
//! downstream (the ring buffer, the `EventSink` fan-out, the frontend) only
//! ever sees this enum, so a second backend is "write the match arms", not a
//! UI rewrite.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub enum AgentEvent {
    SessionStarted {
        session_id: String,
    },
    /// Incremental assistant text, for a live-typing effect.
    TextDelta {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    /// Incremental extended-thinking text.
    ThinkingDelta {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    /// A finished assistant/user text turn (superseded `TextDelta`s).
    Message {
        role: String,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    ToolCall {
        id: String,
        name: String,
        #[ts(type = "unknown")]
        input: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
    /// A subagent ("Task"/"Agent" tool) was dispatched.
    SubagentStarted {
        tool_use_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    SubagentFinished {
        tool_use_id: String,
    },
    /// A tool call is waiting on a human decision via the permission-hook relay.
    PermissionRequest {
        request_id: String,
        tool_name: String,
        #[ts(type = "unknown")]
        tool_input: Value,
        #[serde(default)]
        #[ts(type = "unknown[]")]
        suggestions: Vec<Value>,
    },
    PermissionResolved {
        request_id: String,
        allowed: bool,
    },
    Todos {
        #[ts(type = "unknown")]
        items: Value,
    },
    Usage {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        total_cost_usd: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(type = "unknown")]
        model_usage: Option<Value>,
    },
    /// The turn finished (normally or via interrupt).
    TurnDone {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        final_text: Option<String>,
    },
    /// The backend process exited; the run is over.
    Exited {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        code: Option<i32>,
    },
    /// A wire line we didn't need to normalize but don't want to drop —
    /// mainly diagnostic (`system` subtypes with no UI meaning yet).
    Raw {
        line: String,
    },
}
