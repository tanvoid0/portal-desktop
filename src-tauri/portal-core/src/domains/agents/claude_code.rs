//! `AgentBackend` for the real `claude` CLI, spawned as a subprocess SDK host
//! (`-p --input-format stream-json --output-format stream-json
//! --include-partial-messages`), confirmed against the installed CLI (v2.1.317):
//! one JSON object per stdout line, multi-turn over one process via stdin,
//! `control_request`/`interrupt` aborts a turn without ending the session.
//!
//! ## Permissions
//!
//! `--permission-prompt-tool` doesn't exist and `--permission-mode manual`
//! alone just auto-denies anything that would prompt — headless `-p` mode has
//! no terminal to ask. The actual ask-back channel is Claude Code's
//! `PermissionRequest` **hook**, installed as an `http` hook (not a
//! subprocess hook, matching `agent-monitor`'s `permission-relay.js`): Claude
//! Code itself POSTs the tool name/input to a URL and blocks for the JSON
//! decision. So each run gets its own ephemeral axum listener
//! (`spawn_permission_hook`) and a per-run `--settings` file pointing the
//! hook at it — verified end to end against the real CLI (Write + Bash both
//! correctly held for and obeyed an http-hook decision).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{mpsc::UnboundedSender, oneshot, Mutex};

use crate::process_ext::NoWindowExt;

use super::backend::{AgentBackend, BackendModel, RunControl, StartParams};
use super::events::AgentEvent;
use super::process;

/// How long an unanswered permission prompt stays open before the hook
/// replies with "no decision" (Claude Code then auto-denies — headless has no
/// terminal to fall through to). Mirrors `permission-relay.js`'s `HOLD_MS`.
const PERMISSION_HOLD: Duration = Duration::from_secs(300);

pub struct ClaudeCodeBackend;

