//! Git push for the coder workspace. Mirrors `git_commit.rs`'s shelling-out
//! style. Never force-pushes: no `--force`/`-f` ever appears in `push_args`,
//! and there is no parameter that could add one.

use std::path::Path;
use std::process::Command;

use crate::domains::projects::move_lock;
use crate::process_ext::NoWindowExt;

/// Pushes the current branch to `origin`, setting the upstream when the
/// branch doesn't have one yet (every remote this app ever reads is named
/// `origin` — see `discovery::git::probe`). Returns the branch pushed.
pub fn git_push(workspace: &str) -> Result<String, String> {
    let branch = git_output(workspace, &["branch", "--show-current"])?;
    if branch.is_empty() {
        return Err("Detached HEAD — check out a branch before pushing".into());
    }

    let has_upstream = git_output(
        workspace,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
    .is_ok();

    let args = push_args(&branch, has_upstream);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git_output(workspace, &arg_refs)?;

    Ok(branch)
}

/// The exact argv for `git push`, isolated so it's testable without a real
/// repo or remote.
fn push_args(branch: &str, has_upstream: bool) -> Vec<String> {
    if has_upstream {
        vec!["push".to_string()]
    } else {
        vec![
            "push".to_string(),
            "-u".to_string(),
            "origin".to_string(),
            branch.to_string(),
        ]
    }
}

fn git_output(workspace: &str, args: &[&str]) -> Result<String, String> {
    super::tools::reject_unc(workspace)?;
    move_lock::ensure_unlocked(Path::new(workspace))?;
    let output = Command::new("git")
        .no_window()
        .args(["-c", "core.fsmonitor="])
        // ponytail: forces a stalled auth prompt into a fast, real git error
        // instead of hanging the UI forever; SSH keys and credential helpers
        // still authenticate normally, this only disables the interactive
        // username/password prompt git falls back to without them.
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_args_never_force_pushes() {
        for has_upstream in [true, false] {
            let args = push_args("main", has_upstream);
            assert!(!args.iter().any(|a| a == "--force" || a == "-f"));
        }
    }

    #[test]
    fn push_args_sets_upstream_when_missing() {
        assert_eq!(
            push_args("feature/x", false),
            vec!["push", "-u", "origin", "feature/x"]
        );
    }

    #[test]
    fn push_args_plain_when_upstream_exists() {
        assert_eq!(push_args("main", true), vec!["push".to_string()]);
    }
}
