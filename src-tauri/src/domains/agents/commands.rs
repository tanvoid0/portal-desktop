//! Tauri command surface for the `agents` domain.

use std::sync::Arc;

use tauri::State;

use super::backend::StartParams;
use super::events::AgentEvent;
use super::service::{AgentRunService, BackendInfo};

// `agents` (read-only: which backends/models exist, replaying a run's own
// events) is in `DEFAULT_CAPS`. `agents.exec` (spawning a process, and
// approving/denying the tool calls it makes) is RCE-equivalent and lives in
// `SENSITIVE_CAPS` instead — same split as `terminal`/`terminal.exec`.

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
) -> Result<String, String> {
    let params = StartParams {
        cwd,
        model,
        prompt,
        resume_session_id,
        permission_mode: "manual".to_string(),
        isolate_settings: true,
    };
    service.inner().start_run(&backend_id, params).await
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
) -> Result<Vec<AgentEvent>, String> {
    Ok(service.get_events(&run_id).await)
}
