use crate::domains::tasks::repositories::task_repository::{TaskRepository, CreateTaskRequest, UpdateTaskRequest, TaskFilters};
use crate::domains::tasks::entities::task::Model as TaskModel;
use sea_orm::DatabaseConnection;

pub struct TaskService {
    repository: TaskRepository,
}

impl TaskService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            repository: TaskRepository::new(db),
        }
    }

    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<TaskModel, sea_orm::DbErr> {
        self.repository.create(request).await
    }

    pub async fn update_task(&self, id: i32, request: UpdateTaskRequest) -> Result<TaskModel, sea_orm::DbErr> {
        self.repository.update(id, request).await
    }

    pub async fn delete_task(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        self.repository.delete(id).await
    }

    pub async fn get_task(&self, id: i32) -> Result<Option<TaskModel>, sea_orm::DbErr> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_tasks(&self, filters: Option<TaskFilters>) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_all(filters).await
    }

    pub async fn get_subtasks(&self, parent_id: i32) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_subtasks(parent_id).await
    }

    pub async fn get_main_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_main_tasks().await
    }

    pub async fn get_task_count(&self) -> Result<u64, sea_orm::DbErr> {
        self.repository.count().await
    }
}
