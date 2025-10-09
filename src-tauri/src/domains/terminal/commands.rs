use std::process::Command;
use std::collections::HashMap;
use tauri::{command, State, Window};
use crate::domains::terminal::types::*;
use crate::domains::terminal::manager::TerminalManager;
use crate::domains::terminal::shell_integration::ShellHooks;
use serde::{Deserialize, Serialize};

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
    let processes = processes.lock().unwrap();
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
            "wsl.exe"
        ];
        
        for shell in terminal_shells {
            if let Ok(_) = Command::new("where").arg(shell).output() {
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
                            if name.ends_with(".exe") && (
                                name == "cmd.exe" ||
                                name == "powershell.exe" ||
                                name == "powershell_ise.exe" ||
                                name == "pwsh.exe" ||
                                name == "bash.exe" ||
                                name == "wsl.exe" ||
                                name == "zsh.exe" ||
                                name == "fish.exe"
                            ) {
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
            let wt_profiles_path = format!("{}\\Packages\\Microsoft.WindowsTerminal_8wekyb3d8bbwe\\LocalState\\settings.json", profiles_path);
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
                            let settings_path = format!("{}\\{}\\LocalState\\settings.json", program_files, name);
                            if let Ok(contents) = std::fs::read_to_string(&settings_path) {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) {
                                    if let Some(profiles_obj) = parsed.get("profiles").and_then(|p| p.get("list")) {
                                        profiles.insert("windows_terminal".to_string(), profiles_obj.clone());
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
                _ => shell.trim_end_matches(".exe")
            };
            
            let args = match shell.as_str() {
                "powershell.exe" | "pwsh.exe" => vec!["-NoLogo", "-NoProfile"],
                "cmd.exe" => vec!["/k"],
                "bash.exe" => vec!["-l"],
                "wsl.exe" => vec![],
                _ => vec![]
            };
            
            shell_profiles.insert(profile_name.to_string(), serde_json::json!({
                "command": shell,
                "args": args,
                "icon": shell.trim_end_matches(".exe").to_lowercase()
            }));
        }
        profiles.insert("available_shells".to_string(), serde_json::Value::Object(shell_profiles));
        
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
                _ => shell
            };
            
            let args = match shell.as_str() {
                "bash" | "zsh" | "sh" | "dash" => vec!["-l"],
                "fish" | "tcsh" | "csh" => vec![],
                _ => vec![]
            };
            
            shell_profiles.insert(profile_name.to_string(), serde_json::json!({
                "command": shell,
                "args": args,
                "icon": shell
            }));
        }
        profiles.insert("available_shells".to_string(), serde_json::Value::Object(shell_profiles));
        
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
                (".profile", "sh")
            ];
            
            for (config_file, shell_type) in config_files {
                let config_path = format!("{}/{}", home, config_file);
                if std::path::Path::new(&config_path).exists() {
                    user_profiles.insert(config_file.to_string(), serde_json::json!({
                        "path": config_path,
                        "type": shell_type,
                        "shell": shell_type
                    }));
                }
            }
            
            if !user_profiles.is_empty() {
                profiles.insert("user_profiles".to_string(), serde_json::Value::Object(user_profiles));
            }
        }
    }
    
    serde_json::Value::Object(profiles)
}

#[command]
pub async fn get_shell_integration_hooks() -> Result<ShellHooks, String> {
    Ok(ShellHooks::new())
}

// Command History Persistence
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

// In-memory storage for command history (in production, use a database)
static COMMAND_HISTORY: std::sync::Lazy<std::sync::Mutex<HashMap<String, Vec<CommandHistoryEntry>>>> = 
    std::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

#[command]
pub async fn save_command_history(tab_id: String, entries: Vec<CommandHistoryEntry>) -> Result<(), String> {
    let mut history = COMMAND_HISTORY.lock().map_err(|e| format!("Failed to lock history: {}", e))?;
    history.insert(tab_id, entries);
    println!("Saved command history for tab: {}", tab_id);
    Ok(())
}

#[command]
pub async fn load_command_history(tab_id: String) -> Result<Vec<CommandHistoryEntry>, String> {
    let history = COMMAND_HISTORY.lock().map_err(|e| format!("Failed to lock history: {}", e))?;
    let entries = history.get(&tab_id).cloned().unwrap_or_default();
    println!("Loaded {} command history entries for tab: {}", entries.len(), tab_id);
    Ok(entries)
}

#[command]
pub async fn clear_command_history(tab_id: Option<String>) -> Result<(), String> {
    let mut history = COMMAND_HISTORY.lock().map_err(|e| format!("Failed to lock history: {}", e))?;
    if let Some(tab) = tab_id {
        history.remove(&tab);
        println!("Cleared command history for tab: {}", tab);
    } else {
        history.clear();
        println!("Cleared all command history");
    }
    Ok(())
}
