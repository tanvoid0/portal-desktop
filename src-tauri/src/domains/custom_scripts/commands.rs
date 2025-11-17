use tauri::command;
use crate::domains::custom_scripts::services::CustomScriptService;
use crate::database::DatabaseManager;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptParameter {
    pub name: String,
    pub label: String,
    pub parameter_type: String, // "file", "string", "number", "boolean", "password"
    pub required: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub file_filters: Option<Vec<String>>, // For file type: e.g., ["*.ovpn", "*.txt"]
}

#[command]
pub async fn get_all_custom_scripts(
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Vec<crate::entities::custom_script::Model>, String> {
    let service = CustomScriptService::new(&db_manager);
    service.get_all_scripts().await
}

#[command]
pub async fn get_custom_script(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<Option<crate::entities::custom_script::Model>, String> {
    let service = CustomScriptService::new(&db_manager);
    service.get_script(id).await
}

#[command]
pub async fn create_custom_script(
    name: String,
    description: Option<String>,
    command: String,
    parameters_json: String,
    category: Option<String>,
    icon: Option<String>,
    requires_sudo: bool,
    is_interactive: bool,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<crate::entities::custom_script::Model, String> {
    let service = CustomScriptService::new(&db_manager);
    service.create_script(
        name,
        description,
        command,
        parameters_json,
        category,
        icon,
        requires_sudo,
        is_interactive,
    ).await
}

#[command]
pub async fn update_custom_script(
    id: i32,
    name: Option<String>,
    description: Option<String>,
    command: Option<String>,
    parameters_json: Option<String>,
    category: Option<String>,
    icon: Option<String>,
    requires_sudo: Option<bool>,
    is_interactive: Option<bool>,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<crate::entities::custom_script::Model, String> {
    let service = CustomScriptService::new(&db_manager);
    service.update_script(
        id,
        name,
        description,
        command,
        parameters_json,
        category,
        icon,
        requires_sudo,
        is_interactive,
    ).await
}

#[command]
pub async fn delete_custom_script(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<(), String> {
    let service = CustomScriptService::new(&db_manager);
    service.delete_script(id).await
}

#[command]
pub async fn record_script_run(
    id: i32,
    db_manager: tauri::State<'_, Arc<DatabaseManager>>
) -> Result<crate::entities::custom_script::Model, String> {
    let service = CustomScriptService::new(&db_manager);
    service.record_script_run(id).await
}

#[command]
pub async fn select_file(
    app_handle: tauri::AppHandle,
    title: Option<String>,
    filters: Option<Vec<(String, Vec<String>)>>, // e.g., [("Config Files", vec!["*.ovpn", "*.conf"])]
    default_path: Option<String>, // Path to open the dialog at (simplified - just for reference)
    select_folder: Option<bool>, // If true, select folder instead of file
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    let (tx, rx) = mpsc::channel();
    
    let dialog_title = title.unwrap_or_else(|| {
        if select_folder.unwrap_or(false) {
            "Select Folder".to_string()
        } else {
            "Select File".to_string()
        }
    });
    
    if select_folder.unwrap_or(false) {
        // Select folder
        let mut dialog = app_handle.dialog().file();
        dialog = dialog.set_title(&dialog_title);
        dialog.pick_folder(move |path| {
            let _ = tx.send(path);
        });
    } else {
        // Select file
        let mut dialog = app_handle.dialog().file();
        dialog = dialog.set_title(&dialog_title);
        
        if let Some(filters) = filters {
            for (name, extensions) in filters {
                // Convert Vec<String> to &[&str]
                let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                dialog = dialog.add_filter(name, &ext_refs);
            }
        }
        
        dialog.pick_file(move |path| {
            let _ = tx.send(path);
        });
    }
    
    // Wait for the result
    match rx.recv() {
        Ok(Some(path)) => Ok(Some(path.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

