use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use crate::entities::framework_ide_mapping::{Entity, Model, ActiveModel};
use crate::entities::framework_ide_mapping;

pub struct FrameworkIdeMappingRepository;

impl FrameworkIdeMappingRepository {
    /// Get all framework IDE mappings
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find().all(db).await
    }

    /// Get mapping by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get mapping by framework name
    pub async fn get_by_framework(db: &DatabaseConnection, framework: &str) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(framework_ide_mapping::Column::Framework.eq(framework))
            .one(db)
            .await
    }

    /// Create or update a framework IDE mapping
    pub async fn create_or_update(
        db: &DatabaseConnection,
        framework: String,
        ide_id: i32,
    ) -> Result<Model, sea_orm::DbErr> {
        // Check if mapping already exists
        if let Ok(Some(existing)) = Self::get_by_framework(db, &framework).await {
            // Update existing
            let mut active_model: ActiveModel = existing.into();
            active_model.ide_id = Set(ide_id);
            active_model.updated_at = Set(Some(chrono::Utc::now().into()));
            active_model.update(db).await
        } else {
            // Create new
            let active_model = ActiveModel {
                framework: Set(framework),
                ide_id: Set(ide_id),
                created_at: Set(Some(chrono::Utc::now().into())),
                updated_at: Set(Some(chrono::Utc::now().into())),
                ..Default::default()
            };
            active_model.insert(db).await
        }
    }

    /// Delete a framework IDE mapping
    pub async fn delete(db: &DatabaseConnection, framework: &str) -> Result<(), sea_orm::DbErr> {
        Entity::delete_many()
            .filter(framework_ide_mapping::Column::Framework.eq(framework))
            .exec(db)
            .await?;
        Ok(())
    }

    /// Delete a framework IDE mapping by ID
    pub async fn delete_by_id(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}

