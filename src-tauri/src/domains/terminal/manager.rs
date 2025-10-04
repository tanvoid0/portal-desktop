use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Window, Emitter};
use uuid::Uuid;
use crate::domains::terminal::types::*;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};

// Global state for interactive processes (PTY-backed)
use once_cell::sync::Lazy;
static INTERACTIVE_PROCESSES: Lazy<std::sync::Mutex<HashMap<String, Box<dyn portable_pty::Child + Send>>>> = 
    Lazy::new(|| std::sync::Mutex::new(HashMap::new()));
static MASTER_PTYS: Lazy<std::sync::Mutex<HashMap<String, Box<dyn MasterPty + Send>>>> =
    Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

pub type ProcessMap = Arc<Mutex<HashMap<String, TerminalProcess>>>;
pub type OutputCallbacks = Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(TerminalOutput) + Send + Sync>>>>>;
pub type StdinHandles = Arc<Mutex<HashMap<String, Box<dyn Write + Send>>>>;

pub struct TerminalManager {
    processes: ProcessMap,
    output_callbacks: OutputCallbacks,
    stdin_handles: StdinHandles,
    command_interceptors: Arc<Mutex<Vec<CommandInterceptor>>>,
    output_parsers: Arc<Mutex<Vec<OutputParser>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            output_callbacks: Arc::new(Mutex::new(HashMap::new())),
            stdin_handles: Arc::new(Mutex::new(HashMap::new())),
            command_interceptors: Arc::new(Mutex::new(Vec::new())),
            output_parsers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_processes(&self) -> ProcessMap {
        self.processes.clone()
    }

    pub fn get_output_callbacks(&self) -> OutputCallbacks {
        self.output_callbacks.clone()
    }

    pub async fn create_process(
        &self,
        request: CreateProcessRequest,
        window: Window,
    ) -> Result<TerminalProcess, String> {
        let process_id = Uuid::new_v4().to_string();
        
        println!("Creating process with shell: {}", request.shell);
        
        // Use the raw shell command and add appropriate arguments internally
        let (shell_cmd, shell_args) = {
            let shell_lower = request.shell.to_lowercase();
            if cfg!(target_os = "windows") {
                if shell_lower.contains("cmd") || shell_lower == "cmd.exe" {
                    ("cmd.exe".to_string(), vec!["/k".to_string()])
                } else if shell_lower.contains("powershell") || shell_lower == "powershell.exe" {
                    ("powershell.exe".to_string(), vec!["-NoLogo".to_string(), "-NoProfile".to_string(), "-NoExit".to_string()])
                } else if shell_lower.contains("pwsh") || shell_lower == "pwsh.exe" {
                    ("pwsh.exe".to_string(), vec!["-NoLogo".to_string(), "-NoProfile".to_string(), "-NoExit".to_string()])
                } else if shell_lower.contains("bash") || shell_lower == "bash.exe" {
                    ("bash.exe".to_string(), vec![])
                } else if shell_lower.contains("wsl") {
                    ("wsl.exe".to_string(), vec![])
                } else {
                    // For any other shell, try to use it as-is
                    (request.shell.clone(), vec![])
                }
            } else {
                if shell_lower.contains("bash") {
                    ("bash".to_string(), vec![])
                } else if shell_lower.contains("zsh") {
                    ("zsh".to_string(), vec![])
                } else if shell_lower.contains("fish") {
                    ("fish".to_string(), vec![])
                } else {
                    // For any other shell, try to use it as-is
                    (request.shell.clone(), vec![])
                }
            }
        };

        println!("Using shell command: {} with args: {:?}", shell_cmd, shell_args);

        let process = TerminalProcess {
            id: process_id.clone(),
            tab_id: request.tab_id.clone(),
            command: format!("{} {}", shell_cmd, shell_args.join(" ")),
            working_directory: request.working_directory.clone(),
            environment: request.environment.clone(),
            status: "starting".to_string(),
            pid: None,
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: None,
            exit_code: None,
        };

        // Store the process
        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(process_id.clone(), process.clone());
        }

        // Spawn the shell in a PTY and handle output streaming
        let pty_system: NativePtySystem = NativePtySystem::default();
        let size = PtySize {
            cols: request.cols as u16,
            rows: request.rows as u16,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        println!("Opening PTY with size: {}x{}", size.cols, size.rows);
        let pair = pty_system
            .openpty(size)
            .map_err(|e| {
                println!("PTY creation failed: {}", e);
                format!("Failed to open PTY: {}", e)
            })?;
        
        println!("PTY opened successfully");

        // Use request working directory and environment directly
        let working_dir = request.working_directory.clone();
        let environment = request.environment.clone();

        let mut cmd = CommandBuilder::new(&shell_cmd);
        for a in &shell_args {
            cmd.arg(a);
        }
        cmd.cwd(&working_dir);
        for (k, v) in &environment {
            cmd.env(k, v);
        }

        println!("Spawning command: {} with args: {:?}", shell_cmd, shell_args);
        println!("Working directory: {}", working_dir);
        
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| {
                println!("Failed to spawn PTY shell: {}", e);
                format!("Failed to spawn PTY shell: {}", e)
            })?;
        
