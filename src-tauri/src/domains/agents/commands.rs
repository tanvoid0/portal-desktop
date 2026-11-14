//! Tauri command surface for the `agents` domain.

use std::sync::Arc;

use tauri::State;

use super::backend::StartParams;
use super::service::{
    AgentEventSeq, AgentRunService, AgentSessionDetail, AgentSessionSummary, BackendInfo,
};

// `agents` (read-only: which backends/models exist, replaying a *live* run's
// own events) is in `DEFAULT_CAPS`. `agents.exec` (spawning a process, and
// approving/denying the tool calls it makes) is RCE-equivalent and lives in
// `SENSITIVE_CAPS` instead — same split as `terminal`/`terminal.exec`.
// `agents.history` (reading back *past* runs' full persisted transcripts —
// every tool input/output, file content, command output any run ever
// touched) is sensitive for the same reason: granting it hands a remote
// device everything, not just the one run it's currently watching.
//
// `agents.exec` also gates the coder domain's seven commands that can drive
// its claude-cli engine (`domains::coder::commands::require_engine`) — same
// RCE-equivalent shape, checked per-call via `Caller` on top of the plain
// `coder` cap those commands already carry.

#[portal_rpc_macro::rpc(cap = "agents")]
pub async fn agents_list_backends(
    service: State<'_, Arc<AgentRunService>>,
) -> Result<Vec<BackendInfo>, String> {
    Ok(service.list_backends().await)
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_start_run(
    service: State<'_, Arc<AgentRunService>>,
    backend_id: String,
    cwd: String,
    prompt: String,
    model: Option<String>,
    resume_session_id: Option<String>,
    task_id: Option<String>,
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
    service.inner().start_run(&backend_id, params).await
}

/// Resumes a prior session in place (same row, same id, same backend
/// conversation) rather than starting a new one.
#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_resume_session(
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    text: String,
) -> Result<String, String> {
    service.inner().resume_session(&run_id, text).await
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
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    text: String,
) -> Result<(), String> {
    service.send_user(&run_id, text).await
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_decide_permission(
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
    request_id: String,
    allow: bool,
) -> Result<(), String> {
    service.decide_permission(&run_id, request_id, allow).await
}

#[portal_rpc_macro::rpc(cap = "agents.exec")]
pub async fn agents_interrupt(
    service: State<'_, Arc<AgentRunService>>,
    run_id: String,
) -> Result<(), String> {
    service.interrupt(&run_id).await
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

/// Run ids with a live backend process right now — lets `/agents/run`
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

/// One session's full persisted transcript + event log, in order — the
/// Session Detail page (Conversation + Timeline tabs).
#[portal_rpc_macro::rpc(cap = "agents.history")]
pub async fn agents_get_session(
    service: State<'_, Arc<AgentRunService>>,
    session_id: String,
) -> Result<Option<AgentSessionDetail>, String> {
    service.get_session_detail(&session_id).await
}
