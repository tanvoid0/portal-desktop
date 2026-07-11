use crate::domains::terminal::shell_integration::{
    ShellIntegrationEventV2, ShellIntegrationParser,
};
use crate::domains::terminal::types::*;
use crate::process_ext::NoWindowExt;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tauri::{Emitter, Window};
use tokio::sync::Mutex;
use uuid::Uuid;

/// All OS-level resources owned by a single PTY-backed terminal session.
///
/// Keeping child / master / writer / temp-rc in one entry means a single
/// `sessions.remove(id)` tears the whole session down: dropping `master`
/// closes the PTY (so the reader thread hits EOF and exits), dropping
/// `child` / `writer` releases the process and stdin handles, and
/// `temp_rc_path` is deleted from disk. Previously these lived in four
/// separate maps and only the child map was ever cleaned, leaking PTYs,
/// stdin writers and temp rc files for the life of the app.
struct PtySessionResources {
    child: Box<dyn portable_pty::Child + Send>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    /// zsh: temp ZDOTDIR directory; bash: temp rcfile. Deleted on teardown.
    temp_rc_path: Option<PathBuf>,
}

pub type ProcessMap = Arc<Mutex<HashMap<String, TerminalProcess>>>;
type SessionMap = Arc<Mutex<HashMap<String, PtySessionResources>>>;

pub struct TerminalManager {
    processes: ProcessMap,
    sessions: SessionMap,
    command_interceptors: Arc<Mutex<Vec<CommandInterceptor>>>,
    output_parsers: Arc<Mutex<Vec<OutputParser>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        // Best-effort sweep of temp rc files left behind by crashed sessions
        // in a previous run (normal teardown deletes them per-session).
        sweep_stale_temp_rc();

        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            command_interceptors: Arc::new(Mutex::new(Vec::new())),
            output_parsers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_processes(&self) -> ProcessMap {
        self.processes.clone()
    }

