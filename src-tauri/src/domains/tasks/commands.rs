use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::database::DatabaseManager;
use crate::domains::tasks::services::task_service::TaskService;
use crate::domains::tasks::repositories::task_repository::{CreateTaskRequest, UpdateTaskRequest, TaskFilters};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub type_: Option<String>,
    pub parent_id: Option<i32>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<crate::domains::tasks::entities::task::Model> for TaskResponse {
    fn from(model: crate::domains::tasks::entities::task::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: model.status,
            priority: model.priority,
            type_: model.type_,
            parent_id: model.parent_id,
            resource_id: model.resource_id,
            resource_type: model.resource_type,
            due_date: model.due_date.map(|dt| dt.into()),
            completed_at: model.completed_at.map(|dt| dt.into()),
            created_at: model.created_at.map(|dt| dt.into()),
            updated_at: model.updated_at.map(|dt| dt.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommand {
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
pub struct UpdateTaskCommand {
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
pub struct TaskFiltersCommand {
    pub status: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub type_: Option<Vec<String>>,
    pub parent_id: Option<i32>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
}

#[tauri::command]
pub async fn create_task(
    db_manager: State<'_, Arc<DatabaseManager>>,
    command: CreateTaskCommand,
) -> Result<TaskResponse, String> {
    // Validation
    if command.title.trim().is_empty() {
        return Err("Task title is required".to_string());
    }
    
    if command.title.len() > 200 {
        return Err("Task title must be less than 200 characters".to_string());
    }
    
    if let Some(ref description) = command.description {
        if description.len() > 1000 {
            return Err("Task description must be less than 1000 characters".to_string());
        }
    }
    
    // Validate status
    let valid_statuses = ["pending", "in-progress", "completed", "cancelled"];
    if !valid_statuses.contains(&command.status.as_str()) {
        return Err(format!("Invalid status: {}. Must be one of: {}", command.status, valid_statuses.join(", ")));
    }
    
    // Validate priority
    let valid_priorities = ["low", "medium", "high"];
    if !valid_priorities.contains(&command.priority.as_str()) {
        return Err(format!("Invalid priority: {}. Must be one of: {}", command.priority, valid_priorities.join(", ")));
    }

    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    let request = CreateTaskRequest {
        title: command.title.trim().to_string(),
        description: command.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
        status: command.status,
        priority: command.priority,
        type_: command.type_.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
        parent_id: command.parent_id,
        resource_id: command.resource_id.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()),
        resource_type: command.resource_type.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()),
        due_date: command.due_date,
    };

    task_service
        .create_task(request)
        .await
        .map(TaskResponse::from)
        .map_err(|e| {
            eprintln!("Failed to create task: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn update_task(
    db_manager: State<'_, Arc<DatabaseManager>>,
    id: i32,
    command: UpdateTaskCommand,
) -> Result<TaskResponse, String> {
    // Validation
    if let Some(ref title) = command.title {
        if title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }
        
        if title.len() > 200 {
            return Err("Task title must be less than 200 characters".to_string());
        }
    }
    
    if let Some(ref description) = command.description {
        if description.len() > 1000 {
            return Err("Task description must be less than 1000 characters".to_string());
        }
    }
    
    // Validate status if provided
    if let Some(ref status) = command.status {
        let valid_statuses = ["pending", "in-progress", "completed", "cancelled"];
        if !valid_statuses.contains(&status.as_str()) {
            return Err(format!("Invalid status: {}. Must be one of: {}", status, valid_statuses.join(", ")));
        }
    }
    
    // Validate priority if provided
    if let Some(ref priority) = command.priority {
        let valid_priorities = ["low", "medium", "high"];
        if !valid_priorities.contains(&priority.as_str()) {
            return Err(format!("Invalid priority: {}. Must be one of: {}", priority, valid_priorities.join(", ")));
        }
    }

    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    let request = UpdateTaskRequest {
        title: command.title.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
        description: command.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
        status: command.status,
        priority: command.priority,
        type_: command.type_.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
        parent_id: command.parent_id,
        resource_id: command.resource_id.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()),
        resource_type: command.resource_type.map(|r| r.trim().to_string()).filter(|r| !r.is_empty()),
        due_date: command.due_date,
    };

    task_service
        .update_task(id, request)
        .await
        .map(TaskResponse::from)
        .map_err(|e| {
            eprintln!("Failed to update task: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn delete_task(
    db_manager: State<'_, Arc<DatabaseManager>>,
    id: i32,
) -> Result<(), String> {
    if id <= 0 {
        return Err("Invalid task ID".to_string());
    }

    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    // Check if task exists before deleting
    match task_service.get_task(id).await {
        Ok(Some(_)) => {
            task_service
                .delete_task(id)
                .await
                .map_err(|e| {
                    eprintln!("Failed to delete task: {}", e);
                    e.to_string()
                })
        }
        Ok(None) => Err("Task not found".to_string()),
        Err(e) => {
            eprintln!("Failed to check if task exists: {}", e);
            Err("Failed to delete task".to_string())
        }
    }
}

#[tauri::command]
pub async fn get_task(
    db_manager: State<'_, Arc<DatabaseManager>>,
    id: i32,
) -> Result<Option<TaskResponse>, String> {
    if id <= 0 {
        return Err("Invalid task ID".to_string());
    }

    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    task_service
        .get_task(id)
        .await
        .map(|opt| opt.map(TaskResponse::from))
        .map_err(|e| {
            eprintln!("Failed to get task: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn get_tasks(
    db_manager: State<'_, Arc<DatabaseManager>>,
    filters: Option<TaskFiltersCommand>,
) -> Result<Vec<TaskResponse>, String> {
    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    let filters = filters.map(|f| TaskFilters {
        status: f.status,
        priority: f.priority,
        type_: f.type_,
        parent_id: f.parent_id,
        resource_id: f.resource_id,
        resource_type: f.resource_type,
    });

    task_service
        .get_tasks(filters)
        .await
        .map(|tasks| tasks.into_iter().map(TaskResponse::from).collect())
        .map_err(|e| {
            eprintln!("Failed to get tasks: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn get_subtasks(
    db_manager: State<'_, Arc<DatabaseManager>>,
    parent_id: i32,
) -> Result<Vec<TaskResponse>, String> {
    if parent_id <= 0 {
        return Err("Invalid parent task ID".to_string());
    }

    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    task_service
        .get_subtasks(parent_id)
        .await
        .map(|tasks| tasks.into_iter().map(TaskResponse::from).collect())
        .map_err(|e| {
            eprintln!("Failed to get subtasks: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn get_main_tasks(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<TaskResponse>, String> {
    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    task_service
        .get_main_tasks()
        .await
        .map(|tasks| tasks.into_iter().map(TaskResponse::from).collect())
        .map_err(|e| {
            eprintln!("Failed to get main tasks: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub async fn get_task_count(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<u64, String> {
    let task_service = TaskService::new(db_manager.get_connection_clone());
    
    task_service
        .get_task_count()
        .await
        .map_err(|e| {
            eprintln!("Failed to get task count: {}", e);
            e.to_string()
        })
}
