mod database;
mod entities;
mod migrations;
mod domains;

use database::DatabaseManager;
use domains::terminal::manager::TerminalManager;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize domain managers
    let terminal_manager = TerminalManager::new();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            println!("[Tauri] Starting setup function...");
            
            // Initialize database manager asynchronously in setup
            let app_handle = app.handle().clone();
            
            println!("[Tauri] Initializing database manager...");
            
            // Use block_on to make the async initialization synchronous
            let db_manager = tauri::async_runtime::block_on(async {
                DatabaseManager::new(&app_handle)
                    .await
                    .expect("Failed to initialize database manager")
            });
            
            println!("[Tauri] Database manager initialized, managing state...");
            
            // Manage the database manager wrapped in Arc
            app.manage(std::sync::Arc::new(db_manager));
            
            println!("[Tauri] Database manager state managed successfully");
            println!("[Tauri] Setup function completed");
            
            Ok(())
        })
        .manage(terminal_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Terminal commands
            domains::terminal::create_terminal_process,
            domains::terminal::send_terminal_input,
            domains::terminal::execute_command,
            domains::terminal::kill_terminal_process,
            domains::terminal::get_terminal_processes,
            domains::terminal::get_terminal_process,
            domains::terminal::get_process_exit_code,
            domains::terminal::resize_terminal,
            domains::terminal::add_command_interceptor,
            domains::terminal::remove_command_interceptor,
            domains::terminal::add_output_parser,
            domains::terminal::remove_output_parser,
            domains::terminal::get_system_info,
            domains::terminal::get_shell_integration_hooks,
            // Command History Persistence
            domains::terminal::save_command_history,
            domains::terminal::load_command_history,
            domains::terminal::clear_command_history,
            // Project commands
            domains::projects::get_all_projects,
            domains::projects::get_project,
            domains::projects::create_project,
            domains::projects::update_project,
            domains::projects::delete_project,
            domains::projects::toggle_project_star,
            domains::projects::open_project,
            domains::projects::refresh_project_metadata,
            domains::projects::get_projects_with_filters,
            domains::projects::get_frameworks,
            domains::projects::get_project_stats,
            domains::projects::validate_project_path,
            domains::projects::generate_project_name,
            domains::projects::detect_framework,
            domains::projects::analyze_project_directory,
            domains::projects::open_project_in_explorer,
            domains::projects::select_directory,
            domains::projects::execute_command_in_directory,
            // Task commands
            domains::tasks::create_task,
            domains::tasks::update_task,
            domains::tasks::delete_task,
            domains::tasks::get_task,
            domains::tasks::get_tasks,
            domains::tasks::get_subtasks,
            domains::tasks::get_main_tasks,
            domains::tasks::get_task_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
