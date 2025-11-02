/**
 * SDK Manager Entities
 */

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sdk_installations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub sdk_type: String,
    pub manager_type: String,
    pub version: String,
    pub path: Option<String>,
    pub active: bool,
    pub installed_at: DateTime,
    pub last_used: Option<DateTime>,
    pub project_path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
