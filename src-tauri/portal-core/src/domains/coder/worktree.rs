use std::path::{Path, PathBuf};
use std::process::Command;

use crate::process_ext::NoWindowExt;

pub struct WorktreeSpec {
    pub path: PathBuf,
    pub branch: String,
    pub base_ref: String,
}

pub fn is_git_repo(path: &Path) -> bool {
    git_ok(path, ["rev-parse", "--show-toplevel"])
}

pub fn has_uncommitted_changes(path: &Path) -> bool {
    match git_output(path, ["status", "--porcelain"]) {
        Ok(out) => !out.trim().is_empty(),
        Err(_) => false,
    }
}

pub fn create_worktree(repo_root: &Path, spec: &WorktreeSpec) -> Result<(), String> {
    if let Some(parent) = spec.path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create worktree dir: {e}"))?;
    }
    let args = vec![
        "worktree".to_string(),
        "add".to_string(),
        "-b".to_string(),
        spec.branch.clone(),
        spec.path.to_string_lossy().to_string(),
        spec.base_ref.clone(),
    ];
    git_run(repo_root, args)
}

pub fn remove_worktree(repo_root: &Path, path: &Path, force: bool) -> Result<(), String> {
    let mut args = vec!["worktree".to_string(), "remove".to_string()];
    if force {
        args.push("--force".to_string());
    }
    args.push(path.to_string_lossy().to_string());
    git_run(repo_root, args)
}

pub fn worktree_root(repo_root: &Path, coordinator_thread_id: &str) -> PathBuf {
    repo_root
        .join(".portal-worktrees")
        .join(sanitize_slug(coordinator_thread_id))
}

pub fn sanitize_slug(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let keep = ch.is_ascii_alphanumeric();
        if keep {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn git_ok(path: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    git_run(path, args).is_ok()
}

fn git_output(
    path: &Path,
    args: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(path).no_window();
    for arg in args {
        cmd.arg(arg.as_ref());
    }
    let output = cmd.output().map_err(|e| format!("git spawn failed: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_run(path: &Path, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<(), String> {
    git_output(path, args).map(|_| ())
}
