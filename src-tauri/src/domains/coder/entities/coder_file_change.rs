use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Persisted agent file change. The full `FileChange` (before + hunks) is
/// stored as a JSON blob; `status`/`thread_id`/`path` are duplicated as
/// columns for querying.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "coder_file_changes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub thread_id: String,
    pub path: String,
    pub status: String,
    pub data_json: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
