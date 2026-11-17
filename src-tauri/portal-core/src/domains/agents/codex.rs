//! `AgentBackend` for OpenAI's `codex` CLI, driven as `codex app-server`:
//! JSON-RPC over stdio, one JSON object per line, `"jsonrpc":"2.0"` header
//! omitted on the wire.
//!
//! Wire shapes below are taken from openai/codex `codex-rs/app-server-protocol`
//! (`protocol/common.rs` method table; `protocol/v1.rs` initialize;
//! `protocol/v2/{thread,turn,item,notification,model,shared}.rs` params and
//! notifications), read 2026-11-17. **Never run against a live `codex`** — the
//! CLI is not installed here; the tests replay hand-built transcripts only.
//!
//! Flow: `initialize` -> `initialized` -> `thread/start` | `thread/resume`
//! (-> `SessionStarted{thread.id}`) -> `turn/start`. Follow-up prompts are more
//! `turn/start`s on the same thread; `turn/interrupt` needs the live turn id
//! (tracked from `turn/started`). Approvals arrive as server-initiated
//! requests (`item/commandExecution/requestApproval`,
//! `item/fileChange/requestApproval`) and are answered with
//! `{decision:"accept"|"decline"}` once the user decides.
//!
//! The only launcher knob is `StartParams::permission_mode`, mapped in
//! [`policy_for_mode`] to codex's approval policy + sandbox.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{mpsc::UnboundedSender, oneshot, Mutex};

use crate::process_ext::NoWindowExt;

use super::backend::{AgentBackend, BackendModel, RunControl, StartParams};
use super::events::AgentEvent;
use super::process;

const REQUEST_TIMEOUT: u64 = 30;
const MODEL_TTL: Duration = Duration::from_secs(600);

type ModelCache = StdMutex<Option<(Instant, Vec<BackendModel>)>>;
static MODEL_CACHE: ModelCache = StdMutex::new(None);

pub struct CodexBackend;

#[async_trait]
impl AgentBackend for CodexBackend {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    async fn detect(&self) -> bool {
        which::which("codex").is_ok()
    }

    async fn list_models(&self) -> Vec<BackendModel> {
        if let Some((at, models)) = &*MODEL_CACHE.lock().unwrap() {
            if at.elapsed() < MODEL_TTL {
                return models.clone();
            }
        }
        if !self.detect().await {
            return Vec::new();
        }
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let Ok(rpc) = Rpc::spawn(None, tx) else {
            return Vec::new();
        };
        // One budget for spawn handshake + list; a timeout yields an empty list.
        let res = tokio::time::timeout(Duration::from_secs(5), async {
            rpc.init().await?;
            rpc.request("model/list", json!({}), 5).await
        })
        .await;
        rpc.kill().await;
        let models = res
            .ok()
            .and_then(Result::ok)
            .map(|v| parse_models(&v))
            .unwrap_or_default();
        // ponytail: failures are cached too (10 min) so a broken codex isn't
        // respawned per call; add an invalidate if the launcher needs a retry.
        *MODEL_CACHE.lock().unwrap() = Some((Instant::now(), models.clone()));
        models
    }

    async fn start(
        &self,
        params: StartParams,
        events_tx: UnboundedSender<AgentEvent>,
    ) -> Result<Box<dyn RunControl>, String> {
        which::which("codex").map_err(|_| {
            "`codex` CLI not found on PATH. Install Codex (npm i -g @openai/codex).".to_string()
        })?;
        crate::domains::projects::move_lock::ensure_unlocked(std::path::Path::new(&params.cwd))?;

        let rpc = Rpc::spawn(Some(&params.cwd), events_tx.clone())?;
        let thread_id = match Self::open_thread(&rpc, &params).await {
            Ok(id) => id,
            Err(e) => {
                rpc.kill().await;
                return Err(e);
            }
        };
        let _ = events_tx.send(AgentEvent::SessionStarted {
            session_id: thread_id.clone(),
        });
        let run = CodexRun { rpc, thread_id };
        if !params.prompt.trim().is_empty() {
            if let Err(e) = run.send_user(params.prompt).await {
                run.rpc.kill().await;
                return Err(e);
            }
        }
        Ok(Box::new(run))
    }
}

