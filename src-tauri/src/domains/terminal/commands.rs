use crate::domains::terminal::manager::TerminalManager;
use crate::domains::terminal::types::*;
use crate::database::DatabaseManager;
use crate::entities::terminal_command_history as terminal_command_history_entity;
use crate::entities::terminal_note as terminal_note_entity;
use crate::entities::terminal_session as terminal_session_entity;
use serde::{Deserialize, Serialize};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::HashMap;
use crate::process_ext::NoWindowExt;
use std::process::Command;
use std::sync::Arc;
use tauri::{command, State, Window};

#[command]
pub async fn create_terminal_process(
    request: CreateProcessRequest,
    manager: State<'_, TerminalManager>,
    window: Window,
) -> Result<TerminalProcess, String> {
    manager.create_process(request, window).await
}

#[command]
pub async fn send_terminal_input(
    process_id: String,
    input: String,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.send_input(process_id, input).await
}

#[command]
pub async fn execute_command(
    request: ExecuteCommandRequest,
    manager: State<'_, TerminalManager>,
    window: Window,
) -> Result<String, String> {
    manager.execute_command(request, window).await
}

#[command]
pub async fn kill_terminal_process(
    process_id: String,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.kill_process(process_id).await
}

#[command]
pub async fn get_terminal_processes(
    manager: State<'_, TerminalManager>,
) -> Result<Vec<TerminalProcess>, String> {
    manager.get_all_processes().await
}

#[command]
pub async fn get_terminal_process(
    process_id: String,
    manager: State<'_, TerminalManager>,
) -> Result<Option<TerminalProcess>, String> {
    manager.get_process(process_id).await
}

#[command]
pub async fn get_process_exit_code(
    process_id: String,
    manager: State<'_, TerminalManager>,
) -> Result<Option<i32>, String> {
    let processes = manager.get_processes();
    let processes = processes.lock().await;
    if let Some(process) = processes.get(&process_id) {
        Ok(process.exit_code)
    } else {
        Err(format!("Process {} not found", process_id))
    }
}

#[command]
pub async fn resize_terminal(
    process_id: String,
    cols: u32,
    rows: u32,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    manager.resize_terminal(process_id, cols, rows).await
}

#[command]
pub async fn add_command_interceptor(
    interceptor: CommandInterceptor,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    let _ = manager.add_command_interceptor(interceptor).await;
    Ok(())
}

#[command]
pub async fn remove_command_interceptor(
    pattern: String,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    let _ = manager.remove_command_interceptor(pattern).await;
    Ok(())
}

#[command]
pub async fn add_output_parser(
    parser: OutputParser,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    let _ = manager.add_output_parser(parser).await;
    Ok(())
}

#[command]
pub async fn remove_output_parser(
    pattern: String,
    manager: State<'_, TerminalManager>,
) -> Result<(), String> {
    let _ = manager.remove_output_parser(pattern).await;
    Ok(())
}

#[command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let available_shells = get_available_shells().await;
    let default_shell = get_default_shell().await;

    let info = serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "shell": default_shell,
        "working_directory": std::env::current_dir().unwrap_or_default().to_string_lossy(),
        "available_shells": available_shells,
        "terminal_profiles": get_terminal_profiles().await
    });

    Ok(info)
}

