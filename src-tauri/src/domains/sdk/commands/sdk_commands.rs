use crate::domains::sdk::manager_detector::{detect_sdk_managers as detect_managers, SDKInfo};
use crate::domains::sdk::ollama_manager::{OllamaManager, OllamaModel, OllamaVersion};
use crate::domains::sdk::version_fetcher::{
    fetch_go_versions, fetch_java_versions, fetch_nodejs_versions, fetch_php_versions,
    fetch_python_versions, fetch_ruby_versions, fetch_rust_versions, SDKVersion,
};
use crate::domains::sdk::version_installer::{
    install_go_version, install_java_version, install_nodejs_version, install_php_version,
    install_python_version, install_ruby_version, install_rust_version,
};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub status: String,
}

#[tauri::command]
pub async fn update_project_version(
    project_path: String,
    sdk_type: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Updating project version: {} to {} for {}",
        project_path, version, sdk_type
    );

    Err("Project version management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn remove_project_version(
    project_path: String,
    sdk_type: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Removing project version: {} from {} for {}",
        version, sdk_type, project_path
    );

    Err("Project version management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_project_versions(project_path: String) -> Result<Vec<SDKVersion>, String> {
    println!("[SDK] Getting project versions for: {}", project_path);

    // Return empty array - project version detection not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn setup_shell_integration(sdk_type: String) -> Result<String, String> {
    println!("[SDK] Setting up shell integration for: {}", sdk_type);

    Err("Shell integration is not yet implemented".to_string())
}

#[tauri::command]
pub async fn activate_project_environment(
    project_path: String,
    sdk_type: String,
) -> Result<String, String> {
    println!(
        "[SDK] Activating project environment: {} for {}",
        project_path, sdk_type
    );

    Err("Project environment activation is not yet implemented".to_string())
}

#[tauri::command]
pub async fn deactivate_project_environment(
    project_path: String,
    sdk_type: String,
) -> Result<String, String> {
    println!(
        "[SDK] Deactivating project environment: {} for {}",
        project_path, sdk_type
    );

    Err("Project environment deactivation is not yet implemented".to_string())
}

#[tauri::command]
pub async fn find_projects_with_versions(sdk_type: String) -> Result<Vec<String>, String> {
    println!("[SDK] Finding projects with versions for: {}", sdk_type);

    // Return empty array - project scanning not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn get_terminal_integration_status() -> Result<bool, String> {
    println!("[SDK] Getting terminal integration status");

    // Terminal integration not yet implemented
    Ok(false)
}

#[tauri::command]
pub async fn remove_terminal_integration() -> Result<String, String> {
    println!("[SDK] Removing terminal integration");

    Err("Terminal integration is not yet implemented".to_string())
}

// Real version fetching
#[tauri::command]
pub async fn fetch_available_versions(sdk_type: String) -> Result<Vec<SDKVersion>, String> {
    println!("[SDK] Fetching available versions for {}", sdk_type);

    match sdk_type.as_str() {
        "node" | "nodejs" => fetch_nodejs_versions().await,
        "python" => fetch_python_versions().await,
        "java" => fetch_java_versions().await,
        "rust" => fetch_rust_versions().await,
        "go" => fetch_go_versions().await,
        "php" => fetch_php_versions().await,
        "ruby" => fetch_ruby_versions().await,
        "ollama" => {
            let ollama_versions = OllamaManager::fetch_available_versions().await?;
            let sdk_versions: Vec<SDKVersion> = ollama_versions
                .into_iter()
                .map(|v| SDKVersion {
                    version: v.version,
                    installed: v.installed,
                    active: v.active,
                    size: v.size,
                    release_date: v.release_date,
                })
                .collect();
            Ok(sdk_versions)
        }
        _ => {
            println!("[SDK] Unknown SDK type: {}", sdk_type);
            Ok(vec![])
        }
    }
}

// Real version installation
#[tauri::command]
pub async fn download_and_install_version(
    sdk_type: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Downloading and installing {} version {}",
        sdk_type, version
    );

    match sdk_type.as_str() {
        "nodejs" => install_nodejs_version(&version).await,
        "python" => install_python_version(&version).await,
        "java" => install_java_version(&version).await,
        "rust" => install_rust_version(&version).await,
        "go" => install_go_version(&version).await,
        "php" => install_php_version(&version).await,
        "ruby" => install_ruby_version(&version).await,
        _ => Err(format!("Unknown SDK type: {}", sdk_type)),
    }
}

