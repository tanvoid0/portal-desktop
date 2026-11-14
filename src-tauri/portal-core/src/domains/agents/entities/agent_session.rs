use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// One row per "Start run" launch.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub backend_id: String,
    pub model: Option<String>,
    pub cwd: String,
    /// `"running" | "exited" | "cancelled"`.
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub exit_code: Option<i32>,
    /// First line of the launching prompt, truncated — the sidebar's label.
    pub title: Option<String>,
    /// The task that launched this run, if any (IDE-shaped session view).
    pub task_id: Option<String>,
    /// The backend CLI's own session id (e.g. Claude Code's `--resume` id),
    /// set once the backend reports `SessionStarted`. Required to
    /// resume-in-place — see `AgentRunService::resume_session`.
    pub backend_session_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
