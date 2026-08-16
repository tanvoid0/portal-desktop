use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "terminal_command_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub tab_id: String,

    #[sea_orm(column_type = "Text")]
    pub command: String,

    #[sea_orm(column_type = "Text")]
    pub output: String,

    // Stored as RFC3339 string for easy interchange with frontend
    pub timestamp: String,

    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub intercepted: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