impl CodexBackend {
    /// Handshake, then `thread/start` or `thread/resume`; returns the thread id.
    async fn open_thread(rpc: &Rpc, params: &StartParams) -> Result<String, String> {
        rpc.init().await?;
        let (approval, sandbox) = policy_for_mode(&params.permission_mode);
        let mut p = json!({ "cwd": params.cwd, "approvalPolicy": approval, "sandbox": sandbox });
        if let Some(m) = params.model.as_deref().filter(|m| !m.trim().is_empty()) {
            p["model"] = json!(m);
        }
        let method = if let Some(sid) = &params.resume_session_id {
            p["threadId"] = json!(sid);
            "thread/resume"
        } else {
            "thread/start"
        };
        let res = rpc.request(method, p, REQUEST_TIMEOUT).await?;
        res.pointer("/thread/id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("codex {method}: response had no thread id"))
    }
}

/// `permission_mode` (the claude-code vocabulary the launcher already sends)
/// -> codex `(approvalPolicy, sandbox)`.
fn policy_for_mode(mode: &str) -> (&'static str, &'static str) {
    match mode {
        // Must be gated like coder's bypass before any caller passes it:
        // no approvals and no sandbox.
        "bypassPermissions" => ("never", "danger-full-access"),
        "acceptEdits" => ("untrusted", "workspace-write"),
        // "manual" / empty: ask for anything codex doesn't consider trusted.
        _ => ("untrusted", "workspace-write"),
    }
}

struct CodexRun {
    rpc: Arc<Rpc>,
    thread_id: String,
}

#[async_trait]
impl RunControl for CodexRun {
    async fn send_user(&self, text: String) -> Result<(), String> {
        // Detached: callers hold the service's `runs` lock. turn_id comes from
        // `turn/started` only — writing it from the response could resurrect a
        // turn the reader already completed.
        self.rpc
            .request_detached(
                "turn/start",
                json!({ "threadId": self.thread_id, "input": [{ "type": "text", "text": text }] }),
            )
            .await
    }

    async fn decide_permission(&self, request_id: String, allow: bool) -> Result<(), String> {
        let id = self
            .rpc
            .approvals
            .lock()
            .unwrap()
            .remove(&request_id)
            .ok_or_else(|| "no such pending permission request".to_string())?;
        let decision = if allow { "accept" } else { "decline" };
        self.rpc
            .write(&json!({ "id": id, "result": { "decision": decision } }))
            .await?;
        let _ = self.rpc.events_tx.send(AgentEvent::PermissionResolved {
            request_id,
            allowed: allow,
        });
        Ok(())
    }

    async fn interrupt(&self) -> Result<(), String> {
        let turn_id = self.rpc.state.lock().unwrap().turn_id.clone();
        let Some(turn_id) = turn_id else {
            return Ok(()); // no turn in flight
        };
        // Detached: the reply only comes after the abort completes.
        self.rpc
            .request_detached(
                "turn/interrupt",
                json!({ "threadId": self.thread_id, "turnId": turn_id }),
            )
            .await
    }

