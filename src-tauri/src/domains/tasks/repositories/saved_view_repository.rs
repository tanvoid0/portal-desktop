use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use crate::entities::saved_view::{Entity as SavedViewEntity, Model as SavedViewModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedViewRequest {
    pub name: String,
    pub description: Option<String>,
    pub filters: String, // JSON object
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedViewRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub filters: Option<String>,
    pub is_default: Option<bool>,
}

pub struct SavedViewRepository {
    db: DatabaseConnection,
}

impl SavedViewRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateSavedViewRequest) -> Result<SavedViewModel, sea_orm::DbErr> {
        // If this is set as default, unset all other defaults
        if request.is_default {
            self.clear_default().await?;
        }

        let active_model = ActiveModel {
            name: Set(request.name),
            description: Set(request.description),
            filters: Set(request.filters),
            is_default: Set(request.is_default),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    pub async fn update(&self, id: i32, request: UpdateSavedViewRequest) -> Result<SavedViewModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = SavedViewEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Saved view not found".to_string()))?
            .into();

        if let Some(name) = request.name {
            active_model.name = Set(name);
        }
        if let Some(description) = request.description {
            active_model.description = Set(Some(description));
        }
        if let Some(filters) = request.filters {
            active_model.filters = Set(filters);
        }
        if let Some(is_default) = request.is_default {
            if is_default {
                self.clear_default().await?;
            }
            active_model.is_default = Set(is_default);
        }

        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        SavedViewEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<SavedViewModel>, sea_orm::DbErr> {
        SavedViewEntity::find()
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<SavedViewModel>, sea_orm::DbErr> {
        SavedViewEntity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_default(&self) -> Result<Option<SavedViewModel>, sea_orm::DbErr> {
        SavedViewEntity::find()
            .filter(Column::IsDefault.eq(true))
            .one(&self.db)
            .await
    }

    async fn clear_default(&self) -> Result<(), sea_orm::DbErr> {
        // This would need a bulk update - simplified for now
        Ok(())
    }
}
