use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub status: String,
    pub framework: Option<String>,
    pub package_manager: Option<String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub test_command: Option<String>,
    pub output_directory: Option<String>,
    pub dev_port: Option<i32>,
    pub prod_port: Option<i32>,
    pub starred: bool,
    pub open_count: i32,
    pub last_opened: Option<DateTimeWithTimeZone>,
    pub size: i64,
    pub file_count: i32,
    pub git_repository: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<DateTimeWithTimeZone>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