// Real SDK manager detection
#[tauri::command]
pub async fn detect_sdk_managers() -> Result<Vec<SDKInfo>, String> {
    detect_managers().await
}

// Service management commands (legacy - delegates to start_service/stop_service)
#[tauri::command]
pub async fn start_sdk_service(sdk_type: String) -> Result<String, String> {
    println!("[SDK] Starting SDK service: {}", sdk_type);
    // Delegate to the real start_service implementation
    start_service(sdk_type).await
}

#[tauri::command]
pub async fn stop_sdk_service(sdk_type: String) -> Result<String, String> {
    println!("[SDK] Stopping SDK service: {}", sdk_type);
    // Delegate to the real stop_service implementation
    stop_service(sdk_type).await
}

#[tauri::command]
pub async fn get_service_status(sdk_type: String) -> Result<ServiceStatus, String> {
    println!("[SDK] Getting service status for {}", sdk_type);

    match sdk_type.as_str() {
        "ollama" => {
            let ollama_status = OllamaManager::get_service_status().await?;
            Ok(ServiceStatus {
                running: ollama_status.running,
                pid: ollama_status.pid,
                port: ollama_status.port,
                status: if ollama_status.running {
                    "running".to_string()
                } else {
                    "stopped".to_string()
                },
            })
        }
        "docker" => {
            let running = check_docker_running().await;
            Ok(ServiceStatus {
                running,
                pid: None,
                port: None,
                status: if running {
                    "running".to_string()
                } else {
                    "stopped".to_string()
                },
            })
        }
        _ => {
            // For system services, check via systemctl or service command
            let running = check_system_service_status(&sdk_type).await;
            // Get port from SDK config if available
            let port = get_service_port(&sdk_type).await;
            Ok(ServiceStatus {
                running,
                pid: None,
                port,
                status: if running {
                    "running".to_string()
                } else {
                    "stopped".to_string()
                },
            })
        }
    }
}

/// Check if Docker is running
async fn check_docker_running() -> bool {
    use crate::process_ext::NoWindowExt;
    use std::process::Command;

    let result = Command::new("docker").no_window().args(&["info"]).output();

    if let Ok(output) = result {
        output.status.success()
    } else {
        false
    }
}

/// Check system service status
async fn check_system_service_status(service_name: &str) -> bool {
    use std::process::Command;

    // Try systemctl first
    let systemctl_result = Command::new("systemctl")
        .args(&["is-active", service_name])
        .output();

    if let Ok(output) = systemctl_result {
        if output.status.success() {
            let status_str = String::from_utf8_lossy(&output.stdout);
            let status = status_str.trim();
            return status == "active";
        }
    }

    // Fallback: check if process is running by checking port
    // This is a simplified check - in production, you'd want more robust detection
    false
}

/// Get service port from SDK config
async fn get_service_port(sdk_type: &str) -> Option<u16> {
    use crate::domains::sdk::configs::language_config::get_sdk_config;

    if let Some(config) = get_sdk_config(sdk_type) {
        if let Some(service_config) = &config.service_config {
            if let Some(port) = service_config.get("port") {
                if let Some(port_num) = port.as_u64() {
                    return Some(port_num as u16);
                }
            }
        }
    }

    None
}

// Additional commands for FlyEnv-style functionality
#[tauri::command]
pub async fn setup_project_version_file(
    project_path: String,
    sdk_type: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Setting up project version file: {} for {} at {}",
        sdk_type, version, project_path
    );

    Err("Project version file management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_running_services_count() -> Result<u32, String> {
    println!("[SDK] Getting running services count");

    // Count running services by checking status for known service types
    let service_types = vec![
        "ollama",
        "docker",
        "postgresql",
        "mysql",
        "mongodb",
        "redis",
        "nginx",
        "apache",
    ];
    let mut count = 0u32;

    for service_type in service_types {
        if let Ok(status) = get_service_status(service_type.to_string()).await {
            if status.running {
                count += 1;
            }
        }
    }

    Ok(count)
}

