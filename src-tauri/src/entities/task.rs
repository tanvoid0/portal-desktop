use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String, // pending, in-progress, completed, cancelled
    pub priority: String, // low, medium, high
    pub type_: Option<String>, // Story, Bug, Note, etc.
    pub parent_id: Option<i32>, // For subtasks
    pub resource_id: Option<String>, // Link to other resources
    pub resource_type: Option<String>, // Type of linked resource
    pub due_date: Option<DateTimeWithTimeZone>,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::ParentId",
        to = "super::task::Column::Id"
    )]
    Parent,
    
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}


impl ActiveModelBehavior for ActiveModel {}
