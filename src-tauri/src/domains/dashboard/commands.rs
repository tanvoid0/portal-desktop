use std::sync::Arc;

use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::projects::services::{ProjectService, ProjectStats};
use crate::domains::sdk::commands::sdk_commands::get_running_services_count;
use crate::domains::tasks::services::task_service::{MainTaskStats, TaskService};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardOverview {
    pub project_stats: ProjectStats,
    pub task_stats: MainTaskStats,
    pub running_services_count: u32,
}

#[tauri::command]
pub async fn get_dashboard_overview(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<DashboardOverview, String> {
    let project_service = ProjectService::new(&db_manager);
    let task_service = TaskService::new(db_manager.get_connection_clone());

    let project_stats = project_service.get_project_stats().await?;
    let task_stats = task_service
        .get_main_task_stats()
        .await
        .map_err(|e| e.to_string())?;
    let running_services_count = get_running_services_count().await?;

    Ok(DashboardOverview {
        project_stats,
        task_stats,
        running_services_count,
    })
}
