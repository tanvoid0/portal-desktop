use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use crate::entities::task_attachment::{Entity as TaskAttachmentEntity, Model as TaskAttachmentModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskAttachmentRequest {
    pub task_id: i32,
    pub name: String,
    pub url: String,
    pub type_: String,
    pub size: i64,
}

pub struct TaskAttachmentRepository {
    db: DatabaseConnection,
}

impl TaskAttachmentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateTaskAttachmentRequest) -> Result<TaskAttachmentModel, sea_orm::DbErr> {
        let active_model = ActiveModel {
            task_id: Set(request.task_id),
            name: Set(request.name),
            url: Set(request.url),
            type_: Set(request.type_),
            size: Set(request.size),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        TaskAttachmentEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_task_id(&self, task_id: i32) -> Result<Vec<TaskAttachmentModel>, sea_orm::DbErr> {
        TaskAttachmentEntity::find()
            .filter(Column::TaskId.eq(task_id))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<TaskAttachmentModel>, sea_orm::DbErr> {
        TaskAttachmentEntity::find_by_id(id).one(&self.db).await
    }
}
