use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Raw event log, in order, for the future Timeline tab. The full
/// `AgentEvent` is stored as JSON (same approach as coder's
/// `coder_file_changes.data_json`).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub session_id: String,
    pub seq: i64,
    pub event_json: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
