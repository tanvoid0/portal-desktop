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
            "Run a shell command in the workspace root and return combined stdout/stderr.",
            json!({
                "type": "object",
                "properties": { "command": { "type": "string" } },
                "required": ["command"]
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

/// A tool routed to the platform over HTTP rather than executed locally.
pub fn is_platform_tool(tool: &str) -> bool {
    matches!(tool, "delegate_task")
}

/// The relative path a file-mutating tool targets, for change tracking.
pub fn mutated_path(tool: &str, args: &Value) -> Option<String> {
    match tool {
        "write_file" | "edit_file" => {
            args.get("path").and_then(Value::as_str).map(str::to_string)
        }
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
    matches!(tool, "read_file" | "list_dir" | "search_files")
}

/// Parse a tool call's JSON argument string.
pub fn parse_args(call: &ToolCall) -> Result<Value, String> {
    if call.function.arguments.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&call.function.arguments)
        .map_err(|e| format!("invalid tool arguments JSON: {e}"))
}

/// Short human summary for the approval UI.
pub fn summarize(tool: &str, args: &Value) -> String {
    match tool {
        "run_command" => format!("$ {}", args.get("command").and_then(Value::as_str).unwrap_or("")),
        "write_file" => format!("write {}", args.get("path").and_then(Value::as_str).unwrap_or("")),
        "edit_file" => format!("edit {}", args.get("path").and_then(Value::as_str).unwrap_or("")),
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
            match Path::new(p).parent().and_then(|d| d.to_str()).filter(|s| !s.is_empty()) {
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
        "read_file" => read_file(root, arg_str(args, "path")?),
        "write_file" => write_file(root, arg_str(args, "path")?, arg_str(args, "content")?),
        "edit_file" => edit_file(
            root,
            arg_str(args, "path")?,
            arg_str(args, "old_string")?,
            arg_str(args, "new_string")?,
        ),
        "list_dir" => list_dir(root, args.get("path").and_then(Value::as_str).unwrap_or(".")),
        "search_files" => search_files(
            root,
            arg_str(args, "query")?,
            args.get("path").and_then(Value::as_str).unwrap_or("."),
        ),
        "run_command" => run_command(root, arg_str(args, "command")?),
        other => Err(format!("unknown tool: {other}")),
    }
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing arg: {key}"))
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

fn read_file(root: &str, rel: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    std::fs::read_to_string(&path).map_err(|e| format!("read {rel}: {e}"))
}

fn write_file(root: &str, rel: &str, content: &str) -> Result<String, String> {
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
        return Err(format!("old_string occurs {count} times in {rel}; make it unique"));
    }
    let updated = content.replacen(old, new, 1);
    std::fs::write(&path, updated).map_err(|e| format!("write {rel}: {e}"))?;
    Ok(format!("edited {rel}"))
}

fn list_dir(root: &str, rel: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&path).map_err(|e| format!("list {rel}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let suffix = if entry.path().is_dir() { "/" } else { "" };
        out.push(format!("{name}{suffix}"));
    }
    out.sort();
    Ok(out.join("\n"))
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
    let Ok(entries) = std::fs::read_dir(dir) else { return true };
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
        format!("exit {}\n{stdout}{stderr}", output.status.code().unwrap_or(-1))
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
    fn resolve_blocks_escape() {
        let root = tmp_root();
        let err = read_file(root.to_str().unwrap(), "../secret");
        assert!(err.is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}
