use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use crate::entities::framework::{Entity, Model, ActiveModel, Column};

pub struct FrameworkRepository;

impl FrameworkRepository {
    /// Get all frameworks
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find().all(db).await
    }

    /// Get framework by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get framework by name
    pub async fn get_by_name(db: &DatabaseConnection, name: &str) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(Column::Name.eq(name))
            .one(db)
            .await
    }

    /// Create a new framework
    pub async fn create(
        db: &DatabaseConnection,
        name: String,
        icon: String,
        icon_type: String,
        category: String,
    ) -> Result<Model, sea_orm::DbErr> {
        let active_model = ActiveModel {
            name: Set(name),
            icon: Set(icon),
            icon_type: Set(icon_type),
            category: Set(category),
            created_at: Set(Some(chrono::Utc::now().into())),
            updated_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    /// Update a framework
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        name: Option<String>,
        icon: Option<String>,
        icon_type: Option<String>,
        category: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut active_model: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("Framework with id {} not found", id)))?
            .into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        if let Some(icon) = icon {
            active_model.icon = Set(icon);
        }
        if let Some(icon_type) = icon_type {
            active_model.icon_type = Set(icon_type);
        }
        if let Some(category) = category {
            active_model.category = Set(category);
        }
        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(db).await
    }

    /// Delete a framework
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}

