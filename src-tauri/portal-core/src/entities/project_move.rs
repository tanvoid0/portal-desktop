use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "project_moves")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub project_id: i32,
    pub source_path: String,
    pub dest_path: String,
    pub staging_path: String,
    pub state: String,
    pub include_regenerable: bool,
    pub skipped_dirs: String,
    pub manifest_path: Option<String>,
    pub manifest_sha256: Option<String>,
    pub manifest_version: Option<i32>,
    pub progress_path: Option<String>,
    pub file_count: i32,
    pub total_bytes: i64,
    pub files_done: i32,
    pub bytes_done: i64,
    pub resume_count: i32,
    pub unrewritten: String,
    pub error: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
