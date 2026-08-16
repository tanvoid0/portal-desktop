use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "script_executions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub block_id: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub command: String,
    #[sea_orm(column_type = "Text")]
    pub parameters_json: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub working_directory: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub pid: Option<i32>,
    #[sea_orm(column_type = "Text")]
    pub output: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub error: Option<String>,
    pub started_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
    pub triggered_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::block::Entity",
        from = "Column::BlockId",
        to = "super::block::Column::Id"
    )]
    Block,
}

impl ActiveModelBehavior for ActiveModel {}
