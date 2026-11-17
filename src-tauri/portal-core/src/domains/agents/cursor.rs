//! `AgentBackend` for Cursor's `cursor-agent` CLI in print mode:
//! `cursor-agent -p --output-format stream-json --stream-partial-output
//! [--resume <chatId>] [--model m] [--force]` with the prompt piped on stdin,
//! **one process per turn**. A follow-up prompt is a new process resumed by chat id; the run
//! (and its `Exited`) lives until `kill`.
//!
//! Flags and event shapes are taken from Cursor's official docs, read
//! 2026-11-17: <https://cursor.com/docs/cli/reference/parameters>,
//! <https://cursor.com/docs/cli/reference/output-format> (system/init, user,
//! assistant, tool_call started/completed, result) and
//! <https://cursor.com/docs/cli/headless> (print-mode approval rules).
//! **Never run against a live `cursor-agent`** — not installed here; the tests
//! replay a transcript hand-built from those docs.
//!
//! Unverified: the prompt goes on stdin, not argv (a newline in an argv entry
//! breaks `.cmd` shims on Windows, and `--` may not stop commander dispatching
//! a prompt like `logout` as a subcommand). The headless/parameters docs do not
//! mention stdin; that `-p` reads the prompt from piped stdin is the reviewer's
//! report of the CLI's behaviour, not something these docs confirm.
//! Unverified: the docs name the binary `agent`; `cursor-agent` is the
//! historical installer name and is what we look for. The tool-result shape on
//! failure, `function`-style tool calls and the `--list-models` output are not
//! documented in enough detail, so errors are inferred (a result without
//! `success`) and `list_models` is empty (CLI picks its default).
//!
//! No approvals: print mode cannot ask, it rejects whatever needs approval, so
//! this backend never emits `PermissionRequest`. Default is that safe mode;
//! `--force` (run everything) is only passed for `bypassPermissions`, which
//! must be gated like coder's bypass before any caller sends it.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc::UnboundedSender;

use crate::process_ext::NoWindowExt;

use super::backend::{AgentBackend, BackendModel, RunControl, StartParams};
use super::events::AgentEvent;
use super::process;

const BIN: &str = "cursor-agent";

pub struct CursorBackend;

#[async_trait]
impl AgentBackend for CursorBackend {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn display_name(&self) -> &'static str {
        "Cursor"
    }

    async fn detect(&self) -> bool {
        find_bin().is_ok()
    }

    async fn list_models(&self) -> Vec<BackendModel> {
        // ponytail: `--list-models` output format is undocumented; empty means
        // the launcher uses the CLI default. Parse it once a sample exists.
        Vec::new()
    }

    async fn start(
        &self,
        params: StartParams,
        events_tx: UnboundedSender<AgentEvent>,
    ) -> Result<Box<dyn RunControl>, String> {
        // Resolved path, not `cmd /C`. The prompt is piped on stdin, never argv.
        let bin = find_bin().map_err(|_| {
            "`cursor-agent` CLI not found on PATH. Install Cursor CLI (https://cursor.com/cli)."
                .to_string()
        })?;
        crate::domains::projects::move_lock::ensure_unlocked(std::path::Path::new(&params.cwd))?;

        let run = Arc::new(CursorRun {
            bin,
            cwd: params.cwd,
            model: params.model.filter(|m| !m.trim().is_empty()),
            force: params.permission_mode == "bypassPermissions",
            chat_id: Mutex::new(params.resume_session_id),
            child: Mutex::new(None),
            busy: AtomicBool::new(false),
            interrupted: AtomicBool::new(false),
            killed: AtomicBool::new(false),
            events_tx,
        });
        if let Some(id) = run.chat_id.lock().unwrap().clone() {
            let _ = run
                .events_tx
                .send(AgentEvent::SessionStarted { session_id: id });
        }
        if !params.prompt.trim().is_empty() {
            run.spawn_turn(params.prompt)?;
        }
        Ok(Box::new(run))
    }
}

/// `cursor-agent` first; the docs' name `agent` is only a fallback (it is too
/// generic to prefer).
fn find_bin() -> which::Result<PathBuf> {
    which::which(BIN).or_else(|_| which::which("agent"))
}

struct CursorRun {
    bin: PathBuf,
    cwd: String,
    model: Option<String>,
    force: bool,
    /// Cursor's chat id: learned from `system/init`, passed as `--resume`.
    chat_id: Mutex<Option<String>>,
    /// The current turn's process; `None` between turns.
    child: Mutex<Option<Child>>,
    busy: AtomicBool,
    interrupted: AtomicBool,
    killed: AtomicBool,
    events_tx: UnboundedSender<AgentEvent>,
}

