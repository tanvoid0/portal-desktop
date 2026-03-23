use crate::domains::tasks::repositories::task_repository::{TaskRepository, CreateTaskRequest, UpdateTaskRequest, TaskFilters};
use crate::domains::tasks::entities::task::Model as TaskModel;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

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

    /// Lightweight stats for dashboard usage.
    /// Counts only "main tasks" (tasks without a parent), plus completion percentage.
    pub async fn get_main_task_stats(&self) -> Result<MainTaskStats, sea_orm::DbErr> {
        let total = self.repository.count_main_tasks().await?;

        let pending = self.repository.count_main_tasks_by_status("pending").await?;
        let in_progress = self.repository.count_main_tasks_by_status("in-progress").await?;
        let completed = self.repository.count_main_tasks_by_status("completed").await?;
        let cancelled = self.repository.count_main_tasks_by_status("cancelled").await?;

        let completion_percentage = if total > 0 {
            // Keep rounding consistent with the frontend (Math.round).
            ((completed as f64 / total as f64) * 100.0).round() as u64
        } else {
            0
        };

        Ok(MainTaskStats {
            total,
            pending,
            in_progress,
            completed,
            cancelled,
            completion_percentage,
        })
    }

    // New advanced methods
    pub async fn get_overdue_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_overdue().await
    }

    pub async fn get_due_today_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_due_today().await
    }

    pub async fn get_unestimated_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_unestimated().await
    }

    pub async fn get_recurring_tasks(&self) -> Result<Vec<TaskModel>, sea_orm::DbErr> {
        self.repository.find_recurring().await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainTaskStats {
    pub total: u64,
    pub pending: u64,
    pub in_progress: u64,
    pub completed: u64,
    pub cancelled: u64,
    pub completion_percentage: u64,
}
