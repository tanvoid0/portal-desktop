use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Projection: one row per `ChatMessage`, `seq` = the event that appended it.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "thread_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub thread_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub idx: i64,
    pub role: String,
    pub message_json: String,
    pub seq: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