    async fn kill(&self) -> Result<(), String> {
        self.rpc.kill().await;
        // Same as claude_code: emit the terminal event ourselves rather than
        // wait for the reader's EOF; the service loop takes the first one.
        let _ = self.rpc.events_tx.send(AgentEvent::Exited { code: None });
        Ok(())
    }
}

// ---- JSON-RPC plumbing -----------------------------------------------------

#[derive(Default)]
struct TurnState {
    turn_id: Option<String>,
    /// Last finished agent message of the turn, reported as `TurnDone.final_text`.
    last_text: Option<String>,
}

struct Rpc {
    stdin: Mutex<ChildStdin>,
    child: Mutex<Child>,
    next_id: AtomicI64,
    pending: StdMutex<HashMap<i64, oneshot::Sender<Result<Value, String>>>>,
    /// `PermissionRequest.request_id` -> the server request's JSON-RPC id.
    approvals: StdMutex<HashMap<String, Value>>,
    state: StdMutex<TurnState>,
    killed: AtomicBool,
    /// Reader hit EOF; set under the `pending` lock so no request slips in after.
    closed: AtomicBool,
    events_tx: UnboundedSender<AgentEvent>,
}

impl Rpc {
    fn spawn(
        cwd: Option<&str>,
        events_tx: UnboundedSender<AgentEvent>,
    ) -> Result<Arc<Self>, String> {
        // npm's `codex` shim is a `.cmd` on Windows — same as claude_code.rs.
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C").arg("codex");
            c
        } else {
            tokio::process::Command::new("codex")
        };
        cmd.no_window();
        cmd.arg("app-server");
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn codex: {e}"))?;
        let stdin = child.stdin.take().ok_or("codex produced no stdin")?;
        let stdout = child.stdout.take().ok_or("codex produced no stdout")?;
        let mut stderr = child.stderr.take().ok_or("codex produced no stderr")?;

        let rpc = Arc::new(Rpc {
            stdin: Mutex::new(stdin),
            child: Mutex::new(child),
            next_id: AtomicI64::new(1),
            pending: StdMutex::new(HashMap::new()),
            approvals: StdMutex::new(HashMap::new()),
            state: StdMutex::new(TurnState::default()),
            killed: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            events_tx,
        });

        let (stderr_tx, stderr_rx) = oneshot::channel::<String>();
        tokio::spawn(async move {
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf).await;
            let _ = stderr_tx.send(buf);
        });

