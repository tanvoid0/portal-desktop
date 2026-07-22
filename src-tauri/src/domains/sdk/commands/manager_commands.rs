/**
 * SDK Manager Commands
 *
 * Tauri commands for SDK manager-specific operations (version management, installation, etc.)
 */
use crate::command_executor::{CommandExecutor, CommandOptions, ShellType};
use crate::domains::sdk::configs::{get_all_sdk_configs, SDKManagerConfig};
use crate::domains::sdk::factory::SDKManagerFactory;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SDKManagerWorkflowSupport {
    pub install_available: bool,
    pub install_unavailable_reason: Option<String>,
    pub uninstall_available: bool,
    pub uninstall_unavailable_reason: Option<String>,
}

enum ManagerWorkflow {
    Shell {
        command: String,
        shell: ShellType,
    },
    Process {
        program: String,
        args: Vec<String>,
    },
}

pub fn get_sdk_manager_workflow_support(
    manager_name: &str,
    installed: bool,
) -> SDKManagerWorkflowSupport {
    let Some(config) = find_manager_config(manager_name) else {
        return SDKManagerWorkflowSupport {
            install_available: false,
            install_unavailable_reason: Some(format!(
                "No SDK manager configuration was found for '{}'.",
                manager_name
            )),
            uninstall_available: false,
            uninstall_unavailable_reason: Some(format!(
                "No SDK manager configuration was found for '{}'.",
                manager_name
            )),
        };
    };

    let install_workflow = resolve_install_workflow(&config);
    let uninstall_workflow = resolve_uninstall_workflow(&config);

    let install_unavailable_reason = if installed {
        Some(format!("{} is already installed.", config.display_name))
    } else if install_workflow.is_none() {
        Some(explain_missing_install_workflow(&config))
    } else {
        None
    };

    let uninstall_unavailable_reason = if !installed {
        Some(format!("{} is not currently installed.", config.display_name))
    } else if uninstall_workflow.is_none() {
        Some(explain_missing_uninstall_workflow(&config))
    } else {
        None
    };

    SDKManagerWorkflowSupport {
        install_available: !installed && install_workflow.is_some(),
        install_unavailable_reason,
        uninstall_available: installed && uninstall_workflow.is_some(),
        uninstall_unavailable_reason,
    }
}

fn find_manager_config(manager_name: &str) -> Option<SDKManagerConfig> {
    let normalized = manager_name.to_ascii_lowercase();

    get_all_sdk_configs()
        .into_iter()
        .flat_map(|config| config.sdk_managers.into_iter())
        .find(|manager| {
            let aliases = [
                manager.id.to_ascii_lowercase(),
                manager.name.to_ascii_lowercase(),
                manager.binary.to_ascii_lowercase(),
            ];
            aliases.iter().any(|alias| alias == &normalized)
        })
}

fn explain_missing_install_workflow(config: &SDKManagerConfig) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "Automatic install is not available for {} on Windows yet.",
            config.display_name
        )
    } else if !config.supports_installation {
        format!("{} does not support automatic installation.", config.display_name)
    } else {
        format!(
            "No executable install workflow is configured for {}.",
            config.display_name
        )
    }
}

fn explain_missing_uninstall_workflow(config: &SDKManagerConfig) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "Automatic uninstall is not available for {} on Windows yet.",
            config.display_name
        )
    } else {
        format!(
            "Automatic uninstall is not implemented for {}. Remove it manually using its documented uninstall steps.",
            config.display_name
        )
    }
}

