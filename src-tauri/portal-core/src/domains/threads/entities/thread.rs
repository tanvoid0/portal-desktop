use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "threads")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider: String,
    pub kind: String,
    pub title: String,
    pub workspace_root: String,
    pub project_id: Option<i64>,
    pub model: Option<String>,
    pub llm_provider: Option<String>,
    /// `idle | running | waiting | error`.
    pub status: String,
    /// Folded `ThreadState` minus the messages (those are `thread_messages` rows).
    pub state_json: String,
    pub last_seq: i64,
    pub message_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
