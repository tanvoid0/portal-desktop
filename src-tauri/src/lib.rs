mod command_executor;
mod database;
mod domains;
mod entities;
mod error;
mod invoke_handler;
mod migrations;
mod process_ext;
mod utils;

// Re-export error types for use throughout the codebase
pub use error::{AppError, AppResult};

use database::DatabaseManager;
use domains::ai::services::AIService;
use domains::ai::services::AISettingsService;
use domains::automation::services::automation_service::AutomationService;
use domains::deployments::services::deployment_service::DeploymentService;
use domains::kubernetes::manager::KubernetesManager;
use domains::projects::pipelines::services::{ExecutionService, PipelineService};
use domains::scripts::commands::ScriptExecutionState;
use domains::sdk::services::navigation_service::NavigationService;
use domains::settings::services::settings_service::SettingsService;
use domains::terminal::manager::TerminalManager;
use std::sync::Arc;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    utils::logger::init_logger(None);
    log_info!("Tauri", "Application starting...");

    // Initialize domain managers
    let terminal_manager = TerminalManager::new();
    let kubernetes_manager = tokio::sync::Mutex::new(KubernetesManager::new());
    let navigation_service = NavigationService::new();

    // Configure updater plugin
    // Note: Endpoints and pubkey can be configured via environment variables or builder
    // For now, using default configuration - endpoints and pubkey should be set via
    // TAURI_UPDATER_ENDPOINTS and TAURI_UPDATER_PUBKEY environment variables,
    // or configure in tauri.conf.json under plugins section if supported
    let updater_builder = tauri_plugin_updater::Builder::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(updater_builder.build())
        .setup(|app| {
            // Set app handle for logger to emit events to frontend
            utils::logger::set_app_handle(app.handle().clone());

            log_info!("Tauri", "Starting setup function...");

            log_info!("Tauri", "Initializing database manager...");

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

            let db_manager =
                tauri::async_runtime::block_on(async { DatabaseManager::new(app_data_dir).await })
                    .map_err(|e| format!("Failed to initialize database manager: {}", e))?;

            log_info!("Tauri", "Database manager initialized, managing state...");

            // Wrap in Arc for sharing
            let db_manager_arc = std::sync::Arc::new(db_manager);

            // Manage the database manager wrapped in Arc
            app.manage(db_manager_arc.clone());

            // Initialize automation service
            let automation_service = AutomationService::new(
                "http://localhost:5678".to_string(),
                None, // No API key for local n8n
            );
            app.manage(std::sync::Arc::new(automation_service));

            // Initialize settings service
            let settings_service = SettingsService::new();
            app.manage(std::sync::Arc::new(settings_service));

            // Initialize AI services
            let ai_settings_service = AISettingsService::new();
            let ai_service = AIService::new();

            // Load AI provider configurations and register them
            let settings = ai_settings_service.load_settings().unwrap_or_default();
            for (_, config) in &settings.providers {
                if config.enabled {
                    let _ = tauri::async_runtime::block_on(
                        ai_service.register_provider_from_config(config.clone()),
                    );
                }
            }

            // Set default provider if configured
            if let Ok(Some(default_type)) = ai_settings_service.get_default_provider() {
                let _ =
                    tauri::async_runtime::block_on(ai_service.set_default_provider(default_type));
            }

            app.manage(std::sync::Arc::new(ai_settings_service));
            app.manage(std::sync::Arc::new(ai_service));

            // Initialize disk-cleanup domain (ported from portal_disk_utility).
            // Own rusqlite DB alongside the main app data; state for scan/verify
            // cancellation flags. See docs/development/DISK_UTILITY_MIGRATION.md.
            let disk_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
            let disk_db = domains::disk::db::Db::open(disk_data_dir.join("disk_utility.db"))
                .map_err(|e| format!("Failed to initialize disk utility database: {}", e))?;
            app.manage(std::sync::Arc::new(disk_db));
            app.manage(domains::disk::commands::ScanControl::default());
            app.manage(domains::disk::commands::VerifyControl::default());
            log_info!("Tauri", "Disk utility domain initialized");

            // Initialize IDE storage
            domains::ide::commands::init_ide_storage(app.handle());

            // Initialize deployment service
            let deployment_service = tauri::async_runtime::block_on(async {
                DeploymentService::new(db_manager_arc.clone()).await
            })
            .map_err(|e| format!("Failed to initialize deployment service: {}", e))?;
            app.manage(std::sync::Arc::new(deployment_service));

            // Initialize coder agent service (loads threads + rules from DB)
            let coder_service = tauri::async_runtime::block_on(async {
                domains::coder::CoderService::new(db_manager_arc.clone()).await
            });
            coder_service.init_app_handle(app.handle().clone());
            app.manage(std::sync::Arc::new(coder_service));

            // Initialize pipeline services
            let pipeline_service = PipelineService::new(db_manager_arc.clone());
            let execution_service = ExecutionService::new(db_manager_arc.clone());
            app.manage(Arc::new(pipeline_service));
            app.manage(Arc::new(execution_service));

            // Initialize script execution state
            let script_execution_state = ScriptExecutionState::new();
            app.manage(script_execution_state);

            // Sync any running script executions from previous session
            let db_for_sync = db_manager_arc.clone();
            tauri::async_runtime::spawn(async move {
                let service = domains::scripts::ScriptExecutionService::new(db_for_sync);
                if let Err(e) = service.sync_running_executions().await {
                    log_warn!("Scripts", "Failed to sync script executions: {}", e);
                }
            });

            log_info!("Tauri", "Automation service initialized");
            log_info!("Tauri", "Settings service initialized");
            log_info!("Tauri", "AI services initialized");
            log_info!("Tauri", "IDE storage initialized");
            log_info!("Tauri", "Deployment service initialized");
            log_info!("Tauri", "Pipeline services initialized");
            log_info!("Tauri", "Setup function completed");

            Ok(())
        })
        .manage(terminal_manager)
        .manage(kubernetes_manager)
        .manage(navigation_service)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Coder agent commands
            domains::coder::coder_create_thread,
            domains::coder::coder_list_threads,
            domains::coder::coder_list_thread_summaries,
            domains::coder::coder_get_thread,
            domains::coder::coder_delete_thread,
            domains::coder::coder_update_thread_model,
            domains::coder::coder_set_thread_kind,
            domains::coder::coder_send,
            domains::coder::coder_retry,
            domains::coder::coder_edit_message,
            domains::coder::coder_approve,
            domains::coder::coder_get_mode,
            domains::coder::coder_set_mode,
            domains::coder::coder_get_permission_mode,
            domains::coder::coder_set_permission_mode,
            domains::coder::coder_list_rules,
            domains::coder::coder_add_rule,
            domains::coder::coder_remove_rule,
            domains::coder::coder_list_changes,
            domains::coder::coder_accept_change,
            domains::coder::coder_reject_change,
            domains::coder::coder_set_hunk,
            domains::coder::coder_modify_change,
            domains::coder::coder_stop,
            domains::coder::coder_list_running,
            domains::coder::coder_get_context_usage,
            domains::coder::coder_get_git_diff_stats,
            domains::coder::coder_list_git_changes,
            domains::coder::coder_prepare_git_commit,
            domains::coder::coder_git_commit,
            domains::coder::coder_list_dir,
            domains::coder::coder_read_file,
            domains::coder::coder_write_file,
            domains::coder::coder_open_in_explorer,
            domains::coder::coder_submit_command_result,
            domains::coder::coder_submit_terminal_list,
            domains::coder::coder_multitask_spawn,
            domains::coder::coder_multitask_list,
            domains::coder::coder_multitask_cancel,
            domains::coder::coder_multitask_cleanup,
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
            // Terminal Notes Persistence
            domains::terminal::save_terminal_note,
            domains::terminal::load_terminal_note,
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
            domains::dashboard::commands::get_dashboard_overview,
            domains::projects::validate_project_path,
            domains::projects::generate_project_name,
            domains::projects::detect_framework,
            domains::projects::analyze_project_directory,
            domains::projects::open_project_in_explorer,
            domains::projects::select_directory,
            domains::projects::execute_command_in_directory,
            // Pipeline commands
            domains::projects::pipelines::create_pipeline,
            domains::projects::pipelines::get_pipeline,
            domains::projects::pipelines::get_pipelines,
            domains::projects::pipelines::update_pipeline,
            domains::projects::pipelines::delete_pipeline,
            domains::projects::pipelines::execute_pipeline,
            domains::projects::pipelines::get_pipeline_execution,
            domains::projects::pipelines::get_pipeline_executions,
            domains::projects::pipelines::get_project_pipeline_executions,
            domains::projects::pipelines::get_all_pipeline_executions,
            domains::projects::pipelines::cancel_pipeline_execution,
            domains::projects::pipelines::get_pipeline_variables,
            domains::projects::pipelines::set_pipeline_variable,
            domains::projects::pipelines::delete_pipeline_variable,
            domains::projects::pipelines::get_pipeline_secrets,
            domains::projects::pipelines::add_pipeline_secret,
            domains::projects::pipelines::remove_pipeline_secret,
            domains::projects::pipelines::get_blocks,
            domains::projects::pipelines::create_block,
            domains::projects::pipelines::update_block,
            domains::projects::pipelines::delete_block,
            domains::projects::pipelines::get_step_execution_logs,
            domains::projects::pipelines::retry_step_execution,
            // Task commands
            domains::tasks::create_task,
            domains::tasks::update_task,
            domains::tasks::delete_task,
            domains::tasks::get_task,
            domains::tasks::get_tasks,
            domains::tasks::get_subtasks,
            domains::tasks::get_main_tasks,
            domains::tasks::get_task_count,
            // AI task generation
            domains::tasks::generate_tasks_from_story,
            // New advanced task commands
            domains::tasks::get_overdue_tasks,
            domains::tasks::get_due_today_tasks,
            domains::tasks::get_unestimated_tasks,
            // Document commands
            domains::documents::commands::create_document,
            domains::documents::commands::get_document,
            domains::documents::commands::get_documents,
            domains::documents::commands::update_document,
            domains::documents::commands::update_document_draft,
            domains::documents::commands::save_document,
            domains::documents::commands::delete_document,
            domains::documents::commands::search_documents,
            domains::documents::commands::generate_document_with_ai,
            // GitHub commands
            domains::github::commands::github_get_connection_status,
            domains::github::commands::github_start_device_flow,
            domains::github::commands::github_poll_device_flow,
            domains::github::commands::github_disconnect,
            domains::github::commands::github_list_repositories,
            domains::github::commands::github_list_linked_repos,
            domains::github::commands::github_get_repository,
            domains::github::commands::github_list_issues,
            domains::github::commands::github_get_issue,
            domains::github::commands::github_create_issue,
            domains::github::commands::github_update_issue,
            domains::github::commands::github_clone_repository,
            domains::github::commands::github_link_existing_repository,
            domains::github::commands::github_get_project_link,
            domains::github::commands::github_detect_local_repository,
            domains::github::commands::github_list_workflow_runs,
            domains::github::commands::github_get_workflow_run,
            domains::github::commands::github_get_workflow_job_logs,
            // Credential commands
            domains::credentials::commands::create_credential,
            domains::credentials::commands::get_credentials,
            domains::credentials::commands::get_credential,
            domains::credentials::commands::update_credential,
            domains::credentials::commands::delete_credential,
            domains::credentials::commands::decrypt_credential,
            domains::credentials::commands::search_credentials,
            // Automation commands
            domains::automation::trigger_n8n_workflow,
            domains::automation::get_workflow_status,
            domains::automation::list_available_workflows,
            domains::automation::get_suggested_workflows,
            domains::automation::check_n8n_health,
            // Embedded workflow engine commands
            domains::automation::register_embedded_workflow,
            domains::automation::execute_embedded_workflow,
            domains::automation::list_embedded_workflows,
            domains::automation::check_workflow_trigger,
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
            domains::deployments::commands::get_docker_status_command,
            domains::deployments::commands::start_docker_command,
            domains::deployments::commands::build_docker_image_command,
            domains::deployments::commands::get_process_status_command,
            domains::deployments::commands::start_container_command,
            domains::deployments::commands::stop_container_command,
            domains::deployments::commands::remove_container_command,
            // SDK commands (removed non-existent commands)
            domains::sdk::commands::sdk_commands::get_terminal_integration_status,
            domains::sdk::commands::sdk_commands::remove_terminal_integration,
            // SDK Navigation commands
            domains::sdk::commands::navigation_commands::get_sdk_navigation_items,
            domains::sdk::commands::navigation_commands::get_sdk_details,
            // SDK Configuration commands
            domains::sdk::commands::language_config_commands::get_sdk_config,
            domains::sdk::commands::language_config_commands::get_all_sdk_configs,
            domains::sdk::commands::language_config_commands::get_sdks_by_category,
            domains::sdk::commands::language_config_commands::get_all_sdk_managers,
            // SDK Manager version management commands
            domains::sdk::commands::manager_commands::get_manager_installed_versions,
            domains::sdk::commands::manager_commands::get_manager_available_versions,
            domains::sdk::commands::manager_commands::get_manager_current_version,
            domains::sdk::commands::manager_commands::install_version_via_manager,
            domains::sdk::commands::manager_commands::switch_version_via_manager,
            domains::sdk::commands::manager_commands::uninstall_version_via_manager,
            domains::sdk::commands::manager_commands::is_manager_version_installed,
            // Package manager commands
            domains::sdk::commands::package_manager_commands::get_available_package_managers,
            domains::sdk::commands::package_manager_commands::package_manager_search,
            domains::sdk::commands::package_manager_commands::package_manager_list_installed,
            domains::sdk::commands::package_manager_commands::package_manager_get_details,
            domains::sdk::commands::package_manager_commands::package_manager_install,
            domains::sdk::commands::package_manager_commands::package_manager_upgrade,
            domains::sdk::commands::package_manager_commands::package_manager_uninstall,
            domains::sdk::commands::package_manager_commands::package_manager_check_updates,
            domains::sdk::commands::package_manager_commands::package_manager_info,
            // Legacy language config commands (for backward compatibility)
            domains::sdk::commands::language_config_commands::get_language_config,
            domains::sdk::commands::language_config_commands::get_all_language_configs,
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
            // Runtime (AI) model management wrappers
            domains::sdk::commands::sdk_commands::get_runtime_models,
            domains::sdk::commands::sdk_commands::install_runtime_model,
            domains::sdk::commands::sdk_commands::remove_runtime_model,
            domains::sdk::commands::sdk_commands::get_runtime_available_models,
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
            domains::ide::commands::get_all_frameworks,
            domains::ide::commands::create_framework,
            domains::ide::commands::update_framework,
            domains::ide::commands::delete_framework,
            // Package manager commands
            domains::package_managers::commands::get_suggested_package_managers,
            domains::package_managers::commands::get_all_package_managers,
            domains::package_managers::commands::create_package_manager,
            domains::package_managers::commands::update_package_manager,
            domains::package_managers::commands::delete_package_manager,
            // Language commands
            domains::languages::commands::get_suggested_languages,
            domains::languages::commands::get_all_languages,
            domains::languages::commands::create_language,
            domains::languages::commands::update_language,
            domains::languages::commands::delete_language,
            // Learning commands
            domains::learning::commands::record_learning_event,
            domains::learning::commands::learn_pattern,
            domains::learning::commands::record_pattern_outcome,
            domains::learning::commands::get_suggestions,
            domains::learning::commands::learn_preference,
            domains::learning::commands::get_preference,
            domains::learning::commands::cleanup_learning_data,
            domains::learning::commands::get_memory_stats,
            domains::learning::commands::get_cleanup_preview,
            domains::learning::commands::mark_pattern_important,
            domains::learning::commands::get_ml_intensity,
            domains::learning::commands::set_ml_intensity,
            domains::learning::commands::get_ml_enabled,
            domains::learning::commands::set_ml_enabled,
            domains::learning::commands::get_all_patterns,
            domains::learning::commands::get_recent_events,
            domains::learning::commands::get_all_preferences,
            // Autonomy commands
            domains::autonomy::commands::evaluate_autonomous_action,
            domains::autonomy::commands::record_autonomous_action_outcome,
            domains::autonomy::commands::get_autonomy_level,
            domains::autonomy::commands::set_autonomy_level,
            domains::autonomy::commands::get_autonomy_enabled,
            domains::autonomy::commands::set_autonomy_enabled,
            domains::autonomy::commands::get_approval_stats,
            // Kubernetes commands
            domains::kubernetes::commands::k8s_initialize_manager,
            domains::kubernetes::commands::k8s_load_clusters,
            domains::kubernetes::commands::k8s_connect_cluster,
            domains::kubernetes::commands::k8s_list_pods,
            domains::kubernetes::commands::k8s_get_pod_logs,
            domains::kubernetes::commands::k8s_get_pod_yaml,
            domains::kubernetes::commands::k8s_delete_pod,
            domains::kubernetes::commands::k8s_scale_deployment,
            domains::kubernetes::commands::k8s_start_watching_pods,
            domains::kubernetes::commands::k8s_start_watching_services,
            domains::kubernetes::commands::k8s_start_watching_deployments,
            domains::kubernetes::commands::k8s_exec_pod,
            domains::kubernetes::commands::k8s_start_port_forward,
            domains::kubernetes::commands::k8s_list_port_forwards,
            domains::kubernetes::commands::k8s_stop_port_forward,
            domains::kubernetes::commands::k8s_stop_all_watches,
            domains::kubernetes::commands::k8s_list_services,
            domains::kubernetes::commands::k8s_list_deployments,
            domains::kubernetes::commands::k8s_list_statefulsets,
            domains::kubernetes::commands::k8s_list_daemonsets,
            domains::kubernetes::commands::k8s_list_jobs,
            domains::kubernetes::commands::k8s_list_cronjobs,
            domains::kubernetes::commands::k8s_list_ingresses,
            domains::kubernetes::commands::k8s_list_events,
            domains::kubernetes::commands::k8s_list_configmaps,
            domains::kubernetes::commands::k8s_list_secrets,
            domains::kubernetes::commands::k8s_get_resource_yaml,
            domains::kubernetes::commands::k8s_apply_resource_yaml,
            domains::kubernetes::commands::k8s_delete_configmap,
            domains::kubernetes::commands::k8s_delete_secret,
            domains::kubernetes::commands::k8s_rollback_deployment,
            domains::kubernetes::commands::k8s_get_pod_metrics,
            domains::kubernetes::commands::k8s_get_all_pods_metrics,
            domains::kubernetes::commands::k8s_list_namespaces,
            domains::kubernetes::commands::k8s_get_current_cluster,
            domains::kubernetes::commands::k8s_is_connected,
            domains::kubernetes::commands::k8s_detect_setup_tools,
            domains::kubernetes::commands::k8s_generate_kubeconfig,
            // AI commands
            // AI Provider commands
            domains::ai::commands::get_ai_provider_config_status,
            domains::ai::commands::get_ai_providers,
            domains::ai::commands::get_default_ai_provider,
            domains::ai::commands::set_default_ai_provider,
            domains::ai::commands::save_ai_provider_config,
            domains::ai::commands::get_ai_provider_config,
            domains::ai::commands::test_ai_provider,
            domains::ai::commands::get_ai_provider_models,
            domains::ai::commands::get_ai_platform_catalog,
            domains::ai::commands::generate_ai_text,
            domains::ai::commands::generate_ai_text_with_system,
            // AI Chat commands
            domains::ai::commands::ai_send_message,
            domains::ai::commands::ai_send_message_stream,
            // AI Conversation commands
            domains::ai::commands::ai_create_conversation,
            domains::ai::commands::ai_save_conversation,
            domains::ai::commands::ai_load_conversation,
            domains::ai::commands::ai_list_conversations,
            domains::ai::commands::ai_delete_conversation,
            domains::ai::commands::ai_update_conversation_title,
            domains::ai::commands::ai_update_conversation_model,
            // AI Log commands
            domains::ai::commands::ai_get_logs,
            domains::ai::commands::ai_search_logs,
            domains::ai::commands::ai_export_logs,
            // AI Training Data commands
            domains::ai::commands::ai_list_training_data,
            domains::ai::commands::ai_delete_training_data,
            // Custom Scripts commands
            domains::custom_scripts::commands::get_all_custom_scripts,
            domains::custom_scripts::commands::get_custom_script,
            domains::custom_scripts::commands::create_custom_script,
            domains::custom_scripts::commands::update_custom_script,
            domains::custom_scripts::commands::delete_custom_script,
            domains::custom_scripts::commands::record_script_run,
            domains::custom_scripts::commands::select_file,
            // Update commands
            domains::updates::commands::get_app_version_command,
            // Network commands
            domains::network::commands::get_local_network_ip,
            domains::network::commands::generate_device_passcode,
            domains::network::commands::verify_device_passcode,
            domains::network::commands::approve_device,
            domains::network::commands::verify_access_token,
            domains::network::commands::get_pending_device_approvals,
            domains::network::commands::get_device_status,
            // Script execution commands
            domains::scripts::commands::execute_script,
            domains::scripts::commands::get_script_execution,
            domains::scripts::commands::get_script_execution_live_output,
            domains::scripts::commands::cancel_script_execution,
            domains::scripts::commands::get_script_executions_by_block,
            domains::scripts::commands::get_running_script_executions,
            domains::scripts::commands::get_recent_script_executions,
            domains::scripts::commands::sync_script_executions,
            domains::scripts::commands::delete_script_execution,
            // Disk-cleanup commands (ported from portal_disk_utility)
            domains::disk::commands::scan_directory,
            domains::disk::commands::scan_projects,
            domains::disk::commands::scan_dev_cleaners,
            domains::disk::commands::clean_dev_items,
            domains::disk::commands::cancel_scan,
            domains::disk::commands::get_cached_scan,
            domains::disk::commands::remove_cached_scan,
            domains::disk::commands::quarantine_paths,
            domains::disk::commands::get_audit_log,
            domains::disk::commands::list_protected,
            domains::disk::commands::add_protected,
            domains::disk::commands::remove_protected,
            domains::disk::commands::list_locations,
            domains::disk::commands::disk_usage,
            domains::disk::commands::verify_proposals,
            domains::disk::commands::cancel_verify,
            domains::disk::commands::open_recycle_bin,
            // Environment variable utilities
            domains::environment::commands::env_list_variables,
            domains::environment::commands::env_get_permissions,
            domains::environment::commands::env_set_variable,
            domains::environment::commands::env_delete_variable,
            domains::environment::commands::env_apply_changes,
            domains::environment::commands::env_refresh_process,
            domains::environment::commands::env_request_elevation,
        ])
        .run(tauri::generate_context!()) // Note: OUT_DIR linter error is a false positive - resolves after build
        .expect("error while running tauri application");
}
