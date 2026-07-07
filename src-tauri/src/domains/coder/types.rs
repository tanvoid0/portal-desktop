//! Wire + persistence types for the coder agent.

use serde::{Deserialize, Serialize};

use super::diff::Hunk;

/// A single chat message in a thread transcript. This mirrors the OpenAI
/// chat-completions message shape so it can be sent to the platform `/v1`
/// proxy verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// `system` | `user` | `assistant` | `tool`.
    pub role: String,
    /// Text content. May be empty for an assistant turn that only issues tool
    /// calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Present on assistant turns that call tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Present on `tool` messages — the id of the call this result answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(text: impl Into<String>) -> Self {
        Self { role: "system".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
    }
    pub fn user(text: impl Into<String>) -> Self {
        Self { role: "user".into(), content: Some(text.into()), tool_calls: None, tool_call_id: None }
    }
    pub fn tool_result(call_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: Some(call_id.into()),
        }
    }
}

/// An OpenAI-format tool call emitted by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(default = "default_tool_type")]
    pub r#type: String,
    pub function: FunctionCall,
}

fn default_tool_type() -> String {
    "function".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    /// Raw JSON string of arguments (OpenAI encodes this as a string).
    #[serde(default)]
    pub arguments: String,
}

/// A coder session/thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoderThread {
    pub id: String,
    pub title: String,
    /// Absolute path the agent's relative tool paths resolve against.
    pub workspace_root: String,
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// A tool call awaiting a human decision because the current permission mode +
/// allowlist did not auto-grant it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub call_id: String,
    pub tool: String,
    /// Parsed arguments, for display.
    pub arguments: serde_json::Value,
    /// The allowlist pattern that *would* be stored if the user picks
    /// "accept and remember" (e.g. `git status` or `src/**`).
    pub suggested_rule: String,
    /// Human summary of the effect (command text, target path, ...).
    pub summary: String,
}

/// Returned by `send` / `approve`: either the run finished (final assistant
/// text present) or it paused on an approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoderRunResult {
    pub thread_id: String,
    pub messages: Vec<ChatMessage>,
    /// Set when the loop paused awaiting a decision.
    pub pending: Option<PendingApproval>,
    /// Set when the loop finished; the last assistant text.
    pub final_text: Option<String>,
    /// True if the loop stopped because it hit the iteration ceiling.
    pub exhausted: bool,
}

/// How aggressively tool calls are auto-approved for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionMode {
    /// Every mutating tool auto-runs. Read-only tools always run.
    AutoAcceptAll,
    /// Mutating tools prompt unless matched by the allowlist.
    Review,
    /// Read-only only: mutating tools are hard-rejected (dry run / planning).
    Plan,
}

impl Default for PermissionMode {
    fn default() -> Self {
        PermissionMode::Review
    }
}

/// Review status of a file change produced by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeStatus {
    /// Applied to disk, awaiting the user's accept/reject decision.
    Pending,
    /// User kept the change.
    Accepted,
    /// User reverted the change (disk restored to `before`).
    Rejected,
}

/// A single file edit made by the agent, tracked for Cursor-style review.
/// The edit is already on disk; `before` + `hunks` let the user accept/reject
/// the whole change or individual hunks (which rewrites the file on disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub id: String,
    pub thread_id: String,
    /// Path relative to the workspace root.
    pub path: String,
    /// The tool that produced it (`write_file` | `edit_file`).
    pub tool: String,
    /// Snapshot of the file before the edit (empty string if newly created).
    pub before: String,
    /// The full content originally written by the agent.
    pub original_after: String,
    /// Per-block decisions; disk content = rebuild(before, hunks).
    pub hunks: Vec<Hunk>,
    /// True when the file did not exist before this change.
    pub created: bool,
    pub status: ChangeStatus,
    pub created_at: String,
}

/// A persisted allow/deny rule for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub tool: String,
    /// Glob (for paths) or command prefix (for run_command). Empty = any.
    pub pattern: String,
    /// true = allow, false = deny.
    pub allow: bool,
}
