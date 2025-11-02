use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use crate::entities::task_template::{Entity as TaskTemplateEntity, Model as TaskTemplateModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub default_status: String,
    pub default_priority: String,
    pub default_type: Option<String>,
    pub default_tags: Option<String>, // JSON array of strings
    pub default_estimated_time: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_status: Option<String>,
    pub default_priority: Option<String>,
    pub default_type: Option<String>,
    pub default_tags: Option<String>,
    pub default_estimated_time: Option<i32>,
}

pub struct TaskTemplateRepository {
    db: DatabaseConnection,
}

impl TaskTemplateRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateTaskTemplateRequest) -> Result<TaskTemplateModel, sea_orm::DbErr> {
        let active_model = ActiveModel {
            name: Set(request.name),
            description: Set(request.description),
            default_status: Set(request.default_status),
            default_priority: Set(request.default_priority),
            default_type: Set(request.default_type),
            default_tags: Set(request.default_tags),
            default_estimated_time: Set(request.default_estimated_time),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    pub async fn update(&self, id: i32, request: UpdateTaskTemplateRequest) -> Result<TaskTemplateModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = TaskTemplateEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Template not found".to_string()))?
            .into();

        if let Some(name) = request.name {
            active_model.name = Set(name);
        }
        if let Some(description) = request.description {
            active_model.description = Set(Some(description));
        }
        if let Some(default_status) = request.default_status {
            active_model.default_status = Set(default_status);
        }
        if let Some(default_priority) = request.default_priority {
            active_model.default_priority = Set(default_priority);
        }
        if let Some(default_type) = request.default_type {
            active_model.default_type = Set(Some(default_type));
        }
        if let Some(default_tags) = request.default_tags {
            active_model.default_tags = Set(Some(default_tags));
        }
        if let Some(default_estimated_time) = request.default_estimated_time {
            active_model.default_estimated_time = Set(Some(default_estimated_time));
        }

        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        TaskTemplateEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<TaskTemplateModel>, sea_orm::DbErr> {
        TaskTemplateEntity::find()
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<TaskTemplateModel>, sea_orm::DbErr> {
        TaskTemplateEntity::find_by_id(id).one(&self.db).await
    }
}
