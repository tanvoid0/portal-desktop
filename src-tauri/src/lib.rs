mod database;
mod entities;
mod migrations;
mod domains;
mod command_executor;

use database::DatabaseManager;
use domains::terminal::manager::TerminalManager;
use domains::automation::services::automation_service::AutomationService;
use domains::sdk::services::navigation_service::NavigationService;
use domains::settings::services::settings_service::SettingsService;
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
    let navigation_service = NavigationService::new();
    
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
            
            // Initialize automation service
            let automation_service = AutomationService::new(
                "http://localhost:5678".to_string(),
                None, // No API key for local n8n
            );
            app.manage(std::sync::Arc::new(automation_service));
            
            // Initialize settings service
            let settings_service = SettingsService::new();
            app.manage(std::sync::Arc::new(settings_service));
            
            // Initialize IDE storage
            domains::ide::commands::init_ide_storage(app.handle());
            
            println!("[Tauri] Automation service initialized");
            println!("[Tauri] Settings service initialized");
            println!("[Tauri] IDE storage initialized");
            println!("[Tauri] Setup function completed");
            
            Ok(())
        })
        .manage(terminal_manager)
        .manage(navigation_service)
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
            // domains::terminal::get_shell_integration_hooks,
            // Command History Persistence
            domains::terminal::save_command_history,
            domains::terminal::load_command_history,
            domains::terminal::clear_command_history,
            // Session Persistence
            domains::terminal::save_terminal_session,
            domains::terminal::load_terminal_session,
            domains::terminal::list_terminal_sessions,
            domains::terminal::delete_terminal_session,
            domains::terminal::clear_all_sessions,
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
            domains::tasks::get_task_count,
            // New advanced task commands
            domains::tasks::get_overdue_tasks,
            domains::tasks::get_due_today_tasks,
            domains::tasks::get_unestimated_tasks,
            // Automation commands
            domains::automation::trigger_n8n_workflow,
            domains::automation::get_workflow_status,
            domains::automation::list_available_workflows,
            domains::automation::get_suggested_workflows,
            domains::automation::check_n8n_health,
            // Deployment commands
            domains::deployments::commands::create_deployment_command,
            domains::deployments::commands::get_deployments_command,
            domains::deployments::commands::get_deployment_command,
            domains::deployments::commands::start_deployment_command,
            domains::deployments::commands::stop_deployment_command,
            domains::deployments::commands::delete_deployment_command,
            domains::deployments::commands::update_deployment_command,
            domains::deployments::commands::get_deployment_logs_command,
            domains::deployments::commands::refresh_deployment_statuses_command,
            domains::deployments::commands::list_containers_command,
            // SDK commands (removed non-existent commands)
            domains::sdk::commands::sdk_commands::get_terminal_integration_status,
            domains::sdk::commands::sdk_commands::remove_terminal_integration,
            // SDK Navigation commands
            domains::sdk::commands::navigation_commands::get_sdk_navigation_items,
            domains::sdk::commands::navigation_commands::get_sdk_details,
            // FlyEnv-style download commands
            domains::sdk::commands::sdk_commands::fetch_available_versions,
            domains::sdk::commands::sdk_commands::download_and_install_version,
            domains::sdk::commands::sdk_commands::detect_sdk_managers,
            domains::sdk::commands::sdk_commands::get_all_available_sdks,
            domains::sdk::commands::sdk_commands::start_sdk_service,
            domains::sdk::commands::sdk_commands::stop_sdk_service,
            domains::sdk::commands::sdk_commands::get_service_status,
            domains::sdk::commands::sdk_commands::setup_project_version_file,
            // Ollama-specific commands
            domains::sdk::commands::sdk_commands::get_ollama_versions,
            domains::sdk::commands::sdk_commands::get_ollama_models,
            domains::sdk::commands::sdk_commands::install_ollama_model,
            domains::sdk::commands::sdk_commands::remove_ollama_model,
            domains::sdk::commands::sdk_commands::get_available_ollama_models,
            // Service management commands
            domains::sdk::commands::sdk_commands::start_service,
            domains::sdk::commands::sdk_commands::stop_service,
            // Ollama update commands
            domains::sdk::commands::sdk_commands::check_ollama_updates,
            domains::sdk::commands::sdk_commands::update_ollama,
            // Project management commands
            domains::sdk::commands::sdk_commands::update_project_version,
            domains::sdk::commands::sdk_commands::remove_project_version,
            domains::sdk::commands::sdk_commands::get_project_versions,
            domains::sdk::commands::sdk_commands::setup_shell_integration,
            domains::sdk::commands::sdk_commands::activate_project_environment,
            domains::sdk::commands::sdk_commands::deactivate_project_environment,
            domains::sdk::commands::sdk_commands::find_projects_with_versions,
            // Enhanced SDK commands (FlyEnv-inspired)
            domains::sdk::commands::sdk_commands::add_custom_sdk_directory,
            domains::sdk::commands::sdk_commands::remove_custom_sdk_directory,
            domains::sdk::commands::sdk_commands::get_custom_directories,
            domains::sdk::commands::sdk_commands::get_service_logs,
            domains::sdk::commands::sdk_commands::update_service_config,
            domains::sdk::commands::sdk_commands::restart_service,
            domains::sdk::commands::sdk_commands::get_service_health,
            domains::sdk::commands::sdk_commands::set_path_environment,
            domains::sdk::commands::sdk_commands::get_path_status,
            domains::sdk::commands::sdk_commands::create_alias,
            domains::sdk::commands::sdk_commands::remove_alias,
            domains::sdk::commands::sdk_commands::list_aliases,
            domains::sdk::commands::sdk_commands::detect_version_files,
            domains::sdk::commands::sdk_commands::create_version_file,
            domains::sdk::commands::sdk_commands::get_running_services_count,
            // Settings commands
            domains::settings::commands::get_settings_command,
            domains::settings::commands::save_settings_command,
            domains::settings::commands::update_settings_command,
            domains::settings::commands::reset_settings_command,
            domains::settings::commands::export_settings_command,
            domains::settings::commands::import_settings_command,
            // IDE commands
            domains::ide::commands::detect_installed_ides,
            domains::ide::commands::get_all_ides,
            domains::ide::commands::get_suggested_frameworks,
            domains::ide::commands::add_ide,
            domains::ide::commands::update_ide,
            domains::ide::commands::delete_ide,
            domains::ide::commands::set_default_ide,
            domains::ide::commands::get_default_ide,
            domains::ide::commands::get_all_framework_ide_mappings,
            domains::ide::commands::set_framework_ide_mapping,
            domains::ide::commands::get_framework_ide_mapping,
            domains::ide::commands::delete_framework_ide_mapping,
            domains::ide::commands::get_suggested_frameworks,
            domains::ide::commands::get_all_frameworks,
            domains::ide::commands::create_framework,
            domains::ide::commands::update_framework,
            domains::ide::commands::delete_framework,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
