use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "coder_sub_agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub coordinator_thread_id: String,
    pub child_thread_id: String,
    pub title: String,
    pub workspace_root: String,
    pub branch: String,
    pub status: String,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub github_issue_number: Option<i64>,
    pub github_issue_url: Option<String>,
    pub result_summary: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
