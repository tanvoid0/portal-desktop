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
    // New advanced fields
    pub estimated_time: Option<i32>,
    pub actual_time: Option<i32>,
    pub tags: Option<String>, // JSON array of strings
    pub assignee: Option<String>,
    pub recurring_pattern: Option<String>,
    pub recurring_interval: Option<i32>,
    pub recurring_end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurring_last_generated: Option<chrono::DateTime<chrono::Utc>>,
    pub blocked_by: Option<String>, // JSON array of task IDs
    pub blocks: Option<String>, // JSON array of task IDs
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
    // New advanced fields
    pub estimated_time: Option<i32>,
    pub actual_time: Option<i32>,
    pub tags: Option<String>, // JSON array of strings
    pub assignee: Option<String>,
    pub recurring_pattern: Option<String>,
    pub recurring_interval: Option<i32>,
    pub recurring_end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurring_last_generated: Option<chrono::DateTime<chrono::Utc>>,
    pub blocked_by: Option<String>, // JSON array of task IDs
    pub blocks: Option<String>, // JSON array of task IDs
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
            // New advanced fields
            estimated_time: Set(request.estimated_time),
            actual_time: Set(request.actual_time),
            tags: Set(request.tags),
            assignee: Set(request.assignee),
            recurring_pattern: Set(request.recurring_pattern),
            recurring_interval: Set(request.recurring_interval),
            recurring_end_date: Set(request.recurring_end_date.map(|dt| dt.into())),
            recurring_last_generated: Set(request.recurring_last_generated.map(|dt| dt.into())),
            blocked_by: Set(request.blocked_by),
            blocks: Set(request.blocks),
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
        // Handle new advanced fields
        if let Some(estimated_time) = request.estimated_time {
            active_model.estimated_time = Set(Some(estimated_time));
        }
        if let Some(actual_time) = request.actual_time {
            active_model.actual_time = Set(Some(actual_time));
        }
        if let Some(tags) = request.tags {
            active_model.tags = Set(Some(tags));
        }
        if let Some(assignee) = request.assignee {
            active_model.assignee = Set(Some(assignee));
        }
        if let Some(recurring_pattern) = request.recurring_pattern {
            active_model.recurring_pattern = Set(Some(recurring_pattern));
        }
        if let Some(recurring_interval) = request.recurring_interval {
            active_model.recurring_interval = Set(Some(recurring_interval));
        }
        if let Some(recurring_end_date) = request.recurring_end_date {
            active_model.recurring_end_date = Set(Some(recurring_end_date.into()));
        }
        if let Some(recurring_last_generated) = request.recurring_last_generated {
            active_model.recurring_last_generated = Set(Some(recurring_last_generated.into()));
        }
        if let Some(blocked_by) = request.blocked_by {
            active_model.blocked_by = Set(Some(blocked_by));
        }
        if let Some(blocks) = request.blocks {
            active_model.blocks = Set(Some(blocks));
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

    // New advanced methods
    pub async fn find_by_tags(&self, _tags: Vec<String>) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        // This would need JSON query support - simplified for now
        // TODO: Implement proper JSON tag filtering when SeaORM supports it
        TaskEntity::find()
            .filter(Column::Tags.is_not_null())
            .all(&self.db)
            .await
    }

    pub async fn find_overdue(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        let now = chrono::Utc::now();
        TaskEntity::find()
            .filter(Column::DueDate.lt(now))
            .filter(Column::Status.ne("completed"))
            .filter(Column::Status.ne("cancelled"))
            .all(&self.db)
            .await
    }

    pub async fn find_due_today(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        let today = chrono::Utc::now().date_naive();
        let start_of_day = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = today.and_hms_opt(23, 59, 59).unwrap().and_utc();
        
        TaskEntity::find()
            .filter(Column::DueDate.gte(start_of_day))
            .filter(Column::DueDate.lte(end_of_day))
            .filter(Column::Status.ne("completed"))
            .filter(Column::Status.ne("cancelled"))
            .all(&self.db)
            .await
    }

    pub async fn find_recurring(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        TaskEntity::find()
            .filter(Column::RecurringPattern.is_not_null())
            .all(&self.db)
            .await
    }

    pub async fn find_unestimated(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        TaskEntity::find()
            .filter(Column::EstimatedTime.is_null())
            .filter(Column::Status.ne("completed"))
            .filter(Column::Status.ne("cancelled"))
            .all(&self.db)
            .await
    }
}
