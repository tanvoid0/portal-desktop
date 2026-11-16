use crate::database::DatabaseManager;
use crate::domains::projects::move_lock;
use crate::domains::scripts::repositories::ScriptExecutionRepository;
use crate::process_ext::NoWindowExt;
use crate::utils::pnpm_workspace::prepare_shell_command;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use uuid::Uuid;

type OutputBuffers = Mutex<HashMap<String, Vec<String>>>;

/// Decode one chunk of process output, carrying an incomplete trailing UTF-8
/// sequence over to the next chunk so a multi-byte char split across two reads
/// is not mangled. Invalid bytes become U+FFFD rather than ending the stream.
fn decode_utf8_chunk(carry: &mut Vec<u8>, chunk: &[u8]) -> String {
    carry.extend_from_slice(chunk);
    let mut out = String::new();
    let mut i = 0;
    loop {
        match std::str::from_utf8(&carry[i..]) {
            Ok(s) => {
                out.push_str(s);
                i = carry.len();
                break;
            }
            Err(e) => {
                let end = i + e.valid_up_to();
                out.push_str(std::str::from_utf8(&carry[i..end]).unwrap_or_default());
                match e.error_len() {
                    // Invalid bytes: replace them and keep decoding the rest.
                    Some(n) => {
                        out.push('\u{FFFD}');
                        i = end + n;
                    }
                    // An unfinished char at the very end — carry it over.
                    None => {
                        i = end;
                        break;
                    }
                }
            }
        }
    }
    carry.drain(..i);
    out
}

/// Copy a child pipe into the execution's live buffer as raw chunks — not
/// lines — so partial lines (progress, prompts) show while the run is going.
/// stdout and stderr both land in the one buffer, in arrival order.
async fn pump<R: AsyncRead + Unpin>(stream: Option<R>, id: &str, bufs: &OutputBuffers) {
    let Some(mut stream) = stream else { return };
    let mut chunk = [0u8; 8192];
    let mut carry = Vec::new();
    loop {
        let text = match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => decode_utf8_chunk(&mut carry, &chunk[..n]),
        };
        if !text.is_empty() {
            if let Some(buf) = bufs.lock().unwrap().get_mut(id) {
                buf.push(text);
            }
        }
    }
    if !carry.is_empty() {
        if let Some(buf) = bufs.lock().unwrap().get_mut(id) {
            buf.push(String::from_utf8_lossy(&carry).into_owned());
        }
    }
}

