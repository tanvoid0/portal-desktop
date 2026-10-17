use crate::database::DatabaseManager;
use crate::domains::projects::entities::ProjectAnalysis;
use crate::domains::projects::services::ProjectService;
use std::sync::Arc;
use tauri::command;

#[portal_rpc_macro::rpc]
pub async fn get_all_projects(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_all_projects().await.map_err(|e| e.to_string())
}

#[portal_rpc_macro::rpc]
pub async fn get_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_project(id).await.map_err(|e| e.to_string())
}

#[portal_rpc_macro::rpc]
pub async fn create_project(
    name: String,
    description: Option<String>,
    path: String,
    framework_ids: Option<Vec<i32>>,
    package_manager_ids: Option<Vec<i32>>,
    language_ids: Option<Vec<i32>>,
    build_command: Option<String>,
    start_command: Option<String>,
    test_command: Option<String>,
    output_directory: Option<String>,
    dev_port: Option<i32>,
    prod_port: Option<i32>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<crate::domains::projects::entities::ProjectResponse, String> {
    let service = ProjectService::new(&db_manager);
    service
        .create_project(
            name,
            description,
            path,
            framework_ids.unwrap_or_default(),
            package_manager_ids.unwrap_or_default(),
            language_ids.unwrap_or_default(),
            build_command,
            start_command,
            test_command,
            output_directory,
            dev_port,
            prod_port,
        )
        .await
}

#[portal_rpc_macro::rpc]
pub async fn update_project(
    id: i32,
    name: Option<String>,
    description: Option<String>,
    path: Option<String>,
    status: Option<String>,
    framework_ids: Option<Vec<i32>>,
    package_manager_ids: Option<Vec<i32>>,
    language_ids: Option<Vec<i32>>,
    build_command: Option<String>,
    start_command: Option<String>,
    test_command: Option<String>,
    output_directory: Option<String>,
    dev_port: Option<i32>,
    prod_port: Option<i32>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service
        .update_project(
            id,
            name,
            description,
            path,
            status,
            framework_ids,
            package_manager_ids,
            language_ids,
            build_command,
            start_command,
            test_command,
            output_directory,
            dev_port,
            prod_port,
        )
        .await
}

#[portal_rpc_macro::rpc]
pub async fn delete_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<bool, String> {
    let service = ProjectService::new(&db_manager);
    service.delete_project(id).await
}

#[portal_rpc_macro::rpc]
pub async fn toggle_project_star(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service.toggle_project_star(id).await
}

#[portal_rpc_macro::rpc]
pub async fn open_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service.open_project(id).await
}

#[portal_rpc_macro::rpc]
pub async fn refresh_project_metadata(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Option<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service.refresh_project_metadata(id).await
}

#[portal_rpc_macro::rpc]
pub async fn get_projects_with_filters(
    status_filter: Option<String>,
    sort_by: String,
    search_query: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<crate::domains::projects::entities::ProjectResponse>, String> {
    let service = ProjectService::new(&db_manager);
    service
        .get_projects_with_filters(status_filter, sort_by, search_query)
        .await
}

#[portal_rpc_macro::rpc]
pub async fn get_frameworks(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<String>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_frameworks().await
}

#[portal_rpc_macro::rpc]
pub async fn get_project_stats(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<crate::domains::projects::services::ProjectStats, String> {
    let service = ProjectService::new(&db_manager);
    service.get_project_stats().await
}

#[portal_rpc_macro::rpc]
pub async fn validate_project_path(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<bool, String> {
    let service = ProjectService::new(&db_manager);
    service
        .validate_project_path(&path)
        .await
        .map(|_| true)
        .or_else(|e| Err(e))
}

#[portal_rpc_macro::rpc]
pub async fn generate_project_name(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<String, String> {
    let service = ProjectService::new(&db_manager);
    service.generate_project_name(&path).await
}

#[portal_rpc_macro::rpc]
pub async fn detect_framework(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<String>, String> {
    let service = ProjectService::new(&db_manager);
    service.detect_frameworks(&path).await
}

#[portal_rpc_macro::rpc]
pub async fn analyze_project_directory(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<ProjectAnalysis, String> {
    let service = ProjectService::new(&db_manager);
    service.analyze_project_directory(&path).await
}

#[portal_rpc_macro::rpc]
pub async fn open_project_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[portal_rpc_macro::rpc]
pub async fn select_directory(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = mpsc::channel();

    app_handle
        .dialog()
        .file()
        .set_title("Select Project Directory")
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });

    // Wait for the result
    match rx.recv() {
        Ok(Some(path)) => Ok(Some(path.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

#[portal_rpc_macro::rpc]
pub async fn execute_command_in_directory(
    command: String,
    args: Vec<String>,
    working_directory: String,
) -> Result<String, String> {
    use crate::process_ext::NoWindowExt;
    use std::process::Command;

    let output = Command::new(&command)
        .no_window()
        .args(&args)
        .current_dir(&working_directory)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ── Modules: the apps / services / packages inside a project ─────────────

use crate::domains::projects::modules::ModuleService;
use crate::entities::project_module::Model as ProjectModule;

#[portal_rpc_macro::rpc]
pub async fn get_project_modules(
    project_id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<ProjectModule>, String> {
    ModuleService::new(&db_manager).list(project_id).await
}

#[portal_rpc_macro::rpc]
pub async fn get_all_project_modules(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<ProjectModule>, String> {
    ModuleService::new(&db_manager).list_all().await
}

/// Rescan one project's folders and return what it now holds.
#[portal_rpc_macro::rpc]
pub async fn refresh_project_modules(
    project_id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<ProjectModule>, String> {
    ModuleService::new(&db_manager).refresh(project_id).await
}

/// Rescan every project; returns all modules when done.
#[portal_rpc_macro::rpc]
pub async fn refresh_all_project_modules(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<ProjectModule>, String> {
    ModuleService::new(&db_manager).refresh_all().await
}
