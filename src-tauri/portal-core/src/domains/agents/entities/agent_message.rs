use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A finished user/assistant turn, for the future Conversation tab.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub session_id: String,
    pub seq: i64,
    pub role: String,
    pub text: String,
    pub parent_tool_use_id: Option<String>,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