async fn get_available_shells() -> Vec<String> {
    let mut shells = Vec::new();
    println!("Detecting available shells on Windows...");

    if cfg!(target_os = "windows") {
        // Check for specific terminal shells only
        let terminal_shells = vec![
            "cmd.exe",
            "powershell.exe",
            "powershell_ise.exe",
            "pwsh.exe",
            "bash.exe",
            "wsl.exe",
        ];

        for shell in terminal_shells {
            if let Ok(_) = Command::new("where").no_window().arg(shell).output() {
                if !shells.contains(&shell.to_string()) {
                    shells.push(shell.to_string());
                    println!("Found terminal shell: {}", shell);
                }
            }
        }

        // Also check PATH environment for common shells
        if let Ok(path_env) = std::env::var("PATH") {
            for path_dir in path_env.split(';') {
                if let Ok(entries) = std::fs::read_dir(path_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // Only include known terminal shells
                            if name.ends_with(".exe")
                                && (name == "cmd.exe"
                                    || name == "powershell.exe"
                                    || name == "powershell_ise.exe"
                                    || name == "pwsh.exe"
                                    || name == "bash.exe"
                                    || name == "wsl.exe"
                                    || name == "zsh.exe"
                                    || name == "fish.exe")
                            {
                                if !shells.contains(&name.to_string()) {
                                    shells.push(name.to_string());
                                    println!("Found terminal shell in PATH: {}", name);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Read /etc/shells for available shells
        if let Ok(contents) = std::fs::read_to_string("/etc/shells") {
            for line in contents.lines() {
                let shell = line.trim();
                if !shell.is_empty() && !shell.starts_with('#') {
                    if let Some(shell_name) = std::path::Path::new(shell).file_name() {
                        if let Some(name) = shell_name.to_str() {
                            if !shells.contains(&name.to_string()) {
                                shells.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Also check which command for additional shells
        if let Ok(output) = Command::new("which")
            .args(&["-a", "bash", "zsh", "fish", "sh", "dash", "tcsh", "csh"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if let Some(shell_name) = std::path::Path::new(line.trim()).file_name() {
                        if let Some(name) = shell_name.to_str() {
                            if !shells.contains(&name.to_string()) {
                                shells.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to default if none found
    if shells.is_empty() {
        println!("No shells found, using fallback");
        shells.push(if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            "bash".to_string()
        });
    }

    println!("Final shells list: {:?}", shells);
    shells
}

async fn get_default_shell() -> String {
    if cfg!(target_os = "windows") {
        // Try to get the default shell from environment
        if let Ok(shell) = std::env::var("COMSPEC") {
            return shell;
        }
        if let Ok(shell) = std::env::var("SHELL") {
            return shell;
        }
        "powershell.exe".to_string()
    } else {
        // Try to get the default shell from environment
        if let Ok(shell) = std::env::var("SHELL") {
            return shell;
        }
        // Try to get from /etc/passwd or similar
        if let Ok(output) = Command::new("getent")
            .args(&["passwd", &std::env::var("USER").unwrap_or_default()])
            .output()
        {
            if let Ok(passwd_line) = String::from_utf8(output.stdout) {
                if let Some(shell) = passwd_line.split(':').nth(6) {
                    if !shell.is_empty() && shell != "/bin/false" {
                        return shell.to_string();
                    }
                }
            }
        }
        "bash".to_string()
    }
}

async fn get_terminal_profiles() -> serde_json::Value {
    let mut profiles = serde_json::Map::new();
    let available_shells = get_available_shells().await;

    if cfg!(target_os = "windows") {
        // Windows Terminal profiles - read from actual Windows Terminal settings
        if let Ok(profiles_path) = std::env::var("LOCALAPPDATA") {
            let wt_profiles_path = format!(
                "{}\\Packages\\Microsoft.WindowsTerminal_8wekyb3d8bbwe\\LocalState\\settings.json",
                profiles_path
            );
            if let Ok(contents) = std::fs::read_to_string(&wt_profiles_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(profiles_obj) = parsed.get("profiles").and_then(|p| p.get("list")) {
                        profiles.insert("windows_terminal".to_string(), profiles_obj.clone());
                    }
                }
            }
        }

        // Also check for Windows Terminal in Program Files
        if let Ok(program_files) = std::env::var("PROGRAMFILES") {
            // Use glob pattern to find Windows Terminal settings
            if let Ok(entries) = std::fs::read_dir(&program_files) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with("Microsoft.WindowsTerminal_") {
                            let settings_path =
                                format!("{}\\{}\\LocalState\\settings.json", program_files, name);
                            if let Ok(contents) = std::fs::read_to_string(&settings_path) {
                                if let Ok(parsed) =
                                    serde_json::from_str::<serde_json::Value>(&contents)
                                {
                                    if let Some(profiles_obj) =
                                        parsed.get("profiles").and_then(|p| p.get("list"))
                                    {
                                        profiles.insert(
                                            "windows_terminal".to_string(),
                                            profiles_obj.clone(),
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Generate profiles from available shells
        let mut shell_profiles = serde_json::Map::new();
        for shell in &available_shells {
            let profile_name = match shell.as_str() {
                "powershell.exe" => "PowerShell",
                "pwsh.exe" => "PowerShell Core",
                "cmd.exe" => "Command Prompt",
                "bash.exe" => "Git Bash",
                "wsl.exe" => "WSL",
                _ => shell.trim_end_matches(".exe"),
            };

            let args = match shell.as_str() {
                "powershell.exe" | "pwsh.exe" => vec!["-NoLogo", "-NoProfile"],
                "cmd.exe" => vec!["/k"],
                "bash.exe" => vec!["-l"],
                "wsl.exe" => vec![],
                _ => vec![],
            };

            shell_profiles.insert(
                profile_name.to_string(),
                serde_json::json!({
                    "command": shell,
                    "args": args,
                    "icon": shell.trim_end_matches(".exe").to_lowercase()
                }),
            );
        }
        profiles.insert(
            "available_shells".to_string(),
            serde_json::Value::Object(shell_profiles),
        );
    } else {
        // Unix terminal profiles - read from actual system
        let mut shell_profiles = serde_json::Map::new();
        for shell in &available_shells {
            let profile_name = match shell.as_str() {
                "bash" => "Bash",
                "zsh" => "Zsh",
                "fish" => "Fish",
                "sh" => "Sh",
                "dash" => "Dash",
                "tcsh" => "Tcsh",
                "csh" => "Csh",
                _ => shell,
            };

            let args = match shell.as_str() {
                "bash" | "zsh" | "sh" | "dash" => vec!["-l"],
                "fish" | "tcsh" | "csh" => vec![],
                _ => vec![],
            };

            shell_profiles.insert(
                profile_name.to_string(),
                serde_json::json!({
                    "command": shell,
                    "args": args,
                    "icon": shell
                }),
            );
        }
        profiles.insert(
            "available_shells".to_string(),
            serde_json::Value::Object(shell_profiles),
        );

        // Read shell-specific profiles from user home
        if let Ok(home) = std::env::var("HOME") {
            let mut user_profiles = serde_json::Map::new();

            // Check for various shell config files
            let config_files = vec![
                (".bashrc", "bash"),
                (".bash_profile", "bash"),
                (".zshrc", "zsh"),
                (".zprofile", "zsh"),
                (".fishrc", "fish"),
                (".config/fish/config.fish", "fish"),
                (".profile", "sh"),
            ];

            for (config_file, shell_type) in config_files {
                let config_path = format!("{}/{}", home, config_file);
                if std::path::Path::new(&config_path).exists() {
                    user_profiles.insert(
                        config_file.to_string(),
                        serde_json::json!({
                            "path": config_path,
                            "type": shell_type,
                            "shell": shell_type
                        }),
                    );
                }
            }

            if !user_profiles.is_empty() {
                profiles.insert(
                    "user_profiles".to_string(),
                    serde_json::Value::Object(user_profiles),
                );
            }
        }
    }

    serde_json::Value::Object(profiles)
}

// Command History Persistence (DB-backed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryEntry {
    pub id: String,
    pub tab_id: String,
    pub timestamp: String,
    pub command: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub duration: Option<u64>,
    pub intercepted: Option<bool>,
}

#[command]
pub async fn save_command_history(
    tab_id: String,
    entries: Vec<CommandHistoryEntry>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();

    // Replace all history for this tab.
    terminal_command_history_entity::Entity::delete_many()
        .filter(terminal_command_history_entity::Column::TabId.eq(tab_id.clone()))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    for entry in entries {
        let active = terminal_command_history_entity::ActiveModel {
            id: Set(entry.id),
            tab_id: Set(tab_id.clone()),
            command: Set(entry.command),
            output: Set(entry.output),
            timestamp: Set(entry.timestamp),
            exit_code: Set(entry.exit_code),
            duration_ms: Set(entry.duration.map(|d| d as i64)),
            intercepted: Set(entry.intercepted),
        };

        terminal_command_history_entity::Entity::insert(active)
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;
    }

    println!("Saved command history for tab: {}", tab_id);
    Ok(())
}

#[command]
pub async fn load_command_history(
    tab_id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<CommandHistoryEntry>, String> {
    let db = db_manager.get_connection();

    let rows = terminal_command_history_entity::Entity::find()
        .filter(terminal_command_history_entity::Column::TabId.eq(tab_id.clone()))
        .order_by_desc(terminal_command_history_entity::Column::Timestamp)
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let entries: Vec<CommandHistoryEntry> = rows
        .into_iter()
        .map(|m| CommandHistoryEntry {
            id: m.id,
            tab_id: m.tab_id,
            timestamp: m.timestamp,
            command: m.command,
            output: m.output,
            exit_code: m.exit_code,
            duration: m.duration_ms.map(|d| d as u64),
            intercepted: m.intercepted,
        })
        .collect();

    println!(
        "Loaded {} command history entries for tab: {}",
        entries.len(),
        tab_id
    );

    Ok(entries)
}

#[command]
pub async fn clear_command_history(
    tab_id: Option<String>,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();

    if let Some(tab) = tab_id {
        terminal_command_history_entity::Entity::delete_many()
            .filter(terminal_command_history_entity::Column::TabId.eq(tab.clone()))
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;

        println!("Cleared command history for tab: {}", tab);
    } else {
        terminal_command_history_entity::Entity::delete_many()
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;

        println!("Cleared all command history");
    }

    Ok(())
}

// Session State Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub tab_id: String,
    pub working_directory: String,
    pub environment: std::collections::HashMap<String, String>,
    pub scrollback_buffer: Vec<String>,
    pub cursor_position: (u16, u16),
    pub terminal_size: (u16, u16),
    pub last_activity: String,
    pub process_id: Option<String>,
}

#[command]
pub async fn save_terminal_session(
    session: TerminalSession,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();

    let environment_json = serde_json::to_string(&session.environment).map_err(|e| e.to_string())?;
    let scrollback_buffer_json =
        serde_json::to_string(&session.scrollback_buffer).map_err(|e| e.to_string())?;

    // Replace existing row for this tab.
    terminal_session_entity::Entity::delete_many()
        .filter(
            terminal_session_entity::Column::TabId.eq(session.tab_id.clone()),
        )
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let active = terminal_session_entity::ActiveModel {
        tab_id: Set(session.tab_id.clone()),
        working_directory: Set(session.working_directory.clone()),
        environment_json: Set(environment_json),
        scrollback_buffer_json: Set(scrollback_buffer_json),
        cursor_x: Set(session.cursor_position.0 as i32),
        cursor_y: Set(session.cursor_position.1 as i32),
        terminal_cols: Set(session.terminal_size.0 as i32),
        terminal_rows: Set(session.terminal_size.1 as i32),
        last_activity: Set(session.last_activity.clone()),
        process_id: Set(session.process_id.clone()),
    };

    terminal_session_entity::Entity::insert(active)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    println!("Saved terminal session for tab: {}", session.tab_id);
    Ok(())
}

#[command]
pub async fn load_terminal_session(
    tab_id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Option<TerminalSession>, String> {
    let db = db_manager.get_connection();

    let row = terminal_session_entity::Entity::find_by_id(tab_id.clone())
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = row {
        let environment: HashMap<String, String> =
            serde_json::from_str(&row.environment_json).map_err(|e| e.to_string())?;
        let scrollback_buffer: Vec<String> =
            serde_json::from_str(&row.scrollback_buffer_json).map_err(|e| e.to_string())?;

        println!("Loaded terminal session for tab: {}", tab_id);

        Ok(Some(TerminalSession {
            tab_id: row.tab_id,
            working_directory: row.working_directory,
            environment,
            scrollback_buffer,
            cursor_position: (row.cursor_x as u16, row.cursor_y as u16),
            terminal_size: (row.terminal_cols as u16, row.terminal_rows as u16),
            last_activity: row.last_activity,
            process_id: row.process_id,
        }))
    } else {
        Ok(None)
    }
}

#[command]
pub async fn list_terminal_sessions(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<String>, String> {
    let db = db_manager.get_connection();
    let rows = terminal_session_entity::Entity::find()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| r.tab_id).collect())
}

#[command]
pub async fn delete_terminal_session(
    tab_id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    terminal_session_entity::Entity::delete_many()
        .filter(terminal_session_entity::Column::TabId.eq(tab_id.clone()))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    println!("Deleted terminal session for tab: {}", tab_id);
    Ok(())
}

#[command]
pub async fn clear_all_sessions(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();
    terminal_session_entity::Entity::delete_many()
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    println!("Cleared all terminal sessions");
    Ok(())
}

// Terminal notes persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalNote {
    pub tab_id: String,
    pub markdown: String,
    pub updated_at: String,
}

#[command]
pub async fn save_terminal_note(
    tab_id: String,
    markdown: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let db = db_manager.get_connection();

    let updated_at = chrono::Utc::now().to_rfc3339();

    terminal_note_entity::Entity::delete_many()
        .filter(terminal_note_entity::Column::TabId.eq(tab_id.clone()))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let active = terminal_note_entity::ActiveModel {
        tab_id: Set(tab_id.clone()),
        markdown: Set(markdown),
        updated_at: Set(updated_at),
    };

    terminal_note_entity::Entity::insert(active)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn load_terminal_note(
    tab_id: String,
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<String, String> {
    let db = db_manager.get_connection();
    let row = terminal_note_entity::Entity::find_by_id(tab_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.markdown).unwrap_or_default())
}