// Real Ollama commands using OllamaManager
#[tauri::command]
pub async fn get_ollama_versions() -> Result<Vec<OllamaVersion>, String> {
    println!("[SDK] Getting Ollama versions...");

    OllamaManager::fetch_available_versions().await
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<OllamaModel>, String> {
    println!("[SDK] Getting Ollama models...");

    OllamaManager::get_installed_models().await
}

#[tauri::command]
pub async fn install_ollama_model(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<String, String> {
    println!("[SDK] Installing Ollama model: {}", model_name);

    // Start installation in background task to emit progress events
    let model_name_clone = model_name.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let result =
            OllamaManager::install_model_with_progress(&model_name_clone, app_clone.clone()).await;
        match result {
            Ok(_) => {
                // Success already emitted in install_model_with_progress
            }
            Err(e) => {
                eprintln!("[SDK] Model installation failed: {}", e);
                let _ = app_clone.emit(
                    "ollama-model-progress",
                    serde_json::json!({
                        "model": model_name_clone,
                        "status": "error",
                        "message": e.clone(),
                        "progress": 0
                    }),
                );
            }
        }
    });

    // Return immediately, progress will come via events
    Ok("Model installation started".to_string())
}

#[tauri::command]
pub async fn remove_ollama_model(model_name: String) -> Result<String, String> {
    println!("[SDK] Removing Ollama model: {}", model_name);

    OllamaManager::remove_model(&model_name).await
}

#[tauri::command]
pub async fn get_available_ollama_models(
) -> Result<std::collections::HashMap<String, Vec<serde_json::Value>>, String> {
    println!("[SDK] Getting available Ollama models...");

    let result = OllamaManager::get_available_models().await;
    match &result {
        Ok(models) => println!("[SDK] Successfully got {} model families", models.len()),
        Err(e) => println!("[SDK] Failed to get models: {}", e),
    }
    result
}

// ========== Runtime (AI) model management wrappers ==========
// These commands let the frontend manage models for whichever runtime SDK is selected.
// For now, this delegates to Ollama, but the dispatch point is centralized here.

#[tauri::command]
pub async fn get_runtime_models(sdk_type: String) -> Result<Vec<OllamaModel>, String> {
    match sdk_type.as_str() {
        "ollama" => get_ollama_models().await,
        _ => Err(format!("Unknown runtime sdk type: {}", sdk_type)),
    }
}

#[tauri::command]
pub async fn remove_runtime_model(
    sdk_type: String,
    model_name: String,
) -> Result<String, String> {
    match sdk_type.as_str() {
        "ollama" => remove_ollama_model(model_name).await,
        _ => Err(format!("Unknown runtime sdk type: {}", sdk_type)),
    }
}

#[tauri::command]
pub async fn install_runtime_model(
    app: tauri::AppHandle,
    sdk_type: String,
    model_name: String,
) -> Result<String, String> {
    match sdk_type.as_str() {
        "ollama" => install_ollama_model(app, model_name).await,
        _ => Err(format!("Unknown runtime sdk type: {}", sdk_type)),
    }
}

#[tauri::command]
pub async fn get_runtime_available_models(
    sdk_type: String,
) -> Result<std::collections::HashMap<String, Vec<serde_json::Value>>, String> {
    match sdk_type.as_str() {
        "ollama" => get_available_ollama_models().await,
        _ => Err(format!("Unknown runtime sdk type: {}", sdk_type)),
    }
}

#[tauri::command]
pub async fn start_service(sdk_type: String) -> Result<String, String> {
    println!("[SDK] Starting service: {}", sdk_type);

    match sdk_type.as_str() {
        "ollama" => OllamaManager::start_service().await,
        "docker" => start_docker_service().await,
        "postgresql" | "postgres" => start_system_service("postgresql").await,
        "mysql" => start_system_service("mysql").await,
        "mongodb" | "mongo" => start_system_service("mongod").await,
        "redis" => start_system_service("redis").await,
        "nginx" => start_system_service("nginx").await,
        "apache" | "httpd" => start_system_service("apache2").await,
        _ => {
            // Try systemctl first, then fallback to direct command
            start_system_service(&sdk_type).await
        }
    }
}

#[tauri::command]
pub async fn stop_service(sdk_type: String) -> Result<String, String> {
    println!("[SDK] Stopping service: {}", sdk_type);

    match sdk_type.as_str() {
        "ollama" => OllamaManager::stop_service().await,
        "docker" => stop_docker_service().await,
        "postgresql" | "postgres" => stop_system_service("postgresql").await,
        "mysql" => stop_system_service("mysql").await,
        "mongodb" | "mongo" => stop_system_service("mongod").await,
        "redis" => stop_system_service("redis").await,
        "nginx" => stop_system_service("nginx").await,
        "apache" | "httpd" => stop_system_service("apache2").await,
        _ => {
            // Try systemctl first, then fallback to direct command
            stop_system_service(&sdk_type).await
        }
    }
}

