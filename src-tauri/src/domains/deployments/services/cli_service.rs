use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Error,
}

pub struct CliService {
    // Map deployment_id -> (Child process, ProcessInfo)
    processes: Arc<Mutex<HashMap<String, (Child, ProcessInfo)>>>,
    // Map deployment_id -> logs
    logs: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl CliService {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            logs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a background process
    pub async fn spawn_process(
        &self,
        deployment_id: &str,
        command: &str,
        working_directory: Option<&str>,
        environment: &HashMap<String, String>,
    ) -> Result<u32, String> {
        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Command cannot be empty".to_string());
        }

        let program = parts[0];
        let args = &parts[1..];

        // Build command
        let mut cmd = Command::new(program);
        cmd.args(args);

        // Set working directory
        if let Some(wd) = working_directory {
            let path = PathBuf::from(wd);
            if !path.exists() {
                return Err(format!("Working directory does not exist: {}", wd));
            }
            cmd.current_dir(path);
        }

        // Set environment variables
        for (key, value) in environment {
            cmd.env(key, value);
        }

        // Spawn process
        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let pid = child.id().ok_or("Failed to get process ID")?;

        // Initialize logs
        {
            let mut logs = self.logs.lock().unwrap();
            logs.insert(deployment_id.to_string(), Vec::new());
        }

        // Spawn tasks to read stdout and stderr
        let deployment_id_stdout = deployment_id.to_string();
        let deployment_id_stderr = deployment_id.to_string();
        let logs_stdout = Arc::clone(&self.logs);
        let logs_stderr = Arc::clone(&self.logs);

        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        // Read stdout
        let mut stdout_reader = BufReader::new(stdout).lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                let mut logs = logs_stdout.lock().unwrap();
                if let Some(log_vec) = logs.get_mut(&deployment_id_stdout) {
                    log_vec.push(format!("[STDOUT] {}", line));
                }
            }
        });

        // Read stderr
        let mut stderr_reader = BufReader::new(stderr).lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                let mut logs = logs_stderr.lock().unwrap();
                if let Some(log_vec) = logs.get_mut(&deployment_id_stderr) {
                    log_vec.push(format!("[STDERR] {}", line));
                }
            }
        });

        // Store process info
        let process_info = ProcessInfo {
            pid,
            command: command.to_string(),
            working_directory: working_directory.unwrap_or("").to_string(),
            environment: environment.clone(),
            started_at: chrono::Utc::now(),
            status: ProcessStatus::Running,
        };

        let mut processes = self.processes.lock().unwrap();
        processes.insert(deployment_id.to_string(), (child, process_info.clone()));

        Ok(pid)
    }

    /// Stop a process
    pub async fn stop_process(&self, deployment_id: &str) -> Result<(), String> {
        // Remove child from HashMap first, then release lock before await
        let mut child_opt = {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(deployment_id).map(|(child, _)| child)
        }; // Lock is released here
        
        if let Some(mut child) = child_opt {
            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                return Err(format!("Failed to kill process: {}", e));
            }
            
            // Wait for process to exit
            let _ = child.wait().await;
            
            Ok(())
        } else {
            Err(format!("Process not found for deployment: {}", deployment_id))
        }
    }

    /// Check if a process is still running
    pub async fn is_process_running(&self, deployment_id: &str) -> Result<bool, String> {
        // Check if process exists and get its status, then release lock
        let should_remove = {
            let mut processes = self.processes.lock().unwrap();
            if let Some((child, _)) = processes.get_mut(deployment_id) {
                // Try to get exit status without consuming the child
                match child.try_wait() {
                    Ok(Some(_)) => {
                        // Process has exited, mark for removal
                        true
                    }
                    Ok(None) => {
                        // Still running
                        return Ok(true);
                    }
                    Err(e) => return Err(format!("Failed to check process status: {}", e)),
                }
            } else {
                // Process not found
                return Ok(false);
            }
        }; // Lock is released here
        
        // Remove the process if it exited
        if should_remove {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(deployment_id);
        }
        
        Ok(false)
    }

    /// Get process info
    pub fn get_process_info(&self, deployment_id: &str) -> Option<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        processes.get(deployment_id).map(|(_, info)| info.clone())
    }

    /// Get process logs
    pub fn get_process_logs(&self, deployment_id: &str, tail: Option<usize>) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        if let Some(log_vec) = logs.get(deployment_id) {
            if let Some(tail_count) = tail {
                log_vec.iter().rev().take(tail_count).rev().cloned().collect()
            } else {
                log_vec.clone()
            }
        } else {
            Vec::new()
        }
    }

    /// Clean up process resources
    pub async fn cleanup_process(&self, deployment_id: &str) {
        let mut processes = self.processes.lock().unwrap();
        processes.remove(deployment_id);
        
        let mut logs = self.logs.lock().unwrap();
        logs.remove(deployment_id);
    }

    /// List all running processes
    pub fn list_processes(&self) -> Vec<(String, ProcessInfo)> {
        let processes = self.processes.lock().unwrap();
        processes
            .iter()
            .map(|(id, (_, info))| (id.clone(), info.clone()))
            .collect()
    }
}

