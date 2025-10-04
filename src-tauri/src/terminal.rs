use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{command, State, Window};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalProcess {
    pub id: String,
    pub command: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub status: String,
    pub pid: Option<u32>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub process_id: String,
    pub content: String,
    pub output_type: String, // "stdout", "stderr", "exit"
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProcessRequest {
    pub shell: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub working_directory: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

pub type ProcessMap = Arc<Mutex<HashMap<String, TerminalProcess>>>;
pub type OutputCallback = Arc<Mutex<HashMap<String, Box<dyn Fn(TerminalOutput) + Send + Sync>>>>;

#[command]
pub async fn create_terminal_process(
    request: CreateProcessRequest,
    processes: State<'_, ProcessMap>,
    window: Window,
) -> Result<TerminalProcess, String> {
    let process_id = Uuid::new_v4().to_string();
    
    // Determine shell command based on platform
    let (shell_cmd, shell_args) = if cfg!(target_os = "windows") {
        if request.shell.contains("powershell") {
            ("powershell.exe", vec!["-NoLogo", "-NoProfile"])
        } else if request.shell.contains("cmd") {
            ("cmd.exe", vec!["/k"])
        } else {
            ("powershell.exe", vec!["-NoLogo", "-NoProfile"])
        }
    } else {
        if request.shell.contains("bash") {
            ("bash", vec!["-l"])
        } else if request.shell.contains("zsh") {
            ("zsh", vec!["-l"])
        } else {
            ("bash", vec!["-l"])
        }
    };

    let mut process = TerminalProcess {
        id: process_id.clone(),
        command: format!("{} {}", shell_cmd, shell_args.join(" ")),
        working_directory: request.working_directory,
        environment: request.environment,
        status: "starting".to_string(),
        pid: None,
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: None,
        exit_code: None,
    };

    // Store the process
    {
        let mut processes = processes.lock().unwrap();
        processes.insert(process_id.clone(), process.clone());
    }

    // Start the actual process in a separate thread
    let processes_clone = processes.inner().clone();
    let window_clone = window.clone();
    
    thread::spawn(move || {
        let mut child = match Command::new(shell_cmd)
            .args(&shell_args)
            .current_dir(&request.working_directory)
            .envs(&request.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to spawn process: {}", e);
                return;
            }
        };

        let pid = child.id();
        
        // Update process with PID
        {
            let mut processes = processes_clone.lock().unwrap();
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.pid = Some(pid);
                proc.status = "running".to_string();
            }
        }

        // Handle stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let processes_clone = processes_clone.clone();
            let window_clone = window_clone.clone();
            
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let output = TerminalOutput {
                            process_id: process_id.clone(),
                            content: line + "\n",
                            output_type: "stdout".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        // Emit to frontend
                        let _ = window_clone.emit("terminal-output", &output);
                    }
                }
            });
        }

        // Handle stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let processes_clone = processes_clone.clone();
            let window_clone = window_clone.clone();
            
            thread::spawn(move || {
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let output = TerminalOutput {
                            process_id: process_id.clone(),
                            content: line + "\n",
                            output_type: "stderr".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        // Emit to frontend
                        let _ = window_clone.emit("terminal-output", &output);
                    }
                }
            });
        }

        // Wait for process to complete
        match child.wait() {
            Ok(status) => {
                let exit_code = status.code();
                let output = TerminalOutput {
                    process_id: process_id.clone(),
                    content: format!("Process exited with code: {:?}\n", exit_code),
                    output_type: "exit".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                
                // Update process status
                {
                    let mut processes = processes_clone.lock().unwrap();
                    if let Some(proc) = processes.get_mut(&process_id) {
                        proc.status = "completed".to_string();
                        proc.end_time = Some(chrono::Utc::now().to_rfc3339());
                        proc.exit_code = exit_code;
                    }
                }
                
                // Emit exit event
                let _ = window_clone.emit("terminal-output", &output);
            }
            Err(e) => {
                eprintln!("Error waiting for process: {}", e);
            }
        }
    });

    Ok(process)
}

#[command]
pub async fn send_terminal_input(
    process_id: String,
    input: String,
    processes: State<'_, ProcessMap>,
) -> Result<(), String> {
    // For now, we'll execute the input as a command
    // In a real PTY implementation, we'd write to the process stdin
    execute_command(
        ExecuteCommandRequest {
            command: input.trim().to_string(),
            working_directory: None,
            environment: None,
        },
        processes,
    ).await?;
    
    Ok(())
}

#[command]
pub async fn execute_command(
    request: ExecuteCommandRequest,
    processes: State<'_, ProcessMap>,
) -> Result<String, String> {
    let working_dir = request.working_directory.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/"))
            .to_string_lossy()
            .to_string()
    });

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/c", &request.command])
            .current_dir(&working_dir)
            .envs(request.environment.unwrap_or_default())
            .output()
    } else {
        Command::new("sh")
            .args(&["-c", &request.command])
            .current_dir(&working_dir)
            .envs(request.environment.unwrap_or_default())
            .output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stderr.is_empty() {
                Ok(format!("{}\n{}", stdout, stderr))
            } else {
                Ok(stdout.to_string())
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

#[command]
pub async fn kill_terminal_process(
    process_id: String,
    processes: State<'_, ProcessMap>,
) -> Result<(), String> {
    let mut processes = processes.lock().unwrap();
    if let Some(process) = processes.get_mut(&process_id) {
        if let Some(pid) = process.pid {
            // Kill the process
            if cfg!(target_os = "windows") {
                let _ = Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .output();
            } else {
                let _ = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output();
            }
        }
        
        process.status = "killed".to_string();
        process.end_time = Some(chrono::Utc::now().to_rfc3339());
    }
    
    Ok(())
}

#[command]
pub async fn get_terminal_processes(
    processes: State<'_, ProcessMap>,
) -> Result<Vec<TerminalProcess>, String> {
    let processes = processes.lock().unwrap();
    Ok(processes.values().cloned().collect())
}

#[command]
pub async fn get_terminal_process(
    process_id: String,
    processes: State<'_, ProcessMap>,
) -> Result<Option<TerminalProcess>, String> {
    let processes = processes.lock().unwrap();
    Ok(processes.get(&process_id).cloned())
}

#[command]
pub async fn resize_terminal(
    process_id: String,
    cols: u32,
    rows: u32,
    processes: State<'_, ProcessMap>,
) -> Result<(), String> {
    // In a real PTY implementation, we'd send SIGWINCH to resize the terminal
    // For now, we'll just acknowledge the resize
    let _processes = processes.lock().unwrap();
    println!("Resizing terminal {} to {}x{}", process_id, cols, rows);
    Ok(())
}

#[command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let info = serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "shell": if cfg!(target_os = "windows") { "powershell.exe" } else { "bash" },
        "working_directory": std::env::current_dir().unwrap_or_default().to_string_lossy(),
        "available_shells": if cfg!(target_os = "windows") {
            vec!["powershell.exe", "cmd.exe", "bash"]
        } else {
            vec!["bash", "zsh", "sh"]
        }
    });
    
    Ok(info)
}
