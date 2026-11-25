//! Git probing for discovery.
//!
//! Shells out rather than linking libgit2, matching `ProjectService::get_git_info`.
//! Every call is scoped with `-C <dir>` so no working directory is mutated, and
//! `NoWindowExt` keeps console windows from flashing on Windows.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::domains::projects::move_lock;
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

/// `origin` URL only -- skips the expensive parts of `probe`.
pub fn remote_url(dir: &Path) -> Option<String> {
    git(dir, &["remote", "get-url", "origin"])
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

/// Hard ceiling for one network git call; a stalled remote must not hang a sync.
const NET_TIMEOUT: Duration = Duration::from_secs(120);

/// Runs a networked git command: no credential prompts, killed on `cancel` or
/// after `NET_TIMEOUT`. Error = last non-empty stderr line.
fn git_net(dir: &Path, args: &[&str], cancel: &AtomicBool) -> Result<String, String> {
    move_lock::ensure_unlocked(dir)?;
    let mut child = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "core.fsmonitor=",
            "-c",
            "http.lowSpeedLimit=1000",
            "-c",
            "http.lowSpeedTime=30",
        ])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .no_window()
        .spawn()
        .map_err(|e| format!("Failed to run git: {e}"))?;

    // Drain on threads so a chatty command can't block on a full pipe.
    fn drain(mut r: impl Read + Send + 'static) -> std::thread::JoinHandle<String> {
        std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = r.read_to_end(&mut b);
            String::from_utf8_lossy(&b).into_owned()
        })
    }
    let out = drain(child.stdout.take().expect("piped"));
    let err = drain(child.stderr.take().expect("piped"));

    let deadline = Instant::now() + NET_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => {}
            Err(e) => return Err(e.to_string()),
        }
        let stop = if cancel.load(Ordering::Relaxed) {
            Some("cancelled")
        } else if Instant::now() >= deadline {
            Some("timed out")
        } else {
            None
        };
        if let Some(why) = stop {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("git {} {why}", args.first().unwrap_or(&"")));
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let (out, err) = (
        out.join().unwrap_or_default(),
        err.join().unwrap_or_default(),
    );
    if status.success() {
        Ok(out.trim().to_string())
    } else {
        Err(err
            .lines()
            .rev()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("git failed")
            .to_string())
    }
}

/// Fetches the upstream's remote, else `origin`.
pub fn git_fetch(dir: &Path, cancel: &AtomicBool) -> Result<(), String> {
    let remote = git(dir, &["branch", "--show-current"])
        .and_then(|b| git(dir, &["config", "--get", &format!("branch.{b}.remote")]))
        .unwrap_or_else(|| "origin".into());
    git_net(dir, &["fetch", "--no-recurse-submodules", &remote], cancel).map(|_| ())
}

/// `(ahead, behind)` of HEAD vs upstream; `None` = no upstream.
pub fn ahead_behind(dir: &Path) -> Option<(i64, i64)> {
    let s = git(dir, &["rev-list", "--left-right", "--count", "HEAD...@{u}"])?;
    let mut it = s.split_whitespace().map(|n| n.parse::<i64>());
    Some((it.next()?.ok()?, it.next()?.ok()?))
}

/// Fast-forwards to the already-fetched upstream; never creates a merge commit.
pub fn git_pull_ff_only(dir: &Path, cancel: &AtomicBool) -> Result<(), String> {
    git_net(dir, &["merge", "--ff-only", "@{u}"], cancel).map(|_| ())
}

/// Pushes the current branch (never forced). Returns the branch pushed.
pub fn git_push_net(dir: &Path, cancel: &AtomicBool) -> Result<String, String> {
    let branch = git(dir, &["branch", "--show-current"])
        .ok_or("Detached HEAD — check out a branch before pushing")?;
    let args = crate::domains::coder::git_push::push_args(&branch, ahead_behind(dir).is_some());
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    git_net(dir, &args, cancel).map(|_| branch)
}

pub fn has_remote(dir: &Path) -> bool {
    git(dir, &["remote"]).is_some()
}

pub fn toplevel(dir: &Path) -> Option<PathBuf> {
    git(dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

/// Why a pull/push must not touch this repo right now, if anything.
pub fn busy(dir: &Path) -> Option<&'static str> {
    let gd = dir.join(git(dir, &["rev-parse", "--git-dir"])?);
    if gd.join("rebase-merge").exists() || gd.join("rebase-apply").exists() {
        Some("rebase in progress")
    } else if gd.join("MERGE_HEAD").exists() {
        Some("merge in progress")
    } else if git(dir, &["branch", "--show-current"]).is_none() {
        Some("detached HEAD")
    } else {
        None
    }
}
