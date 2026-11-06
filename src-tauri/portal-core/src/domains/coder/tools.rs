//! Native tool registry + executor.
//!
//! Each tool maps to real desktop capability (filesystem, shell). Read-only
//! tools always run; mutating tools are gated by the permission layer. All
//! paths are resolved *inside* the thread's `workspace_root` — a `..` escape
//! is rejected before any IO happens.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::utils::pnpm_workspace::{prepare_shell_command, warn_if_broken_pnpm_workspace};
use serde_json::{json, Value};

use super::types::ToolCall;

/// OpenAI-format tool specs advertised to the model on every request.
pub fn tool_specs() -> Vec<Value> {
    vec![
        spec(
            "read_file",
            "Read a UTF-8 text file relative to the workspace root.",
            json!({
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Path relative to workspace root." } },
                "required": ["path"]
            }),
        ),
        spec(
            "write_file",
            "Create or overwrite a file with the given full content. Read the file first when editing.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
        ),
        spec(
            "edit_file",
            "Replace an exact substring in a file. `old_string` must occur exactly once. \
             Prefer this over write_file for small changes.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "old_string": { "type": "string", "description": "Exact text to replace (must be unique)." },
                    "new_string": { "type": "string" }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        ),
        spec(
            "list_dir",
            "List entries of a directory relative to the workspace root.",
            json!({
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Defaults to '.'." } }
            }),
        ),
        spec(
            "search_files",
            "Search file contents recursively for a substring. Returns matching path:line lines.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "path": { "type": "string", "description": "Subdir to search. Defaults to '.'." }
                },
                "required": ["query"]
            }),
        ),
        spec(
            "run_command",
            "Run a shell command in the workspace root. Commands run in a fast background \
             shell by default. Pass terminal_id from list_terminals only when you need to run \
             inside a specific interactive session tab.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "terminal_id": {
                        "type": "string",
                        "description": "Optional: run inside a specific interactive terminal tab from list_terminals."
                    }
                },
                "required": ["command"]
            }),
        ),
        spec(
            "list_terminals",
            "List interactive terminals attached to this coder session (id + label).",
            json!({
                "type": "object",
                "properties": {},
            }),
        ),
    ]
}

fn spec(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}

/// Extra tools that delegate to the agent-platform's higher-level AI
/// capabilities (multi-agent orchestration). Only advertised when delegation
/// is configured (a team-template id + auth). Executed by the *service*
/// (async HTTP), not by [`execute`].
pub fn platform_tool_specs() -> Vec<Value> {
    vec![spec(
        "delegate_task",
        "Delegate a hard planning/analysis goal to the agent-platform's multi-agent \
         team, which runs a planning+execution process and returns its result. Use for \
         work that benefits from decomposition beyond your own local tools.",
        json!({
            "type": "object",
            "properties": { "goal": { "type": "string", "description": "The goal for the agent team." } },
            "required": ["goal"]
        }),
    )]
}

/// Web search, run by the platform's `/api/v1/search`.
///
/// Separate from [`platform_tool_specs`] because it needs no team template —
/// only a reachable platform, which the coder already requires for its turn
/// loop. Read-only, so it is offered in Ask mode too.
pub fn web_search_tool_specs() -> Vec<Value> {
    vec![spec(
        "web_search",
        "Search the web for current information — documentation, error messages,          library versions, anything outside the workspace or your training data.          Returns titles, URLs and snippets.",
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "What to search for, in plain words." },
                "limit": {
                    "type": "integer",
                    "description": "How many results to return, 1-10. Defaults to 5."
                }
            },
            "required": ["query"]
        }),
    )]
}

pub fn multitask_tool_specs() -> Vec<Value> {
    vec![spec(
        "spawn_parallel_tasks",
        "Split independent work into parallel git worktree-backed sub-agents. Use this only in coordinator threads when the tasks are independent enough to run separately.",
        json!({
            "type": "object",
            "properties": {
                "base_ref": {
                    "type": "string",
                    "description": "Optional git base ref to branch worktrees from. Defaults to HEAD."
                },
                "issue_urls": {
                    "type": "array",
                    "description": "Optional GitHub issue URLs to fan out directly.",
                    "items": { "type": "string" }
                },
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "prompt": { "type": "string" },
                            "github_issue_url": { "type": "string" }
                        },
                        "required": ["title"]
                    }
                }
            }
        }),
    )]
}

/// A tool routed to the platform over HTTP rather than executed locally.
pub fn is_platform_tool(tool: &str) -> bool {
    matches!(tool, "delegate_task" | "web_search")
}