/// Start a system service using systemctl or service command
async fn start_system_service(service_name: &str) -> Result<String, String> {
    use std::process::Command;

    // Try systemctl first (systemd)
    let systemctl_result = Command::new("systemctl")
        .args(&["start", service_name])
        .output();

    if let Ok(output) = systemctl_result {
        if output.status.success() {
            return Ok(format!("Started {} service via systemctl", service_name));
        }
    }

    // Fallback to service command (SysV init)
    let service_result = Command::new("service")
        .args(&[service_name, "start"])
        .output();

    if let Ok(output) = service_result {
        if output.status.success() {
            return Ok(format!(
                "Started {} service via service command",
                service_name
            ));
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to start {} service: {}",
                service_name, error_msg
            ));
        }
    }

    Err(format!(
        "Failed to start {} service: systemctl and service commands both failed",
        service_name
    ))
}

/// Stop a system service using systemctl or service command
async fn stop_system_service(service_name: &str) -> Result<String, String> {
    use std::process::Command;

    // Try systemctl first (systemd)
    let systemctl_result = Command::new("systemctl")
        .args(&["stop", service_name])
        .output();

    if let Ok(output) = systemctl_result {
        if output.status.success() {
            return Ok(format!("Stopped {} service via systemctl", service_name));
        }
    }

    // Fallback to service command (SysV init)
    let service_result = Command::new("service")
        .args(&[service_name, "stop"])
        .output();

    if let Ok(output) = service_result {
        if output.status.success() {
            return Ok(format!(
                "Stopped {} service via service command",
                service_name
            ));
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to stop {} service: {}",
                service_name, error_msg
            ));
        }
    }

    Err(format!(
        "Failed to stop {} service: systemctl and service commands both failed",
        service_name
    ))
}

/// Start Docker service
async fn start_docker_service() -> Result<String, String> {
    use crate::process_ext::NoWindowExt;
    use std::process::Command;

    // Check if Docker is already running
    let check_result = Command::new("docker").no_window().args(&["info"]).output();

    if let Ok(output) = check_result {
        if output.status.success() {
            return Ok("Docker service is already running".to_string());
        }
    }

    // Try to start Docker daemon via systemctl
    let systemctl_result = Command::new("systemctl")
        .args(&["start", "docker"])
        .output();

    if let Ok(output) = systemctl_result {
        if output.status.success() {
            // Wait a moment for Docker to start
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            return Ok("Docker service started successfully".to_string());
        }
    }

    Err("Failed to start Docker service. Please ensure Docker is installed and you have permission to start system services.".to_string())
}

/// Stop Docker service
async fn stop_docker_service() -> Result<String, String> {
    use std::process::Command;

    // Try to stop Docker daemon via systemctl
    let systemctl_result = Command::new("systemctl").args(&["stop", "docker"]).output();

    if let Ok(output) = systemctl_result {
        if output.status.success() {
            return Ok("Docker service stopped successfully".to_string());
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to stop Docker service: {}", error_msg));
        }
    }

    Err(
        "Failed to stop Docker service. Please ensure you have permission to stop system services."
            .to_string(),
    )
}

#[tauri::command]
pub async fn check_ollama_updates() -> Result<String, String> {
    println!("[SDK] Checking for Ollama updates...");

    OllamaManager::check_for_updates().await
}

#[tauri::command]
pub async fn update_ollama() -> Result<String, String> {
    println!("[SDK] Updating Ollama...");

    OllamaManager::update_ollama().await
}

