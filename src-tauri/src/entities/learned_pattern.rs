use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "learned_patterns")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pattern_type: String, // 'command', 'workflow', 'config', 'framework', 'code'
    pub pattern_data: String, // JSON string containing pattern data
    pub context: Option<String>, // Project type, framework, etc.
    #[sea_orm(default_value = 1)]
    pub frequency: i32,
    pub last_used: Option<DateTimeWithTimeZone>,
    #[sea_orm(default_value = 1.0)]
    pub success_rate: f64,
    #[sea_orm(default_value = false)]
    pub is_important: bool, // User-marked important patterns are never auto-deleted
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

