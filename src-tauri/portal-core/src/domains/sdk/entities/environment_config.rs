use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sdk_environment_configs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sdk_type: String,
    pub path_managed_by: String, // 'app', 'system', 'none'
    pub last_updated: DateTimeWithTimeZone,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
