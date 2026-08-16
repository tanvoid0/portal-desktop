//! Git working-tree and branch diff stats for the coder workspace bar.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::process_ext::NoWindowExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileChange {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffStats {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    pub changed_files: u32,
    pub has_changes: bool,
}

impl GitDiffStats {
    fn empty() -> Self {
        Self {
            is_repo: false,
            branch: None,
            additions: 0,
            deletions: 0,
            changed_files: 0,
            has_changes: false,
        }
    }
}

pub fn git_diff_stats(workspace: &str) -> GitDiffStats {
    let root = Path::new(workspace);
    if workspace.is_empty() || !root.join(".git").exists() {
        return GitDiffStats::empty();
    }

    let branch = git_output(workspace, &["branch", "--show-current"])
        .ok()
        .filter(|s| !s.is_empty());

    let has_commits = git_output(workspace, &["rev-parse", "--verify", "HEAD"]).is_ok();

    let mut additions = 0u32;
    let mut deletions = 0u32;
    let mut changed_files = 0u32;

    if let Some(default) = default_branch(workspace) {
        if branch.as_deref() == Some(default.as_str()) {
            accumulate_numstat(
                workspace,
                &["diff", "HEAD", "--numstat"],
                &mut additions,
                &mut deletions,
                &mut changed_files,
            );
        } else if has_commits {
            let range = format!("{default}...HEAD");
            accumulate_numstat(
                workspace,
                &["diff", "--numstat", &range],
                &mut additions,
                &mut deletions,
                &mut changed_files,
            );
            accumulate_numstat(
                workspace,
                &["diff", "HEAD", "--numstat"],
                &mut additions,
                &mut deletions,
                &mut changed_files,
            );
        }
    } else if has_commits {
        accumulate_numstat(
            workspace,
            &["diff", "HEAD", "--numstat"],
            &mut additions,
            &mut deletions,
            &mut changed_files,
        );
    } else {
        accumulate_numstat(
            workspace,
            &["diff", "--numstat"],
            &mut additions,
            &mut deletions,
            &mut changed_files,
        );
    }

    accumulate_untracked(root, workspace, &mut additions, &mut changed_files);

    GitDiffStats {
        is_repo: true,
        branch,
        additions,
        deletions,
        changed_files,
        has_changes: additions > 0 || deletions > 0 || changed_files > 0,
    }
}

fn default_branch(workspace: &str) -> Option<String> {
    for name in ["main", "master"] {
        if git_output(workspace, &["rev-parse", "--verify", name]).is_ok() {
            return Some(name.to_string());
        }
    }

    git_output(workspace, &["symbolic-ref", "refs/remotes/origin/HEAD"])
        .ok()
        .and_then(|sym| sym.strip_prefix("refs/remotes/origin/").map(str::to_string))
}

fn git_output(workspace: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .no_window()
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn accumulate_numstat(
    workspace: &str,
    args: &[&str],
    additions: &mut u32,
    deletions: &mut u32,
    changed_files: &mut u32,
) {
    let Ok(output) = git_output(workspace, args) else {
        return;
    };

    for line in output.lines() {
        if let Some((add, del)) = parse_numstat_line(line) {
            *additions += add;
            *deletions += del;
            *changed_files += 1;
        }
    }
}

fn accumulate_untracked(
    root: &Path,
    workspace: &str,
    additions: &mut u32,
    changed_files: &mut u32,
) {
    let Ok(output) = git_output(workspace, &["ls-files", "--others", "--exclude-standard"]) else {
        return;
    };

    for rel in output.lines().filter(|l| !l.is_empty()) {
        let full = root.join(rel);
        if !full.is_file() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&full) {
            let lines = content.lines().count() as u32;
            if lines > 0 {
                *additions += lines;
                *changed_files += 1;
            }
        }
    }
}

pub fn git_list_changes(workspace: &str) -> Vec<GitFileChange> {
    let root = Path::new(workspace);
    if workspace.is_empty() || !root.join(".git").exists() {
        return Vec::new();
    }

    let has_commits = git_output(workspace, &["rev-parse", "--verify", "HEAD"]).is_ok();
    let name_status_args: &[&str] = if has_commits {
        &["diff", "HEAD", "--name-status"]
    } else {
        &["diff", "--name-status"]
    };

    let mut changes = Vec::new();

    if let Ok(output) = git_output(workspace, name_status_args) {
        for line in output.lines().filter(|l| !l.is_empty()) {
            let Some((status, path)) = parse_name_status(line) else {
                continue;
            };
            let diff_args: Vec<&str> = if has_commits {
                vec!["diff", "HEAD", "--no-color", "--", &path]
            } else {
                vec!["diff", "--no-color", "--", &path]
            };
            let diff = git_output(workspace, &diff_args).unwrap_or_default();
            let (additions, deletions) = count_diff_lines(&diff);
            changes.push(GitFileChange {
                path,
                status,
                additions,
                deletions,
                diff,
            });
        }
    }

    if let Ok(output) = git_output(workspace, &["ls-files", "--others", "--exclude-standard"]) {
        for rel in output.lines().filter(|l| !l.is_empty()) {
            let full = root.join(rel);
            if !full.is_file() {
                continue;
            }
            let content = std::fs::read_to_string(&full).unwrap_or_default();
            let lines: Vec<&str> = content.lines().collect();
            let additions = lines.len() as u32;
            let diff = if lines.is_empty() {
                String::new()
            } else {
                lines
                    .iter()
                    .map(|line| format!("+{line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            changes.push(GitFileChange {
                path: rel.to_string(),
                status: "untracked".into(),
                additions,
                deletions: 0,
                diff,
            });
        }
    }

    changes.sort_by(|a, b| a.path.cmp(&b.path));
    changes
}

fn parse_name_status(line: &str) -> Option<(String, String)> {
    let mut parts = line.split('\t');
    let code = parts.next()?;
    let status = match code.chars().next()? {
        'M' => "modified",
        'A' => "added",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        'T' => "type changed",
        _ => "changed",
    }
    .to_string();

    let first_path = parts.next()?.to_string();
    let path = parts.next().unwrap_or(&first_path).to_string();
    Some((status, path))
}

fn count_diff_lines(diff: &str) -> (u32, u32) {
    let mut additions = 0u32;
    let mut deletions = 0u32;
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix('+') {
            if !rest.starts_with("+++") {
                additions += 1;
            }
        } else if let Some(rest) = line.strip_prefix('-') {
            if !rest.starts_with("---") {
                deletions += 1;
            }
        }
    }
    (additions, deletions)
}

fn parse_numstat_line(line: &str) -> Option<(u32, u32)> {
    let mut parts = line.split('\t');
    let add = parts.next()?;
    let del = parts.next()?;
    let _path = parts.next()?;

    if add == "-" || del == "-" {
        return None;
    }

    Some((add.parse().ok()?, del.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numstat_adds_and_dels() {
        assert_eq!(parse_numstat_line("12\t3\tsrc/foo.rs"), Some((12, 3)));
        assert_eq!(parse_numstat_line("0\t5\tREADME.md"), Some((0, 5)));
    }

    #[test]
    fn parse_numstat_skips_binary() {
        assert_eq!(parse_numstat_line("-\t-\timage.png"), None);
    }
}