/// The relative path a file-mutating tool targets, for change tracking.
pub fn mutated_path(tool: &str, args: &Value) -> Option<String> {
    match tool {
        "write_file" | "edit_file" => args.get("path").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// Read a file under `root`, returning `None` if it does not exist. Used to
/// snapshot `before` content for change review.
pub fn read_raw(root: &str, rel: &str) -> Option<String> {
    let path = resolve(root, rel).ok()?;
    std::fs::read_to_string(path).ok()
}

/// Write raw content to a file under `root` (used when accepting/rejecting a
/// reviewed change rewrites the file). Creates parent dirs.
pub fn write_raw(root: &str, rel: &str, content: &str) -> Result<(), String> {
    let path = resolve(root, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("write {rel}: {e}"))
}

/// A tool that does not change disk or run processes — never needs approval.
/// Platform delegation is *not* read-only (it spends compute / runs agents).
pub fn is_read_only(tool: &str) -> bool {
    matches!(
        tool,
        "read_file" | "list_dir" | "search_files" | "list_terminals" | "web_search"
    )
}

/// Parse a tool call's JSON argument string.
pub fn parse_args(call: &ToolCall) -> Result<Value, String> {
    if call.function.arguments.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&call.function.arguments)
        .map(|args| normalize_args(&call.function.name, args))
        .map_err(|e| format!("invalid tool arguments JSON: {e}"))
}

/// Short human summary for the approval UI.
pub fn summarize(tool: &str, args: &Value) -> String {
    match tool {
        "run_command" => format!(
            "$ {}",
            args.get("command").and_then(Value::as_str).unwrap_or("")
        ),
        "write_file" => format!(
            "write {}",
            args.get("path").and_then(Value::as_str).unwrap_or("")
        ),
        "edit_file" => format!(
            "edit {}",
            args.get("path").and_then(Value::as_str).unwrap_or("")
        ),
        _ => tool.to_string(),
    }
}

/// The allowlist rule proposed when the user picks "accept & remember".
/// For commands this is the first token (e.g. `git`); for writes, the file's
/// parent dir glob (e.g. `src/*`). Kept coarse on purpose — the user can edit.
pub fn suggested_rule(tool: &str, args: &Value) -> String {
    match tool {
        "run_command" => args
            .get("command")
            .and_then(Value::as_str)
            .and_then(|c| c.split_whitespace().next())
            .unwrap_or("")
            .to_string(),
        "write_file" | "edit_file" => {
            let p = args.get("path").and_then(Value::as_str).unwrap_or("");
            match Path::new(p)
                .parent()
                .and_then(|d| d.to_str())
                .filter(|s| !s.is_empty())
            {
                Some(dir) => format!("{dir}/*"),
                None => "*".to_string(),
            }
        }
        _ => String::new(),
    }
}

/// Execute a tool. `root` is the absolute workspace root.
pub fn execute(root: &str, tool: &str, args: &Value) -> Result<String, String> {
    match tool {
        "read_file" => read_file(root, arg_str(args, tool, "path")?),
        "write_file" => write_file(
            root,
            arg_str(args, tool, "path")?,
            arg_str(args, tool, "content")?,
        ),
        "edit_file" => edit_file(
            root,
            arg_str(args, tool, "path")?,
            arg_str(args, tool, "old_string")?,
            arg_str(args, tool, "new_string")?,
        ),
        "list_dir" => list_dir(
            root,
            args.get("path").and_then(Value::as_str).unwrap_or("."),
        ),
        "search_files" => search_files(
            root,
            arg_str(args, tool, "query")?,
            args.get("path").and_then(Value::as_str).unwrap_or("."),
        ),
        "run_command" => run_command(root, arg_str(args, tool, "command")?),
        "list_terminals" => Ok("[]".into()),
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Argument names weak models reach for instead of the ones we declare. A model
/// trained on other tool schemas emits `file_path`, `old_str` or `cmd`; the
/// intent is right, so rename rather than bounce the turn. Local 8B models burn
/// whole turns looping on `missing arg: path` otherwise.
fn aliases(key: &str) -> &'static [&'static str] {
    match key {
        "path" => &[
            "file_path",
            "filepath",
            "filename",
            "file",
            "file_name",
            "target_file",
            "path_to_file",
        ],
        "content" => &["contents", "text", "file_text", "new_content", "body", "data"],
        "old_string" => &["old_str", "old_text", "old", "search", "find"],
        "new_string" => &["new_str", "new_text", "new", "replace", "replacement"],
        "command" => &["cmd", "shell_command", "command_line", "script"],
        "query" => &["pattern", "search", "q", "text"],
        "terminal_id" => &["terminalId", "terminal"],
        _ => &[],
    }
}

/// Every argument name a tool declares, read back off the advertised spec so
/// this cannot drift from what the model was told.
fn spec_arg_names(tool: &str) -> Vec<String> {
    spec_field(tool, "properties")
        .and_then(|p| p.as_object().map(|m| m.keys().cloned().collect()))
        .unwrap_or_default()
}

/// A tool's required argument names, from the same spec the model receives.
fn required_args(tool: &str) -> Vec<String> {
    spec_field(tool, "required")
        .and_then(|r| serde_json::from_value::<Vec<String>>(r).ok())
        .unwrap_or_default()
}

fn spec_field(tool: &str, field: &str) -> Option<Value> {
    tool_specs()
        .iter()
        .chain(platform_tool_specs().iter())
        .find(|s| s.pointer("/function/name").and_then(Value::as_str) == Some(tool))
        .and_then(|s| s.pointer(&format!("/function/parameters/{field}")).cloned())
}

/// Rename aliased argument keys to the ones this tool declares.
///
/// Call this once where tool arguments enter, *before* the permission check:
/// everything downstream (gating, approval text, change tracking, execution)
/// reads canonical keys, so salvaging later would gate on one path and write
/// another.
pub fn normalize_args(tool: &str, mut args: Value) -> Value {
    let Some(map) = args.as_object_mut() else {
        return args;
    };
    for key in spec_arg_names(tool) {
        if map.contains_key(&key) {
            continue;
        }
        let found = aliases(&key)
            .iter()
            .copied()
            .find(|a| map.get(*a).is_some_and(|v| !v.is_null()));
        if let Some(alias) = found {
            if let Some(v) = map.remove(alias) {
                map.insert(key, v);
            }
        }
    }
    args
}

/// Error text a weak model can act on: what is missing, the tool's full
/// required-arg list, what it actually sent, and a correctly shaped call.
fn arg_error(tool: &str, key: &str, args: &Value) -> String {
    let required = required_args(tool);
    let sent = match args.as_object() {
        Some(m) if !m.is_empty() => m.keys().cloned().collect::<Vec<_>>().join(", "),
        _ => "nothing".to_string(),
    };
    let example = required
        .iter()
        .map(|k| format!("\"{k}\": \"...\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "missing arg: {key}. Tool `{tool}` requires: {req}. You sent: {sent}. \
         Call `{tool}` again with every required argument, e.g. {{{example}}}",
        req = required.join(", "),
    )
}

fn arg_str<'a>(args: &'a Value, tool: &str, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| arg_error(tool, key, args))
}

/// Resolve `rel` under `root`, rejecting escapes outside the workspace.
fn resolve(root: &str, rel: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(root);
    let joined = root.join(rel);
    // Normalize without touching the filesystem (path may not exist yet).
    let mut normalized = PathBuf::new();
    for comp in joined.components() {
        use std::path::Component::*;
        match comp {
            ParentDir => {
                if !normalized.pop() {
                    return Err("path escapes workspace root".into());
                }
            }
            CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(&root) {
        return Err("path escapes workspace root".into());
    }
    Ok(normalized)
}

pub fn read_file(root: &str, rel: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    std::fs::read_to_string(&path).map_err(|e| format!("read {rel}: {e}"))
}

pub fn write_file(root: &str, rel: &str, content: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("write {rel}: {e}"))?;
    Ok(format!("wrote {} bytes to {rel}", content.len()))
}

fn edit_file(root: &str, rel: &str, old: &str, new: &str) -> Result<String, String> {
    if old.is_empty() {
        return Err("old_string must not be empty".into());
    }
    let path = resolve(root, rel)?;
    let content = std::fs::read_to_string(&path).map_err(|e| format!("read {rel}: {e}"))?;
    let count = content.matches(old).count();
    if count == 0 {
        return Err(format!("old_string not found in {rel}"));
    }
    if count > 1 {
        return Err(format!(
            "old_string occurs {count} times in {rel}; make it unique"
        ));
    }
    let updated = content.replacen(old, new, 1);
    std::fs::write(&path, updated).map_err(|e| format!("write {rel}: {e}"))?;
    Ok(format!("edited {rel}"))
}

fn list_dir(root: &str, rel: &str) -> Result<String, String> {
    let entries = list_dir_entries(root, rel)?;
    Ok(entries
        .iter()
        .map(|e| {
            if e.is_dir {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Structured directory listing for the workspace file explorer.
pub fn list_dir_entries(
    root: &str,
    rel: &str,
) -> Result<Vec<crate::domains::coder::types::WorkspaceDirEntry>, String> {
    use crate::domains::coder::types::WorkspaceDirEntry;
    let path = resolve(root, rel)?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| format!("list {rel}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == "node_modules" || name == "target" {
            continue;
        }
        let is_dir = entry.path().is_dir();
        let child_rel = if rel == "." {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        out.push(WorkspaceDirEntry {
            name,
            path: child_rel,
            is_dir,
        });
    }
    out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(out)
}

fn search_files(root: &str, query: &str, rel: &str) -> Result<String, String> {
    let base = resolve(root, rel)?;
    let mut hits = Vec::new();
    walk(&base, &mut |file| {
        if let Ok(text) = std::fs::read_to_string(file) {
            for (i, line) in text.lines().enumerate() {
                if line.contains(query) {
                    let shown = file.strip_prefix(root).unwrap_or(file).to_string_lossy();
                    hits.push(format!("{}:{}: {}", shown, i + 1, line.trim()));
                    if hits.len() >= 200 {
                        return false;
                    }
                }
            }
        }
        true
    });
    if hits.is_empty() {
        Ok("no matches".into())
    } else {
        Ok(hits.join("\n"))
    }
}

/// Depth-first file walk; `f` returns false to stop early. Skips common heavy
/// dirs to keep searches responsive.
fn walk(dir: &Path, f: &mut dyn FnMut(&Path) -> bool) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return true;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skip = matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some(".git" | "node_modules" | "target" | ".svelte-kit" | "dist" | "build")
            );
            if !skip && !walk(&path, f) {
                return false;
            }
        } else if !f(&path) {
            return false;
        }
    }
    true
}

fn run_command(root: &str, command: &str) -> Result<String, String> {
    let exec_command = prepare_shell_command(command, root);
    let warning = warn_if_broken_pnpm_workspace(root);

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &exec_command])
            .current_dir(root)
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&exec_command)
            .current_dir(root)
            .output()
    }
    .map_err(|e| format!("spawn: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let body = if output.status.success() {
        if stdout.is_empty() {
            "(no output)".to_string()
        } else {
            stdout.to_string()
        }
    } else {
        format!(
            "exit {}\n{stdout}{stderr}",
            output.status.code().unwrap_or(-1)
        )
    };

    Ok(match warning {
        Some(w) => format!("{w}\n{body}"),
        None => body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("coder_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn edit_file_replaces_unique() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        write_file(root_s, "a.txt", "hello world").unwrap();
        edit_file(root_s, "a.txt", "world", "caveman").unwrap();
        assert_eq!(read_file(root_s, "a.txt").unwrap(), "hello caveman");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn edit_file_rejects_ambiguous() {
        let root = tmp_root();
        let root_s = root.to_str().unwrap();
        write_file(root_s, "a.txt", "x x").unwrap();
        assert!(edit_file(root_s, "a.txt", "x", "y").is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn normalize_salvages_aliased_args() {
        // The exact shape a Claude-trained model emits for our write_file.
        let out = normalize_args(
            "write_file",
            json!({ "file_path": "App.tsx", "contents": "hi" }),
        );
        assert_eq!(out["path"], "App.tsx");
        assert_eq!(out["content"], "hi");
        assert!(out.get("file_path").is_none());

        let out = normalize_args("edit_file", json!({ "old_str": "a", "new_str": "b" }));
        assert_eq!(out["old_string"], "a");
        assert_eq!(out["new_string"], "b");

        assert_eq!(
            normalize_args("run_command", json!({ "cmd": "ls" }))["command"],
            "ls"
        );
    }

    #[test]
    fn normalize_leaves_canonical_args_alone() {
        let args = json!({ "path": "a.txt", "content": "x", "file_path": "WRONG" });
        let out = normalize_args("write_file", args);
        assert_eq!(out["path"], "a.txt");
        assert_eq!(out["file_path"], "WRONG", "unknown keys are left untouched");
    }

    #[test]
    fn missing_arg_error_names_every_required_arg() {
        let err = execute("/tmp", "write_file", &json!({ "content": "x" })).unwrap_err();
        assert!(err.contains("missing arg: path"), "{err}");
        assert!(err.contains("requires: path, content"), "{err}");
        assert!(err.contains("You sent: content"), "{err}");
        assert!(err.contains(r#"{"path": "...", "content": "..."}"#), "{err}");
    }

    #[test]
    fn resolve_blocks_escape() {
        let root = tmp_root();
        let err = read_file(root.to_str().unwrap(), "../secret");
        assert!(err.is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}