#[async_trait]
impl AgentBackend for ClaudeCodeBackend {
    fn id(&self) -> &'static str {
        "claude-code"
    }

    fn display_name(&self) -> &'static str {
        "Claude Code"
    }

    async fn detect(&self) -> bool {
        which::which("claude").is_ok()
    }

    async fn list_models(&self) -> Vec<BackendModel> {
        // ponytail: the CLI has no "list models" command; the launcher's
        // model field is free text (alias like "sonnet"/"opus"/"fable" or a
        // full name) and empty means "claude's own default". Add a real list
        // if/when the CLI exposes one.
        Vec::new()
    }

    async fn start(
        &self,
        params: StartParams,
        events_tx: UnboundedSender<AgentEvent>,
    ) -> Result<Box<dyn RunControl>, String> {
        which::which("claude")
            .map_err(|_| "`claude` CLI not found on PATH. Install Claude Code.".to_string())?;

        let run_id = uuid::Uuid::new_v4().to_string();
        let (hook_port, hook_token, pending, shutdown_tx) =
            spawn_permission_hook(events_tx.clone()).await?;
        let settings_path = write_hook_settings(&run_id, hook_port, &hook_token)?;
        // Removed on every exit path below — spawn failure, an early `?`
        // return, or (moved into the stdout pump) a normal run end.
        let settings_guard = TempSettingsFile(settings_path.clone());

        // npm's `claude` shim is a `.cmd`, which `CreateProcess` can't run
        // directly — go through `cmd.exe` on Windows, same as coder's
        // `advance_claude_cli` (the `which` call above confirmed `claude` is
        // on PATH either way, so this resolves the same binary).
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C").arg("claude");
            c
        } else {
            tokio::process::Command::new("claude")
        };
        cmd.no_window();
        cmd.current_dir(&params.cwd)
            .arg("-p")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            // Always ask-per-tool via the hook relay above; never pass
            // acceptEdits/bypassPermissions as the default (P1 decision).
            .arg("--permission-mode")
            .arg("manual")
            .arg("--settings")
            .arg(&settings_path)
            // `--settings` merges with the user's real settings files rather
            // than replacing them, so a global `PreToolUse` hook would fire
            // alongside the per-run one above (double permission prompt).
            // Loading no sources at all still leaves `--settings` and any
            // admin-managed policy settings in effect.
            .arg("--setting-sources")
            .arg("");
        if let Some(model) = params.model.as_deref().filter(|m| !m.trim().is_empty()) {
            cmd.arg("--model").arg(model);
        }
        if let Some(sid) = &params.resume_session_id {
            cmd.arg("--resume").arg(sid);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn claude: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("claude produced no stdin")?;
        let stdout = child.stdout.take().ok_or("claude produced no stdout")?;
        let mut stderr = child.stderr.take().ok_or("claude produced no stderr")?;

        write_user_line(&mut stdin, &params.prompt).await?;

        let stdin = Arc::new(Mutex::new(stdin));
        let child = Arc::new(Mutex::new(child));

        // Drains stderr quietly so a full pipe can never block claude's
        // stdout; nothing downstream reads it in P1 — a spawn that produces
        // no events surfaces as an empty run to the UI instead.
        tokio::spawn(async move {
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf).await;
        });

        {
            let events_tx = events_tx.clone();
            let child = Arc::clone(&child);
            tokio::spawn(async move {
                // Dropped at the end of this task — i.e. once the run is
                // over — regardless of which branch below gets there.
                let _settings_guard = settings_guard;
                let mut lines = BufReader::new(stdout).lines();
                let mut subagent_ids: HashSet<String> = HashSet::new();
                loop {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            let line = line.trim();
                            if line.is_empty() {
                                continue;
                            }
                            let Ok(event) = serde_json::from_str::<Value>(line) else {
                                continue; // non-JSON line on a stream-json stream
                            };
                            for e in parse_stream_json_line(&event, &mut subagent_ids) {
                                if events_tx.send(e).is_err() {
                                    break;
                                }
                            }
                        }
                        Ok(None) => break, // stdout closed — process is gone or going
                        Err(_) => break,
                    }
                }
                // Tokio's `Child::wait` is safe to call after `kill()` already
                // reaped it (cached exit status), so this is the one place
                // that both reaps a natural exit and settles a killed one.
                let code = child.lock().await.wait().await.ok().and_then(|s| s.code());
                let _ = events_tx.send(AgentEvent::Exited { code });
                let _ = shutdown_tx.send(());
            });
        }

        Ok(Box::new(ClaudeCodeRun {
            stdin,
            child,
            pending,
            events_tx,
        }))
    }
}

struct ClaudeCodeRun {
    stdin: Arc<Mutex<ChildStdin>>,
    child: Arc<Mutex<Child>>,
    pending: PendingMap,
    events_tx: UnboundedSender<AgentEvent>,
}

#[async_trait]
impl RunControl for ClaudeCodeRun {
    async fn send_user(&self, text: String) -> Result<(), String> {
        let mut stdin = self.stdin.lock().await;
        write_user_line(&mut stdin, &text).await
    }

    async fn decide_permission(&self, request_id: String, allow: bool) -> Result<(), String> {
        let tx = self
            .pending
            .lock()
            .await
            .remove(&request_id)
            .ok_or_else(|| "no such pending permission request".to_string())?;
        let _ = tx.send(permission_decision(allow));
        let _ = self.events_tx.send(AgentEvent::PermissionResolved {
            request_id,
            allowed: allow,
        });
        Ok(())
    }

    async fn interrupt(&self) -> Result<(), String> {
        let mut stdin = self.stdin.lock().await;
        let line = json!({
            "type": "control_request",
            "request_id": uuid::Uuid::new_v4().to_string(),
            "request": { "subtype": "interrupt" },
        });
        write_line(&mut stdin, &line).await
    }

