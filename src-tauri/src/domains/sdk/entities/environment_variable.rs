use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sdk_environment_variables")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub sdk_type: String,
    pub name: String,
    pub value: String,
    pub scope: String, // 'global', 'project', 'session'
    pub is_exported: bool,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

