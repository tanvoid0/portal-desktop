//! Tauri command surface for the `agents` domain.

use std::sync::Arc;

use tauri::State;

use crate::rpc::Caller;

use super::backend::StartParams;
use super::service::{
    AgentAnalytics, AgentEventSeq, AgentFeedPage, AgentRunService, AgentSessionDetail, AgentSessionSummary,
    BackendInfo,
};

// `agents` (read-only: which backends/models exist, replaying a *live* run's
// own events) is in `DEFAULT_CAPS`. `agents.exec` (spawning a process, and
// approving/denying the tool calls it makes) is RCE-equivalent and lives in
// `SENSITIVE_CAPS` instead — same split as `terminal`/`terminal.exec`.
// `agents.history` (reading back *past* runs' full persisted transcripts —
// every tool input/output, file content, command output any run ever
// touched) is sensitive for the same reason: granting it hands a remote
// device everything, not just the one run it's currently watching.

#[portal_rpc_macro::rpc(cap = "agents")]
pub async fn agents_list_backends(
    service: State<'_, Arc<AgentRunService>>,
) -> Result<Vec<BackendInfo>, String> {
    Ok(service.list_backends().await)
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_start_run(
    caller: Caller,
    service: State<'_, Arc<AgentRunService>>,
    backend_id: String,
    cwd: String,
    prompt: String,
    model: Option<String>,
    resume_session_id: Option<String>,
    task_id: Option<String>,
    command_id: Option<String>,
) -> Result<String, String> {
    let params = StartParams {
        cwd,
        model,
        prompt,
        resume_session_id,
        task_id,
        permission_mode: "manual".to_string(),
        // #546: Agents runs should see the user's real `~/.claude/settings.json`
        // and project `CLAUDE.md`, same as a `claude` CLI session by hand —
        // same as coder already does (`coder/service.rs`'s own `start` call).
        isolate_settings: false,
    };
    service.inner().start_run(&backend_id, params, &caller.actor(), command_id).await
}

/// Resumes a prior session in place (same row, same id, same backend
/// conversation) rather than starting a new one.
#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_resume_session(
    caller: Caller,
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    text: String,
    command_id: Option<String>,
) -> Result<String, String> {
    service.resume(&run_id, text, &caller.actor(), command_id).await
}

/// Deletes a persisted session and its transcript/event log. Refused while
/// the session is live or persisted as `"running"`.
#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_delete_session(
    service: State<'_, Arc<AgentRunService>>,
    session_id: String,
) -> Result<(), String> {
    service.delete_session(&session_id).await
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_send_user(
    caller: Caller,
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    text: String,
    command_id: Option<String>,
) -> Result<(), String> {
    service.send_user(&run_id, text, &caller.actor(), command_id).await?;
    Ok(())
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_decide_permission(
    caller: Caller,
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    request_id: String,
    allow: bool,
    command_id: Option<String>,
) -> Result<(), String> {
    service.decide_permission(&run_id, request_id, allow, &caller.actor(), command_id).await
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_interrupt(
    caller: Caller,
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    command_id: Option<String>,
) -> Result<(), String> {
    service.interrupt(&run_id, &caller.actor(), command_id).await
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_cancel(
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
) -> Result<(), String> {
    service.cancel(&run_id).await
}

#[portal_rpc_macro::rpc(cap = "agents")]
pub async fn agents_get_events(
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
) -> Result<Vec<AgentEventSeq>, String> {
    Ok(service.get_events(&run_id).await)
}

/// Run ids with a live backend process right now — lets the agent tab (`/ai/coder/<backend>`)
/// reattach to a run still alive on the backend after a reload (QA-127).
#[portal_rpc_macro::rpc(cap = "agents")]
pub async fn agents_list_running(
    service: State<'_, Arc<AgentRunService>>,
) -> Result<Vec<String>, String> {
    Ok(service.list_running().await)
}

/// Persisted sessions, most recent first — the Sessions list page.
#[portal_rpc_macro::rpc(cap = "agents.history")]
pub async fn agents_list_sessions(
    service: State<'_, Arc<AgentRunService>>,
) -> Result<Vec<AgentSessionSummary>, String> {
    service.list_sessions().await
}

/// Cross-session activity feed (newest first) — pages on a global cursor, see
/// `AgentFeedEvent`. Reads every run's persisted events, so same cap as the
/// session list.
#[portal_rpc_macro::rpc(cap = "agents.history")]
pub async fn agents_list_events(
    service: State<'_, Arc<AgentRunService>>,
    after_seq: Option<i64>,
    before_seq: Option<i64>,
    limit: Option<u64>,
    backend_id: Option<String>,
) -> Result<AgentFeedPage, String> {
    service
        .list_events(after_seq, before_seq, limit, backend_id)
        .await
}

/// Usage/tool/outcome aggregates over every run's events — same cap as the feed.
#[portal_rpc_macro::rpc(cap = "agents.history")]
pub async fn agents_analytics(
    service: State<'_, Arc<AgentRunService>>,
    backend_id: Option<String>,
) -> Result<AgentAnalytics, String> {
    service.analytics(backend_id).await
}

/// One session's full persisted transcript + event log, in order — the
/// Session Detail page (Conversation + Timeline tabs).
#[portal_rpc_macro::rpc(cap = "agents.history")]
pub async fn agents_get_session(
    service: State<'_, Arc<AgentRunService>>,
    session_id: String,
) -> Result<Option<AgentSessionDetail>, String> {
    service.get_session_detail(&session_id).await
}
