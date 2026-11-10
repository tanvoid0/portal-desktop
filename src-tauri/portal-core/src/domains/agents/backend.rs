//! The seam a new agent CLI plugs into: implement [`AgentBackend`] (how to
//! start it) and [`RunControl`] (how to talk to it once running). Everything
//! above this — `AgentRunService`, the Tauri commands, the frontend — only
//! ever sees these two traits plus [`super::events::AgentEvent`], so adding
//! Codex or Cursor later is "write an adapter module", not a rearchitecture.

use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use ts_rs::TS;

use super::events::AgentEvent;

/// What a backend can run with, for the launcher's model picker. Empty when
/// the backend has no enumerable list (the CLI just takes whatever default).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/agents/generated/")]
pub struct BackendModel {
    pub id: String,
    pub label: String,
}

/// Inputs for starting one run.
#[derive(Debug, Clone, Default)]
pub struct StartParams {
    pub cwd: String,
    pub model: Option<String>,
    pub prompt: String,
    /// Resume a prior backend session (e.g. Claude's `--resume <id>`).
    pub resume_session_id: Option<String>,
}

#[async_trait]
pub trait AgentBackend: Send + Sync {
    /// Stable id used in commands/persistence, e.g. `"claude-code"`.
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    /// Is the backend's CLI reachable on this machine right now?
    async fn detect(&self) -> bool;
    async fn list_models(&self) -> Vec<BackendModel>;
    /// Start a run. Normalized events stream out on `events_tx` as they
    /// happen — the caller (the run service) owns fanning them out and
    /// buffering them; the backend only ever produces [`AgentEvent`]s.
    async fn start(
        &self,
        params: StartParams,
        events_tx: UnboundedSender<AgentEvent>,
    ) -> Result<Box<dyn RunControl>, String>;
}

/// Live handle on one running backend process.
#[async_trait]
pub trait RunControl: Send + Sync {
    async fn send_user(&self, text: String) -> Result<(), String>;
    /// Resolve a `PermissionRequest` event this run emitted.
    async fn decide_permission(&self, request_id: String, allow: bool) -> Result<(), String>;
    /// Abort the in-flight turn without ending the session.
    async fn interrupt(&self) -> Result<(), String>;
    /// End the run outright.
    async fn kill(&self) -> Result<(), String>;
}