/// Kill the whole process tree rooted at `pid` — the `cmd /C` / `sh -c`
/// wrapper. Killing only the wrapper orphans what it started (pnpm -> node
/// ...), which keeps running and keeps the output pipes open. Async, so no
/// lock is held and no runtime thread blocks while taskkill/kill runs.
async fn kill_tree(pid: u32) {
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .no_window()
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .output()
        .await;
    // Spawned with process_group(0), so the group id is the wrapper's pid.
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .output()
        .await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteScriptRequest {
    pub block_id: Option<String>,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub working_directory: Option<String>,
    /// Extra environment variables for the spawned process (e.g. CI=true for
    /// non-interactive Action runs). Merged onto the inherited env.
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Who asked for the run: "user" (default) or e.g. "mcp:<client>". Stored
    /// on the row so a stop can be told apart from a failure by whoever waits.
    #[serde(default)]
    pub triggered_by: Option<String>,
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
    // Map execution_id -> working directory, for the relocation in-use check
    // (`relocation::usage::running_under`) — kept alongside `running_processes`
    // rather than folded into it so the `Child` map stays untouched.
    running_dirs: Arc<Mutex<HashMap<String, String>>>,
    // Runs the user asked to stop. The output task reads this when the process
    // exits and records `cancelled` itself — after saving the output.
    cancel_requested: Arc<Mutex<HashSet<String>>>,
}

impl ScriptExecutionService {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            repository: ScriptExecutionRepository::new(db_manager.clone()),
            db_manager,
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            output_buffers: Arc::new(Mutex::new(HashMap::new())),
            running_dirs: Arc::new(Mutex::new(HashMap::new())),
            cancel_requested: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// True if any execution currently running has a working directory at or
    /// under `path`. Used by the project-relocation in-use check.
    pub fn running_under(&self, path: &str) -> bool {
        self.running_dirs
            .lock()
            .unwrap()
            .values()
            .any(|dir| crate::domains::projects::move_lock::is_under(dir, path))
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
        if let Some(wd) = &request.working_directory {
            move_lock::ensure_unlocked_within(Path::new(wd))?;
        }
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
                request
                    .triggered_by
                    .clone()
                    .unwrap_or_else(|| "user".to_string()),
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

        // Extra env (e.g. CI=true so non-interactive runners don't prompt)
        if let Some(ref env) = request.env {
            cmd.envs(env);
        }

        // Configure stdio. stdin is closed (null) so the process can never block
        // waiting on a prompt that no one will answer (e.g. pnpm's purge
        // confirmation) — it fails fast instead of hanging the run forever.
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);
        // Own process group, so Stop can kill everything the shell started.
        #[cfg(unix)]
        cmd.process_group(0);

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

        // Take stdout/stderr before moving child
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let pid = child.id();

        // Register the child before anything else can await, so a Stop from
        // here on finds it (cancel_execution sets its flag, then looks here).
        self.running_processes
            .lock()
            .unwrap()
            .insert(execution_id.clone(), child);
        if let Some(ref wd) = request.working_directory {
            let mut dirs = self.running_dirs.lock().unwrap();
            dirs.insert(execution_id.clone(), wd.clone());
        }
        // A Stop that landed while the process was spawning saw no child.
        let stop_early = self
            .cancel_requested
            .lock()
            .unwrap()
            .contains(&execution_id);
        if let (true, Some(pid)) = (stop_early, pid) {
            kill_tree(pid).await;
        }
        if let Some(pid) = pid {
            self.repository
                .update_pid(&execution_id, pid as i32)
                .await?;
        }

        // Spawn background task to read output and wait for completion
        let exec_id_clone = execution_id.clone();
        let repo = self.repository.clone();
        let output_buffers = Arc::clone(&self.output_buffers);
        let running_processes = Arc::clone(&self.running_processes);
        let running_dirs = Arc::clone(&self.running_dirs);
        let cancel_requested = Arc::clone(&self.cancel_requested);

        tokio::spawn(async move {
            // Pump stdout + stderr concurrently (sequential reads can deadlock on
            // full pipes) into the one arrival-ordered live buffer.
            let mut pumps = Box::pin(async {
                tokio::join!(
                    pump(stdout, &exec_id_clone, &output_buffers),
                    pump(stderr, &exec_id_clone, &output_buffers),
                )
            });
            // After a Stop, give the pipes a moment to close, then stop reading
            // anyway: a process that outlived its (killed) parent can still hold
            // them open, and on Windows taskkill /T can't find it once the parent
            // is gone. Dropping our read ends lets the run finish as cancelled.
            // ponytail: orphans survive on Windows; a Job object per run would
            // kill them too.
            let mut stop_deadline: Option<std::time::Instant> = None;
            loop {
                tokio::select! {
                    _ = &mut pumps => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => {
                        if stop_deadline.is_none()
                            && cancel_requested.lock().unwrap().contains(&exec_id_clone)
                        {
                            stop_deadline =
                                Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
                        }
                        if stop_deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                            break;
                        }
                    }
                }
            }
            drop(pumps); // closes our read ends if we stopped early
            let combined_output = output_buffers
                .lock()
                .unwrap()
                .get(&exec_id_clone)
                .map(|chunks| chunks.concat())
                .unwrap_or_default();

            // Wait for process to complete - extract child from map first, then await
            let child_opt: Option<Child> = {
                let mut processes = running_processes.lock().unwrap();
                processes.remove(&exec_id_clone)
            };
            running_dirs.lock().unwrap().remove(&exec_id_clone);

            let wait_result = if let Some(mut child) = child_opt {
                child.wait().await
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Process handle lost before wait",
                ))
            };

            // Stopped by the user: record `cancelled`, not `failed`, so whoever
            // waits on this run (the UI, an MCP client) reads it was stopped. A
            // clean exit means it finished before the kill landed.
            let stopped = cancel_requested.lock().unwrap().remove(&exec_id_clone);
            let killed = stopped && !matches!(&wait_result, Ok(st) if st.success());

            // Update final status in database
            let (status, exit_code, error) = match wait_result {
                _ if killed => (
                    "cancelled".to_string(),
                    None,
                    Some("Stopped by user".to_string()),
                ),
                Ok(status) if status.success() => ("success".to_string(), status.code(), None),
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

            // Save output, then the final status — a poller that stops at a
            // finished status must already see the output.
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

    /// Record a run that happened in the embedded interactive terminal (no process is spawned here).
    pub async fn record_interactive(
        &self,
        block_id: Option<String>,
        command: String,
        working_directory: Option<String>,
        exit_code: Option<i32>,
        started_at: Option<String>,
    ) -> Result<(), String> {
        let id = Uuid::new_v4().to_string();
        self.repository
            .create(
                id.clone(),
                block_id,
                command,
                "{}".to_string(),
                working_directory,
                "user".to_string(),
            )
            .await?;
        let status = match exit_code {
            Some(0) => "success",
            Some(_) => "failed",
            None => "cancelled",
        };
        self.repository
            .update_status(&id, status.to_string(), exit_code, None)
            .await?;
        // Backdate started_at to when the terminal run began so duration is real.
        if let Some(t) = started_at.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()) {
            use crate::entities::script_execution::{ActiveModel, Entity};
            use sea_orm::{ActiveModelTrait, EntityTrait, Set};
            let db = self.db_manager.get_connection();
            if let Ok(Some(m)) = Entity::find_by_id(&id).one(db).await {
                let mut am: ActiveModel = m.into();
                am.started_at = Set(t);
                am.update(db)
                    .await
                    .map_err(|e| format!("Failed to set started_at: {}", e))?;
            }
        }
        Ok(())
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

    /// Stop a running execution and its whole process tree.
    ///
    /// Only flags and kills: the output task saves what the run printed and
    /// then records `cancelled` (see `execute_script`).
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        let row = self
            .repository
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| format!("Execution not found: {execution_id}"))?;
        if row.status != "running" && row.status != "pending" {
            return Err(format!("Run already finished ({})", row.status));
        }

        // Flag first, then look for the child: execute_script registers the
        // child before checking the flag, so one of the two always sees the other.
        self.cancel_requested
            .lock()
            .unwrap()
            .insert(execution_id.to_string());
        let pid = self
            .running_processes
            .lock()
            .unwrap()
            .get(execution_id)
            .map(|child| child.id());
        match pid {
            Some(Some(pid)) => {
                kill_tree(pid).await;
                if let Some(child) = self.running_processes.lock().unwrap().get_mut(execution_id) {
                    let _ = child.start_kill();
                }
            }
            // Already exited; the output task is finishing it.
            Some(None) => {}
            // Still spawning: execute_script sees the flag once the child is registered.
            None if row.status == "pending" => {}
            // Left over from an earlier Portal session — its process isn't ours to
            // kill (the pid may be reused), so close the row and say so.
            None => {
                self.cancel_requested.lock().unwrap().remove(execution_id);
                self.repository
                    .update_status(
                        execution_id,
                        "cancelled".to_string(),
                        None,
                        Some("Not tracked by this Portal session — marked stopped".to_string()),
                    )
                    .await?;
            }
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
        let mut executions = self.repository.get_running().await?;
        // Pending rows have no pid, so they fall into the "state unknown" branch below.
        executions.extend(self.repository.get_pending().await?);

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
        // A row still `pending` at startup never got a process.
        for execution in self.repository.get_pending().await? {
            self.repository
                .update_status(
                    &execution.id,
                    "failed".to_string(),
                    None,
                    Some("Interrupted by app restart".to_string()),
                )
                .await?;
        }

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
                        Some("Interrupted by app restart".to_string()),
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
            running_dirs: Arc::clone(&self.running_dirs),
            cancel_requested: Arc::clone(&self.cancel_requested),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::decode_utf8_chunk;

    #[test]
    fn utf8_split_across_chunks_is_kept_whole() {
        let bytes = "ok ✓ done\r50%\r100%\n".as_bytes();
        let cut = 4; // inside the 3-byte ✓
        let mut carry = Vec::new();
        let a = decode_utf8_chunk(&mut carry, &bytes[..cut]);
        let b = decode_utf8_chunk(&mut carry, &bytes[cut..]);
        assert_eq!(a, "ok ");
        assert_eq!(a + &b, "ok ✓ done\r50%\r100%\n");
        assert!(carry.is_empty());
    }

    #[test]
    fn invalid_byte_then_split_char_keeps_the_tail() {
        let mut carry = Vec::new();
        // bad byte, then the first 2 of the 3 bytes of a check mark
        let a = decode_utf8_chunk(&mut carry, &[b'x', 0xff, 0xe2, 0x9c]);
        let b = decode_utf8_chunk(&mut carry, &[0x93]);
        assert_eq!(a + &b, "x\u{fffd}\u{2713}");
    }

    #[test]
    fn invalid_bytes_do_not_end_the_stream() {
        let mut carry = Vec::new();
        let out = decode_utf8_chunk(&mut carry, b"a\xffb");
        assert_eq!(out, "a\u{fffd}b");
        assert!(carry.is_empty());
    }
}
