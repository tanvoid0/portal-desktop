use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "framework_ide_mappings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub framework: String,
    #[sea_orm(column_name = "ide_id")]
    pub ide_id: i32,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ide::Entity",
        from = "Column::IdeId",
        to = "super::ide::Column::Id"
    )]
    Ide,
}

impl Related<super::ide::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ide.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

