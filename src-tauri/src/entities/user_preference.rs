use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_preferences")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub preference_type: String, // 'sdk_version', 'ide_config', 'terminal_setup', etc.
    pub context: Option<String>, // Project type or 'global'
    pub preference_value: String, // JSON string containing preference value
    #[sea_orm(default_value = 0.5)]
    pub confidence: f64,
    pub learned_from: Option<String>, // How this preference was learned
    #[sea_orm(default_value = false)]
    pub is_important: bool, // User-marked important preferences are never auto-deleted
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