        println!("Shell spawned successfully");

        // PID (if available)
        let pid = child.process_id();
        {
            let mut processes = self.processes.lock().unwrap();
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.pid = pid;
                proc.status = "running".to_string();
            }
        }

        // Take writer for input handling and store
        let master = pair.master;
        if let Ok(writer) = master.take_writer() {
            let mut stdin_handles = self.stdin_handles.lock().unwrap();
            stdin_handles.insert(process_id.clone(), writer);
            println!("Stdin writer stored for process: {}", process_id);
        } else {
            println!("Warning: Could not take writer from master PTY");
        }

        // Store the PTY child for lifecycle management
        {
            let mut processes = INTERACTIVE_PROCESSES.lock().unwrap();
            processes.insert(process_id.clone(), child);
        }
        // Store the Master PTY for resize operations
        {
            let mut masters = MASTER_PTYS.lock().unwrap();
            masters.insert(process_id.clone(), master);
        }

        // Inform frontend
        let initial_output = TerminalOutput {
            process_id: process_id.clone(),
            content: format!("PTY shell ready: {} {}\r\n", shell_cmd, shell_args.join(" ")),
            output_type: "info".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        let _ = window.emit("terminal-output", &initial_output);

        // Start PTY output streaming (single stream contains both stdout/err)
        {
            let mut masters = MASTER_PTYS.lock().unwrap();
            if let Some(m) = masters.get_mut(&process_id) {
                if let Ok(mut reader) = m.try_clone_reader() {
                    println!("Starting output streaming for process: {}", process_id);
                    let pid_for_thread = process_id.clone();
                    let window_for_reader = window.clone();
                    std::thread::spawn(move || {
                        let mut buf = [0u8; 8192];
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) => {
                                    println!("PTY reader reached EOF for process: {}", pid_for_thread);
                                    break; // EOF
                                }
                                Ok(n) => {
                                    let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let output = TerminalOutput {
                                        process_id: pid_for_thread.clone(),
                                        content: chunk,
                                        output_type: "stdout".to_string(),
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    if let Err(e) = window_for_reader.emit("terminal-output", &output) {
                                        eprintln!("Failed to emit PTY output: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("PTY read error for process {}: {}", pid_for_thread, e);
                                    break;
                                }
                            }
                        }
                    });
                } else {
                    println!("Warning: Could not clone reader from master PTY");
                }
            } else {
                println!("Warning: Master PTY not found for process: {}", process_id);
            }
        }

        // Start process monitoring thread
        self.start_process_monitoring(process_id.clone(), window.clone());

        Ok(process)
    }

    pub async fn send_input(
        &self,
        process_id: String,
        input: String,
    ) -> Result<(), String> {
        println!("send_input called: process_id={}, input={:?}", process_id, input);
        
        let mut stdin_handles = self.stdin_handles.lock().unwrap();
        
        if let Some(stdin) = stdin_handles.get_mut(&process_id) {
            println!("Writing to stdin: '{}' (bytes: {:?})", input, input.as_bytes());
            
            // Write the input as-is - frontend now sends complete commands with newlines
            if let Err(e) = stdin.write_all(input.as_bytes()) {
                println!("Error writing to stdin: {}", e);
                return Err(format!("Failed to send input: {}", e));
            }
            
            // Flush to ensure it's sent immediately
            if let Err(e) = stdin.flush() {
                println!("Error flushing stdin: {}", e);
                return Err(format!("Failed to flush stdin: {}", e));
            }
            
            println!("Successfully sent input to process");
            
            // Small delay to ensure the process has time to process the input
            std::thread::sleep(std::time::Duration::from_millis(50));
        } else {
            println!("No stdin handle found for process: {}", process_id);
            return Err("No stdin handle found for process".to_string());
        }
        
        Ok(())
    }

    pub async fn kill_process(&self, process_id: String) -> Result<(), String> {
        let mut processes = INTERACTIVE_PROCESSES.lock().unwrap();
        
        if let Some(mut child) = processes.remove(&process_id) {
            if let Err(e) = child.kill() {
                println!("Failed to kill process: {}", e);
                return Err(format!("Failed to kill process: {}", e));
            }
        }
        
        // Update process status
        {
            let mut processes = self.processes.lock().unwrap();
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.status = "killed".to_string();
                proc.end_time = Some(chrono::Utc::now().to_rfc3339());
            }
        }
        
        Ok(())
    }

    pub async fn get_process(&self, process_id: String) -> Result<Option<TerminalProcess>, String> {
        let processes = self.processes.lock().unwrap();
        Ok(processes.get(&process_id).cloned())
    }

    pub async fn get_all_processes(&self) -> Result<Vec<TerminalProcess>, String> {
        let processes = self.processes.lock().unwrap();
        Ok(processes.values().cloned().collect())
    }

    pub async fn execute_command(
        &self,
        request: ExecuteCommandRequest,
        _window: Window,
    ) -> Result<String, String> {
        // Parse the command
        let (cmd, _args) = parse_command(&request.command);
        
        if cmd.is_empty() {
            return Ok("".to_string());
        }
        
        // Execute as native OS command
        println!("Executing native command: {}", cmd);
        
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", &request.command])
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&request.command)
                .output()
        }.map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result = if output.status.success() {
            stdout.to_string()
        } else {
            format!("Error: {}\nOutput: {}", stderr, stdout)
        };

        Ok(result)
    }

    pub async fn add_command_interceptor(&self, interceptor: CommandInterceptor) -> Result<(), String> {
        let mut interceptors = self.command_interceptors.lock().unwrap();
        interceptors.push(interceptor);
        Ok(())
    }

    pub async fn remove_command_interceptor(&self, _id: String) -> Result<(), String> {
        // For now, just clear all interceptors
        let mut interceptors = self.command_interceptors.lock().unwrap();
        interceptors.clear();
        Ok(())
    }

    pub async fn add_output_parser(&self, parser: OutputParser) -> Result<(), String> {
        let mut parsers = self.output_parsers.lock().unwrap();
        parsers.push(parser);
        Ok(())
    }

    pub async fn remove_output_parser(&self, _id: String) -> Result<(), String> {
        // For now, just clear all parsers
        let mut parsers = self.output_parsers.lock().unwrap();
        parsers.clear();
        Ok(())
    }

    pub async fn resize_terminal(&self, process_id: String, cols: u32, rows: u32) -> Result<(), String> {
        let mut masters = MASTER_PTYS.lock().unwrap();
        if let Some(m) = masters.get_mut(&process_id) {
            let size = PtySize { cols: cols as u16, rows: rows as u16, pixel_width: 0, pixel_height: 0 };
            if let Err(e) = m.resize(size) {
                eprintln!("Failed to resize PTY: {}", e);
                return Err(format!("Failed to resize PTY: {}", e));
            }
            println!("Resized terminal {} to {}x{}", process_id, cols, rows);
            Ok(())
        } else {
            Err("Process not found".into())
        }
    }

    fn start_process_monitoring(&self, process_id: String, window: Window) {
        let processes = self.processes.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                
                let mut interactive_processes = INTERACTIVE_PROCESSES.lock().unwrap();
                if let Some(child) = interactive_processes.get_mut(&process_id) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Extract exit code from status
                            // portable_pty::ExitStatus doesn't expose the raw exit code
                            // For now, we'll use success/failure, but we can improve this later
                            let exit_code = if status.success() { 0 } else { 1 };
                            
                            // TODO: Find a way to get the actual exit code from portable_pty
                            // This might require switching to a different PTY library or
                            // using platform-specific code to get the real exit code
                            println!("Process {} exited with code: {}", process_id, exit_code);
                            
                            // Update process record with exit code
                            {
                                let mut process_map = processes.lock().unwrap();
                                if let Some(proc) = process_map.get_mut(&process_id) {
                                    proc.exit_code = Some(exit_code);
                                    proc.status = "exited".to_string();
                                    proc.end_time = Some(chrono::Utc::now().to_rfc3339());
                                }
                            }
                            
                            let output = TerminalOutput {
                                process_id: process_id.clone(),
                                content: format!("\nProcess exited with code: {}\n", exit_code),
                                output_type: "exit".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            
                            let _ = window.emit("terminal-output", &output);
                            interactive_processes.remove(&process_id);
                            break;
                        }
                        Ok(None) => {
                            // Process is still running
                        }
                        Err(e) => {
                            println!("Error checking process status: {}", e);
                            break;
                        }
                    }
                } else {
                    println!("Process {} not found in monitoring", process_id);
                    break;
                }
            }
        });
    }
}

// Helper function to parse commands (from terminux)
fn parse_command(input: &str) -> (String, Vec<String>) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let command = parts[0].to_string();
    let args = parts[1..].iter().map(|s| ToString::to_string(s)).collect();
    
    (command, args)
}