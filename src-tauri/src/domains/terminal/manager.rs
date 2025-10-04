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
        
        // Determine shell command based on platform and request
        let (shell_cmd, shell_args) = if cfg!(target_os = "windows") {
            if request.shell.contains("powershell") {
                ("powershell.exe", vec!["-NoLogo", "-NoProfile"])
            } else if request.shell.contains("cmd") {
                ("cmd.exe", vec!["/k"])
            } else {
                // Default to PowerShell on Windows if shell is not recognized
                ("powershell.exe", vec!["-NoLogo", "-NoProfile"])
            }
        } else {
            if request.shell.contains("bash") {
                ("bash", vec!["-i", "-l"])
            } else if request.shell.contains("zsh") {
                ("zsh", vec!["-i", "-l"])
            } else {
                // Default to bash on Unix
                ("bash", vec!["-i", "-l"])
            }
        };

        println!("Using shell command: {} with args: {:?}", shell_cmd, shell_args);

        let process = TerminalProcess {
            id: process_id.clone(),
            tab_id: "default".to_string(),
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
        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(shell_cmd);
        for a in &shell_args {
            cmd.arg(a);
        }
        cmd.cwd(&request.working_directory);
        for (k, v) in &request.environment {
            cmd.env(k, v);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn PTY shell: {}", e))?;

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
        let mut master = pair.master;
        if let Ok(writer) = master.take_writer() {
            let mut stdin_handles = self.stdin_handles.lock().unwrap();
            stdin_handles.insert(process_id.clone(), writer);
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
                    let pid_for_thread = process_id.clone();
                    let window_for_reader = window.clone();
                    std::thread::spawn(move || {
                        let mut buf = [0u8; 8192];
                        loop {
                            match reader.read(&mut buf) {
                                Ok(0) => break, // EOF
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
                                    eprintln!("PTY read error: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
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
            std::thread::sleep(std::time::Duration::from_millis(10));
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
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                
                let mut processes = INTERACTIVE_PROCESSES.lock().unwrap();
                if let Some(child) = processes.get_mut(&process_id) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            println!("Process {} exited with status: {:?}", process_id, status);
                            
                            let output = TerminalOutput {
                                process_id: process_id.clone(),
                                content: format!("\nProcess exited: {:?}\n", status),
                                output_type: "exit".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            
                            let _ = window.emit("terminal-output", &output);
                            processes.remove(&process_id);
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