fn resolve_install_workflow(config: &SDKManagerConfig) -> Option<ManagerWorkflow> {
    if cfg!(target_os = "windows") {
        let package_id = match config.id.as_str() {
            "rustup" => "Rustlang.Rustup",
            "fnm" => "Schniz.fnm",
            "nvm" => "CoreyButler.NVMforWindows",
            "pyenv" => "pyenv-win.pyenv-win",
            "conda" => "Anaconda.Miniconda3",
            _ => return None,
        };

        if !has_command("winget") {
            return None;
        }

        return Some(ManagerWorkflow::Process {
            program: "winget".to_string(),
            args: vec![
                "install".to_string(),
                "--id".to_string(),
                package_id.to_string(),
                "--exact".to_string(),
                "--accept-package-agreements".to_string(),
                "--accept-source-agreements".to_string(),
                "--disable-interactivity".to_string(),
            ],
        });
    }

    config
        .install_command
        .as_ref()
        .map(|command| ManagerWorkflow::Shell {
            command: command.clone(),
            shell: ShellType::Bash,
        })
}

fn resolve_uninstall_workflow(config: &SDKManagerConfig) -> Option<ManagerWorkflow> {
    if cfg!(target_os = "windows") {
        let package_id = match config.id.as_str() {
            "rustup" => "Rustlang.Rustup",
            "fnm" => "Schniz.fnm",
            "nvm" => "CoreyButler.NVMforWindows",
            "pyenv" => "pyenv-win.pyenv-win",
            "conda" => "Anaconda.Miniconda3",
            _ => return None,
        };

        if !has_command("winget") {
            return None;
        }

        return Some(ManagerWorkflow::Process {
            program: "winget".to_string(),
            args: vec![
                "uninstall".to_string(),
                "--id".to_string(),
                package_id.to_string(),
                "--exact".to_string(),
                "--disable-interactivity".to_string(),
            ],
        });
    }

    match config.id.as_str() {
        "rustup" => Some(ManagerWorkflow::Shell {
            command: "rustup self uninstall -y".to_string(),
            shell: ShellType::Bash,
        }),
        _ => None,
    }
}

/// Whether `command` resolves on PATH.
///
/// Deliberately a PATH lookup, not a `<command> --version` spawn: this runs
/// twice per manager while building the SDK manager list, and on Windows a
/// spawn from a GUI process (no attached console) pops a console window each
/// time. `which` resolves in-process and honours PATHEXT.
fn has_command(command: &str) -> bool {
    which::which(command).is_ok()
}

async fn execute_workflow(
    manager_name: &str,
    action_label: &str,
    workflow: ManagerWorkflow,
) -> Result<(), String> {
    let result = match workflow {
        ManagerWorkflow::Shell { command, shell } => CommandExecutor::execute(
            &command,
            Some(CommandOptions {
                shell: Some(shell),
                ..Default::default()
            }),
        )
        .await
        .map_err(|err| format!("Failed to start {} for {}: {}", action_label, manager_name, err))?,
        ManagerWorkflow::Process { program, args } => {
            let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
            CommandExecutor::execute_with_args(&program, &args_ref, None)
                .await
                .map_err(|err| {
                    format!("Failed to start {} for {}: {}", action_label, manager_name, err)
                })?
        }
    };

    if result.success {
        return Ok(());
    }

    let detail = if !result.stderr.trim().is_empty() {
        result.stderr.trim().to_string()
    } else if !result.stdout.trim().is_empty() {
        result.stdout.trim().to_string()
    } else if let Some(exit_code) = result.exit_code {
        format!("process exited with code {}", exit_code)
    } else {
        "the workflow exited without additional output".to_string()
    };

    Err(format!(
        "{} failed for {}: {}",
        action_label, manager_name, detail
    ))
}

/// Get installed versions for a specific SDK manager
#[tauri::command]
pub async fn get_manager_installed_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();

    match factory.list_versions(&manager_name).await {
        Ok(versions) => Ok(versions),
        Err(e) => Err(format!(
            "Failed to list versions for {}: {}",
            manager_name, e
        )),
    }
}