        let pump = Arc::clone(&rpc);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(msg) = serde_json::from_str::<Value>(line.trim()) {
                    pump.dispatch(msg).await;
                }
            }
            // Fail in-flight requests instead of letting them time out.
            {
                let mut pending = pump.pending.lock().unwrap();
                pump.closed.store(true, Ordering::SeqCst);
                pending.clear();
            }
            if !pump.killed.load(Ordering::SeqCst) {
                let detail = tokio::time::timeout(Duration::from_secs(2), stderr_rx)
                    .await
                    .ok()
                    .and_then(|r| r.ok())
                    .unwrap_or_default();
                if !detail.trim().is_empty() {
                    let _ = pump.events_tx.send(AgentEvent::Raw {
                        line: detail.trim().to_string(),
                    });
                }
            }
            let code = pump
                .child
                .lock()
                .await
                .wait()
                .await
                .ok()
                .and_then(|s| s.code());
            let _ = pump.events_tx.send(AgentEvent::Exited { code });
        });
        Ok(rpc)
    }

    async fn write(&self, value: &Value) -> Result<(), String> {
        let mut s = value.to_string();
        s.push('\n');
        self.stdin
            .lock()
            .await
            .write_all(s.as_bytes())
            .await
            .map_err(|e| format!("codex stdin write failed: {e}"))
    }

    /// Registers + writes a request; returns its id and the response channel.
    /// Cleans `pending` on write failure; fails fast once the reader hit EOF.
    async fn send(
        &self,
        method: &str,
        params: Value,
    ) -> Result<(i64, oneshot::Receiver<Result<Value, String>>), String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().unwrap();
            if self.closed.load(Ordering::SeqCst) {
                return Err("codex exited".to_string());
            }
            pending.insert(id, tx);
        }
        if let Err(e) = self
            .write(&json!({ "id": id, "method": method, "params": params }))
            .await
        {
            self.pending.lock().unwrap().remove(&id);
            return Err(e);
        }
        Ok((id, rx))
    }

    async fn request(&self, method: &str, params: Value, secs: u64) -> Result<Value, String> {
        let (id, rx) = self.send(method, params).await?;
        match tokio::time::timeout(Duration::from_secs(secs), rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => Err("codex exited".to_string()),
            Err(_) => {
                self.pending.lock().unwrap().remove(&id);
                Err(format!("codex {method} timed out"))
            }
        }
    }

    /// Writes the request and returns; a spawned task awaits the reply and
    /// reports failures as a `Raw` event. Only write failures are returned.
    async fn request_detached(
        self: &Arc<Self>,
        method: &'static str,
        params: Value,
    ) -> Result<(), String> {
        let (id, rx) = self.send(method, params).await?;
        let rpc = Arc::clone(self);
        tokio::spawn(async move {
            let err = match tokio::time::timeout(Duration::from_secs(REQUEST_TIMEOUT), rx).await {
                Ok(Ok(Ok(_))) | Ok(Err(_)) => return, // ok, or exited (Exited event covers it)
                Ok(Ok(Err(e))) => e,
                Err(_) => {
                    rpc.pending.lock().unwrap().remove(&id);
                    format!("codex {method} timed out")
                }
            };
            let _ = rpc.events_tx.send(AgentEvent::Raw {
                line: format!("codex {method} failed: {err}"),
            });
        });
        Ok(())
    }

    async fn init(&self) -> Result<(), String> {
        self.request("initialize", init_params(), REQUEST_TIMEOUT)
            .await?;
        self.write(&json!({ "method": "initialized" })).await
    }

    async fn kill(&self) {
        self.killed.store(true, Ordering::SeqCst);
        process::start_kill(&mut *self.child.lock().await);
    }

    async fn dispatch(&self, msg: Value) {
        match classify(&msg) {
            Incoming::Response { id, result } => {
                if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
                    let _ = tx.send(result);
                }
            }
            Incoming::Request { id, method, params } => {
                let request_id = id.to_string();
                if let Some(ev) = approval_event(&method, &params, request_id.clone()) {
                    self.approvals.lock().unwrap().insert(request_id, id);
                    let _ = self.events_tx.send(ev);
                } else {
                    let _ = self
                        .write(&json!({ "id": id, "error": {
                            "code": -32601, "message": format!("unsupported request {method}") } }))
                        .await;
                }
            }
            Incoming::Notification { method, params } => {
                let events = parse_notification(&method, &params, &mut self.state.lock().unwrap());
                for e in events {
                    let _ = self.events_tx.send(e);
                }
            }
        }
    }
}

fn init_params() -> Value {
    json!({ "clientInfo": {
        "name": "portal_desktop", "title": "Portal Desktop", "version": env!("CARGO_PKG_VERSION") } })
}

// ---- wire parsing ----------------------------------------------------------

enum Incoming {
    Response {
        id: i64,
        result: Result<Value, String>,
    },
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    Notification {
        method: String,
        params: Value,
    },
}

/// Splits one line into response / server request / notification. `None`-ish
/// garbage (no id and no method) is folded into an empty notification.
fn classify(msg: &Value) -> Incoming {
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    match (msg.get("method").and_then(Value::as_str), msg.get("id")) {
        (Some(m), Some(id)) => Incoming::Request {
            id: id.clone(),
            method: m.to_string(),
            params,
        },
        (Some(m), None) => Incoming::Notification {
            method: m.to_string(),
            params,
        },
        _ => Incoming::Response {
            id: msg.get("id").and_then(Value::as_i64).unwrap_or(-1),
            result: match msg.get("error") {
                Some(e) => Err(e
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("codex request failed")
                    .to_string()),
                None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
            },
        },
    }
}

fn parse_models(res: &Value) -> Vec<BackendModel> {
    res.get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|m| !m.get("hidden").and_then(Value::as_bool).unwrap_or(false))
        .filter_map(|m| {
            let id = m.get("model").and_then(Value::as_str)?;
            let label = m.get("displayName").and_then(Value::as_str).unwrap_or(id);
            Some(BackendModel {
                id: id.to_string(),
                label: label.to_string(),
            })
        })
        .collect()
}

