use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "pipeline_executions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub pipeline_id: Option<i32>,
    pub project_id: Option<i32>,
    pub status: String,
    pub started_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
    pub triggered_by: String,
    #[sea_orm(column_type = "Text")]
    pub step_executions_json: String,
    #[sea_orm(column_type = "Text")]
    pub variables_json: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub error: Option<String>,
    /// "pipeline" or "action" (a one-step run of a command, see `m20261119_000051_unify_runs`).
    pub kind: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub command: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub working_directory: Option<String>,
    pub block_id: Option<String>,
    pub workflow_run_id: Option<String>,
    pub workflow_id: Option<String>,
    pub step_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub workflow_plan_json: Option<String>,
    pub pid: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pipeline::Entity",
        from = "Column::PipelineId",
        to = "super::pipeline::Column::Id"
    )]
    Pipeline,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
}

impl ActiveModelBehavior for ActiveModel {}