    pub async fn create_process(
        &self,
        request: CreateProcessRequest,
        window: Window,
    ) -> Result<TerminalProcess, String> {
        let process_id = Uuid::new_v4().to_string();
        let oneshot_command = request.command.clone();
        let is_oneshot = oneshot_command.is_some();

        // Resolve the concrete shell executable + base args from the requested shell.
        let (shell_cmd, mut shell_args) = {
            let shell_lower = request.shell.to_lowercase();
            if cfg!(target_os = "windows") {
                if shell_lower.contains("cmd") || shell_lower == "cmd.exe" {
                    ("cmd.exe".to_string(), vec!["/k".to_string()])
                } else if shell_lower.contains("powershell") || shell_lower == "powershell.exe" {
                    (
                        "powershell.exe".to_string(),
                        vec![
                            "-NoLogo".to_string(),
                            "-NoProfile".to_string(),
                            "-NoExit".to_string(),
                        ],
                    )
                } else if shell_lower.contains("pwsh") || shell_lower == "pwsh.exe" {
                    (
                        "pwsh.exe".to_string(),
                        vec![
                            "-NoLogo".to_string(),
                            "-NoProfile".to_string(),
                            "-NoExit".to_string(),
                        ],
                    )
                } else if shell_lower.contains("bash") || shell_lower == "bash.exe" {
                    ("bash.exe".to_string(), vec![])
                } else if shell_lower.contains("wsl") {
                    ("wsl.exe".to_string(), vec![])
                } else {
                    (request.shell.clone(), vec![])
                }
            } else if shell_lower.contains("zsh") {
                ("zsh".to_string(), vec![])
            } else if shell_lower.contains("bash") {
                ("bash".to_string(), vec![])
            } else if shell_lower.contains("fish") {
                ("fish".to_string(), vec![])
            } else if request.shell == "bash" || request.shell.is_empty() {
                ("zsh".to_string(), vec![])
            } else {
                (request.shell.clone(), vec![])
            }
        };

        // Oneshot: run the command through the shell's non-interactive flag so
        // quoting/builtins work and the exit status is the command's real code.
        // This replaces the old POSIX-only `stty -echo; exec ...` stdin hack.
        if let Some(cmd) = &oneshot_command {
            shell_args = oneshot_shell_args(&shell_cmd, cmd);
        }

        let process = TerminalProcess {
            id: process_id.clone(),
            tab_id: request.tab_id.clone(),
            command: if let Some(cmd) = &oneshot_command {
                cmd.clone()
            } else {
                format!("{} {}", shell_cmd, shell_args.join(" "))
            },
            working_directory: request.working_directory.clone(),
            environment: request.environment.clone(),
            status: "starting".to_string(),
            pid: None,
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: None,
            exit_code: None,
        };

        {
            let mut processes = self.processes.lock().await;
            processes.insert(process_id.clone(), process.clone());
        }

        // Spawn the shell in a PTY and handle output streaming.
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

        // Empty working directory → fall back to the user's home dir (then the
        // app's current dir) so we never try to spawn in "" (which fails).
        // Frontend sends "" by default.
        let working_dir = if request.working_directory.trim().is_empty() {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .ok()
                .or_else(|| {
                    std::env::current_dir()
                        .ok()
                        .map(|p| p.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| ".".to_string())
        } else {
            request.working_directory.clone()
        };
        let mut environment = request.environment.clone();

        // Fill in essential env vars from the real process environment when the
        // caller did not provide them. No fake/hardcoded HOME/USER values here.
        let mut ensure_env = |key: &str, value: String| {
            environment.entry(key.to_string()).or_insert(value);
        };
        ensure_env("TERM", "xterm-256color".to_string());
        ensure_env("COLORTERM", "truecolor".to_string());
        if let Ok(home) = std::env::var("HOME") {
            ensure_env("HOME", home);
        }
        if let Ok(user) = std::env::var("USER") {
            ensure_env("USER", user);
        }
        if let Ok(path) = std::env::var("PATH") {
            ensure_env("PATH", path);
        }
        ensure_env("SHELL", shell_cmd.clone());
        ensure_env("LANG", "en_US.UTF-8".to_string());
        ensure_env("LC_ALL", "en_US.UTF-8".to_string());
        ensure_env("HISTSIZE", "10000".to_string());
        ensure_env("HISTFILESIZE", "10000".to_string());
        ensure_env("EDITOR", "nano".to_string());
        ensure_env("PAGER", "less".to_string());
        environment.insert("NO_COLOR".to_string(), "0".to_string());
        environment.insert("FORCE_COLOR".to_string(), "1".to_string());

        // --- OSC 133 injection (command start/end tracking) ---
        // Interactive sessions only (no interactive prompt to instrument in
        // oneshot runs). Protocol (internal, Warp-style):
        //   133;A;<cwd>     command started (pre-exec)
        //   133;C;<command> command text for the current block
        //   133;B;<exit>    command finished with exit code
        let mut temp_rc_path: Option<PathBuf> = None;
        if !is_oneshot && (shell_cmd.contains("powershell") || shell_cmd.contains("pwsh")) {
            // PowerShell: PSConsoleHostReadLine fires after Enter and before
            // execution (our pre-exec); prompt fires after each command.
            let ps_profile_path = std::env::temp_dir().join(format!(
                "portal_osc133_ps_{}.ps1",
                process_id.replace('-', "_")
            ));

            // Loads the user's own profile first (we spawn with -NoProfile),
            // then wraps — not replaces — their prompt, waveterm-style.
            let ps_profile = r#"
if (Test-Path $PROFILE) {
  try { . $PROFILE } catch { Write-Host "portal: error loading profile: $_" }
}

function Global:__PortalOscWrite([string]$s) {
  [Console]::Write("$([char]27)]133;$s$([char]27)\")
}

if (Test-Path Function:\prompt) {
  $Global:__portal_original_prompt = $function:prompt
} else {
  $Global:__portal_original_prompt = {
    "PS $($ExecutionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
  }
}

function Global:prompt {
  $code = if ($?) { 0 } elseif ($global:LASTEXITCODE) { $global:LASTEXITCODE } else { 1 }
  __PortalOscWrite "B;$code"
  & $Global:__portal_original_prompt
}

# Pre-exec hook: requires PSReadLine (loaded by default in console hosts).
if (Get-Command -Name Set-PSReadLineOption -ErrorAction SilentlyContinue) {
  function Global:PSConsoleHostReadLine {
    $line = [Microsoft.PowerShell.PSConsoleReadLine]::ReadLine($Host.Runspace, $ExecutionContext)
    if ($line -and $line.Trim()) {
      __PortalOscWrite "A;$($ExecutionContext.SessionState.Path.CurrentLocation.Path)"
      __PortalOscWrite "C;$line"
    }
    $line
  }
}
"#;

            std::fs::write(&ps_profile_path, ps_profile).map_err(|e| {
                format!("Failed to write temporary PowerShell profile for OSC133 injection: {e}")
            })?;

            shell_args = vec![
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                format!(". '{}'", ps_profile_path.to_string_lossy()),
            ];
            temp_rc_path = Some(ps_profile_path);
        } else if !is_oneshot && !cfg!(target_os = "windows") {
            let shell_lower = request.shell.to_lowercase();

            if shell_cmd == "zsh" && shell_lower.contains("zsh") {
                let zsh_dir = std::env::temp_dir().join(format!(
                    "portal_osc133_zsh_{}",
                    process_id.replace('-', "_")
                ));
                std::fs::create_dir_all(&zsh_dir).map_err(|e| {
                    format!("Failed to create temporary ZDOTDIR for OSC133 injection: {e}")
                })?;

                let zshrc_path = zsh_dir.join(".zshrc");
                // Sources the user's real zshrc (aliases/PATH/prompt survive),
                // then registers hooks additively via add-zsh-hook so any
                // user-defined preexec/precmd keep working (waveterm-style).
                let zshrc = r#"
__PORTAL_ZDOTDIR="$ZDOTDIR"
[ -f "$HOME/.zshrc" ] && source "$HOME/.zshrc"

# Don't let history land in our temp ZDOTDIR
if [[ "$HISTFILE" == "$__PORTAL_ZDOTDIR/.zsh_history" ]]; then
  HISTFILE="$HOME/.zsh_history"
fi

__portal_preexec() {
  printf '\033]133;A;%s\033\\' "$PWD"
  printf '\033]133;C;%s\033\\' "$1"
}

__portal_precmd() {
  printf '\033]133;B;%s\033\\' "$?"
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __portal_preexec
add-zsh-hook precmd __portal_precmd
"#;

                std::fs::write(&zshrc_path, zshrc).map_err(|e| {
                    format!("Failed to write temporary .zshrc for OSC133 injection: {e}")
                })?;

                environment.insert("ZDOTDIR".to_string(), zsh_dir.to_string_lossy().to_string());
                temp_rc_path = Some(zsh_dir);
            } else if shell_cmd == "bash" && shell_lower.contains("bash") {
                let bash_rc_path = std::env::temp_dir().join(format!(
                    "portal_osc133_bashrc_{}",
                    process_id.replace('-', "_")
                ));

                // Sources the user's bashrc first so aliases/PATH/PS1 survive.
                // Our DEBUG trap + PROMPT_COMMAND are then set last and win —
                // a user PROMPT_COMMAND hook is dropped (full bash-preexec
                // would be needed to merge them; not worth 400 lines here).
                let bashrc = r#"
[ -f "$HOME/.bashrc" ] && source "$HOME/.bashrc"

__portal_osc133_suppress=0

__portal_osc133_preexec() {
  # Avoid emitting A around the prompt command itself.
  if [ $__portal_osc133_suppress -ne 0 ]; then
    return
  fi

  case "$BASH_COMMAND" in
    *__portal_osc133_precmd* ) return ;;
  esac

  printf '\033]133;A;%s\033\\' "$PWD"
  printf '\033]133;C;%s\033\\' "$BASH_COMMAND"
}

__portal_osc133_precmd() {
  __portal_osc133_suppress=1
  local st=$?
  printf '\033]133;B;%s\033\\' "$st"
  __portal_osc133_suppress=0
}

trap '__portal_osc133_preexec' DEBUG
PROMPT_COMMAND='__portal_osc133_precmd'
"#;

                std::fs::write(&bash_rc_path, bashrc).map_err(|e| {
                    format!("Failed to write temporary bash rcfile for OSC133 injection: {e}")
                })?;

                shell_args = vec![
                    "--noprofile".to_string(),
                    "--norc".to_string(),
                    "--rcfile".to_string(),
                    bash_rc_path.to_string_lossy().to_string(),
                ];
                temp_rc_path = Some(bash_rc_path);
            }
        }

        let mut cmd = CommandBuilder::new(&shell_cmd);
        for a in &shell_args {
            cmd.arg(a);
        }
        cmd.cwd(&working_dir);
        for (k, v) in &environment {
            cmd.env(k, v);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn PTY shell: {}", e))?;

        // Clone the reader and take the writer before the master is moved into
        // the session map (both borrow `&master` immutably).
        let master = pair.master;
        let reader = master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;
        let writer = master
            .take_writer()
            .map_err(|e| format!("Failed to take PTY writer: {}", e))?;

        // Update metadata with the real PID / running status.
        let pid = child.process_id();
        {
            let mut processes = self.processes.lock().await;
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.pid = pid;
                proc.status = "running".to_string();
            }
        }

        // Store the whole session under one key.
        {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(
                process_id.clone(),
                PtySessionResources {
                    child,
                    master,
                    writer,
                    temp_rc_path,
                },
            );
        }

        // No banner line — the shell's own prompt is the "ready" signal,
        // matching Warp/Wave. (The old "PTY shell ready: …" line was noise.)

        // Start PTY output streaming. The shell-integration parser is owned by
        // this thread (no shared map / lock needed) since it is only touched here.
        let pid_for_thread = process_id.clone();
        let window_for_reader = window.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut parser = ShellIntegrationParser::new();
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF: flush any buffered shell-integration events.
                        for event in parser.flush() {
                            let _ = window_for_reader.emit("shell-integration-event", &event);
                            let v2 = ShellIntegrationEventV2 {
                                process_id: pid_for_thread.clone(),
                                event,
                            };
                            let _ = window_for_reader.emit("shell-integration-event-v2", &v2);
                        }
                        break;
                    }
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();

                        for event in parser.process_output(&chunk) {
                            let _ = window_for_reader.emit("shell-integration-event", &event);
                            let v2 = ShellIntegrationEventV2 {
                                process_id: pid_for_thread.clone(),
                                event,
                            };
                            let _ = window_for_reader.emit("shell-integration-event-v2", &v2);
                        }

                        let output = TerminalOutput {
                            process_id: pid_for_thread.clone(),
                            content: chunk,
                            output_type: "stdout".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        if window_for_reader.emit("terminal-output", &output).is_err() {
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

        // Start process monitoring (detects exit, reports real code, tears down).
        self.start_process_monitoring(process_id.clone(), window.clone());

        Ok(process)
    }

    pub async fn send_input(&self, process_id: String, input: String) -> Result<(), String> {
        // NOTE: intentionally does NOT log `input` — it can contain secrets.
        let mut sessions = self.sessions.lock().await;

        let Some(session) = sessions.get_mut(&process_id) else {
            return Err("No stdin handle found for process".to_string());
        };

        session
            .writer
            .write_all(input.as_bytes())
            .map_err(|e| format!("Failed to send input: {}", e))?;
        session
            .writer
            .flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        // Previously slept 50ms here on every write, blocking a tokio worker per
        // keystroke/paste chunk. PTY writes are ordered; the sleep bought nothing.
        Ok(())
    }

    pub async fn kill_process(&self, process_id: String) -> Result<(), String> {
        // Kill the child, then remove the whole session entry (dropping master +
        // writer, closing the PTY) and delete its temp rc file.
        let mut kill_err = None;
        {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.remove(&process_id) {
                let mut session = session;
                if let Err(e) = session.child.kill() {
                    kill_err = Some(format!("Failed to kill process: {}", e));
                }
                remove_temp_rc(session.temp_rc_path.as_deref());
                // session (child/master/writer) dropped here.
            }
        }

        {
            let mut processes = self.processes.lock().await;
            if let Some(proc) = processes.get_mut(&process_id) {
                proc.status = "killed".to_string();
                proc.end_time = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        match kill_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    pub async fn get_process(&self, process_id: String) -> Result<Option<TerminalProcess>, String> {
        let processes = self.processes.lock().await;
        Ok(processes.get(&process_id).cloned())
    }

    pub async fn get_all_processes(&self) -> Result<Vec<TerminalProcess>, String> {
        let processes = self.processes.lock().await;
        Ok(processes.values().cloned().collect())
    }

    pub async fn execute_command(
        &self,
        request: ExecuteCommandRequest,
        _window: Window,
    ) -> Result<String, String> {
        let (cmd, _args) = parse_command(&request.command);

        if cmd.is_empty() {
            return Ok("".to_string());
        }

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .no_window()
                .args(["/C", &request.command])
                .output()
        } else {
            Command::new("sh").arg("-c").arg(&request.command).output()
        }
        .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result = if output.status.success() {
            stdout.to_string()
        } else {
            format!("Error: {}\nOutput: {}", stderr, stdout)
        };

        Ok(result)
    }

    pub async fn add_command_interceptor(
        &self,
        interceptor: CommandInterceptor,
    ) -> Result<(), String> {
        let mut interceptors = self.command_interceptors.lock().await;
        interceptors.push(interceptor);
        Ok(())
    }

    pub async fn remove_command_interceptor(&self, _id: String) -> Result<(), String> {
        let mut interceptors = self.command_interceptors.lock().await;
        interceptors.clear();
        Ok(())
    }

    pub async fn add_output_parser(&self, parser: OutputParser) -> Result<(), String> {
        let mut parsers = self.output_parsers.lock().await;
        parsers.push(parser);
        Ok(())
    }

    pub async fn remove_output_parser(&self, _id: String) -> Result<(), String> {
        let mut parsers = self.output_parsers.lock().await;
        parsers.clear();
        Ok(())
    }

    pub async fn resize_terminal(
        &self,
        process_id: String,
        cols: u32,
        rows: u32,
    ) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&process_id) {
            let size = PtySize {
                cols: cols as u16,
                rows: rows as u16,
                pixel_width: 0,
                pixel_height: 0,
            };
            session
                .master
                .resize(size)
                .map_err(|e| format!("Failed to resize PTY: {}", e))?;
            Ok(())
        } else {
            Err("Process not found".into())
        }
    }

    fn start_process_monitoring(&self, process_id: String, window: Window) {
        let processes = self.processes.clone();
        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // Poll the child for exit under the session lock, then release it.
                let exit_code = {
                    let mut sessions_guard = sessions.lock().await;
                    match sessions_guard.get_mut(&process_id) {
                        Some(session) => match session.child.try_wait() {
                            // portable_pty exposes the real exit code since 0.8.
                            Ok(Some(status)) => Some(status.exit_code() as i32),
                            Ok(None) => None, // still running
                            Err(e) => {
                                eprintln!("Error checking process status: {}", e);
                                // Tear the session down so we don't leak on error.
                                if let Some(s) = sessions_guard.remove(&process_id) {
                                    remove_temp_rc(s.temp_rc_path.as_deref());
                                }
                                break;
                            }
                        },
                        // Session already removed (e.g. killed) — stop polling.
                        None => break,
                    }
                };

                let Some(exit_code) = exit_code else {
                    continue;
                };

                // Full teardown: remove the session (drops child/master/writer,
                // closing the PTY so the reader thread hits EOF) + delete temp rc.
                {
                    let mut sessions_guard = sessions.lock().await;
                    if let Some(session) = sessions_guard.remove(&process_id) {
                        remove_temp_rc(session.temp_rc_path.as_deref());
                    }
                }

                {
                    let mut process_map = processes.lock().await;
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
                break;
            }
        });
    }
}

/// Non-interactive shell args to run a single command and exit with its code.
fn oneshot_shell_args(shell_cmd: &str, command: &str) -> Vec<String> {
    let lower = shell_cmd.to_lowercase();
    if lower.contains("cmd") {
        vec!["/C".to_string(), command.to_string()]
    } else if lower.contains("powershell") || lower.contains("pwsh") {
        vec![
            "-NoLogo".to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            command.to_string(),
        ]
    } else {
        // bash / zsh / sh / fish all accept `-c "<command>"`.
        vec!["-c".to_string(), command.to_string()]
    }
}

/// Delete a session's temp rc artifact (zsh ZDOTDIR dir or bash rcfile).
fn remove_temp_rc(path: Option<&Path>) {
    if let Some(path) = path {
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// On startup, delete OSC133 temp rc artifacts left by crashed sessions.
fn sweep_stale_temp_rc() {
    let tmp = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&tmp) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("portal_osc133_") {
                let path = entry.path();
                if path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
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
