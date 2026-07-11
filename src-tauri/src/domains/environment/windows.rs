use std::fs;
use std::path::PathBuf;

use crate::domains::environment::types::{EnvApplyResult, EnvChange, EnvPermissions, EnvVariable};

use super::platform::PlatformEnv;

pub struct WindowsEnv;

impl PlatformEnv for WindowsEnv {
    fn list_variables(&self) -> Result<Vec<EnvVariable>, String> {
        let script = r#"
$user = [Environment]::GetEnvironmentVariables('User')
$machine = [Environment]::GetEnvironmentVariables('Machine')
$result = @()
foreach ($key in $user.Keys) {
  $result += [PSCustomObject]@{ name = $key; value = [string]$user[$key]; scope = 'user' }
}
foreach ($key in $machine.Keys) {
  $result += [PSCustomObject]@{ name = $key; value = [string]$machine[$key]; scope = 'system' }
}
$result | ConvertTo-Json -Compress
"#;
        let output = run_powershell(script)?;
        if output.trim().is_empty() {
            return Ok(Vec::new());
        }
        parse_variable_json(&output)
    }

    fn get_permissions(&self) -> Result<EnvPermissions, String> {
        let script = r#"
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
[PSCustomObject]@{
  canEditUser = $true
  canEditSystem = $isAdmin
  isElevated = $isAdmin
  platform = 'windows'
} | ConvertTo-Json -Compress
"#;
        let output = run_powershell(script)?;
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse permissions: {}", e))
    }

    fn set_variable(&self, name: &str, value: &str, scope: &str) -> Result<(), String> {
        let target = scope_target(scope)?;
        let script = format!(
            "[Environment]::SetEnvironmentVariable('{}', '{}', '{}')",
            escape_ps_single(name),
            escape_ps_single(value),
            target
        );
        run_powershell(&script)?;
        Ok(())
    }

    fn delete_variable(&self, name: &str, scope: &str) -> Result<(), String> {
        let target = scope_target(scope)?;
        let script = format!(
            "[Environment]::SetEnvironmentVariable('{}', $null, '{}')",
            escape_ps_single(name),
            target
        );
        run_powershell(&script)?;
        Ok(())
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

        let changes_path_str = changes_path.to_string_lossy().replace('\'', "''");
        let helper_script = format!(
            r#"
$changes = Get-Content -Raw -LiteralPath '{path}' | ConvertFrom-Json
foreach ($c in $changes) {{
  if ($c.action -eq 'set') {{
    [Environment]::SetEnvironmentVariable($c.name, $c.value, 'Machine')
  }} elseif ($c.action -eq 'delete') {{
    [Environment]::SetEnvironmentVariable($c.name, $null, 'Machine')
  }}
}}
"#,
            path = changes_path_str
        );

        let helper_path = temp_dir.join(format!("env-elevated-{}.ps1", uuid::Uuid::new_v4()));
        fs::write(&helper_path, helper_script)
            .map_err(|e| format!("Failed to write helper script: {}", e))?;

        let helper_path_str = helper_path.to_string_lossy().replace('\'', "''");
        let launch_script = format!(
            r#"
$proc = Start-Process -FilePath 'powershell.exe' -Verb RunAs -Wait -PassThru -ArgumentList @(
  '-NoProfile',
  '-ExecutionPolicy', 'Bypass',
  '-File', '{path}'
)
if ($proc.ExitCode -ne 0) {{
  exit $proc.ExitCode
}}
exit 0
"#,
            path = helper_path_str
        );

        match run_powershell(&launch_script) {
            Ok(_) => {
                let _ = fs::remove_file(&changes_path);
                let _ = fs::remove_file(&helper_path);
                Ok(EnvApplyResult {
                    success: true,
                    message: "System environment variables updated with elevated privileges."
                        .into(),
                    elevated: true,
                })
            }
            Err(err) => {
                let _ = fs::remove_file(&changes_path);
                let _ = fs::remove_file(&helper_path);
                if err.to_lowercase().contains("canceled")
                    || err.to_lowercase().contains("cancelled")
                    || err.to_lowercase().contains("1223")
                {
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
        let script = r#"
$user = [Environment]::GetEnvironmentVariables('User')
$machine = [Environment]::GetEnvironmentVariables('Machine')
$all = @{}
foreach ($key in $machine.Keys) { $all[$key] = [string]$machine[$key] }
foreach ($key in $user.Keys) { $all[$key] = [string]$user[$key] }
$all | ConvertTo-Json -Compress
"#;
        let output = run_powershell(script)?;
        if output.trim().is_empty() {
            return Ok(());
        }
        let vars: Vec<EnvVariable> = parse_variable_json(&output)?;
        for var in vars {
            std::env::set_var(&var.name, &var.value);
        }
        Ok(())
    }
}

fn scope_target(scope: &str) -> Result<&'static str, String> {
    match scope {
        "user" => Ok("User"),
        "system" => Ok("Machine"),
        _ => Err(format!("Invalid scope: {}", scope)),
    }
}

fn escape_ps_single(value: &str) -> String {
    value.replace('\'', "''")
}

fn run_powershell(script: &str) -> Result<String, String> {
    use crate::process_ext::NoWindowExt;
    let output = std::process::Command::new("powershell")
        .no_window()
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let message = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("PowerShell exited with code {:?}", output.status.code())
        };
        Err(message)
    }
}

fn parse_variable_json(output: &str) -> Result<Vec<EnvVariable>, String> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("Failed to parse env JSON: {}", e))?;

    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        _ => Vec::new(),
    };

    let mut vars = Vec::new();
    for item in items {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let value = item
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let scope = item
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string();
        vars.push(EnvVariable { name, value, scope });
    }

    vars.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(vars)
}

#[allow(dead_code)]
pub fn temp_changes_path() -> PathBuf {
    std::env::temp_dir()
        .join("portal-desktop")
        .join("env-changes.json")
}