fn build_args(force: bool, model: Option<&str>, chat_id: Option<&str>) -> Vec<String> {
    let mut a: Vec<String> = [
        "-p",
        "--output-format",
        "stream-json",
        "--stream-partial-output",
    ]
    .map(String::from)
    .into();
    if force {
        a.push("--force".into());
    }
    if let Some(m) = model {
        a.extend(["--model".into(), m.into()]);
    }
    if let Some(id) = chat_id {
        a.extend(["--resume".into(), id.into()]);
    }
    a
}

impl CursorRun {
    /// Starts one turn's process. Never blocks: spawn is immediate and a task
    /// owns the output (callers hold the service's `runs` lock).
    fn spawn_turn(self: &Arc<Self>, prompt: String) -> Result<(), String> {
        if self.killed.load(Ordering::SeqCst) {
            return Err("run ended".into());
        }
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err("a Cursor turn is still running; wait for it or interrupt".into());
        }
        self.spawn_inner(prompt)
            .inspect_err(|_| self.busy.store(false, Ordering::SeqCst))
    }

    fn spawn_inner(self: &Arc<Self>, prompt: String) -> Result<(), String> {
        self.interrupted.store(false, Ordering::SeqCst);
        let args = build_args(
            self.force,
            self.model.as_deref(),
            self.chat_id.lock().unwrap().as_deref(),
        );
        let mut cmd = tokio::process::Command::new(&self.bin);
        cmd.no_window()
            .args(args)
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn cursor-agent: {e}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or("cursor-agent produced no stdout")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("cursor-agent produced no stderr")?;
        let mut stdin = child.stdin.take().ok_or("cursor-agent has no stdin")?;
        *self.child.lock().unwrap() = Some(child);

        tokio::spawn(async move {
            let _ = stdin.write_all(prompt.as_bytes()).await;
            // dropped: EOF on stdin is what starts the turn
        });
        let mut stderr = stderr;
        let stderr_task = tokio::spawn(async move {
            // Drain everything (a closed pipe can EPIPE-kill the CLI), cap after.
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf).await;
            buf.truncate(buf.floor_char_boundary(64 * 1024));
            buf
        });
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            let mut done = None;
            let mut last_msg = None;
            while let Ok(Some(line)) = lines.next_line().await {
                let events = match serde_json::from_str::<Value>(line.trim()) {
                    Ok(v) => parse_line(&v, &mut this.chat_id.lock().unwrap()),
                    Err(_) if line.trim().is_empty() => vec![],
                    Err(_) => vec![AgentEvent::Raw { line }],
                };
                for e in events {
                    // Held back: TurnDone must follow `busy = false` so the
                    // follow-up send the UI fires on it is not refused.
                    match e {
                        AgentEvent::TurnDone {
                            final_text,
                            is_error,
                        } => {
                            // `result.result` glues every segment; the last
                            // Message is the final answer.
                            let final_text = if is_error {
                                final_text
                            } else {
                                last_msg.take().or(final_text)
                            };
                            done = Some(AgentEvent::TurnDone {
                                final_text,
                                is_error,
                            });
                        }
                        e => {
                            if let AgentEvent::Message { text, .. } = &e {
                                last_msg = Some(text.clone());
                            }
                            let _ = this.events_tx.send(e);
                        }
                    }
                }
                // `result` ends the turn: a grandchild (dev server, MCP) can
                // hold the stdout pipe open, so EOF may never come.
                if done.is_some() {
                    break;
                }
            }
            let child = this.child.lock().unwrap().take();
            if let Some(mut c) = child {
                let grace = Duration::from_secs(2);
                if tokio::time::timeout(grace, c.wait()).await.is_err() {
                    process::start_kill(&mut c);
                    let _ = tokio::time::timeout(grace, c.wait()).await;
                }
            }
            let err = tokio::time::timeout(Duration::from_secs(2), stderr_task)
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_default();
            let interrupted = this.interrupted.load(Ordering::SeqCst);
            this.busy.store(false, Ordering::SeqCst);
            if this.killed.load(Ordering::SeqCst) {
                return; // `kill` already sent Exited
            }
            let err = err.trim().to_string();
            let ev = done.unwrap_or_else(|| {
                // No terminal `result` line: interrupted, or the CLI failed
                // (auth, headless disabled, untrusted workspace...) — stderr says.
                AgentEvent::TurnDone {
                    final_text: (!interrupted && !err.is_empty()).then(|| err.clone()),
                    is_error: !interrupted,
                }
            });
            if !err.is_empty() && matches!(ev, AgentEvent::TurnDone { is_error: true, .. }) {
                let _ = this.events_tx.send(AgentEvent::Raw { line: err });
            }
            let _ = this.events_tx.send(ev);
        });
        Ok(())
    }

    fn kill_child(&self) {
        if let Some(c) = self.child.lock().unwrap().as_mut() {
            process::start_kill(c);
        }
    }
}

