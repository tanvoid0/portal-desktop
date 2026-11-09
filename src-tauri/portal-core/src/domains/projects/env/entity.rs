/**
 * Project Env Links Entity
 *
 * One row per `(project_id, env_key, file_name)` linked to a credential, with
 * the snapshots (`file_value_hash`, `credential_updated_at`) taken at link time
 * that `service::drift_state` compares against live state. `file_name` pins
 * the snapshot to the one `.env*` file it was taken from — the same key can
 * be defined in more than one file with a different value.
 */
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "project_env_links")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub project_id: i32,
    pub env_key: String,
    pub credential_id: String,
    pub file_name: String,
    pub file_value_hash: Option<String>,
    pub credential_updated_at: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
