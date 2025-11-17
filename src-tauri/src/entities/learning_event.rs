use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "learning_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub event_type: String, // 'command_executed', 'project_created', 'suggestion_accepted', etc.
    pub event_data: String, // JSON string containing event data
    pub outcome: Option<String>, // 'success', 'failure', 'ignored'
    pub context: Option<String>, // Project context, framework, etc.
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

