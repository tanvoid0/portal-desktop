//! Git probing for discovery.
//!
//! Shells out rather than linking libgit2, matching `ProjectService::get_git_info`.
//! Every call is scoped with `-C <dir>` so no working directory is mutated, and
//! `NoWindowExt` keeps console windows from flashing on Windows.

use std::path::Path;
use std::process::Command;

use crate::process_ext::NoWindowExt;

/// Raw git state for one repository.
#[derive(Debug, Clone, Default)]
pub struct GitProbe {
    pub present: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub last_commit: Option<String>,
    pub commits: i64,
    /// Commits on HEAD but not upstream. `-1` means no upstream is configured,
    /// which is a different situation from "nothing to push".
    pub ahead: i64,
    pub behind: i64,
    pub stash: i64,
    /// Tracked files with modifications. `-1` when not measured.
    pub dirty: i64,
    pub untracked: i64,
}

impl GitProbe {
    pub fn absent() -> Self {
        Self {
            present: false,
            ahead: -1,
            behind: -1,
            dirty: -1,
            untracked: -1,
            ..Default::default()
        }
    }

    /// Last commit time in epoch ms, or 0 when there is no commit.
    pub fn last_commit_ms(&self) -> i64 {
        self.last_commit
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0)
    }

    /// True when the repo holds history that exists nowhere else.
    pub fn local_only(&self) -> bool {
        self.present && self.commits > 0 && self.remote.as_deref().unwrap_or("").is_empty()
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .no_window()
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn git_count(dir: &Path, args: &[&str]) -> Option<i64> {
    git(dir, args).and_then(|s| s.trim().parse::<i64>().ok())
}

/// Collects the state of one repository. Cheap ref reads first; the working-tree
/// scan (`status`) is last because it is the only expensive call.
pub fn probe(dir: &Path) -> GitProbe {
    let mut p = GitProbe {
        present: true,
        ahead: -1,
        behind: -1,
        dirty: -1,
        untracked: -1,
        ..Default::default()
    };

    p.branch = git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]);
    p.remote = git(dir, &["remote", "get-url", "origin"]);
    p.last_commit = git(dir, &["log", "-1", "--format=%cI"]);
    p.commits = git_count(dir, &["rev-list", "--count", "HEAD"]).unwrap_or(0);

    // Absent upstream makes these fail; -1 records "unknown", not "clean".
    p.ahead = git_count(dir, &["rev-list", "--count", "@{u}..HEAD"]).unwrap_or(-1);
    p.behind = git_count(dir, &["rev-list", "--count", "HEAD..@{u}"]).unwrap_or(-1);

    p.stash = git(dir, &["stash", "list"])
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count() as i64)
        .unwrap_or(0);

    if p.commits > 0 {
        // `git` returns success with empty stdout for a clean tree, which `git()`
        // maps to None — so a None here means clean, not failed.
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["status", "--porcelain"])
            .no_window()
            .output()
            .ok();
        if let Some(out) = status {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
                p.dirty = lines.iter().filter(|l| !l.starts_with("??")).count() as i64;
                p.untracked = lines.iter().filter(|l| l.starts_with("??")).count() as i64;
            }
        }
    }

    p
}
