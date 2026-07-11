//! Wire + persistence types for the coder agent.

use serde::{Deserialize, Serialize, Serializer};

use super::diff::Hunk;

/// A single chat message in a thread transcript. This mirrors the OpenAI
/// chat-completions message shape so it can be sent to the platform `/v1`
/// proxy verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// `system` | `user` | `assistant` | `tool`.
    pub role: String,
    /// Text content. May be empty for an assistant turn that only issues tool
    /// calls. Serialized as `""` when absent — agent-platform rejects null.
    #[serde(default, serialize_with = "serialize_content")]
    pub content: Option<String>,
    /// Present on assistant turns that call tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Present on `tool` messages — the id of the call this result answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Message creation time (UTC RFC3339) for UI timing display.
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "created_at")]
    pub timestamp: Option<String>,
}

fn serialize_content<S>(content: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(content.as_deref().unwrap_or(""))
}

impl ChatMessage {
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
    pub fn tool_result(call_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            content: Some(text.into()),
            tool_calls: None,
            tool_call_id: Some(call_id.into()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

fn tool_calls_match(a: &Option<Vec<ToolCall>>, b: &Option<Vec<ToolCall>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a_calls), Some(b_calls)) => {
            a_calls.len() == b_calls.len()
                && a_calls.iter().zip(b_calls.iter()).all(|(x, y)| {
                    x.id == y.id
                        && x.r#type == y.r#type
                        && x.function.name == y.function.name
                        && x.function.arguments == y.function.arguments
                })
        }
        _ => false,
    }
}

fn messages_match(a: &ChatMessage, b: &ChatMessage) -> bool {
    a.content == b.content
        && a.tool_call_id == b.tool_call_id
        && tool_calls_match(&a.tool_calls, &b.tool_calls)
}

/// Preserve known timestamps and estimate missing ones for UI display.
pub fn with_message_timestamps(
    messages: Vec<ChatMessage>,
    previous: &[ChatMessage],
    anchor_iso: Option<&str>,
) -> Vec<ChatMessage> {
    let anchor_ms = anchor_iso
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis())
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
    let gap_ms: i64 = 1500;
    let count = messages.len() as i64;

    messages
        .into_iter()
        .enumerate()
        .map(|(index, mut message)| {
            if message.timestamp.is_some() {
                return message;
            }
            if let Some(prev) = previous.get(index) {
                if prev.role == message.role && messages_match(prev, &message) {
                    if let Some(ts) = &prev.timestamp {
                        message.timestamp = Some(ts.clone());
                        return message;
                    }
                }
            }
            for prev in previous {
                if prev.role == message.role && messages_match(prev, &message) {
                    if let Some(ts) = &prev.timestamp {
                        message.timestamp = Some(ts.clone());
                        return message;
                    }
                }
            }
            let estimated_ms = anchor_ms - (count - 1 - index as i64) * gap_ms;
            message.timestamp = Some(
                chrono::DateTime::from_timestamp_millis(estimated_ms)
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339(),
            );
            message
        })
        .collect()
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i32>,
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub thread_kind: CoderThreadKind,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// Lightweight thread row for the sessions sidebar (no message payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoderThreadSummary {
    pub id: String,
    pub title: String,
    pub workspace_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i32>,
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub thread_kind: CoderThreadKind,
    pub message_count: usize,
    pub is_running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CoderThreadKind {
    #[default]
    Session,
    Coordinator,
    SubAgent,
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

/// Workspace file explorer entry (relative path under thread root).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoderSubAgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssueRef {
    pub owner: String,
    pub repo: String,
    pub number: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSubAgentTask {
    pub title: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_issue: Option<GitHubIssueRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_issue_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultitaskSpawnRequest {
    pub coordinator_thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
    #[serde(default)]
    pub tasks: Vec<SpawnSubAgentTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultitaskCleanupRequest {
    pub coordinator_thread_id: String,
    #[serde(default)]
    pub sub_agent_ids: Vec<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultitaskCancelRequest {
    pub coordinator_thread_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoderSubAgent {
    pub id: String,
    pub coordinator_thread_id: String,
    pub child_thread_id: String,
    pub title: String,
    pub workspace_root: String,
    pub branch: String,
    pub status: CoderSubAgentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_issue_number: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_issue_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
