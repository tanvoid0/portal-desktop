use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "blocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub category: String,
    pub version: String,
    #[sea_orm(column_type = "Text")]
    pub parameters_json: String,
    #[sea_orm(column_type = "Text")]
    pub command: String,
    pub execution_type: String,
    #[sea_orm(column_type = "Text")]
    pub default_config_json: String,
    #[sea_orm(column_type = "Text")]
    pub tags_json: String,
    pub icon: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