// Additional SDK management commands
#[tauri::command]
pub async fn add_custom_sdk_directory(path: String, sdk_type: String) -> Result<String, String> {
    println!("[SDK] Adding custom directory: {} for {}", path, sdk_type);

    Err("Custom SDK directory management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn remove_custom_sdk_directory(path: String, sdk_type: String) -> Result<String, String> {
    println!("[SDK] Removing custom directory: {} for {}", path, sdk_type);

    Err("Custom SDK directory management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_custom_directories(sdk_type: String) -> Result<Vec<serde_json::Value>, String> {
    println!("[SDK] Getting custom directories for: {}", sdk_type);

    // Return empty array - custom directory management not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn get_service_logs(
    service_id: String,
    lines: usize,
) -> Result<Vec<serde_json::Value>, String> {
    println!(
        "[SDK] Getting service logs for: {} (last {} lines)",
        service_id, lines
    );

    // Service log retrieval not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn update_service_config(
    service_id: String,
    _config: serde_json::Value,
) -> Result<String, String> {
    println!("[SDK] Updating service config for: {}", service_id);

    Err("Service configuration management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn restart_service(service_id: String) -> Result<String, String> {
    println!("[SDK] Restarting service: {}", service_id);

    // Try to restart by stopping and starting
    let stop_result = stop_service(service_id.clone()).await;
    if stop_result.is_err() {
        return stop_result;
    }

    // Wait a moment before starting
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    start_service(service_id).await
}

#[tauri::command]
pub async fn get_service_health(service_id: String) -> Result<serde_json::Value, String> {
    println!("[SDK] Getting service health for: {}", service_id);

    // Get basic status - detailed health metrics not yet implemented
    let status = get_service_status(service_id.clone()).await?;

    let health = serde_json::json!({
        "service_id": service_id,
        "status": if status.running { "healthy" } else { "stopped" },
        "running": status.running,
        "port": status.port,
        "pid": status.pid
    });

    Ok(health)
}

#[tauri::command]
pub async fn set_path_environment(sdk_type: String, version: String) -> Result<String, String> {
    println!(
        "[SDK] Setting PATH environment for: {} version {}",
        sdk_type, version
    );

    Err("PATH environment management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_path_status(sdk_type: String) -> Result<serde_json::Value, String> {
    println!("[SDK] Getting PATH status for: {}", sdk_type);

    // Basic PATH status - detailed management not yet implemented
    let status = serde_json::json!({
        "sdk_type": sdk_type,
        "in_path": false,
        "path_entries": []
    });

    Ok(status)
}

#[tauri::command]
pub async fn create_alias(
    sdk_type: String,
    alias_name: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Creating alias: {} -> {} for {}",
        alias_name, version, sdk_type
    );

    Err("Version alias management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn remove_alias(alias_name: String) -> Result<String, String> {
    println!("[SDK] Removing alias: {}", alias_name);

    Err("Version alias management is not yet implemented".to_string())
}

#[tauri::command]
pub async fn list_aliases(sdk_type: String) -> Result<Vec<serde_json::Value>, String> {
    println!("[SDK] Listing aliases for: {}", sdk_type);

    // Return empty array - alias management not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn detect_version_files(project_path: String) -> Result<Vec<serde_json::Value>, String> {
    println!("[SDK] Detecting version files in: {}", project_path);

    // Version file detection not yet implemented
    Ok(vec![])
}

#[tauri::command]
pub async fn create_version_file(
    project_path: String,
    sdk_type: String,
    version: String,
) -> Result<String, String> {
    println!(
        "[SDK] Creating version file: {} for {} at {}",
        sdk_type, version, project_path
    );

    Err("Version file creation is not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_all_available_sdks() -> Result<Vec<serde_json::Value>, String> {
    println!("[SDK] Getting all available SDKs...");

    // Return a curated list of available SDKs without requiring database connection
    let available_sdks = vec![
        serde_json::json!({
            "id": "node",
            "name": "Node.js",
            "category": "language",
            "description": "JavaScript Runtime",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "python",
            "name": "Python",
            "category": "language",
            "description": "Python Programming Language",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "java",
            "name": "Java",
            "category": "language",
            "description": "Java Development Kit",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "rust",
            "name": "Rust",
            "category": "language",
            "description": "Systems Programming Language",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "go",
            "name": "Go",
            "category": "language",
            "description": "Go Programming Language",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "php",
            "name": "PHP",
            "category": "language",
            "description": "PHP Programming Language",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "ruby",
            "name": "Ruby",
            "category": "language",
            "description": "Ruby Programming Language",
            "installed": false,
            "version": null
        }),
        serde_json::json!({
            "id": "ollama",
            "name": "Ollama",
            "category": "ai",
            "description": "AI Model Runtime",
            "installed": false,
            "version": null
        }),
    ];

    Ok(available_sdks)
}
