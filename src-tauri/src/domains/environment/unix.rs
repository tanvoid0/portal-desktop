use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::domains::environment::types::{EnvApplyResult, EnvChange, EnvPermissions, EnvVariable};

use super::platform::PlatformEnv;

pub struct UnixEnv;

impl PlatformEnv for UnixEnv {
    fn list_variables(&self) -> Result<Vec<EnvVariable>, String> {
        let mut vars: HashMap<String, EnvVariable> = HashMap::new();

        for (name, value) in std::env::vars() {
            vars.insert(
                name.clone(),
                EnvVariable {
                    name,
                    value,
                    scope: "session".into(),
                },
            );
        }

        if let Ok(user_vars) = read_user_env_file() {
            for (name, value) in user_vars {
                vars.insert(
                    name.clone(),
                    EnvVariable {
                        name,
                        value,
                        scope: "user".into(),
                    },
                );
            }
        }

        let mut list: Vec<EnvVariable> = vars.into_values().collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(list)
    }

    fn get_permissions(&self) -> Result<EnvPermissions, String> {
        let is_root = is_unix_root();

        Ok(EnvPermissions {
            can_edit_user: true,
            can_edit_system: is_root,
            is_elevated: is_root,
            platform: if cfg!(target_os = "macos") {
                "macos".into()
            } else {
                "linux".into()
            },
        })
    }

    fn set_variable(&self, name: &str, value: &str, scope: &str) -> Result<(), String> {
        match scope {
            "user" | "session" => {
                upsert_user_env_file(name, value)?;
                std::env::set_var(name, value);
                Ok(())
            }
            "system" => Err(
                "System environment variables require elevation. Use elevated apply.".into(),
            ),
            _ => Err(format!("Invalid scope: {}", scope)),
        }
    }

    fn delete_variable(&self, name: &str, scope: &str) -> Result<(), String> {
        match scope {
            "user" | "session" => {
                remove_from_user_env_file(name)?;
                std::env::remove_var(name);
                Ok(())
            }
            "system" => Err(
                "System environment variables require elevation. Use elevated apply.".into(),
            ),
            _ => Err(format!("Invalid scope: {}", scope)),
        }
    }

    fn apply_changes_elevated(&self, changes: &[EnvChange]) -> Result<EnvApplyResult, String> {
        let system_changes: Vec<&EnvChange> = changes
            .iter()
            .filter(|c| c.scope == "system")
            .collect();
        if system_changes.is_empty() {
            return Ok(EnvApplyResult {
                success: true,
                message: "No system-scoped changes to apply.".into(),
                elevated: false,
            });
        }

        let temp_dir = std::env::temp_dir().join("portal-desktop");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        let changes_path = temp_dir.join(format!("env-changes-{}.json", uuid::Uuid::new_v4()));
        let payload = serde_json::to_string(&system_changes)
            .map_err(|e| format!("Failed to serialize changes: {}", e))?;
        fs::write(&changes_path, payload)
            .map_err(|e| format!("Failed to write changes file: {}", e))?;

        // Use platform elevation helper.
        let result = if cfg!(target_os = "macos") {
            run_macos_elevated(&changes_path)
        } else {
            run_linux_elevated(&changes_path)
        };

        let _ = fs::remove_file(&changes_path);

        match result {
            Ok(_) => Ok(EnvApplyResult {
                success: true,
                message: "System environment variables updated with elevated privileges.".into(),
                elevated: true,
            }),
            Err(err) => {
                if err.to_lowercase().contains("cancel") {
                    Ok(EnvApplyResult {
                        success: false,
                        message: "Elevation was cancelled. System variables were not changed."
                            .into(),
                        elevated: false,
                    })
                } else {
                    Err(err)
                }
            }
        }
    }

    fn refresh_process_environment(&self) -> Result<(), String> {
        if let Ok(user_vars) = read_user_env_file() {
            for (name, value) in user_vars {
                std::env::set_var(&name, &value);
            }
        }
        Ok(())
    }
}