#[async_trait]
impl RunControl for Arc<CursorRun> {
    async fn send_user(&self, text: String) -> Result<(), String> {
        self.spawn_turn(text)
    }

    async fn decide_permission(&self, _request_id: String, _allow: bool) -> Result<(), String> {
        Err("Cursor print mode has no approvals".to_string())
    }

    async fn interrupt(&self) -> Result<(), String> {
        self.interrupted.store(true, Ordering::SeqCst);
        self.kill_child();
        Ok(())
    }

    async fn kill(&self) -> Result<(), String> {
        self.killed.store(true, Ordering::SeqCst);
        self.kill_child();
        let _ = self.events_tx.send(AgentEvent::Exited { code: None });
        Ok(())
    }
}

// ---- wire parsing ----------------------------------------------------------

fn str_of(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Normalizes one stream-json line. `chat_id` is set from `system/init`, which
/// also yields `SessionStarted` (first time only). `user` echoes and unknown
/// types are dropped. Pure — unit-tested without the CLI.
fn parse_line(v: &Value, chat_id: &mut Option<String>) -> Vec<AgentEvent> {
    match v.get("type").and_then(Value::as_str) {
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            let sid = str_of(v, "session_id");
            if sid.is_empty() || chat_id.as_deref() == Some(sid.as_str()) {
                return vec![];
            }
            *chat_id = Some(sid.clone());
            vec![AgentEvent::SessionStarted { session_id: sid }]
        }
        Some("assistant") => {
            let text: String = v
                .pointer("/message/content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|c| c.get("text").and_then(Value::as_str))
                .collect();
            if text.is_empty() {
                return vec![];
            }
            // Per the docs: `timestamp_ms` without `model_call_id` is a new
            // delta; every other assistant line is a full-message flush.
            if v.get("timestamp_ms").is_some() && v.get("model_call_id").is_none() {
                vec![AgentEvent::TextDelta {
                    text,
                    parent_tool_use_id: None,
                }]
            } else {
                vec![AgentEvent::Message {
                    role: "assistant".into(),
                    text,
                    parent_tool_use_id: None,
                }]
            }
        }
        Some("tool_call") => tool_event(v).into_iter().collect(),
        Some("result") => {
            let failed = v.get("is_error").and_then(Value::as_bool).unwrap_or(false)
                || v.get("subtype")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s != "success");
            vec![AgentEvent::TurnDone {
                final_text: v.get("result").and_then(Value::as_str).map(str::to_string),
                is_error: failed,
            }]
        }
        _ => vec![],
    }
}