/// Get available (installable) versions for a specific SDK manager
#[tauri::command]
pub async fn get_manager_available_versions(manager_name: String) -> Result<Vec<String>, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.list_available_versions().await {
            Ok(versions) => Ok(versions),
            Err(e) => Err(format!(
                "Failed to list available versions for {}: {}",
                manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Get current active version for a specific SDK manager
#[tauri::command]
pub async fn get_manager_current_version(manager_name: String) -> Result<Option<String>, String> {
    let factory = SDKManagerFactory::new();

    match factory.get_current_version(&manager_name).await {
        Ok(version) => Ok(version),
        Err(e) => Err(format!(
            "Failed to get current version for {}: {}",
            manager_name, e
        )),
    }
}

/// Install a version using a specific SDK manager
#[tauri::command]
pub async fn install_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    match factory.install_version(&manager_name, &version).await {
        Ok(_) => Ok(format!(
            "Successfully installed {} version {}",
            manager_name, version
        )),
        Err(e) => Err(format!(
            "Failed to install version {} for {}: {}",
            version, manager_name, e
        )),
    }
}

/// Install an SDK manager itself rather than an SDK version.
#[tauri::command]
pub async fn install_sdk_manager(manager_name: String) -> Result<String, String> {
    let config = find_manager_config(&manager_name)
        .ok_or_else(|| format!("Manager '{}' not found", manager_name))?;
    let support = get_sdk_manager_workflow_support(&manager_name, false);

    if !support.install_available {
        return Err(
            support
                .install_unavailable_reason
                .unwrap_or_else(|| explain_missing_install_workflow(&config)),
        );
    }

    let workflow = resolve_install_workflow(&config)
        .ok_or_else(|| explain_missing_install_workflow(&config))?;
    execute_workflow(&manager_name, "Installation", workflow).await?;

    Ok(format!(
        "Successfully installed {}",
        config.display_name
    ))
}

/// Switch to a version using a specific SDK manager
#[tauri::command]
pub async fn switch_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.switch_version(&version).await {
            Ok(_) => Ok(format!(
                "Successfully switched to version {} for {}",
                version, manager_name
            )),
            Err(e) => Err(format!(
                "Failed to switch to version {} for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Uninstall a version using a specific SDK manager
#[tauri::command]
pub async fn uninstall_version_via_manager(
    manager_name: String,
    version: String,
) -> Result<String, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.uninstall_version(&version).await {
            Ok(_) => Ok(format!(
                "Successfully uninstalled {} version {}",
                manager_name, version
            )),
            Err(e) => Err(format!(
                "Failed to uninstall version {} for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}

/// Uninstall an SDK manager itself rather than an SDK version.
#[tauri::command]
pub async fn uninstall_sdk_manager(manager_name: String) -> Result<String, String> {
    let config = find_manager_config(&manager_name)
        .ok_or_else(|| format!("Manager '{}' not found", manager_name))?;
    let support = get_sdk_manager_workflow_support(&manager_name, true);

    if !support.uninstall_available {
        return Err(
            support
                .uninstall_unavailable_reason
                .unwrap_or_else(|| explain_missing_uninstall_workflow(&config)),
        );
    }

    let workflow = resolve_uninstall_workflow(&config)
        .ok_or_else(|| explain_missing_uninstall_workflow(&config))?;
    execute_workflow(&manager_name, "Uninstall", workflow).await?;

    Ok(format!(
        "Successfully uninstalled {}",
        config.display_name
    ))
}

/// Check if a version is installed for a specific SDK manager
#[tauri::command]
pub async fn is_manager_version_installed(
    manager_name: String,
    version: String,
) -> Result<bool, String> {
    let factory = SDKManagerFactory::new();

    if let Some(manager) = factory.get_manager(&manager_name) {
        match manager.is_version_installed(&version).await {
            Ok(installed) => Ok(installed),
            Err(e) => Err(format!(
                "Failed to check if version {} is installed for {}: {}",
                version, manager_name, e
            )),
        }
    } else {
        Err(format!("Manager '{}' not found", manager_name))
    }
}
