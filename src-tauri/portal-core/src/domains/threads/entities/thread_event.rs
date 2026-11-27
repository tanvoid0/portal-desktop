use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Append-only. `payload` is the whole `ThreadEvent` JSON, `kind` its `type` tag.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "thread_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub thread_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub seq: i64,
    pub kind: String,
    pub payload: String,
    pub command_id: Option<String>,
    pub actor: String,
    pub at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