fn tool_event(v: &Value) -> Option<AgentEvent> {
    let id = str_of(v, "call_id");
    // `tool_call` is `{ "<kind>ToolCall": { args, result? } }` or
    // `{ "function": { name, arguments } }`.
    let (kind, body) = v.get("tool_call")?.as_object()?.iter().next()?;
    match v.get("subtype").and_then(Value::as_str)? {
        "started" => {
            let (name, input) = if kind == "function" {
                let args = body.get("arguments").cloned().unwrap_or(Value::Null);
                let args = args
                    .as_str()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or(args);
                (str_of(body, "name"), args)
            } else {
                let name = kind.strip_suffix("ToolCall").unwrap_or(kind);
                (
                    name.to_string(),
                    body.get("args").cloned().unwrap_or_else(|| json!({})),
                )
            };
            Some(AgentEvent::ToolCall {
                id,
                name,
                input,
                parent_tool_use_id: None,
            })
        }
        "completed" => {
            let (content, is_error) = match body.get("result") {
                None => (String::new(), false),
                Some(r) => match r.get("success") {
                    Some(s) => (
                        s.get("content")
                            .and_then(Value::as_str)
                            .map_or_else(|| s.to_string(), str::to_string),
                        false,
                    ),
                    None => (r.to_string(), true),
                },
            };
            Some(AgentEvent::ToolResult {
                tool_use_id: id,
                content,
                is_error,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replay(lines: &[Value]) -> (Vec<AgentEvent>, Option<String>) {
        let mut chat = None;
        let ev = lines
            .iter()
            .flat_map(|l| parse_line(l, &mut chat))
            .collect();
        (ev, chat)
    }

    #[test]
    fn full_turn_transcript() {
        let (ev, chat) = replay(&[
            json!({"type":"system","subtype":"init","apiKeySource":"login","cwd":"/w","session_id":"c1","model":"M","permissionMode":"default"}),
            json!({"type":"user","message":{"role":"user","content":[{"type":"text","text":"hi"}]},"session_id":"c1"}),
            json!({"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"I'll read "}]},"session_id":"c1","timestamp_ms":1}),
            json!({"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"I'll read it"}]},"session_id":"c1","timestamp_ms":2,"model_call_id":"m1"}),
            json!({"type":"tool_call","subtype":"started","call_id":"t1","tool_call":{"readToolCall":{"args":{"path":"README.md"}}},"session_id":"c1"}),
            json!({"type":"tool_call","subtype":"completed","call_id":"t1","tool_call":{"readToolCall":{"args":{"path":"README.md"},"result":{"success":{"content":"# P","totalLines":1}}}},"session_id":"c1"}),
            json!({"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Done."}]},"session_id":"c1"}),
            json!({"type":"result","subtype":"success","is_error":false,"duration_ms":5,"result":"Done.","session_id":"c1"}),
        ]);
        assert_eq!(chat.as_deref(), Some("c1"));
        assert!(matches!(&ev[0], AgentEvent::SessionStarted { session_id } if session_id == "c1"));
        assert!(matches!(&ev[1], AgentEvent::TextDelta { text, .. } if text == "I'll read "));
        assert!(matches!(&ev[2], AgentEvent::Message { text, .. } if text == "I'll read it"));
        assert!(
            matches!(&ev[3], AgentEvent::ToolCall { id, name, input, .. }
            if id == "t1" && name == "read" && input["path"] == "README.md")
        );
        assert!(
            matches!(&ev[4], AgentEvent::ToolResult { tool_use_id, content, is_error }
            if tool_use_id == "t1" && content == "# P" && !is_error)
        );
        assert!(matches!(&ev[5], AgentEvent::Message { text, .. } if text == "Done."));
        assert!(
            matches!(&ev[6], AgentEvent::TurnDone { final_text, is_error }
            if final_text.as_deref() == Some("Done.") && !is_error)
        );
        assert_eq!(ev.len(), 7); // the user echo is dropped
    }

    #[test]
    fn init_is_not_repeated_for_a_resumed_chat() {
        let mut chat = Some("c1".to_string());
        let init = json!({"type":"system","subtype":"init","session_id":"c1"});
        assert!(parse_line(&init, &mut chat).is_empty());
    }

    #[test]
    fn error_result_and_failed_tool() {
        let (ev, _) = replay(&[
            json!({"type":"tool_call","subtype":"completed","call_id":"t2","tool_call":{"writeToolCall":{"args":{},"result":{"rejected":{"reason":"needs approval"}}}}}),
            json!({"type":"tool_call","subtype":"started","call_id":"t3","tool_call":{"function":{"name":"grep","arguments":"{\"q\":\"x\"}"}}}),
            json!({"type":"result","subtype":"error","is_error":true,"result":"boom"}),
        ]);
        assert!(matches!(
            &ev[0],
            AgentEvent::ToolResult { is_error: true, .. }
        ));
        assert!(
            matches!(&ev[1], AgentEvent::ToolCall { name, input, .. } if name == "grep" && input["q"] == "x")
        );
        assert!(
            matches!(&ev[2], AgentEvent::TurnDone { final_text, is_error: true } if final_text.as_deref() == Some("boom"))
        );
    }

    #[test]
    fn args_default_safe_and_force_only_on_request() {
        let a = build_args(false, None, None);
        assert!(!a.contains(&"--force".to_string()));
        assert!(!a.contains(&"--".to_string())); // the prompt is on stdin, never argv
        let b = build_args(true, Some("m"), Some("c1"));
        assert!(b.windows(2).any(|w| w == ["--resume", "c1"]));
        assert!(b.windows(2).any(|w| w == ["--model", "m"]));
        assert!(b.contains(&"--force".to_string()));
    }

    #[test]
    fn spawn_turn_after_kill_is_refused() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let run = Arc::new(CursorRun {
            bin: PathBuf::from("cursor-agent"),
            cwd: ".".into(),
            model: None,
            force: false,
            chat_id: Mutex::new(None),
            child: Mutex::new(None),
            busy: AtomicBool::new(false),
            interrupted: AtomicBool::new(false),
            killed: AtomicBool::new(true),
            events_tx: tx,
        });
        assert_eq!(run.spawn_turn("hi".into()).unwrap_err(), "run ended");
    }
}
