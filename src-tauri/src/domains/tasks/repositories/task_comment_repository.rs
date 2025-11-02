use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use crate::entities::task_comment::{Entity as TaskCommentEntity, Model as TaskCommentModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommentRequest {
    pub task_id: i32,
    pub content: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskCommentRequest {
    pub content: Option<String>,
}

pub struct TaskCommentRepository {
    db: DatabaseConnection,
}

impl TaskCommentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateTaskCommentRequest) -> Result<TaskCommentModel, sea_orm::DbErr> {
        let active_model = ActiveModel {
            task_id: Set(request.task_id),
            content: Set(request.content),
            author: Set(request.author),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    pub async fn update(&self, id: i32, request: UpdateTaskCommentRequest) -> Result<TaskCommentModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = TaskCommentEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Comment not found".to_string()))?
            .into();

        if let Some(content) = request.content {
            active_model.content = Set(content);
        }

        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        TaskCommentEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_task_id(&self, task_id: i32) -> Result<Vec<TaskCommentModel>, sea_orm::DbErr> {
        TaskCommentEntity::find()
            .filter(Column::TaskId.eq(task_id))
            .order_by_asc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<TaskCommentModel>, sea_orm::DbErr> {
        TaskCommentEntity::find_by_id(id).one(&self.db).await
    }
}
