use crate::domains::environment::platform::platform_env;
use crate::domains::environment::types::{
    EnvApplyResult, EnvChange, EnvPermissions, EnvVariable,
};

#[tauri::command]
pub async fn env_list_variables() -> Result<Vec<EnvVariable>, String> {
    platform_env().list_variables()
}

#[tauri::command]
pub async fn env_get_permissions() -> Result<EnvPermissions, String> {
    platform_env().get_permissions()
}

#[tauri::command]
pub async fn env_set_variable(name: String, value: String, scope: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Variable name cannot be empty".into());
    }
    platform_env().set_variable(&name, &value, &scope)
}

#[tauri::command]
pub async fn env_delete_variable(name: String, scope: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Variable name cannot be empty".into());
    }
    platform_env().delete_variable(&name, &scope)
}

#[tauri::command]
pub async fn env_apply_changes(changes: Vec<EnvChange>) -> Result<EnvApplyResult, String> {
    let platform = platform_env();
    let permissions = platform.get_permissions()?;

    let mut user_changes = Vec::new();
    let mut system_changes = Vec::new();
    for change in changes {
        if change.scope == "system" {
            system_changes.push(change);
        } else {
            user_changes.push(change);
        }
    }

    for change in &user_changes {
        match change.action.as_str() {
            "set" => {
                let value = change.value.clone().unwrap_or_default();
                platform.set_variable(&change.name, &value, &change.scope)?;
            }
            "delete" => platform.delete_variable(&change.name, &change.scope)?,
            _ => return Err(format!("Unknown action: {}", change.action)),
        }
    }

    if system_changes.is_empty() {
        platform.refresh_process_environment()?;
        return Ok(EnvApplyResult {
            success: true,
            message: "Environment variables saved.".into(),
            elevated: false,
        });
    }

    if permissions.can_edit_system {
        for change in &system_changes {
            match change.action.as_str() {
                "set" => {
                    let value = change.value.clone().unwrap_or_default();
                    platform.set_variable(&change.name, &value, "system")?;
                }
                "delete" => platform.delete_variable(&change.name, "system")?,
                _ => return Err(format!("Unknown action: {}", change.action)),
            }
        }
        platform.refresh_process_environment()?;
        return Ok(EnvApplyResult {
            success: true,
            message: "Environment variables saved.".into(),
            elevated: permissions.is_elevated,
        });
    }

    let elevated_result = platform.apply_changes_elevated(&system_changes)?;
    if elevated_result.success {
        platform.refresh_process_environment()?;
    }
    Ok(elevated_result)
}

#[tauri::command]
pub async fn env_refresh_process() -> Result<(), String> {
    platform_env().refresh_process_environment()
}

#[tauri::command]
pub async fn env_request_elevation() -> Result<EnvPermissions, String> {
    // Re-check permissions after the user may have approved elevation in a prior action.
    platform_env().get_permissions()
}