    async fn kill(&self) -> Result<(), String> {
        {
            let mut child = self.child.lock().await;
            process::start_kill(&mut child);
        }
        // Emit the terminal event ourselves rather than wait on the
        // stdout-pump task's own EOF-triggered `Exited` (claude_code.rs's
        // pump loop above) — that's a race against the OS actually tearing
        // the process down and, on a kill we requested ourselves, there's no
        // reason to make the caller (and every other run sharing
        // `AgentRunService::runs`) wait on it. The pump task still sends its
        // own `Exited` once it reaps the real exit code; the service-level
        // loop breaks on the first one and ignores the second.
        let _ = self.events_tx.send(AgentEvent::Exited { code: None });
        Ok(())
    }
}

async fn write_user_line(stdin: &mut ChildStdin, text: &str) -> Result<(), String> {
    let line = json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": text }] },
    });
    write_line(stdin, &line).await
}

async fn write_line(stdin: &mut ChildStdin, value: &Value) -> Result<(), String> {
    let mut s = value.to_string();
    s.push('\n');
    stdin
        .write_all(s.as_bytes())
        .await
        .map_err(|e| format!("claude stdin write failed: {e}"))
}

fn permission_decision(allow: bool) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PermissionRequest",
            "decision": { "behavior": if allow { "allow" } else { "deny" } },
        },
    })
}

// ---- permission-hook relay ------------------------------------------------

type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

#[derive(Clone)]
struct HookState {
    events_tx: UnboundedSender<AgentEvent>,
    pending: PendingMap,
    /// Per-run secret baked into the route path (`/hook/<token>`) so nothing
    /// else reaching this loopback port during the run's lifetime can inject
    /// a fake permission decision for it.
    token: String,
}

