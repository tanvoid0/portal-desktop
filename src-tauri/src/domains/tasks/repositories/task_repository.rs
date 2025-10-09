use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait};
use serde::{Deserialize, Serialize};
use crate::domains::tasks::entities::task::{Entity as TaskEntity, Model as TaskModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub type_: Option<String>,
    pub parent_id: Option<i32>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub type_: Option<String>,
    pub parent_id: Option<i32>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub status: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub type_: Option<Vec<String>>,
    pub parent_id: Option<i32>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
}

pub struct TaskRepository {
    db: DatabaseConnection,
}

impl TaskRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateTaskRequest) -> Result<TaskModel, sea_orm::DbErr> {
        let active_model = ActiveModel {
            title: Set(request.title),
            description: Set(request.description),
            status: Set(request.status),
            priority: Set(request.priority),
            type_: Set(request.type_),
            parent_id: Set(request.parent_id),
            resource_id: Set(request.resource_id),
            resource_type: Set(request.resource_type),
            due_date: Set(request.due_date.map(|dt| dt.into())),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    pub async fn update(&self, id: i32, request: UpdateTaskRequest) -> Result<TaskModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = TaskEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Task not found".to_string()))?
            .into();

        if let Some(title) = request.title {
            active_model.title = Set(title);
        }
        if let Some(description) = request.description {
            active_model.description = Set(Some(description));
        }
        // Set completed_at if status is completed (check before moving the value)
        let is_completed = if let Some(ref status) = request.status {
            status == "completed"
        } else {
            false
        };

        if let Some(status) = request.status {
            active_model.status = Set(status);
        }
        if let Some(priority) = request.priority {
            active_model.priority = Set(priority);
        }
        if let Some(type_) = request.type_ {
            active_model.type_ = Set(Some(type_));
        }
        if let Some(parent_id) = request.parent_id {
            active_model.parent_id = Set(Some(parent_id));
        }
        if let Some(resource_id) = request.resource_id {
            active_model.resource_id = Set(Some(resource_id));
        }
        if let Some(resource_type) = request.resource_type {
            active_model.resource_type = Set(Some(resource_type));
        }
        if let Some(due_date) = request.due_date {
            active_model.due_date = Set(Some(due_date.into()));
        }

        // Set completed_at if status is completed
        if is_completed {
            active_model.completed_at = Set(Some(chrono::Utc::now().into()));
        }

        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        TaskEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<TaskModel>, sea_orm::DbErr> {
        TaskEntity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_all(&self, filters: Option<TaskFilters>) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        let mut query = TaskEntity::find();

        if let Some(filters) = filters {
            if let Some(status) = filters.status {
                query = query.filter(Column::Status.is_in(status));
            }
            if let Some(priority) = filters.priority {
                query = query.filter(Column::Priority.is_in(priority));
            }
            if let Some(type_) = filters.type_ {
                query = query.filter(Column::Type.is_in(type_));
            }
            if let Some(parent_id) = filters.parent_id {
                query = query.filter(Column::ParentId.eq(parent_id));
            }
            if let Some(resource_id) = filters.resource_id {
                query = query.filter(Column::ResourceId.eq(resource_id));
            }
            if let Some(resource_type) = filters.resource_type {
                query = query.filter(Column::ResourceType.eq(resource_type));
            }
        }

        query.order_by_desc(Column::CreatedAt).all(&self.db).await
    }

    pub async fn find_subtasks(&self, parent_id: i32) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        TaskEntity::find()
            .filter(Column::ParentId.eq(parent_id))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_main_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        TaskEntity::find()
            .filter(Column::ParentId.is_null())
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn count(&self) -> Result<u64, sea_orm::DbErr> {
        TaskEntity::find().count(&self.db).await
    }
}
