use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "project_package_managers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub package_manager_id: i32,
    pub created_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    
    #[sea_orm(
        belongs_to = "super::package_manager::Entity",
        from = "Column::PackageManagerId",
        to = "super::package_manager::Column::Id"
    )]
    PackageManager,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::package_manager::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PackageManager.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

