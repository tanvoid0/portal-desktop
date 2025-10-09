use tauri::command;
use crate::domains::projects::services::ProjectService;
use crate::domains::projects::entities::ProjectAnalysis;
use crate::database::DatabaseManager;
use std::sync::Arc;

#[command]
pub async fn get_all_projects(db_manager: tauri::State<'_, Arc<DatabaseManager>>) -> Result<Vec<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_all_projects().await
}

#[command]
pub async fn get_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_project(id).await
}

#[command]
pub async fn create_project(
    name: String,
    description: Option<String>,
    path: String,
    framework: Option<String>,
    package_manager: Option<String>,
    build_command: Option<String>,
    start_command: Option<String>,
    test_command: Option<String>,
    output_directory: Option<String>,
    dev_port: Option<i32>,
    prod_port: Option<i32>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<crate::database::ProjectModel, String> {
    let service = ProjectService::new(&db_manager);
    service.create_project(
        name,
        description,
        path,
        framework,
        package_manager,
        build_command,
        start_command,
        test_command,
        output_directory,
        dev_port,
        prod_port,
    ).await
}

#[command]
pub async fn update_project(
    id: i32,
    name: Option<String>,
    description: Option<String>,
    path: Option<String>,
    status: Option<String>,
    framework: Option<String>,
    package_manager: Option<String>,
    build_command: Option<String>,
    start_command: Option<String>,
    test_command: Option<String>,
    output_directory: Option<String>,
    dev_port: Option<i32>,
    prod_port: Option<i32>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.update_project(
        id,
        name,
        description,
        path,
        status,
        framework,
        package_manager,
        build_command,
        start_command,
        test_command,
        output_directory,
        dev_port,
        prod_port,
    ).await
}

#[command]
pub async fn delete_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<bool, String> {
    let service = ProjectService::new(&db_manager);
    service.delete_project(id).await
}

#[command]
pub async fn toggle_project_star(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.toggle_project_star(id).await
}

#[command]
pub async fn open_project(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.open_project(id).await
}

#[command]
pub async fn refresh_project_metadata(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.refresh_project_metadata(id).await
}

#[command]
pub async fn get_projects_with_filters(
    status_filter: Option<String>,
    sort_by: String,
    search_query: Option<String>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Vec<crate::database::ProjectModel>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_projects_with_filters(status_filter, sort_by, search_query).await
}


#[command]
pub async fn get_frameworks(db_manager: tauri::State<'_, Arc<DatabaseManager>>) -> Result<Vec<String>, String> {
    let service = ProjectService::new(&db_manager);
    service.get_frameworks().await
}

#[command]
pub async fn get_project_stats(db_manager: tauri::State<'_, Arc<DatabaseManager>>) -> Result<crate::domains::projects::services::ProjectStats, String> {
    let service = ProjectService::new(&db_manager);
    service.get_project_stats().await
}

#[command]
pub async fn validate_project_path(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<bool, String> {
    let service = ProjectService::new(&db_manager);
    service.validate_project_path(&path).await.map(|_| true).or_else(|e| Err(e))
}

#[command]
pub async fn generate_project_name(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<String, String> {
    let service = ProjectService::new(&db_manager);
    service.generate_project_name(&path).await
}

#[command]
pub async fn detect_framework(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<String>, String> {
    let service = ProjectService::new(&db_manager);
    service.detect_framework(&path).await
}

#[command]
pub async fn analyze_project_directory(
    path: String,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<ProjectAnalysis, String> {
    let service = ProjectService::new(&db_manager);
    service.analyze_project_directory(&path).await
}

#[command]
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

#[command]
pub async fn select_directory(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    
    app_handle.dialog()
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

#[command]
pub async fn execute_command_in_directory(
    command: String,
    args: Vec<String>,
    working_directory: String
) -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new(&command)
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