/// A server-initiated approval request -> `PermissionRequest`; `None` for any
/// other server request (answered with a JSON-RPC error by the caller).
fn approval_event(method: &str, p: &Value, request_id: String) -> Option<AgentEvent> {
    let (tool_name, tool_input) = match method {
        "item/commandExecution/requestApproval" => (
            "commandExecution",
            json!({ "command": p.get("command"), "cwd": p.get("cwd"), "reason": p.get("reason") }),
        ),
        "item/fileChange/requestApproval" => (
            "fileChange",
            json!({ "reason": p.get("reason"), "grantRoot": p.get("grantRoot") }),
        ),
        _ => return None,
    };
    Some(AgentEvent::PermissionRequest {
        request_id,
        tool_name: tool_name.to_string(),
        tool_input,
        suggestions: Vec::new(),
    })
}

fn str_of(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Normalizes one notification into zero or more events. Unmodelled
/// notifications are dropped (codex emits many; `Raw` would flood the feed).
/// Pure — unit-tested without spawning the CLI.
fn parse_notification(method: &str, p: &Value, st: &mut TurnState) -> Vec<AgentEvent> {
    match method {
        "turn/started" => {
            st.turn_id = p
                .pointer("/turn/id")
                .and_then(Value::as_str)
                .map(str::to_string);
            st.last_text = None;
            vec![]
        }
        "item/agentMessage/delta" => vec![AgentEvent::TextDelta {
            text: str_of(p, "delta"),
            parent_tool_use_id: None,
        }],
        "item/reasoning/textDelta" | "item/reasoning/summaryTextDelta" => {
            vec![AgentEvent::ThinkingDelta {
                text: str_of(p, "delta"),
                parent_tool_use_id: None,
            }]
        }
        "item/started" => item_started(&p["item"]).into_iter().collect(),
        "item/completed" => item_completed(&p["item"], st).into_iter().collect(),
        "turn/completed" => {
            st.turn_id = None;
            let failed = p.pointer("/turn/status").and_then(Value::as_str) == Some("failed");
            let final_text = if failed {
                p.pointer("/turn/error/message")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            } else {
                st.last_text.take()
            };
            vec![AgentEvent::TurnDone {
                final_text,
                is_error: failed,
            }]
        }
        "error" if !p.get("willRetry").and_then(Value::as_bool).unwrap_or(false) => {
            vec![AgentEvent::Raw {
                line: format!("codex error: {}", str_of(&p["error"], "message")),
            }]
        }
        _ => vec![],
    }
}

fn item_started(item: &Value) -> Option<AgentEvent> {
    let (name, input) = match item.get("type").and_then(Value::as_str)? {
        "commandExecution" => (
            "commandExecution".to_string(),
            json!({ "command": item.get("command"), "cwd": item.get("cwd") }),
        ),
        "fileChange" => (
            "fileChange".to_string(),
            json!({ "changes": item.get("changes") }),
        ),
        "mcpToolCall" => (
            format!("{}/{}", str_of(item, "server"), str_of(item, "tool")),
            item.get("arguments").cloned().unwrap_or_else(|| json!({})),
        ),
        "dynamicToolCall" => (
            str_of(item, "tool"),
            item.get("arguments").cloned().unwrap_or_else(|| json!({})),
        ),
        _ => return None,
    };
    Some(AgentEvent::ToolCall {
        id: str_of(item, "id"),
        name,
        input,
        parent_tool_use_id: None,
    })
}

fn item_completed(item: &Value, st: &mut TurnState) -> Option<AgentEvent> {
    let status = str_of(item, "status");
    let bad_status = status == "failed" || status == "declined";
    let (content, is_error) = match item.get("type").and_then(Value::as_str)? {
        "agentMessage" => {
            let text = str_of(item, "text");
            if text.trim().is_empty() {
                return None;
            }
            st.last_text = Some(text.clone());
            return Some(AgentEvent::Message {
                role: "assistant".into(),
                text,
                parent_tool_use_id: None,
            });
        }
        "commandExecution" => {
            let code = item.get("exitCode").and_then(Value::as_i64).unwrap_or(0);
            (str_of(item, "aggregatedOutput"), bad_status || code != 0)
        }
        "fileChange" => (status, bad_status),
        "mcpToolCall" => (
            item.get("result")
                .filter(|v| !v.is_null())
                .or_else(|| item.get("error"))
                .map(Value::to_string)
                .unwrap_or_default(),
            bad_status,
        ),
        "dynamicToolCall" => (
            item.get("contentItems")
                .map(Value::to_string)
                .unwrap_or_default(),
            item.get("success").and_then(Value::as_bool) == Some(false),
        ),
        _ => return None,
    };
    Some(AgentEvent::ToolResult {
        tool_use_id: str_of(item, "id"),
        content,
        is_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds server->client lines through the same classify/parse/approval
    /// path `Rpc::dispatch` uses and collects the events.
    fn replay(lines: &[Value]) -> Vec<AgentEvent> {
        let mut st = TurnState::default();
        let mut out = Vec::new();
        for l in lines {
            match classify(l) {
                Incoming::Notification { method, params } => {
                    out.extend(parse_notification(&method, &params, &mut st))
                }
                Incoming::Request { id, method, params } => {
                    out.extend(approval_event(&method, &params, id.to_string()))
                }
                Incoming::Response { .. } => {}
            }
        }
        out
    }

    #[test]
    fn handshake_and_thread_start_responses() {
        assert_eq!(init_params()["clientInfo"]["name"], "portal_desktop");
        let ok = json!({"id":1,"result":{"userAgent":"codex/1.0"}});
        assert!(matches!(
            classify(&ok),
            Incoming::Response {
                id: 1,
                result: Ok(_)
            }
        ));
        let thread = json!({"id":2,"result":{"thread":{"id":"th_1"},"model":"gpt-5"}});
        let Incoming::Response { result: Ok(v), .. } = classify(&thread) else {
            panic!()
        };
        assert_eq!(
            v.pointer("/thread/id").and_then(Value::as_str),
            Some("th_1")
        );
        let err = json!({"id":3,"error":{"code":-32600,"message":"bad"}});
        assert!(
            matches!(classify(&err), Incoming::Response { id: 3, result: Err(m) } if m == "bad")
        );
    }

    #[test]
    fn turn_with_deltas_command_approval_and_completion() {
        let events = replay(&[
            json!({"method":"turn/started","params":{"threadId":"th_1","turn":{"id":"tu_1","status":"inProgress"}}}),
            json!({"method":"item/agentMessage/delta","params":{"threadId":"th_1","turnId":"tu_1","itemId":"m1","delta":"Running "}}),
            json!({"method":"item/started","params":{"threadId":"th_1","turnId":"tu_1","item":{"type":"commandExecution","id":"c1","command":"ls","cwd":"/w","status":"inProgress"}}}),
            json!({"id":7,"method":"item/commandExecution/requestApproval","params":{"threadId":"th_1","turnId":"tu_1","itemId":"c1","command":"ls","cwd":"/w","reason":"outside sandbox"}}),
            json!({"method":"item/completed","params":{"threadId":"th_1","turnId":"tu_1","item":{"type":"commandExecution","id":"c1","status":"completed","aggregatedOutput":"a.txt","exitCode":0}}}),
            json!({"method":"item/completed","params":{"threadId":"th_1","turnId":"tu_1","item":{"type":"agentMessage","id":"m1","text":"Running ls, done."}}}),
            json!({"method":"turn/completed","params":{"threadId":"th_1","turn":{"id":"tu_1","status":"completed"}}}),
        ]);
        assert!(matches!(&events[0], AgentEvent::TextDelta { text, .. } if text == "Running "));
        assert!(
            matches!(&events[1], AgentEvent::ToolCall { id, name, .. } if id == "c1" && name == "commandExecution")
        );
        assert!(
            matches!(&events[2], AgentEvent::PermissionRequest { request_id, tool_name, tool_input, .. }
            if request_id == "7" && tool_name == "commandExecution" && tool_input["command"] == "ls")
        );
        assert!(
            matches!(&events[3], AgentEvent::ToolResult { tool_use_id, content, is_error }
            if tool_use_id == "c1" && content == "a.txt" && !is_error)
        );
        assert!(
            matches!(&events[4], AgentEvent::Message { text, .. } if text == "Running ls, done.")
        );
        assert!(
            matches!(&events[5], AgentEvent::TurnDone { final_text, is_error }
            if final_text.as_deref() == Some("Running ls, done.") && !is_error)
        );
    }

    #[test]
    fn failed_turn_and_error_notification() {
        let events = replay(&[
            json!({"method":"error","params":{"error":{"message":"rate limited"},"willRetry":true,"threadId":"t","turnId":"u"}}),
            json!({"method":"error","params":{"error":{"message":"boom"},"willRetry":false,"threadId":"t","turnId":"u"}}),
            json!({"method":"turn/completed","params":{"threadId":"t","turn":{"id":"u","status":"failed","error":{"message":"boom"}}}}),
        ]);
        assert_eq!(events.len(), 2); // the retryable error is dropped
        assert!(matches!(&events[0], AgentEvent::Raw { line } if line == "codex error: boom"));
        assert!(
            matches!(&events[1], AgentEvent::TurnDone { final_text, is_error: true }
            if final_text.as_deref() == Some("boom"))
        );
    }

    #[test]
    fn fast_turn_leaves_no_turn_id() {
        // turn/start's response is no longer a turn_id source, so a turn that
        // starts and completes before the response is read cannot leave a stale id.
        let mut st = TurnState::default();
        parse_notification("turn/started", &json!({"turn":{"id":"tu_1"}}), &mut st);
        parse_notification(
            "turn/completed",
            &json!({"turn":{"status":"completed"}}),
            &mut st,
        );
        assert!(st.turn_id.is_none());
    }

    #[test]
    fn acceptedits_is_untrusted_and_bypass_unsandboxed() {
        assert_eq!(
            policy_for_mode("acceptEdits"),
            ("untrusted", "workspace-write")
        );
        assert_eq!(
            policy_for_mode("bypassPermissions"),
            ("never", "danger-full-access")
        );
    }

    #[test]
    fn turn_id_tracked_for_interrupt_and_unknown_requests_not_approvals() {
        let mut st = TurnState::default();
        parse_notification("turn/started", &json!({"turn":{"id":"tu_9"}}), &mut st);
        assert_eq!(st.turn_id.as_deref(), Some("tu_9"));
        parse_notification(
            "turn/completed",
            &json!({"turn":{"status":"interrupted"}}),
            &mut st,
        );
        assert!(st.turn_id.is_none());
        assert!(approval_event("item/tool/call", &json!({}), "1".into()).is_none());
    }

    #[test]
    fn model_list_skips_hidden_and_mode_maps_to_policy() {
        let res = json!({"data":[
            {"id":"a","model":"gpt-5","displayName":"GPT-5","hidden":false},
            {"id":"b","model":"old","displayName":"Old","hidden":true}]});
        let models = parse_models(&res);
        assert_eq!(models.len(), 1);
        assert_eq!(
            (models[0].id.as_str(), models[0].label.as_str()),
            ("gpt-5", "GPT-5")
        );
        assert_eq!(policy_for_mode(""), ("untrusted", "workspace-write"));
        assert_eq!(policy_for_mode("bypassPermissions").0, "never");
    }
}