fn user_env_file() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".portal-desktop-env.sh");
    }
    std::env::temp_dir().join(".portal-desktop-env.sh")
}

fn read_user_env_file() -> Result<HashMap<String, String>, String> {
    let path = user_env_file();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read user env file: {}", e))?;
    parse_export_file(&content)
}

fn upsert_user_env_file(name: &str, value: &str) -> Result<(), String> {
    let path = user_env_file();
    let mut vars = read_user_env_file().unwrap_or_default();
    vars.insert(name.to_string(), value.to_string());
    write_export_file(&path, &vars)
}

fn remove_from_user_env_file(name: &str) -> Result<(), String> {
    let path = user_env_file();
    let mut vars = read_user_env_file().unwrap_or_default();
    vars.remove(name);
    write_export_file(&path, &vars)
}

fn write_export_file(path: &PathBuf, vars: &HashMap<String, String>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create env file directory: {}", e))?;
    }
    let mut lines = vec![
        "# Managed by Portal Desktop — environment variable editor".to_string(),
    ];
    let mut keys: Vec<_> = vars.keys().collect();
    keys.sort();
    for key in keys {
        let value = &vars[key];
        lines.push(format!("export {}='{}'", key, value.replace('\'', "'\\''")));
    }
    fs::write(path, lines.join("\n") + "\n")
        .map_err(|e| format!("Failed to write user env file: {}", e))
}

fn parse_export_file(content: &str) -> Result<HashMap<String, String>, String> {
    let mut vars = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("export ") {
            if let Some((name, value)) = rest.split_once('=') {
                let name = name.trim().to_string();
                let value = value
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();
                if !name.is_empty() {
                    vars.insert(name, value);
                }
            }
        }
    }
    Ok(vars)
}

fn run_linux_elevated(changes_path: &PathBuf) -> Result<(), String> {
    use crate::process_ext::NoWindowExt;
    let script = format!(
        r#"
python3 -c "
import json, os, re
path = r'{path}'
with open(path) as f:
    changes = json.load(f)
env_path = '/etc/environment'
existing = {{}}
if os.path.exists(env_path):
    with open(env_path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#') or '=' not in line:
                continue
            k, v = line.split('=', 1)
            existing[k.strip()] = v.strip().strip('\"')
for c in changes:
    if c['action'] == 'set':
        existing[c['name']] = c['value']
    elif c['action'] == 'delete':
        existing.pop(c['name'], None)
with open(env_path, 'w') as f:
    for k in sorted(existing):
        f.write(f'{{k}}=\"{{existing[k]}}\"\n')
"
"#,
        path = changes_path.to_string_lossy()
    );

    let output = std::process::Command::new("pkexec")
        .no_window()
        .args(["bash", "-c", &script])
        .output()
        .or_else(|_| {
            std::process::Command::new("sudo")
                .no_window()
                .args(["bash", "-c", &script])
                .output()
        })
        .map_err(|e| format!("Failed to run elevation helper: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

fn is_unix_root() -> bool {
    use crate::process_ext::NoWindowExt;
    std::process::Command::new("id")
        .no_window()
        .args(["-u"])
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim() == "0")
        })
        .unwrap_or(false)
}

fn run_macos_elevated(changes_path: &PathBuf) -> Result<(), String> {
    use crate::process_ext::NoWindowExt;
    let inner = format!(
        r#"do shell script "python3 -c \"import json; p=r'{path}'; c=json.load(open(p)); [__import__('os').environ.__setitem__(x['name'], x['value']) for x in c if x['action']=='set']\"" with administrator privileges"#,
        path = changes_path.to_string_lossy()
    );
    let output = std::process::Command::new("osascript")
        .no_window()
        .args(["-e", &inner])
        .output()
        .map_err(|e| format!("Failed to run osascript elevation: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}