/// Binds an ephemeral localhost listener for this run's `PermissionRequest`
/// hook and returns its port, a per-run token the hook URL embeds, the
/// pending-request map `decide_permission` resolves into, and a shutdown
/// handle torn down when the run's stdout pump sees the process exit.
async fn spawn_permission_hook(
    events_tx: UnboundedSender<AgentEvent>,
) -> Result<(u16, String, PendingMap, oneshot::Sender<()>), String> {
    let token = uuid::Uuid::new_v4().to_string();
    let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
    let state = HookState {
        events_tx,
        pending: pending.clone(),
        token: token.clone(),
    };
    let app = axum::Router::new()
        .route("/hook/{token}", axum::routing::post(permission_hook_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("failed to bind permission-hook listener: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .port();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    Ok((port, token, pending, shutdown_tx))
}

/// Claude Code's `PermissionRequest` http-hook POST: holds the connection
/// open until `decide_permission` answers or `PERMISSION_HOLD` elapses, then
/// replies with the hook's expected JSON (see `permission_decision`, or `{}`
/// for "no decision" on timeout). Requests to any path/token but this run's
/// own are refused (loopback-only, but cheap to close off).
async fn permission_hook_handler(
    State(state): State<HookState>,
    Path(token): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    if token != state.token {
        return Json(json!({}));
    }
    let tool_name = body
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let tool_input = body.get("tool_input").cloned().unwrap_or_else(|| json!({}));
    let suggestions = body
        .get("permission_suggestions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let request_id = uuid::Uuid::new_v4().to_string();

    let (tx, rx) = oneshot::channel::<Value>();
    state.pending.lock().await.insert(request_id.clone(), tx);

    let _ = state.events_tx.send(AgentEvent::PermissionRequest {
        request_id: request_id.clone(),
        tool_name,
        tool_input,
        suggestions,
    });

    let decision = tokio::time::timeout(PERMISSION_HOLD, rx).await;
    state.pending.lock().await.remove(&request_id);
    match decision {
        Ok(Ok(value)) => Json(value),
        _ => Json(json!({})),
    }
}

fn write_hook_settings(run_id: &str, port: u16, token: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("portal-agents");
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create hook settings dir: {e}"))?;
    let path = dir.join(format!("hook-{run_id}.json"));
    let settings = json!({
        "hooks": {
            "PermissionRequest": [{
                "matcher": "*",
                "hooks": [{
                    "type": "http",
                    "url": format!("http://127.0.0.1:{port}/hook/{token}"),
                    // Comfortably above PERMISSION_HOLD so claude doesn't
                    // cancel the hook before our own timeout fires.
                    "timeout": 310,
                }],
            }],
        },
    });
    let body = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, body).map_err(|e| format!("failed to write hook settings: {e}"))?;
    Ok(path)
}

/// Deletes the per-run `--settings` temp file on drop, whichever way `start`
/// exits — spawn failure, an early `?` return, or (moved into the stdout
/// pump) a normal run end.
struct TempSettingsFile(PathBuf);

impl Drop for TempSettingsFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// ---- wire parsing ----------------------------------------------------------

/// Claude tool_result `content` is either a string or an array of
/// `{type:"text", text}` blocks — same shape coder's `tool_result_text`
/// handles; duplicated here rather than imported so `agents` has no
/// dependency on `coder` (coder depends on `agents`, not the reverse).
fn tool_result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

/// Normalizes one stream-json line into zero or more [`AgentEvent`]s.
/// `subagent_ids` tracks `Agent`-tool dispatches so their matching
/// `tool_result` becomes `SubagentFinished` instead of a plain `ToolResult`.
/// Pure — unit-tested without spawning the CLI.
fn parse_stream_json_line(event: &Value, subagent_ids: &mut HashSet<String>) -> Vec<AgentEvent> {
    let mut out = Vec::new();
    match event.get("type").and_then(Value::as_str) {
        Some("system") if event.get("subtype").and_then(Value::as_str) == Some("init") => {
            if let Some(sid) = event.get("session_id").and_then(Value::as_str) {
                out.push(AgentEvent::SessionStarted {
                    session_id: sid.to_string(),
                });
            }
        }
        Some("stream_event") => {
            let parent = event
                .get("parent_tool_use_id")
                .and_then(Value::as_str)
                .map(str::to_string);
            let delta = event.get("event").and_then(|e| e.get("delta"));
            match delta.and_then(|d| d.get("type")).and_then(Value::as_str) {
                Some("text_delta") => {
                    if let Some(text) = delta.and_then(|d| d.get("text")).and_then(Value::as_str) {
                        if !text.is_empty() {
                            out.push(AgentEvent::TextDelta {
                                text: text.to_string(),
                                parent_tool_use_id: parent,
                            });
                        }
                    }
                }
                Some("thinking_delta") => {
                    if let Some(text) = delta.and_then(|d| d.get("thinking")).and_then(Value::as_str)
                    {
                        if !text.is_empty() {
                            out.push(AgentEvent::ThinkingDelta {
                                text: text.to_string(),
                                parent_tool_use_id: parent,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Some("assistant") => {
            let parent = event
                .get("parent_tool_use_id")
                .and_then(Value::as_str)
                .map(str::to_string);
            let blocks = event
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        let text = block.get("text").and_then(Value::as_str).unwrap_or("");
                        if !text.trim().is_empty() {
                            out.push(AgentEvent::Message {
                                role: "assistant".into(),
                                text: text.to_string(),
                                parent_tool_use_id: parent.clone(),
                            });
                        }
                    }
                    Some("tool_use") => {
                        let id = block.get("id").and_then(Value::as_str).unwrap_or("").to_string();
                        let name = block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                        if name == "Agent" {
                            subagent_ids.insert(id.clone());
                            let label = input
                                .get("description")
                                .and_then(Value::as_str)
                                .or_else(|| input.get("subagent_type").and_then(Value::as_str))
                                .map(str::to_string);
                            out.push(AgentEvent::SubagentStarted {
                                tool_use_id: id.clone(),
                                name: label,
                            });
                        }
                        if name == "TodoWrite" {
                            if let Some(items) = input.get("todos") {
                                out.push(AgentEvent::Todos {
                                    items: items.clone(),
                                });
                            }
                        }
                        out.push(AgentEvent::ToolCall {
                            id,
                            name,
                            input,
                            parent_tool_use_id: parent.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
        Some("user") => {
            let blocks = event
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for block in blocks {
                if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                    let tool_use_id = block
                        .get("tool_use_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let is_error = block.get("is_error").and_then(Value::as_bool).unwrap_or(false);
                    out.push(AgentEvent::ToolResult {
                        tool_use_id: tool_use_id.clone(),
                        content: tool_result_text(block.get("content")),
                        is_error,
                    });
                    if subagent_ids.remove(&tool_use_id) {
                        out.push(AgentEvent::SubagentFinished { tool_use_id });
                    }
                }
            }
        }
        Some("result") => {
            let final_text = event
                .get("result")
                .and_then(Value::as_str)
                .map(str::to_string);
            out.push(AgentEvent::TurnDone { final_text });
            let total_cost_usd = event.get("total_cost_usd").and_then(Value::as_f64);
            let model_usage = event.get("modelUsage").cloned();
            if total_cost_usd.is_some() || model_usage.is_some() {
                out.push(AgentEvent::Usage {
                    total_cost_usd,
                    model_usage,
                });
            }
        }
        _ => {
            out.push(AgentEvent::Raw {
                line: event.to_string(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_use_agent_marks_subagent_started_and_its_result_finishes_it() {
        let mut ids = HashSet::new();
        let dispatch = json!({"type":"assistant","message":{"content":[
            {"type":"tool_use","id":"tu_1","name":"Agent","input":{"description":"Explore the repo"}}
        ]}});
        let events = parse_stream_json_line(&dispatch, &mut ids);
        assert!(matches!(&events[0], AgentEvent::SubagentStarted { tool_use_id, name }
            if tool_use_id == "tu_1" && name.as_deref() == Some("Explore the repo")));
        assert!(matches!(&events[1], AgentEvent::ToolCall { id, name, .. }
            if id == "tu_1" && name == "Agent"));
        assert!(ids.contains("tu_1"));

        let result = json!({"type":"user","message":{"content":[
            {"type":"tool_result","tool_use_id":"tu_1","content":"done"}
        ]}});
        let events = parse_stream_json_line(&result, &mut ids);
        assert!(matches!(&events[0], AgentEvent::ToolResult { tool_use_id, .. } if tool_use_id == "tu_1"));
        assert!(matches!(&events[1], AgentEvent::SubagentFinished { tool_use_id } if tool_use_id == "tu_1"));
        assert!(!ids.contains("tu_1"));
    }

    #[test]
    fn result_line_emits_turn_done_and_usage() {
        let mut ids = HashSet::new();
        let line = json!({"type":"result","result":"all done","total_cost_usd":0.5,"modelUsage":{"sonnet":{"tokens":10}}});
        let events = parse_stream_json_line(&line, &mut ids);
        assert!(matches!(&events[0], AgentEvent::TurnDone { final_text } if final_text.as_deref() == Some("all done")));
        assert!(matches!(&events[1], AgentEvent::Usage { total_cost_usd: Some(c), .. } if (*c - 0.5).abs() < f64::EPSILON));
    }

    #[test]
    fn text_delta_carries_parent_tool_use_id_for_subagent_attribution() {
        let mut ids = HashSet::new();
        let line = json!({
            "type":"stream_event","parent_tool_use_id":"tu_1",
            "event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"hi"}}
        });
        let events = parse_stream_json_line(&line, &mut ids);
        assert!(matches!(&events[0], AgentEvent::TextDelta { text, parent_tool_use_id }
            if text == "hi" && parent_tool_use_id.as_deref() == Some("tu_1")));
    }
}
