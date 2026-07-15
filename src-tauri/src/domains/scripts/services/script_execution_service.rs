use crate::database::DatabaseManager;
use crate::domains::scripts::repositories::ScriptExecutionRepository;
use crate::process_ext::NoWindowExt;
use crate::utils::pnpm_workspace::prepare_shell_command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteScriptRequest {
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptExecutionInfo {
    pub id: String,
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub pid: Option<i32>,
    pub output: String,
    pub error: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub triggered_by: String,
}

pub struct ScriptExecutionService {
    repository: ScriptExecutionRepository,
    db_manager: Arc<DatabaseManager>,
    // Map execution_id -> Child process
    running_processes: Arc<Mutex<HashMap<String, Child>>>,
    // Map execution_id -> live output buffer (for real-time updates)
    output_buffers: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ScriptExecutionService {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            repository: ScriptExecutionRepository::new(db_manager.clone()),
            db_manager,
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            output_buffers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Resolve parameter placeholders in command
    fn resolve_command(command: &str, parameters: &HashMap<String, String>) -> String {
        let mut resolved = command.to_string();
        for (key, value) in parameters {
            // Replace ${key} style placeholders
            resolved = resolved.replace(&format!("${{{}}}", key), value);
            // Also support $key style without braces
            resolved = resolved.replace(&format!("${}", key), value);
        }
        resolved
    }

    /// Build a short error message from exit code + captured output.
    fn failure_message(exit_code: Option<i32>, output: &str) -> String {
        let meaningful: Vec<&str> = output
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let tail = meaningful
            .iter()
            .rev()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        match (exit_code, tail.is_empty()) {
            (Some(code), false) => format!("Exit code {code}\n{tail}"),
            (Some(code), true) => format!("Exit code {code}"),
            (None, false) => tail,
            (None, true) => "Process failed with no output".to_string(),
        }
    }

    /// Execute a script and track it persistently
    pub async fn execute_script(&self, request: ExecuteScriptRequest) -> Result<String, String> {
        let execution_id = Uuid::new_v4().to_string();
        let parameters_json = serde_json::to_string(&request.parameters)
            .map_err(|e| format!("Failed to serialize parameters: {}", e))?;

        // Resolve command with parameters, then apply shell rewrites (e.g. broken pnpm workspace)
        let resolved_command = Self::resolve_command(&request.command, &request.parameters);
        let exec_command = if let Some(ref wd) = request.working_directory {
            prepare_shell_command(&resolved_command, wd)
        } else {
            resolved_command
        };

        // Create execution record in database
        let _execution = self
            .repository
            .create(
                execution_id.clone(),
                request.block_id.clone(),
                exec_command.clone(),
                parameters_json,
                request.working_directory.clone(),
                "user".to_string(),
            )
            .await?;

        // Initialize output buffer
        {
            let mut buffers = self.output_buffers.lock().unwrap();
            buffers.insert(execution_id.clone(), Vec::new());
        }

        // Parse command - use shell for complex commands
        let parts: Vec<&str> = exec_command.split_whitespace().collect();
        if parts.is_empty() {
            self.repository
                .update_status(
                    &execution_id,
                    "failed".to_string(),
                    None,
                    Some("Empty command".to_string()),
                )
                .await?;
            return Err("Command cannot be empty".to_string());
        }

        // Build command - use shell to handle complex commands with pipes, redirects, etc.
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.no_window();
            c.args(["/C", &exec_command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", &exec_command]);
            c
        };

        // Set working directory
        if let Some(ref wd) = request.working_directory {
            cmd.current_dir(wd);
        }

        // Configure stdio
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);

        // Spawn process
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.repository
                    .update_status(
                        &execution_id,
                        "failed".to_string(),
                        None,
                        Some(format!("Failed to spawn process: {}", e)),
                    )
                    .await?;
                return Err(format!("Failed to spawn process: {}", e));
            }
        };

        // Get PID and update record
        let pid = child.id().map(|p| p as i32);
        if let Some(pid) = pid {
            self.repository.update_pid(&execution_id, pid).await?;
        }

        // Take stdout/stderr before moving child
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Store child process for tracking
        {
            let mut processes = self.running_processes.lock().unwrap();
            processes.insert(execution_id.clone(), child);
        }

        // Spawn background task to read output and wait for completion
        let exec_id_clone = execution_id.clone();
        let repo = self.repository.clone();
        let output_buffers = Arc::clone(&self.output_buffers);
        let running_processes = Arc::clone(&self.running_processes);

        tokio::spawn(async move {
            // Read stdout + stderr in parallel (sequential reads can deadlock on full pipes)
            let exec_id_out = exec_id_clone.clone();
            let buffers_out = Arc::clone(&output_buffers);
            let stdout_task = async move {
                let mut lines = Vec::new();
                if let Some(stdout) = stdout {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let formatted = format!("{}\n", line);
                        {
                            let mut bufs = buffers_out.lock().unwrap();
                            if let Some(buf) = bufs.get_mut(&exec_id_out) {
                                buf.push(formatted.clone());
                            }
                        }
                        lines.push(formatted);
                    }
                }
                lines
            };

            let exec_id_err = exec_id_clone.clone();
            let buffers_err = Arc::clone(&output_buffers);
            let stderr_task = async move {
                let mut lines = Vec::new();
                if let Some(stderr) = stderr {
                    let mut reader = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let formatted = format!("[stderr] {}\n", line);
                        {
                            let mut bufs = buffers_err.lock().unwrap();
                            if let Some(buf) = bufs.get_mut(&exec_id_err) {
                                buf.push(formatted.clone());
                            }
                        }
                        lines.push(formatted);
                    }
                }
                lines
            };

            let (stdout_lines, stderr_lines) = tokio::join!(stdout_task, stderr_task);
            let mut combined_output = String::new();
            for line in &stdout_lines {
                combined_output.push_str(line);
            }
            for line in &stderr_lines {
                combined_output.push_str(line);
            }

            // Wait for process to complete - extract child from map first, then await
            let child_opt: Option<Child> = {
                let mut processes = running_processes.lock().unwrap();
                processes.remove(&exec_id_clone)
            };

            let wait_result = if let Some(mut child) = child_opt {
                child.wait().await
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Process handle lost before wait",
                ))
            };

            // Update final status in database
            let (status, exit_code, error) = match wait_result {
                Ok(status) if status.success() => {
                    ("success".to_string(), status.code(), None)
                }
                Ok(status) => {
                    let code = status.code();
                    (
                        "failed".to_string(),
                        code,
                        Some(Self::failure_message(code, &combined_output)),
                    )
                }
                Err(e) => (
                    "failed".to_string(),
                    None,
                    Some(format!(
                        "{}\n{}",
                        e,
                        Self::failure_message(None, &combined_output)
                    )),
                ),
            };

            // Save output and final status
            let _ = repo.append_output(&exec_id_clone, &combined_output).await;
            let _ = repo
                .update_status(&exec_id_clone, status, exit_code, error)
                .await;

            // Clean up output buffer
            {
                let mut bufs = output_buffers.lock().unwrap();
                bufs.remove(&exec_id_clone);
            }
        });

        Ok(execution_id)
    }

    /// Get execution details
    pub async fn get_execution(
        &self,
        execution_id: &str,
    ) -> Result<Option<ScriptExecutionInfo>, String> {
        let execution = self.repository.get_by_id(execution_id).await?;

        Ok(execution.map(|e| ScriptExecutionInfo {
            id: e.id,
            block_id: e.block_id,
            command: e.command,
            parameters: serde_json::from_str(&e.parameters_json).unwrap_or_default(),
            working_directory: e.working_directory,
            status: e.status,
            exit_code: e.exit_code,
            pid: e.pid,
            output: e.output,
            error: e.error,
            started_at: e.started_at.to_rfc3339(),
            finished_at: e.finished_at.map(|d| d.to_rfc3339()),
            triggered_by: e.triggered_by,
        }))
    }

    /// Get live output for a running execution
    pub fn get_live_output(&self, execution_id: &str) -> Vec<String> {
        let buffers = self.output_buffers.lock().unwrap();
        buffers.get(execution_id).cloned().unwrap_or_default()
    }

    /// Cancel a running execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        // Try to kill the process - extract from map first
        let child_opt: Option<Child> = {
            let mut processes = self.running_processes.lock().unwrap();
            processes.remove(execution_id)
        };

        if let Some(mut child) = child_opt {
            let _ = child.kill().await;
        }

        // Update database status
        self.repository
            .update_status(execution_id, "cancelled".to_string(), None, None)
            .await?;

        // Clean up output buffer
        {
            let mut buffers = self.output_buffers.lock().unwrap();
            buffers.remove(execution_id);
        }

        Ok(())
    }

    /// Get executions for a specific block/script
    pub async fn get_executions_by_block(
        &self,
        block_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<ScriptExecutionInfo>, String> {
        let executions = self.repository.get_by_block(block_id, limit).await?;

        Ok(executions
            .into_iter()
            .map(|e| ScriptExecutionInfo {
                id: e.id,
                block_id: e.block_id,
                command: e.command,
                parameters: serde_json::from_str(&e.parameters_json).unwrap_or_default(),
                working_directory: e.working_directory,
                status: e.status,
                exit_code: e.exit_code,
                pid: e.pid,
                output: e.output,
                error: e.error,
                started_at: e.started_at.to_rfc3339(),
                finished_at: e.finished_at.map(|d| d.to_rfc3339()),
                triggered_by: e.triggered_by,
            })
            .collect())
    }

    /// Get all running executions
    pub async fn get_running_executions(&self) -> Result<Vec<ScriptExecutionInfo>, String> {
        let executions = self.repository.get_running().await?;

        Ok(executions
            .into_iter()
            .map(|e| ScriptExecutionInfo {
                id: e.id,
                block_id: e.block_id,
                command: e.command,
                parameters: serde_json::from_str(&e.parameters_json).unwrap_or_default(),
                working_directory: e.working_directory,
                status: e.status,
                exit_code: e.exit_code,
                pid: e.pid,
                output: e.output,
                error: e.error,
                started_at: e.started_at.to_rfc3339(),
                finished_at: e.finished_at.map(|d| d.to_rfc3339()),
                triggered_by: e.triggered_by,
            })
            .collect())
    }

    /// Get recent executions
    pub async fn get_recent_executions(
        &self,
        limit: u64,
    ) -> Result<Vec<ScriptExecutionInfo>, String> {
        let executions = self.repository.get_recent(limit).await?;

        Ok(executions
            .into_iter()
            .map(|e| ScriptExecutionInfo {
                id: e.id,
                block_id: e.block_id,
                command: e.command,
                parameters: serde_json::from_str(&e.parameters_json).unwrap_or_default(),
                working_directory: e.working_directory,
                status: e.status,
                exit_code: e.exit_code,
                pid: e.pid,
                output: e.output,
                error: e.error,
                started_at: e.started_at.to_rfc3339(),
                finished_at: e.finished_at.map(|d| d.to_rfc3339()),
                triggered_by: e.triggered_by,
            })
            .collect())
    }

    /// Check running processes and sync with database
    /// Called on app startup to reconcile state
    pub async fn sync_running_executions(&self) -> Result<(), String> {
        let executions = self.repository.get_running().await?;

        for execution in executions {
            if let Some(pid) = execution.pid {
                // Check if process is still running
                let is_running = self.is_process_alive(pid as u32);

                if !is_running {
                    // Process died while app was closed, mark as failed
                    self.repository
                        .update_status(
                            &execution.id,
                            "failed".to_string(),
                            None,
                            Some("Process terminated while app was closed".to_string()),
                        )
                        .await?;
                }
            } else {
                // No PID recorded, mark as failed
                self.repository
                    .update_status(
                        &execution.id,
                        "failed".to_string(),
                        None,
                        Some("Process state unknown".to_string()),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Check if a process is still alive by PID
    fn is_process_alive(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use std::process::Command as StdCommand;
            // Use kill -0 to check if process exists
            StdCommand::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(windows)]
        {
            use std::process::Command as StdCommand;
            // Use tasklist to check if process exists
            StdCommand::new("tasklist")
                .no_window()
                .args(["/FI", &format!("PID eq {}", pid)])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                .unwrap_or(false)
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    /// Delete an execution record
    pub async fn delete_execution(&self, execution_id: &str) -> Result<(), String> {
        // First try to cancel if running
        let _ = self.cancel_execution(execution_id).await;
        // Then delete from database
        self.repository.delete(execution_id).await
    }
}

impl Clone for ScriptExecutionService {
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
            db_manager: self.db_manager.clone(),
            running_processes: Arc::clone(&self.running_processes),
            output_buffers: Arc::clone(&self.output_buffers),
        }
    }
}
