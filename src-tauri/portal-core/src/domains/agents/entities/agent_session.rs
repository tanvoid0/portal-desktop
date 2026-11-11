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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
