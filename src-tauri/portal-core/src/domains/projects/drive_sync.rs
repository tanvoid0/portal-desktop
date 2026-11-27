//! External-drive sync: mirrors a project onto a removable drive, reports which
//! side is behind, and brings drive changes back. Pure functions over paths.
//!
//! Drive layout:
//!
//!   <mount>/.portal-sync/portal-sync.json     manifest
//!   <mount>/.portal-sync/state/<hash>.json    per-project base file lists, one per PC
//!   <mount>/<source path minus drive/root>/   the replica
//!
//! The drive is removable, so everything read from it is untrusted. Every repo
//! inside a project (root, nested, submodule) is tracked on its own: a commit
//! that exists only on the drive is never mirrored over.
//!
//! ponytail: drive-only tags are not carried to the PC, and the next copy to the
//! drive removes them. Carry `refs/tags` next to branches if that matters.
//! ponytail: a rebased / amended / reset commit on the PC still counts as
//! "only on the drive" (tips are compared, not history of syncs).
//! ponytail: a run with failures deletes nothing, so a project holding a file
//! that can never be copied keeps deleted files and refs on the drive. Delete
//! stale git refs anyway if that shows up as "PC is behind".

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::domains::projects::discovery::git::{self, error_line};
use crate::domains::projects::discovery::scan::REGENERABLE;
use crate::domains::projects::move_lock::{self, norm};
use crate::process_ext::NoWindowExt;

const META: &str = ".portal-sync";
const MANIFEST: &str = "portal-sync.json";
const TMP: &str = ".portal-sync-tmp";
/// FAT stores mtimes in 2 s steps. Only for comparisons that involve the drive.
// ponytail: a same-size edit made on the drive within 2 s goes unseen; hash if it matters.
const MTIME_SLACK: i64 = 2_000_000_000;
const LIST_CAP: usize = 20;
/// Names too generic to skip unless git says they are ignored.
const IGNORED_ONLY: &[&str] = &[
    "Library",
    "Temp",
    "obj",
    "bin",
    ".intellijPlatform",
    ".cache",
    ".pnpm-store",
    "Pods",
    ".angular",
    ".expo",
    ".terraform",
    ".kotlin",
    ".cxx",
    "Intermediate",
    "vendor",
];

// ── Wire types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncDrive {
    pub mount_point: String,
    pub label: String,
    pub fs_kind: String,
    #[ts(type = "number")]
    pub total_bytes: u64,
    #[ts(type = "number")]
    pub available_bytes: u64,
    pub removable: bool,
    pub has_mirror: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum DriveSyncState {
    NotOnDrive,
    InSync,
    DriveBehind,
    LocalBehind,
    Diverged,
    /// Inside another registered project, whose replica holds it.
    Covered,
    Unsupported,
    Missing,
    /// The drive copy exists but cannot be compared.
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DriveBranch {
    pub name: String,
    #[ts(type = "number")]
    pub local_ahead: u64,
    #[ts(type = "number")]
    pub drive_ahead: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DriveProjectStatus {
    pub state: DriveSyncState,
    /// Replica path relative to the drive root.
    pub drive_folder: Option<String>,
    pub synced_at: Option<String>,
    #[ts(type = "number")]
    pub local_ahead: u64,
    #[ts(type = "number")]
    pub drive_ahead: u64,
    pub branches: Vec<DriveBranch>,
    #[ts(type = "number")]
    pub local_dirty: usize,
    #[ts(type = "number")]
    pub drive_dirty: usize,
    #[ts(type = "number")]
    pub local_only_files: usize,
    #[ts(type = "number")]
    pub drive_only_files: usize,
    pub covered_by: Option<String>,
    pub note: Option<String>,
}

impl DriveProjectStatus {
    fn of(state: DriveSyncState) -> Self {
        Self {
            state,
            drive_folder: None,
            synced_at: None,
            local_ahead: 0,
            drive_ahead: 0,
            branches: Vec::new(),
            local_dirty: 0,
            drive_dirty: 0,
            local_only_files: 0,
            drive_only_files: 0,
            covered_by: None,
            note: None,
        }
    }

    fn noted(state: DriveSyncState, note: impl Into<String>) -> Self {
        Self {
            note: Some(note.into()),
            ..Self::of(state)
        }
    }

    pub fn covered(parent: String) -> Self {
        Self {
            covered_by: Some(parent),
            ..Self::of(DriveSyncState::Covered)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum DriveSyncDirection {
    ToDrive,
    ToLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DriveSyncResult {
    pub status: DriveProjectStatus,
    pub notes: Vec<String>,
    /// Left untouched; needs your hand.
    pub conflicts: Vec<String>,
    /// Files that could not be read or written.
    pub failures: Vec<String>,
    #[ts(type = "number")]
    pub files_copied: u64,
    #[ts(type = "number")]
    pub bytes_copied: u64,
}

fn cap(mut v: Vec<String>) -> Vec<String> {
    if v.len() > LIST_CAP {
        let more = v.len() - LIST_CAP;
        v.truncate(LIST_CAP);
        v.push(format!("…and {more} more"));
    }
    v
}

// ── Paths ────────────────────────────────────────────────────────────────────

/// `root.join("")` would append a separator.
fn join(root: &Path, rel: &str) -> PathBuf {
    if rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel)
    }
}

fn slashes(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// `.git`, in any case, or its NTFS short name.
fn is_git_name(c: &str) -> bool {
    c.eq_ignore_ascii_case(".git") || c.to_ascii_lowercase().starts_with("git~")
}

fn has_git(rel: &str) -> bool {
    rel.split('/').any(is_git_name)
}

/// A relative path that cannot leave the directory it is joined to.
fn valid_rel(rel: &str) -> bool {
    !rel.is_empty()
        && rel.split('/').all(|c| {
            let mut parts = Path::new(c).components();
            matches!(parts.next(), Some(Component::Normal(_)))
                && parts.next().is_none()
                // Windows drops trailing dots and spaces, and `:` names a drive or stream.
                && (!cfg!(windows) || (!c.contains([':', '\\']) && !c.ends_with(['.', ' '])))
        })
}

/// Manifest key: also keeps clear of our own folder.
fn valid_key(key: &str) -> bool {
    valid_rel(key)
        && !has_git(key)
        && !key
            .split('/')
            .next()
            .is_some_and(|c| c.eq_ignore_ascii_case(META))
}

/// The project path minus drive / root: `D:\a\b` -> `a/b`.
fn rel_key(local: &Path) -> Result<String, String> {
    let refuse = || format!("Cannot mirror {}", local.display());
    if !local.is_absolute() {
        return Err(refuse());
    }
    let mut parts = Vec::new();
    for c in local.components() {
        match c {
            Component::Prefix(_) | Component::RootDir => {}
            Component::Normal(n) => parts.push(n.to_str().ok_or_else(refuse)?),
            _ => return Err(refuse()),
        }
    }
    let key = parts.join("/");
    if valid_key(&key) {
        Ok(key)
    } else {
        Err(refuse())
    }
}

/// True when any existing component of `rel` under `root` is a link or junction.
fn has_link(root: &Path, rel: &str) -> bool {
    let mut p = root.to_path_buf();
    for c in rel.split('/').filter(|c| !c.is_empty()) {
        p.push(c);
        match fs::symlink_metadata(&p) {
            Ok(m) if m.file_type().is_symlink() => return true,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    false
}

/// `canonicalize` for a path whose tail may not exist yet.
fn canon(p: &Path) -> PathBuf {
    let mut base = p;
    let mut rest = Vec::new();
    loop {
        if let Ok(mut c) = fs::canonicalize(base) {
            c.extend(rest.iter().rev());
            return c;
        }
        match (base.parent(), base.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name);
                base = parent;
            }
            _ => return p.to_path_buf(),
        }
    }
}

/// As written and as resolved: a volume that cannot `canonicalize`, or the
/// verbatim prefix on one side only, must not hide that two paths overlap.
fn overlap(a: &Path, b: &Path, test: fn(&str, &str) -> bool) -> bool {
    // `\\?\`, as `canonicalize` returns it on Windows.
    let verbatim = String::from_iter(['\\', '\\', '?', '\\']);
    let forms = |p: &Path| {
        [p.to_path_buf(), canon(p)].map(|p| {
            let s = p.to_string_lossy().into_owned();
            s.strip_prefix(&verbatim).map(str::to_string).unwrap_or(s)
        })
    };
    let b = forms(b);
    forms(a).iter().any(|a| b.iter().any(|b| test(a, b)))
}

fn on_volume(local: &Path, mount: &Path) -> bool {
    overlap(local, mount, move_lock::is_within)
}

/// Nothing here but other projects' copies (`foreign`, see [`foreign`]).
fn vacant(root: &Path, rel: &str, foreign: &[String]) -> bool {
    let entries = match fs::read_dir(join(root, rel)) {
        Ok(entries) => entries,
        Err(e) => return e.kind() == io::ErrorKind::NotFound,
    };
    for entry in entries {
        let Ok(entry) = entry else { return false };
        let name = entry.file_name().to_string_lossy().into_owned();
        let child = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        let low = child.to_lowercase();
        let theirs = foreign.contains(&low)
            || (entry.file_type().is_ok_and(|t| t.is_dir())
                && foreign.iter().any(|f| move_lock::is_within(f, &low))
                && vacant(root, &child, foreign));
        if !theirs {
            return false;
        }
    }
    true
}

// ── Repositories ─────────────────────────────────────────────────────────────

/// Target of a `.git` file.
fn gitfile(dot_git: &Path) -> Option<String> {
    if !dot_git.is_file() {
        return None;
    }
    let text = fs::read_to_string(dot_git).ok()?;
    Some(text.strip_prefix("gitdir:")?.trim().to_string())
}

/// A `.git` file with an absolute target is a linked worktree (a relative one is a submodule).
fn linked(dir: &Path) -> bool {
    gitfile(&dir.join(".git")).is_some_and(|g| g.starts_with('/') || Path::new(&g).is_absolute())
}

fn gitdir(dir: &Path) -> Option<PathBuf> {
    let dot = dir.join(".git");
    if dot.is_dir() {
        return Some(dot);
    }
    // Resolve `..` lexically: git wants a clean path for safe.directory.
    let mut out = PathBuf::new();
    for c in dir.join(gitfile(&dot)?).components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Some(out)
}

/// Structural test, no git run: a failing git must never read as "no repo here".
fn is_repo(dir: &Path) -> bool {
    !linked(dir) && gitdir(dir).is_some_and(|g| g.join("HEAD").is_file())
}

/// Every git call here. `peer` is the other side's copy, for fetches.
/// `safe.directory` is needed on exFAT/FAT drives, which have no owners.
fn git_cmd(repo: &Path, peer: Option<&Path>) -> Command {
    let mut c = Command::new("git");
    c.arg("-C").arg(repo);
    for dir in [Some(repo), peer].into_iter().flatten() {
        let mut safe = vec![dir.to_path_buf(), dir.join(".git")];
        safe.extend(gitdir(dir));
        for s in safe {
            c.arg("-c").arg(format!("safe.directory={}", slashes(&s)));
        }
    }
    let hooks = if cfg!(windows) {
        "core.hooksPath=NUL"
    } else {
        "core.hooksPath=/dev/null"
    };
    for kv in [
        "core.fsmonitor=false",
        hooks,
        "core.quotepath=false",
        "protocol.version=2",
        "gc.auto=0",
        "maintenance.auto=false",
    ] {
        c.arg("-c").arg(kv);
    }
    c.env("GIT_TERMINAL_PROMPT", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .stdin(Stdio::null())
        .no_window();
    c
}

fn run(mut c: Command, input: Option<String>) -> Result<String, String> {
    let fail = |e: io::Error| format!("Failed to run git: {e}");
    let out = match input {
        None => c.output().map_err(fail)?,
        Some(input) => {
            c.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = c.spawn().map_err(fail)?;
            let mut stdin = child.stdin.take().expect("piped");
            // On a thread: git may fill its stdout before it has read everything.
            let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
            let out = child.wait_with_output().map_err(fail)?;
            let _ = writer.join();
            out
        }
    };
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(error_line(&String::from_utf8_lossy(&out.stderr)))
    }
}

fn git_at(repo: &Path, peer: Option<&Path>, args: &[&str]) -> Result<String, String> {
    let mut c = git_cmd(repo, peer);
    c.args(args);
    run(c, None)
}

fn git_in(repo: &Path, args: &[&str], input: String) -> Result<String, String> {
    let mut c = git_cmd(repo, None);
    c.args(args);
    run(c, Some(input))
}

/// Whether the ignore rules cover `dir`, tracked content or not.
fn ignored(repo: &Path, dir: &Path) -> Result<bool, String> {
    let rel = dir.strip_prefix(repo).map_err(|e| e.to_string())?;
    let mut c = git_cmd(repo, None);
    c.args(["check-ignore", "-q", "--no-index", "--"])
        .arg(slashes(rel));
    let out = c.output().map_err(|e| format!("Failed to run git: {e}"))?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        // Unknown is an error: guessing "not ignored" copies whole build folders.
        _ => Err(format!(
            "git check-ignore: {}",
            error_line(&String::from_utf8_lossy(&out.stderr))
        )),
    }
}

/// `discovery::git::busy` through [`git_cmd`]: a foreign-owner repo must not
/// read as free, so a git failure is an error here.
fn busy(repo: &Path) -> Result<Option<&'static str>, String> {
    let git = |args: &[&str]| git_at(repo, None, args);
    let dir = repo.join(git(&["rev-parse", "--git-dir"])?.trim());
    Ok(
        if dir.join("rebase-merge").exists() || dir.join("rebase-apply").exists() {
            Some("rebase in progress")
        } else if dir.join("MERGE_HEAD").exists() {
            Some("merge in progress")
        } else if git(&["branch", "--show-current"])?.trim().is_empty() {
            Some("detached HEAD")
        } else {
            None
        },
    )
}

/// What fsck holds against the repo; empty when it is complete.
fn fsck_problems(repo: &Path) -> Result<Vec<String>, String> {
    let mut c = git_cmd(repo, None);
    c.args([
        "fsck",
        "--connectivity-only",
        "--no-dangling",
        "--no-progress",
    ]);
    let out = c.output().map_err(|e| format!("Failed to run git: {e}"))?;
    if out.status.success() {
        return Ok(Vec::new());
    }
    // Missing objects are listed on stdout.
    let text = [out.stderr, out.stdout].concat();
    let mut lines: Vec<String> = String::from_utf8_lossy(&text)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("notice") && !l.starts_with("warning"))
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        lines.push(out.status.to_string());
    }
    Ok(lines)
}

/// Every object the copy's refs, reflogs and index name is really there.
fn fsck(copy: &Path, source: &Path) -> Result<(), String> {
    // Also refuses a broken `.git`, which would make git check an outer repo.
    tips(copy)?;
    let mut problems = fsck_problems(copy)?;
    // A dangling symref (`origin/HEAD` after a branch rename) loses nothing,
    // as long as the source has it too. Likewise the empty tree, which git
    // never has to store.
    if !problems.is_empty() {
        let theirs = fsck_problems(source)?;
        let nowhere = format!("invalid sha1 pointer {}", "0".repeat(40));
        let empty_tree = "missing tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904";
        problems.retain(|l| !((l.contains(&nowhere) || l == empty_tree) && theirs.contains(l)));
    }
    match problems.first() {
        None => Ok(()),
        Some(first) => Err(format!("git fsck failed: {first}")),
    }
}

#[derive(Debug, Default)]
struct Tips {
    heads: BTreeMap<String, String>,
    head: Option<String>,
    /// Checked-out branch; `None` when detached.
    branch: Option<String>,
    stash: Vec<String>,
}

impl Tips {
    fn all(&self) -> Vec<String> {
        let set: BTreeSet<&String> = self
            .heads
            .values()
            .chain(self.head.iter())
            .chain(self.stash.iter())
            .collect();
        set.into_iter().cloned().collect()
    }
}

fn tips(repo: &Path) -> Result<Tips, String> {
    let bad = |e: String| format!("Cannot read the repository at {}: {e}", repo.display());
    let git = |args: &[&str]| git_at(repo, None, args);
    // A broken nested `.git` makes git answer for the parent repo instead.
    if !git(&["rev-parse", "--show-prefix"])
        .map_err(bad)?
        .trim()
        .is_empty()
    {
        return Err(bad("not a repository".into()));
    }
    let heads = git(&[
        "for-each-ref",
        "--format=%(objectname) %(refname)",
        "refs/heads",
    ])
    .map_err(bad)?
    .lines()
    .filter_map(|l| {
        let (sha, name) = l.split_once(' ')?;
        Some((name.strip_prefix("refs/heads/")?.to_string(), sha.into()))
    })
    .collect();
    let one = |args: &[&str]| git(args).ok().map(|s| s.trim().to_string());
    let mut stash: Vec<String> = Vec::new();
    if let Some(top) = one(&["rev-parse", "-q", "--verify", "refs/stash"]) {
        // rev-list, not log: a drive's `log.showSignature` must not run its gpg.program.
        stash = git(&["rev-list", "--walk-reflogs", "refs/stash"])
            .map_err(bad)?
            .lines()
            .map(str::to_string)
            .collect();
        if !stash.contains(&top) {
            stash.insert(0, top);
        }
    }
    Ok(Tips {
        heads,
        head: one(&["rev-parse", "-q", "--verify", "HEAD^{commit}"]),
        branch: one(&["symbolic-ref", "-q", "HEAD"])
            .and_then(|r| r.strip_prefix("refs/heads/").map(str::to_string)),
        stash,
    })
}

/// Commits reachable from `include` and not from `exclude`.
fn count(repo: &Path, include: &[String], exclude: &[String]) -> Result<u64, String> {
    if include.is_empty() {
        return Ok(0);
    }
    let mut input = include.join("\n");
    for e in exclude {
        input.push_str("\n^");
        input.push_str(e);
    }
    git_in(repo, &["rev-list", "--count", "--stdin"], input)?
        .trim()
        .parse()
        .map_err(|e| format!("rev-list: {e}"))
}

/// Brings the drive's objects into the local repo; creates no refs.
fn fetch_missing(local: &Path, drive: &Path, shas: &[String]) -> Result<(), String> {
    if shas.is_empty() {
        return Ok(());
    }
    let missing: Vec<String> = git_in(local, &["cat-file", "--batch-check"], shas.join("\n"))?
        .lines()
        .filter_map(|l| l.strip_suffix(" missing").map(str::to_string))
        .collect();
    let from = slashes(drive);
    for chunk in missing.chunks(200) {
        let mut c = git_cmd(local, Some(drive));
        c.args([
            "fetch",
            "--quiet",
            "--no-write-fetch-head",
            "--no-tags",
            "--no-recurse-submodules",
        ])
        .arg(&from)
        .args(chunk);
        run(c, None)?;
    }
    Ok(())
}

/// path -> blob id of everything committed at `rev`.
fn tree(repo: &Path, rev: Option<&str>) -> HashMap<String, String> {
    let Some(rev) = rev else {
        return HashMap::new();
    };
    git_at(repo, None, &["ls-tree", "-r", "-z", "--full-tree", rev])
        .unwrap_or_default()
        .split('\0')
        .filter_map(|e| {
            let (meta, path) = e.split_once('\t')?;
            Some((path.to_string(), meta.split(' ').nth(2)?.to_string()))
        })
        .collect()
}

/// `git status` runs clean filters named in the repo's own config, so a drive
/// repo is only inspected when its config cannot define any.
fn status_safe(repo: &Path) -> bool {
    let Some(dir) = gitdir(repo) else {
        return false;
    };
    if dir.join("commondir").exists() {
        return false;
    }
    let config = dir.join("config");
    if !config.exists() {
        return true;
    }
    let mut c = git_cmd(repo, None);
    c.args(["config", "--file"])
        .arg(config)
        .args(["--name-only", "--list"]);
    run(c, None).is_ok_and(|keys| {
        !keys.lines().any(|k| {
            [
                "filter.",
                "include.",
                "includeif.",
                "extensions.worktreeconfig",
            ]
            .iter()
            .any(|p| k.starts_with(p))
        })
    })
}

/// Modified and untracked paths; `None` when it cannot be asked.
fn dirty(repo: &Path, on_drive: bool) -> Option<HashSet<String>> {
    if on_drive && !status_safe(repo) {
        return None;
    }
    let mut c = git_cmd(repo, None);
    if on_drive {
        // The copied index carries the PC's inode data; compare size and mtime only.
        c.args([
            "-c",
            "core.checkStat=minimal",
            "-c",
            "core.trustctime=false",
        ]);
    }
    c.args([
        "--no-optional-locks",
        "status",
        "--porcelain",
        "-z",
        "--untracked-files=all",
        "--no-renames",
        "--ignore-submodules=all",
    ]);
    let out = run(c, None).ok()?;
    Some(
        out.split('\0')
            .filter_map(|e| e.get(3..))
            // `dir/` is a nested repository, which is inspected on its own.
            .filter(|p| !p.ends_with('/'))
            .map(str::to_string)
            .collect(),
    )
}

// ── Walk ─────────────────────────────────────────────────────────────────────

/// rel (forward slashes) -> (length, mtime in nanoseconds).
type Files = BTreeMap<String, (u64, i64)>;

#[derive(Default)]
struct Walk {
    files: Files,
    dirs: BTreeSet<String>,
    /// Directories holding a repo; `""` is the root.
    repos: BTreeSet<String>,
    /// What could not be read. Nothing under these is compared or deleted.
    errors: BTreeMap<String, String>,
    /// Half-written copies an interrupted apply left behind.
    tmps: Vec<String>,
}

/// Whether the walk leaves `dir` out. `repo` is the nearest enclosing repository.
fn skip_dir(dir: &Path, name: &str, repo: Option<&Path>) -> Result<bool, String> {
    if linked(dir) || dir.join("CACHEDIR.TAG").is_file() {
        return Ok(true);
    }
    let strict = REGENERABLE.contains(&name);
    match repo {
        None => Ok(strict),
        Some(repo) if strict || IGNORED_ONLY.contains(&name) => ignored(repo, dir),
        Some(_) => Ok(false),
    }
}

fn stat(m: &fs::Metadata) -> (u64, i64) {
    let mtime = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as i64);
    (m.len(), mtime)
}

/// What git tracks inside a skipped folder still goes along; the rest does not.
fn walk_tracked(w: &mut Walk, repo: &Path, dir: &Path, rel: &str, with_git: bool) {
    let Ok(sub) = dir.strip_prefix(repo).map(slashes) else {
        return;
    };
    let args = ["--literal-pathspecs", "ls-files", "-z", "--", &sub];
    let listed = match git_at(repo, None, &args) {
        Ok(listed) => listed,
        Err(e) => {
            w.errors.insert(rel.to_string(), e);
            return;
        }
    };
    for path in listed.split('\0').filter(|p| !p.is_empty()) {
        // Empty when the folder itself is a submodule.
        let tail = path.get(sub.len()..).unwrap_or("").trim_start_matches('/');
        if !tail.is_empty() && (!valid_rel(tail) || has_git(tail) || has_link(dir, tail)) {
            continue;
        }
        let full = join(dir, tail);
        let child = if tail.is_empty() {
            rel.to_string()
        } else {
            format!("{rel}/{tail}")
        };
        let mut tmp = full.clone().into_os_string();
        tmp.push(TMP);
        if Path::new(&tmp).is_file() {
            w.tmps.push(format!("{child}{TMP}"));
        }
        let meta = match fs::symlink_metadata(&full) {
            Ok(meta) => meta,
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    w.errors.insert(child, e.to_string());
                }
                continue;
            }
        };
        let submodule = meta.is_dir() && is_repo(&full);
        if !meta.is_file() && !submodule {
            continue;
        }
        w.dirs.insert(rel.to_string());
        for (i, _) in child.match_indices('/').filter(|(i, _)| *i >= rel.len()) {
            w.dirs.insert(child[..i].to_string());
        }
        if submodule {
            w.dirs.insert(child.clone());
            w.repos.insert(child.clone());
            walk_dir(w, &full, &child, Some(&full), false, with_git);
        } else {
            w.files.insert(child, stat(&meta));
        }
    }
}

/// `with_git` also lists the contents of `.git` directories (for mirroring).
fn walk(root: &Path, with_git: bool) -> Result<Walk, String> {
    let mut w = Walk::default();
    if !root.exists() {
        return Ok(w);
    }
    let repo = is_repo(root).then(|| root.to_path_buf());
    if repo.is_some() {
        w.repos.insert(String::new());
    }
    walk_dir(&mut w, root, "", repo.as_deref(), false, with_git);
    match w.errors.get("") {
        Some(e) => Err(format!("Cannot read {}: {e}", root.display())),
        None => Ok(w),
    }
}

fn walk_dir(
    w: &mut Walk,
    dir: &Path,
    rel: &str,
    repo: Option<&Path>,
    in_git: bool,
    with_git: bool,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            w.errors.insert(rel.to_string(), e.to_string());
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                w.errors.insert(rel.to_string(), e.to_string());
                continue;
            }
        };
        let raw = entry.file_name();
        let name = raw.to_string_lossy();
        let child = if rel.is_empty() {
            name.to_string()
        } else {
            format!("{rel}/{name}")
        };
        if raw.to_str().is_none() {
            w.errors.insert(child, "unsupported file name".into());
            continue;
        }
        if name.ends_with(TMP) {
            if entry.file_type().is_ok_and(|t| t.is_file()) {
                w.tmps.push(child);
            }
            continue;
        }
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(e) => {
                w.errors.insert(child, e.to_string());
                continue;
            }
        };
        // Symlinks and junctions are never followed or copied.
        if kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        if kind.is_dir() {
            let dot_git = name.eq_ignore_ascii_case(".git");
            if dot_git && !with_git {
                continue;
            }
            if in_git {
                if name == "worktrees" && dir.file_name().is_some_and(|n| n == ".git") {
                    continue;
                }
            } else if !dot_git {
                match skip_dir(&path, &name, repo) {
                    Ok(false) => {}
                    Ok(true) => {
                        if let Some(repo) = repo {
                            walk_tracked(w, repo, &path, &child, with_git);
                        }
                        continue;
                    }
                    Err(e) => {
                        w.errors.insert(child, e);
                        continue;
                    }
                }
            }
            w.dirs.insert(child.clone());
            let nested = !in_git && !dot_git && is_repo(&path);
            if nested {
                w.repos.insert(child.clone());
            }
            let below = if nested { Some(path.as_path()) } else { repo };
            walk_dir(w, &path, &child, below, in_git || dot_git, with_git);
        } else if kind.is_file() {
            if in_git && name.ends_with(".lock") {
                continue;
            }
            // A submodule is a repo with a `.git` file; its directory was added above.
            match entry.metadata() {
                Ok(m) => {
                    w.files.insert(child, stat(&m));
                }
                Err(e) => {
                    w.errors.insert(child, e.to_string());
                }
            }
        }
    }
}

/// Name of the registered project whose replica already holds `local`.
/// A nested project the walk would skip is not covered.
pub fn covered_by(local: &Path, others: &[(String, PathBuf)]) -> Option<String> {
    let me = norm(&local.to_string_lossy());
    let mut parents: Vec<&(String, PathBuf)> = others
        .iter()
        .filter(|(_, p)| {
            let p_norm = norm(&p.to_string_lossy());
            p_norm != me
                && move_lock::is_within(&me, &p_norm)
                && p.is_dir()
                && !p.join(".git").is_file()
        })
        .collect();
    parents.sort_by_key(|(_, p)| p.components().count());
    parents
        .into_iter()
        .find(|(_, p)| reaches(p, local))
        .map(|(name, _)| name.clone())
}

fn reaches(parent: &Path, child: &Path) -> bool {
    let mut dir = parent.to_path_buf();
    let mut repo = is_repo(&dir).then(|| dir.clone());
    for c in child.components().skip(parent.components().count()) {
        let name = c.as_os_str().to_string_lossy();
        dir.push(c);
        if !fs::symlink_metadata(&dir).is_ok_and(|m| m.is_dir())
            || skip_dir(&dir, &name, repo.as_deref()).unwrap_or(true)
        {
            return false;
        }
        if is_repo(&dir) {
            repo = Some(dir.clone());
        }
    }
    true
}

/// Git object stores found by the walk (the repo's own and submodules').
fn object_stores(w: &Walk) -> Vec<String> {
    w.dirs
        .iter()
        .filter(|d| {
            has_git(d)
                && d.strip_suffix("/objects")
                    .is_some_and(|g| w.files.contains_key(&format!("{g}/HEAD")))
        })
        .cloned()
        .collect()
}

fn list_objects(root: &Path, stores: &[String]) -> Walk {
    let mut w = Walk::default();
    for rel in stores {
        w.dirs.insert(rel.clone());
        walk_dir(&mut w, &join(root, rel), rel, None, true, true);
    }
    // git's own temporary files come and go under our feet.
    w.files.retain(|rel, _| {
        let name = rel.rsplit('/').next().unwrap_or(rel);
        !name.starts_with("tmp_") && !name.starts_with(".tmp-")
    });
    w
}

/// A git file the repository cannot do without: losing one means "not synced".
fn essential(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    has_git(rel)
        && (is_git_name(name)
            || matches!(
                name,
                "HEAD" | "packed-refs" | "index" | "config" | "shallow"
            )
            || rel
                .split('/')
                .any(|c| matches!(c, "objects" | "refs" | "reftable")))
}

// ── Manifest and state ───────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    name: String,
    source_path: String,
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    synced_at: Option<String>,
    #[serde(default)]
    in_progress: bool,
    /// Which PC and folder last started a copy.
    #[serde(default)]
    writer: Option<String>,
    /// What the last finished copy had to leave out.
    #[serde(default)]
    failed: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    projects: BTreeMap<String, Entry>,
}

/// What one PC saw when it last agreed with the drive.
#[derive(Clone, Default, Serialize, Deserialize)]
struct Base {
    local: Files,
    drive: Files,
    #[serde(default)]
    repos: BTreeMap<String, Vec<String>>,
}

/// Keyed by [`owner`]: another PC's base says nothing about this one.
type State = BTreeMap<String, Base>;

fn owner(local: &Path) -> String {
    format!(
        "{}|{}",
        sysinfo::System::host_name().unwrap_or_default(),
        norm(&local.to_string_lossy())
    )
}

fn load_manifest(mount: &Path) -> Result<Manifest, String> {
    let path = mount.join(META).join(MANIFEST);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(Manifest {
                version: 1,
                projects: BTreeMap::new(),
            })
        }
        Err(e) => return Err(format!("Cannot read {}: {e}", path.display())),
    };
    let corrupt = |why: String| {
        format!(
            "The sync manifest on the drive is invalid ({}): {why}",
            path.display()
        )
    };
    let m: Manifest = serde_json::from_str(&text).map_err(|e| corrupt(e.to_string()))?;
    if m.version != 1 {
        return Err(corrupt(format!("unknown version {}", m.version)));
    }
    if let Some(key) = m.projects.keys().find(|k| !valid_key(k)) {
        return Err(corrupt(format!("unsafe project folder {key:?}")));
    }
    Ok(m)
}

/// Temp file + rename, so a pulled cable never leaves half a file.
fn write_meta(mount: &Path, rel: &str, value: &impl Serialize) -> Result<(), String> {
    if has_link(mount, &format!("{META}/{rel}")) {
        return Err("Refusing to write through a link on the drive".into());
    }
    let path = mount.join(META).join(rel);
    let put = || -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&serde_json::to_vec(value)?)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, &path)
    };
    put().map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

fn state_file(key: &str) -> String {
    let hash = hex::encode(Sha256::digest(key.to_lowercase().as_bytes()));
    format!("state/{}.json", &hash[..32])
}

/// Unreadable state is no state: everything then counts as changed, the cautious answer.
fn load_state(mount: &Path, key: &str) -> State {
    let mut state: State = fs::read(mount.join(META).join(state_file(key)))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    for base in state.values_mut() {
        // These paths get joined onto local folders; drop anything that could escape.
        base.local.retain(|r, _| valid_rel(r) && !has_git(r));
        base.drive.retain(|r, _| valid_rel(r) && !has_git(r));
    }
    state
}

struct Target {
    manifest: Manifest,
    key: String,
    entry: Option<Entry>,
}

/// Which replica belongs to this project.
fn locate(local: &Path, mount: &Path) -> Result<Target, String> {
    let manifest = load_manifest(mount)?;
    let mirrored = rel_key(local)?;
    let me = norm(&local.to_string_lossy());
    let remote = is_repo(local).then(|| git::remote_url(local)).flatten();
    let at_path = manifest
        .projects
        .iter()
        .find(|(k, _)| k.to_lowercase() == mirrored.to_lowercase())
        .map(|(k, e)| (k.clone(), e.clone()));
    if let Some((key, e)) = at_path {
        if norm(&e.source_path) != me {
            let other_repo = matches!((&e.remote, &remote), (Some(a), Some(b)) if a != b);
            if Path::new(&e.source_path).is_dir() || other_repo {
                return Err(format!(
                    "{key} on the drive is the copy of a different project ({})",
                    e.source_path
                ));
            }
        }
        return Ok(Target {
            manifest,
            key,
            entry: Some(e),
        });
    }
    // Moved here, or synced from a PC that keeps it elsewhere.
    let moved = {
        let mut hits = manifest.projects.iter().filter(|(_, e)| {
            remote.is_some() && e.remote == remote && !Path::new(&e.source_path).exists()
        });
        match (hits.next(), hits.next()) {
            (Some((k, e)), None) => Some((k.clone(), e.clone())),
            _ => None,
        }
    };
    Ok(match moved {
        Some((key, e)) => Target {
            manifest,
            key,
            entry: Some(e),
        },
        None => Target {
            manifest,
            key: mirrored,
            entry: None,
        },
    })
}

// ── Nested copies ────────────────────────────────────────────────────────────

/// Subfolders of this replica (lowercased) that are other projects' copies and
/// that this project's walk does not reach: not ours to compare or delete.
fn foreign(manifest: &Manifest, key: &str, local: &Path) -> Vec<String> {
    let depth = key.split('/').count();
    manifest
        .projects
        .keys()
        .filter(|k| move_lock::is_within(k, key))
        .filter_map(|k| {
            let sub = k.split('/').skip(depth).collect::<Vec<_>>().join("/");
            (!sub.is_empty() && !reaches(local, &join(local, &sub))).then(|| sub.to_lowercase())
        })
        .collect()
}

/// Whether Portal may have written into this folder for another project: it is
/// inside that project's copy, or holds one this project's walk reaches.
fn shared(manifest: &Manifest, key: &str, foreign: &[String]) -> bool {
    manifest.projects.keys().any(|k| {
        let low = k.to_lowercase();
        low != key.to_lowercase()
            && move_lock::is_under(k, key)
            && !foreign
                .iter()
                .any(|f| low == format!("{key}/{f}").to_lowercase())
    })
}

/// Drops other projects' copies from a replica listing, except what `ours` also
/// has (tracked files in a skipped folder).
fn prune(w: &mut Walk, foreign: &[String], ours: &Walk) {
    let theirs = |rel: &str| foreign.iter().any(|f| move_lock::is_within(rel, f));
    w.files
        .retain(|r, _| ours.files.contains_key(r) || !theirs(r));
    w.dirs.retain(|r| ours.dirs.contains(r) || !theirs(r));
    w.repos.retain(|r| ours.repos.contains(r) || !theirs(r));
    w.errors.retain(|r, _| !theirs(r));
}

// ── Compare ──────────────────────────────────────────────────────────────────

struct Repo {
    rel: String,
    local: Option<Tips>,
    drive: Option<Tips>,
    local_ahead: u64,
    drive_ahead: u64,
    local_dirty: Option<HashSet<String>>,
    drive_dirty: Option<HashSet<String>>,
}

struct Cmp {
    local: Walk,
    drive: Walk,
    repos: Vec<Repo>,
    /// Changed since this PC's base, whether or not the other side agrees.
    local_changed: BTreeSet<String>,
    drive_changed: BTreeSet<String>,
    local_only: BTreeSet<String>,
    drive_only: BTreeSet<String>,
}

fn differs(a: &(u64, i64), b: &(u64, i64), slack: i64) -> bool {
    a.0 != b.0 || (a.1 - b.1).abs() > slack
}

fn under_error(rel: &str, errors: &BTreeMap<String, String>) -> bool {
    errors.keys().any(|e| {
        rel.strip_prefix(e.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

/// Positively absent: any other error (unplugged, access denied) is not proof.
fn gone(p: &Path) -> bool {
    fs::symlink_metadata(p).is_err_and(|e| e.kind() == io::ErrorKind::NotFound)
}

/// Added, modified or deleted since `base`. No base: everything is changed.
fn changed(w: &Walk, base: Option<&Files>, root: &Path, slack: i64) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = w
        .files
        .iter()
        .filter(|(rel, st)| {
            !has_git(rel)
                && base
                    .and_then(|b| b.get(*rel))
                    .is_none_or(|was| differs(st, was, slack))
        })
        .map(|(rel, _)| rel.clone())
        .collect();
    for rel in base.into_iter().flat_map(|b| b.keys()) {
        // Gone from the walk but still on disk means newly skipped, not deleted.
        if !w.files.contains_key(rel) && gone(&join(root, rel)) {
            out.insert(rel.clone());
        }
    }
    out
}

fn same_bytes(a: &Path, b: &Path) -> io::Result<bool> {
    let (mut a, mut b) = (fs::File::open(a)?, fs::File::open(b)?);
    let (mut x, mut y) = ([0u8; 16 * 1024], [0u8; 16 * 1024]);
    loop {
        let n = a.read(&mut x)?;
        if n == 0 {
            return Ok(b.read(&mut y)? == 0);
        }
        if b.read_exact(&mut y[..n]).is_err() || x[..n] != y[..n] {
            return Ok(false);
        }
    }
}

/// Both missing, or both present with identical bytes.
fn same(local: &Path, drive: &Path, rel: &str, c: &Walk, d: &Walk) -> bool {
    match (c.files.get(rel), d.files.get(rel)) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.0 == b.0 && same_bytes(&join(local, rel), &join(drive, rel)).unwrap_or(false)
        }
        _ => false,
    }
}

/// The innermost repo holding `rel`, and `rel` relative to it.
fn repo_of<'a>(repos: &'a [Repo], rel: &'a str) -> Option<(&'a Repo, &'a str)> {
    repos
        .iter()
        .filter_map(|r| {
            if r.rel.is_empty() {
                Some((r, rel))
            } else {
                Some((r, rel.strip_prefix(r.rel.as_str())?.strip_prefix('/')?))
            }
        })
        .max_by_key(|(r, _)| r.rel.len())
}

/// Committed trees per repo: (local HEAD, drive HEAD), read in the local repo.
/// path -> blob id.
type Blobs = HashMap<String, String>;

#[derive(Default)]
struct Trees(HashMap<String, (Blobs, Blobs)>);

impl Trees {
    fn get(&mut self, local: &Path, r: &Repo) -> &(Blobs, Blobs) {
        self.0.entry(r.rel.clone()).or_insert_with(|| {
            let at = join(local, &r.rel);
            let head = |t: &Option<Tips>| t.as_ref().and_then(|t| t.head.clone());
            (
                tree(&at, head(&r.local).as_deref()),
                tree(&at, head(&r.drive).as_deref()),
            )
        })
    }
}

fn compare(
    local: &Path,
    replica: &Path,
    foreign: &[String],
    base: Option<&Base>,
) -> Result<Cmp, String> {
    // A drive pulled mid-run must never read as "everything was deleted".
    let here = || {
        replica
            .is_dir()
            .then_some(())
            .ok_or("The drive copy is not available")
    };
    here()?;
    let l = walk(local, false)?;
    let mut d = walk(replica, false)?;
    prune(&mut d, foreign, &l);
    let rels: BTreeSet<&String> = l.repos.iter().chain(&d.repos).collect();
    let mut repos = Vec::new();
    for rel in rels {
        let (lp, dp) = (join(local, rel), join(replica, rel));
        let lt = is_repo(&lp).then(|| tips(&lp)).transpose()?;
        let dt = is_repo(&dp).then(|| tips(&dp)).transpose()?;
        let (mut local_ahead, mut drive_ahead) = (0, 0);
        match (&lt, &dt) {
            (Some(l), Some(d)) => {
                fetch_missing(&lp, &dp, &d.all())?;
                local_ahead = count(&lp, &l.all(), &d.all())?;
                drive_ahead = count(&lp, &d.all(), &l.all())?;
            }
            (Some(l), None) => local_ahead = count(&lp, &l.all(), &[])?,
            (None, Some(d)) => {
                // Unchanged since we last agreed: it was removed here, not added there.
                let known = base.and_then(|b| b.repos.get(rel));
                let all = d.all();
                if !all.iter().all(|s| known.is_some_and(|k| k.contains(s))) {
                    drive_ahead = count(&dp, &all, &[])?;
                }
            }
            (None, None) => {}
        }
        repos.push(Repo {
            rel: rel.clone(),
            local_dirty: lt.is_some().then(|| dirty(&lp, false)).flatten(),
            drive_dirty: dt.is_some().then(|| dirty(&dp, true)).flatten(),
            local: lt,
            drive: dt,
            local_ahead,
            drive_ahead,
        });
    }

    let local_changed = changed(&l, base.map(|b| &b.local), local, 0);
    let drive_changed = changed(&d, base.map(|b| &b.drive), replica, MTIME_SLACK);
    here()?;
    let differing = |set: &BTreeSet<String>| -> BTreeSet<String> {
        set.iter()
            .filter(|rel| {
                !under_error(rel, &l.errors)
                    && !under_error(rel, &d.errors)
                    && !same(local, replica, rel, &l, &d)
            })
            .cloned()
            .collect()
    };
    let local_only = differing(&local_changed);
    let mut drive_only = differing(&drive_changed);
    // A clean file on a drive whose commits we all have, committed differently
    // here, is just an older version: nothing to pull.
    let mut trees = Trees::default();
    drive_only.retain(|rel| {
        let Some((r, sub)) = repo_of(&repos, rel) else {
            return true;
        };
        let clean = r.drive_dirty.as_ref().is_some_and(|s| !s.contains(sub));
        if r.local.is_none() || r.drive.is_none() || r.drive_ahead > 0 || !clean {
            return true;
        }
        let (ours, theirs) = trees.get(local, r);
        let committed = theirs.get(sub);
        !(committed.is_some() == d.files.contains_key(rel) && committed != ours.get(sub))
    });
    Ok(Cmp {
        local: l,
        drive: d,
        repos,
        local_changed,
        drive_changed,
        local_only,
        drive_only,
    })
}

fn report(local: &Path, cmp: &Cmp) -> Result<DriveProjectStatus, String> {
    let mut s = DriveProjectStatus::of(DriveSyncState::InSync);
    for r in &cmp.repos {
        s.local_ahead += r.local_ahead;
        s.drive_ahead += r.drive_ahead;
        s.local_dirty += r.local_dirty.as_ref().map_or(0, HashSet::len);
        s.drive_dirty += r.drive_dirty.as_ref().map_or(0, HashSet::len);
        let (Some(l), Some(d)) = (&r.local, &r.drive) else {
            continue;
        };
        let at = join(local, &r.rel);
        let names: BTreeSet<&String> = l.heads.keys().chain(d.heads.keys()).collect();
        for name in names {
            let (a, b) = (l.heads.get(name), d.heads.get(name));
            if a == b || s.branches.len() >= LIST_CAP {
                continue;
            }
            let ahead = |mine: Option<&String>, theirs: Option<&String>, all: &Tips| {
                let Some(mine) = mine else { return Ok(0) };
                let not = theirs.map_or_else(|| all.all(), |t| vec![t.clone()]);
                count(&at, std::slice::from_ref(mine), &not)
            };
            let (local_ahead, drive_ahead) = (ahead(a, b, d)?, ahead(b, a, l)?);
            if local_ahead + drive_ahead > 0 {
                let name = if r.rel.is_empty() {
                    name.clone()
                } else {
                    format!("{}: {name}", r.rel)
                };
                s.branches.push(DriveBranch {
                    name,
                    local_ahead,
                    drive_ahead,
                });
            }
        }
    }
    s.local_only_files = cmp.local_only.len();
    s.drive_only_files = cmp.drive_only.len();
    let unpulled = s.drive_only_files > 0 || s.drive_ahead > 0;
    let unpushed = s.local_only_files > 0 || s.local_ahead > 0;
    s.state = match (unpulled, unpushed) {
        (true, true) => DriveSyncState::Diverged,
        (true, false) => DriveSyncState::LocalBehind,
        (false, true) => DriveSyncState::DriveBehind,
        (false, false) => DriveSyncState::InSync,
    };
    Ok(s)
}

// ── Status ───────────────────────────────────────────────────────────────────

pub fn status(local: &Path, mount: &Path) -> Result<DriveProjectStatus, String> {
    use DriveSyncState as S;
    if !local.is_dir() {
        return Ok(DriveProjectStatus::of(S::Missing));
    }
    if local.join(".git").is_file() {
        let why = "Linked worktree or submodule (.git is a file)";
        return Ok(DriveProjectStatus::noted(S::Unsupported, why));
    }
    if !mount.is_dir() {
        return Err("The drive is not available".into());
    }
    if on_volume(local, mount) {
        let why = "The project is on the selected drive";
        return Ok(DriveProjectStatus::noted(S::Unsupported, why));
    }
    let t = locate(local, mount)?;
    let replica = join(mount, &t.key);
    let foreign = foreign(&t.manifest, &t.key, local);
    let fresh = vacant(&replica, "", &foreign);
    let Some(entry) = t.entry else {
        let mut s = DriveProjectStatus::of(S::NotOnDrive);
        if !fresh && !shared(&t.manifest, &t.key, &foreign) {
            s.note = Some(format!(
                "{} on the drive has files Portal did not put there",
                t.key
            ));
        }
        return Ok(s);
    };
    // A removed or emptied replica must not read as "everything was deleted".
    if fresh {
        let mut s = DriveProjectStatus::of(S::NotOnDrive);
        s.drive_folder = Some(t.key);
        return Ok(s);
    }
    if has_link(mount, &t.key) {
        return Err("The drive copy is behind a link".into());
    }
    let me = owner(local);
    let state = load_state(mount, &t.key);
    let compared =
        compare(local, &replica, &foreign, state.get(&me)).and_then(|c| report(local, &c));
    let unfinished = "The last copy to the drive did not finish";
    let mut s = match compared {
        Ok(mut s) if entry.in_progress => {
            // Our own half-finished copy is just stale, unless the drive has
            // something we lack.
            let mine = entry.writer.as_deref() == Some(me.as_str());
            s.state = if mine && s.drive_ahead == 0 && s.drive_only_files == 0 {
                S::DriveBehind
            } else {
                S::Diverged
            };
            s.note = Some(unfinished.into());
            s
        }
        Ok(mut s) => {
            if !entry.failed.is_empty() {
                if s.state == S::InSync {
                    s.state = S::DriveBehind;
                }
                let first: Vec<&str> = entry.failed.iter().take(3).map(String::as_str).collect();
                let more = if entry.failed.len() > 3 { "; …" } else { "" };
                s.note = Some(format!(
                    "Not everything could be copied: {}{more}",
                    first.join("; ")
                ));
            }
            s
        }
        Err(e) if entry.in_progress => {
            DriveProjectStatus::noted(S::Error, format!("{unfinished}: {e}"))
        }
        Err(e) => DriveProjectStatus::noted(S::Error, e),
    };
    s.drive_folder = Some(t.key);
    s.synced_at = entry.synced_at;
    Ok(s)
}

// ── Copy ─────────────────────────────────────────────────────────────────────

/// Clears the read-only bit (git objects have it); returns what to restore.
fn make_writable(p: &Path) -> Option<fs::Permissions> {
    let was = fs::metadata(p).ok()?.permissions();
    if !was.readonly() {
        return None;
    }
    let mut next = was.clone();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        next.set_mode(next.mode() | 0o200);
    }
    #[cfg(not(unix))]
    #[allow(clippy::permissions_set_readonly_false)]
    next.set_readonly(false);
    fs::set_permissions(p, next).ok()?;
    Some(was)
}

fn remove_file(p: &Path) -> io::Result<()> {
    make_writable(p);
    match fs::remove_file(p) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Removes what [`copy_file`] left behind when it was cut off.
fn sweep(root: &Path, w: &Walk) {
    for rel in &w.tmps {
        if !has_link(root, rel) {
            let _ = remove_file(&join(root, rel));
        }
    }
}

/// One apply per drive across processes (a dev and a release Portal). Held until
/// the returned file is dropped.
fn lock_drive(mount: &Path) -> Result<fs::File, String> {
    let rel = format!("{META}/apply.lock");
    if has_link(mount, &rel) {
        return Err("Refusing to write through a link on the drive".into());
    }
    let fail = |e: io::Error| format!("Cannot lock the drive: {e}");
    fs::create_dir_all(mount.join(META)).map_err(fail)?;
    let file = fs::File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(join(mount, &rel))
        .map_err(fail)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(fs::TryLockError::WouldBlock) => {
            Err("Another Portal is syncing with this drive. Try again when it is done.".into())
        }
        Err(fs::TryLockError::Error(e)) => Err(fail(e)),
    }
}

/// Copies next to `dst`, then renames over it: an interrupted copy never
/// truncates the destination. `recycle` keeps the replaced file recoverable.
/// `keep_mtime` carries the source's mtime over; otherwise the copy is "now".
fn copy_file(src: &Path, dst: &Path, recycle: bool, keep_mtime: bool) -> Result<u64, String> {
    let mut tmp = dst.as_os_str().to_owned();
    tmp.push(TMP);
    let tmp = PathBuf::from(tmp);
    let put = || -> Result<u64, String> {
        let io = |e: io::Error| e.to_string();
        remove_file(&tmp).map_err(io)?;
        // Read first: a source rewritten during the copy then still reads as newer.
        let source = fs::metadata(src).and_then(|m| m.modified()).map_err(io)?;
        let bytes = fs::copy(src, &tmp).map_err(io)?;
        // Set either way: `fs::copy` itself keeps the mtime on some systems.
        let mtime = if keep_mtime {
            source
        } else {
            std::time::SystemTime::now()
        };
        if fs::metadata(&tmp).and_then(|m| m.modified()).map_err(io)? != mtime {
            let was = make_writable(&tmp);
            fs::File::options()
                .write(true)
                .open(&tmp)
                .and_then(|f| f.set_modified(mtime))
                .map_err(io)?;
            if let Some(was) = was {
                fs::set_permissions(&tmp, was).map_err(io)?;
            }
        }
        if dst.is_file() {
            if recycle {
                trash::delete(dst).map_err(|e| e.to_string())?;
            } else {
                make_writable(dst);
            }
        }
        fs::rename(&tmp, dst).map_err(io)?;
        Ok(bytes)
    };
    let out = put();
    if out.is_err() {
        let _ = remove_file(&tmp);
    }
    out
}

/// Shared refusals before anything is written.
fn precheck(local: &Path, mount: &Path) -> Result<(), String> {
    if !local.is_dir() {
        return Err("The project folder is missing on this PC".into());
    }
    if local.join(".git").is_file() {
        return Err("Linked worktrees and submodules cannot be synced on their own".into());
    }
    if !mount.is_dir() {
        return Err("The drive is not available".into());
    }
    if on_volume(local, mount) {
        return Err("The project is on the selected drive".into());
    }
    Ok(())
}

fn check_replica(local: &Path, mount: &Path, key: &str) -> Result<PathBuf, String> {
    let replica = join(mount, key);
    if has_link(mount, key) {
        return Err("Refusing to write through a link on the drive".into());
    }
    // Replica == source would mirror a folder onto itself and delete from it.
    if overlap(local, &replica, move_lock::is_under) {
        return Err("The project and its drive copy contain each other".into());
    }
    Ok(replica)
}

fn final_status(local: &Path, mount: &Path) -> DriveProjectStatus {
    status(local, mount).unwrap_or_else(|e| DriveProjectStatus::noted(DriveSyncState::Error, e))
}

#[cfg(test)]
thread_local! {
    /// Lets a test change the source between the walk and the copy.
    static AFTER_WALK: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = Default::default();
}

/// Mirrors the project onto the drive. Refuses to run over changes this PC does
/// not have unless `force`. `free` is the space left on the drive.
pub fn to_drive(
    local: &Path,
    mount: &Path,
    name: &str,
    force: bool,
    free: u64,
) -> Result<DriveSyncResult, String> {
    precheck(local, mount)?;
    let _drive = lock_drive(mount)?;
    let mut t = locate(local, mount)?;
    let replica = check_replica(local, mount, &t.key)?;
    let foreign = foreign(&t.manifest, &t.key, local);
    let occupied = !vacant(&replica, "", &foreign);
    if t.entry.is_none() && occupied {
        if !shared(&t.manifest, &t.key, &foreign) {
            return Err(format!(
                "{} already exists on the drive and was not created by Portal. Move or rename it first.",
                replica.display()
            ));
        }
        // Part of another project's copy: take it over only if nothing is lost.
        if !force {
            let c = compare(local, &replica, &foreign, None)?;
            if !c.drive_only.is_empty() || c.repos.iter().any(|r| r.drive_ahead > 0) {
                return Err(format!(
                    "{} belongs to another project's drive copy and has changes this PC does not have. Move it away first.",
                    replica.display()
                ));
            }
        }
    } else if !force && occupied {
        let s = status(local, mount)?;
        match s.state {
            DriveSyncState::LocalBehind | DriveSyncState::Diverged => {
                return Err(
                    "The drive has changes this PC does not have. Update this PC first, or overwrite the drive."
                        .into(),
                );
            }
            DriveSyncState::Error => {
                return Err(format!(
                    "The drive copy cannot be checked ({}). Overwrite the drive to replace it.",
                    s.note.unwrap_or_default()
                ));
            }
            _ => {}
        }
    }

    let src = walk(local, true)?;
    #[cfg(test)]
    if let Some(change) = AFTER_WALK.take() {
        change();
    }
    let mut dst = walk(&replica, true)?;
    sweep(local, &src);
    sweep(&replica, &dst);
    prune(&mut dst, &foreign, &src);
    if src.files.is_empty() && !dst.files.is_empty() {
        return Err("The folder on this PC is empty; not clearing the drive copy".into());
    }
    let me = owner(local);
    let mut state = load_state(mount, &t.key);
    // The slack for the drive must not hide an edit made here since the last copy.
    let stale = |rel: &str, st: &(u64, i64)| {
        dst.files
            .get(rel)
            .is_none_or(|d| differs(st, d, MTIME_SLACK))
            || state
                .get(&me)
                .and_then(|b| b.local.get(rel))
                .is_some_and(|was| was != st)
    };
    let stores = object_stores(&src);
    let is_object = |rel: &str| {
        stores.iter().any(|s| {
            rel.strip_prefix(s.as_str())
                .is_some_and(|r| r.starts_with('/'))
        })
    };
    // Copied after the loose refs.
    let mut packed: Vec<String> = stores
        .iter()
        .map(|s| format!("{}packed-refs", s.trim_end_matches("objects")))
        .collect();
    // Everything else but objects. Git bookkeeping is always rewritten.
    let copies: Vec<&String> = src
        .files
        .iter()
        .filter(|(rel, st)| {
            let bookkeeping = has_git(rel) && !rel.split('/').any(|c| c == "objects");
            !is_object(rel) && !packed.contains(rel) && (bookkeeping || stale(rel, st))
        })
        .map(|(rel, _)| rel)
        .collect();
    // (bytes of new space needed)
    let need: u64 = src
        .files
        .iter()
        .filter(|(rel, st)| stale(rel, st))
        .map(|(_, st)| st.0)
        .sum();
    if need > free {
        let mb = |b: u64| b.div_ceil(1024 * 1024);
        return Err(format!(
            "Not enough space on the drive: {} MB needed, {} MB free",
            mb(need),
            mb(free)
        ));
    }

    let entry = Entry {
        name: name.to_string(),
        source_path: local.to_string_lossy().into_owned(),
        remote: is_repo(local).then(|| git::remote_url(local)).flatten(),
        synced_at: t.entry.as_ref().and_then(|e| e.synced_at.clone()),
        in_progress: true,
        writer: Some(me.clone()),
        failed: Vec::new(),
    };
    t.manifest.projects.insert(t.key.clone(), entry);
    write_meta(mount, MANIFEST, &t.manifest)?;
    fs::create_dir_all(&replica)
        .map_err(|e| format!("Cannot create {}: {e}", replica.display()))?;

    // rel -> why it did not make it.
    let mut failures = src.errors.clone();

    // Case renames and file <-> folder swaps have to be cleared before copying.
    let files: HashSet<String> = src.files.keys().map(|r| r.to_lowercase()).collect();
    let dirs: HashSet<String> = src.dirs.iter().map(|r| r.to_lowercase()).collect();
    let in_way = |rel: &str| {
        let low = rel.to_lowercase();
        files.contains(&low)
            || dirs.contains(&low)
            || low
                .match_indices('/')
                .any(|(i, _)| files.contains(&low[..i]))
    };
    let blocking: HashSet<&String> = dst
        .files
        .keys()
        .filter(|rel| !src.files.contains_key(*rel) && in_way(rel))
        .collect();
    if src.errors.is_empty() {
        for rel in &blocking {
            if let Err(e) = remove_file(&join(&replica, rel)) {
                failures.insert((*rel).clone(), e.to_string());
            }
        }
        for rel in dst.dirs.iter().rev() {
            if !src.dirs.contains(rel) && in_way(rel) {
                let _ = fs::remove_dir(join(&replica, rel));
            }
        }
    }

    let mkdir = |rel: &str| -> Result<(), String> {
        if has_link(&replica, rel) {
            return Err("a link on the drive is in the way".into());
        }
        fs::create_dir_all(join(&replica, rel)).map_err(|e| e.to_string())
    };
    let put = |rel: &str| -> Result<u64, String> {
        if has_link(&replica, rel) {
            return Err("a link on the drive is in the way".into());
        }
        copy_file(&join(local, rel), &join(&replica, rel), false, true)
    };
    for rel in &src.dirs {
        if let Err(e) = mkdir(rel) {
            failures.insert(rel.clone(), e);
        }
    }
    let (mut files_copied, mut bytes_copied) = (0, 0);
    // Git files that went away since the walk (a ref packed or deleted).
    let mut vanished: HashSet<&String> = HashSet::new();
    for rel in copies {
        match put(rel) {
            Ok(n) => {
                files_copied += 1;
                bytes_copied += n;
            }
            Err(_) if has_git(rel) && gone(&join(local, rel)) => {
                vanished.insert(rel);
            }
            Err(e) => {
                failures.insert(rel.clone(), e);
            }
        }
    }
    // Looked for on disk, not in the listing: a ref packed since the walk has no
    // loose file left, and `packed-refs` may not have existed then.
    packed.retain(|rel| join(local, rel).is_file());
    for rel in &packed {
        match put(rel) {
            Ok(n) => {
                files_copied += 1;
                bytes_copied += n;
            }
            Err(e) => {
                failures.insert(rel.clone(), e);
            }
        }
    }

    // Objects are listed only now, after the refs: whatever those name exists.
    let mut done = HashSet::new();
    let mut attempt = 0;
    let objects = loop {
        attempt += 1;
        let listed = list_objects(local, &stores);
        let mut missed = listed.errors.clone();
        for rel in &listed.dirs {
            if let Err(e) = mkdir(rel) {
                missed.insert(rel.clone(), e);
            }
        }
        for (rel, st) in &listed.files {
            if done.contains(rel) || !stale(rel, st) {
                continue;
            }
            match put(rel) {
                Ok(n) => {
                    done.insert(rel.clone());
                    files_copied += 1;
                    bytes_copied += n;
                }
                Err(e) => {
                    missed.insert(rel.clone(), e);
                }
            }
        }
        // A concurrent `git gc` moves loose objects into packs: list again.
        if attempt == 3 || !missed.keys().any(|rel| gone(&join(local, rel))) {
            failures.extend(missed);
            break listed;
        }
    };

    // Nothing extraneous is deleted unless every copy succeeded.
    if failures.is_empty() {
        for rel in dst.files.keys() {
            let keep = objects.files.contains_key(rel)
                || packed.contains(rel)
                || (src.files.contains_key(rel) && !is_object(rel) && !vanished.contains(rel));
            if !keep && !blocking.contains(rel) {
                if let Err(e) = remove_file(&join(&replica, rel)) {
                    failures.insert(rel.clone(), e.to_string());
                }
            }
        }
        for rel in dst.dirs.iter().rev() {
            // A case-only match may be the very folder just written (NTFS, exFAT).
            if !src.dirs.contains(rel)
                && !objects.dirs.contains(rel)
                && !dirs.contains(&rel.to_lowercase())
            {
                let _ = fs::remove_dir(join(&replica, rel));
            }
        }
    }

    // A commit made during the copy must not leave refs without their objects.
    let mut broken = false;
    for rel in &src.repos {
        let label = if rel.is_empty() { "." } else { rel };
        if !gitdir(&join(local, rel)).is_some_and(|g| g.starts_with(local)) {
            let why = "its git data lives outside the project and was not copied";
            failures.insert(label.to_string(), why.into());
        } else if let Err(e) = fsck(&join(&replica, rel), &join(local, rel)) {
            broken = true;
            failures.insert(label.to_string(), e);
        }
    }
    let verified = !broken && !failures.keys().any(|rel| essential(rel));

    // Record what did arrive, finished or not: our own files on the drive must
    // not later read as changes made there.
    let mut after = walk(&replica, false)?;
    prune(&mut after, &foreign, &src);
    failures.extend(after.errors.clone());
    let repos = after
        .repos
        .iter()
        .filter_map(|rel| Some((rel.clone(), tips(&join(&replica, rel)).ok()?.all())))
        .collect();
    let tracked = |files: &Files| -> Files {
        files
            .iter()
            .filter(|(rel, _)| !has_git(rel))
            .map(|(rel, st)| (rel.clone(), *st))
            .collect()
    };
    // The listing taken before copying: an edit made meanwhile still shows.
    let mut mirrored = tracked(&src.files);
    mirrored.retain(|rel, _| !under_error(rel, &failures));
    state.insert(
        me,
        Base {
            local: mirrored,
            drive: tracked(&after.files),
            repos,
        },
    );
    write_meta(mount, &state_file(&t.key), &state)?;

    let problems = failures.len();
    let failures = cap(failures
        .into_iter()
        .map(|(rel, e)| format!("{rel}: {e}"))
        .collect());
    let mut notes = Vec::new();
    if verified {
        if let Some(e) = t.manifest.projects.get_mut(&t.key) {
            e.in_progress = false;
            e.synced_at = Some(chrono::Utc::now().to_rfc3339());
            e.failed = failures.clone();
        }
        write_meta(mount, MANIFEST, &t.manifest)?;
        if problems > 0 {
            notes.push(format!(
                "{problems} items could not be copied. Extra files on the drive were left in place."
            ));
        }
    } else {
        notes.push(format!(
            "Copy incomplete ({problems} problems) and not marked as synced; run it again."
        ));
    }

    Ok(DriveSyncResult {
        status: final_status(local, mount),
        notes,
        conflicts: Vec::new(),
        failures,
        files_copied,
        bytes_copied,
    })
}

// ── Pull ─────────────────────────────────────────────────────────────────────

fn fast_forward(repo: &Path, target: &str) -> Result<(), String> {
    // Uncommitted files that already equal the incoming version (synced as work
    // in progress, then committed on the drive) would block git. Staging them
    // loses nothing. A path the user staged is theirs: left alone.
    let diff = [
        "diff",
        "--cached",
        "--name-only",
        "-z",
        "--no-renames",
        "HEAD",
    ];
    let staged = git_at(repo, None, &diff)?;
    let staged: HashSet<&str> = staged.split('\0').collect();
    let incoming = tree(repo, Some(target));
    let changed = git_at(
        repo,
        None,
        &["diff", "--name-only", "-z", "--no-renames", "HEAD", target],
    )?;
    let present: Vec<&str> = changed
        .split('\0')
        .filter(|f| incoming.contains_key(*f) && !staged.contains(f) && repo.join(f).is_file())
        .collect();
    let mut identical: Vec<&str> = Vec::new();
    if !present.is_empty() {
        let hashes = git_in(repo, &["hash-object", "--stdin-paths"], present.join("\n"))?;
        identical = present
            .iter()
            .zip(hashes.lines())
            .filter(|(f, hash)| incoming.get(**f).is_some_and(|b| b == hash))
            .map(|(f, _)| *f)
            .collect();
    }
    let each = |args: &[&str]| -> Result<(), String> {
        for chunk in identical.chunks(100) {
            let mut c = git_cmd(repo, None);
            c.arg("--literal-pathspecs").args(args).args(chunk);
            run(c, None)?;
        }
        Ok(())
    };
    let merged = each(&["add", "-f", "--"])
        .and_then(|()| git_at(repo, None, &["merge", "--ff-only", target]).map(drop));
    if merged.is_err() {
        // None of these were staged before, so HEAD is what the index held.
        let _ = each(&["reset", "-q", "HEAD", "--"]);
    }
    merged
}

#[derive(Default)]
struct Log {
    notes: Vec<String>,
    conflicts: Vec<String>,
    failures: Vec<String>,
}

/// Brings the drive repo's commits into the local one. Never rewrites or drops
/// local work. `label` prefixes messages for nested repos.
fn pull_repo(local: &Path, drive: &Path, label: &str, log: &mut Log) -> Result<(), String> {
    let (mine, theirs) = (tips(local)?, tips(drive)?);
    fetch_missing(local, drive, &theirs.all())?;
    let have = mine.all();
    let news = |sha: &String| count(local, std::slice::from_ref(sha), &have).map(|n| n > 0);
    let git = |args: &[&str]| git_at(local, None, args);
    let ancestor = |a: &str, b: &str| git(&["merge-base", "--is-ancestor", a, b]).is_ok();

    for (branch, sha) in &theirs.heads {
        if !news(sha)? {
            continue;
        }
        let current = mine.branch.as_ref() == Some(branch);
        match mine.heads.get(branch) {
            None => {
                git(&["branch", "--", branch, sha])?;
                // Checked out but unborn: give it an index, leaving files alone.
                if current {
                    git(&["reset", "-q"])?;
                }
                log.notes.push(format!("{label}added branch {branch}"));
            }
            Some(old) if ancestor(old, sha) => {
                let moved = if current {
                    fast_forward(local, sha)
                } else {
                    let spec = format!("refs/heads/{branch}:refs/heads/{branch}");
                    let from = slashes(drive);
                    git_at(
                        local,
                        Some(drive),
                        &[
                            "fetch",
                            "--no-write-fetch-head",
                            "--no-tags",
                            "--no-recurse-submodules",
                            &from,
                            &spec,
                        ],
                    )
                    .map(drop)
                };
                match moved {
                    Ok(()) => log.notes.push(format!("{label}updated {branch}")),
                    Err(e) => log
                        .conflicts
                        .push(format!("{label}{branch}: not updated ({e})")),
                }
            }
            Some(_) if current => {
                if !git(&["status", "--porcelain", "--untracked-files=no"])?
                    .trim()
                    .is_empty()
                {
                    log.conflicts.push(format!(
                        "{label}{branch}: diverged with uncommitted changes. Commit them, then retry."
                    ));
                } else if let Err(e) = git(&["merge", "--no-edit", sha]) {
                    let _ = git(&["merge", "--abort"]);
                    log.conflicts
                        .push(format!("{label}{branch}: conflicts, merge by hand ({e})"));
                } else {
                    log.notes.push(format!("{label}merged {branch}"));
                }
            }
            Some(_) => log
                .conflicts
                .push(format!("{label}{branch}: diverged, merge by hand")),
        }
    }
    // Detached (usual for submodules): only ever move forward.
    if mine.branch.is_none() {
        if let (Some(old), Some(new)) = (&mine.head, &theirs.head) {
            if old != new && news(new)? && ancestor(old, new) {
                if let Err(e) = fast_forward(local, new) {
                    log.conflicts.push(format!("{label}not updated ({e})"));
                }
            }
        }
    }
    for sha in theirs.stash.iter().rev() {
        if news(sha)? {
            git(&["stash", "store", "-m", "from external drive", sha])?;
            log.notes
                .push(format!("{label}added a stash from the drive"));
        }
    }
    if let (None, Some(head)) = (&theirs.branch, &theirs.head) {
        if count(local, std::slice::from_ref(head), &tips(local)?.all())? > 0 {
            let name = format!("drive-detached-{}", &head[..8]);
            git(&["branch", "--", &name, head])?;
            log.notes
                .push(format!("{label}saved drive commits as {name}"));
        }
    }
    Ok(())
}

/// A repo that exists only on the drive. A fresh `init` plus fetch, so the
/// drive's config and hooks never reach this PC.
fn adopt_repo(local: &Path, drive: &Path, label: &str, log: &mut Log) -> Result<(), String> {
    if !drive.join(".git").is_dir() || local.join(".git").exists() {
        log.conflicts.push(format!(
            "{label}this repository exists only on the drive. Set it up here by hand."
        ));
        return Ok(());
    }
    let theirs = tips(drive)?;
    fs::create_dir_all(local).map_err(|e| e.to_string())?;
    let git = |args: &[&str]| git_at(local, Some(drive), args);
    git(&["init", "-q"])?;
    if !theirs.heads.is_empty() {
        let from = slashes(drive);
        git(&[
            "fetch",
            "-q",
            "--update-head-ok",
            "--no-write-fetch-head",
            "--no-tags",
            &from,
            "+refs/heads/*:refs/heads/*",
        ])?;
    }
    fetch_missing(local, drive, &theirs.all())?;
    match (&theirs.branch, &theirs.head) {
        (Some(b), _) => drop(git(&["symbolic-ref", "HEAD", &format!("refs/heads/{b}")])?),
        (None, Some(head)) => drop(git(&["update-ref", "--no-deref", "HEAD", head])?),
        (None, None) => {}
    }
    if theirs.head.is_some() {
        git(&["reset", "-q"])?;
    }
    let url = git_at(drive, None, &["config", "--get", "remote.origin.url"]).unwrap_or_default();
    let url = url.trim();
    if !url.is_empty() && !url.starts_with('-') {
        git(&["remote", "add", "origin", url])?;
    }
    log.notes
        .push(format!("{label}repository created from the drive"));
    pull_repo(local, drive, label, log)
}

/// Brings drive changes to this PC. Local work is never destructively
/// overwritten: what cannot be applied safely is reported instead.
pub fn to_local(local: &Path, mount: &Path) -> Result<DriveSyncResult, String> {
    precheck(local, mount)?;
    let _drive = lock_drive(mount)?;
    let t = locate(local, mount)?;
    let replica = check_replica(local, mount, &t.key)?;
    let foreign = foreign(&t.manifest, &t.key, local);
    let entry = match t.entry {
        Some(entry) if !vacant(&replica, "", &foreign) => entry,
        _ => return Err("This project is not on the drive yet".into()),
    };
    if entry.in_progress {
        return Err("The last copy to the drive did not finish. Update the drive first.".into());
    }
    if is_repo(local) {
        if let Some(why) = busy(local)? {
            return Err(format!("This repository is not ready: {why}"));
        }
    }
    let me = owner(local);
    let mut state = load_state(mount, &t.key);
    let base = state.get(&me).cloned();
    let before = compare(local, &replica, &foreign, base.as_ref())?;
    sweep(local, &before.local);
    sweep(&replica, &before.drive);
    for r in &before.repos {
        if r.local.is_some() && !r.rel.is_empty() {
            if let Some(why @ ("rebase in progress" | "merge in progress")) =
                busy(&join(local, &r.rel))?
            {
                return Err(format!("{} is not ready: {why}", r.rel));
            }
        }
    }

    let mut log = Log::default();
    for r in before.repos.iter().filter(|r| r.drive_ahead > 0) {
        let (lp, dp) = (join(local, &r.rel), join(&replica, &r.rel));
        let label = if r.rel.is_empty() {
            String::new()
        } else {
            format!("{}: ", r.rel)
        };
        let done = if has_link(local, &r.rel) {
            Err("a link is in the way".to_string())
        } else if r.local.is_some() {
            pull_repo(&lp, &dp, &label, &mut log)
        } else {
            adopt_repo(&lp, &dp, &label, &mut log)
        };
        if let Err(e) = done {
            log.failures.push(format!("{label}{e}"));
        }
    }

    // Files git did not already take care of.
    let after = compare(local, &replica, &foreign, base.as_ref())?;
    let touched: HashSet<String> = before
        .local_changed
        .iter()
        .map(|r| r.to_lowercase())
        .collect();
    let mut todo: Vec<&String> = before.drive_only.intersection(&after.drive_only).collect();
    // Deletions first, so a case-only rename does not delete what it just wrote.
    todo.sort_by_key(|rel| after.drive.files.contains_key(*rel));

    let mut trees = Trees::default();
    let mut tracked: HashMap<String, HashSet<String>> = HashMap::new();
    let (mut files_copied, mut bytes_copied, mut waiting) = (0, 0, 0);
    for rel in todo {
        if touched.contains(&rel.to_lowercase()) {
            log.conflicts
                .push(format!("{rel}: changed on this PC and on the drive"));
            continue;
        }
        // Whether git still has the version about to be replaced.
        let mut recoverable = false;
        if let Some((r, sub)) = repo_of(&after.repos, rel).filter(|(r, _)| r.local.is_some()) {
            let known = tracked.entry(r.rel.clone()).or_insert_with(|| {
                git_at(&join(local, &r.rel), None, &["ls-files", "-z"])
                    .unwrap_or_default()
                    .split('\0')
                    .map(str::to_string)
                    .collect()
            });
            let is_tracked = known.contains(sub);
            let (ours, theirs) = trees.get(local, r);
            let committed_there = theirs.contains_key(sub)
                && r.drive_dirty.as_ref().is_some_and(|d| !d.contains(sub));
            if r.drive_ahead > 0 && committed_there {
                waiting += 1;
                continue;
            }
            // Tracked files only follow when both sides committed the same version.
            if is_tracked && ours.get(sub) != theirs.get(sub) {
                log.conflicts.push(format!(
                    "{rel}: skipped, the two sides are on different commits"
                ));
                continue;
            }
            recoverable = is_tracked && r.local_dirty.as_ref().is_some_and(|d| !d.contains(sub));
        }
        if has_link(local, rel) {
            log.conflicts.push(format!("{rel}: a link is in the way"));
            continue;
        }
        let dest = join(local, rel);
        let exists = fs::symlink_metadata(&dest).is_ok();
        let listed = after.local.files.contains_key(rel);
        if !after.drive.files.contains_key(rel) {
            // Checked again now: the drive must still be there and the file not.
            if exists && listed && replica.is_dir() && gone(&join(&replica, rel)) {
                match trash::delete(&dest) {
                    Ok(()) => log.notes.push(format!("{rel}: moved to the Recycle Bin")),
                    Err(e) => log.failures.push(format!("{rel}: {e}")),
                }
            }
        } else if exists && !listed {
            // A skipped folder, a case clash, a directory: not ours to replace.
            log.conflicts
                .push(format!("{rel}: something else is already at that path"));
        } else {
            let copied = dest
                .parent()
                .map_or(Ok(()), fs::create_dir_all)
                .map_err(|e| e.to_string())
                .and_then(|()| copy_file(&join(&replica, rel), &dest, !recoverable, false));
            match copied {
                Ok(n) => {
                    files_copied += 1;
                    bytes_copied += n;
                }
                Err(e) => log.failures.push(format!("{rel}: {e}")),
            }
        }
    }
    if waiting > 0 {
        log.notes.push(format!(
            "{waiting} committed files arrive once that history is merged"
        ));
    }

    // Record what now matches, so a later one-sided edit is seen as one-sided.
    let end = compare(local, &replica, &foreign, base.as_ref())?;
    let mut next = base.unwrap_or_default();
    let candidates: BTreeSet<&String> = [&before, &end]
        .into_iter()
        .flat_map(|c| c.local_changed.iter().chain(&c.drive_changed))
        .collect();
    for rel in candidates {
        if under_error(rel, &end.local.errors)
            || under_error(rel, &end.drive.errors)
            || !same(local, &replica, rel, &end.local, &end.drive)
        {
            continue;
        }
        for (map, w) in [(&mut next.local, &end.local), (&mut next.drive, &end.drive)] {
            match w.files.get(rel) {
                Some(st) => map.insert(rel.clone(), *st),
                None => map.remove(rel),
            };
        }
    }
    for r in &end.repos {
        if let (0, Some(d)) = (r.drive_ahead, &r.drive) {
            next.repos.insert(r.rel.clone(), d.all());
        }
    }
    state.insert(me, next);
    write_meta(mount, &state_file(&t.key), &state)?;

    Ok(DriveSyncResult {
        status: final_status(local, mount),
        notes: cap(log.notes),
        conflicts: cap(log.conflicts),
        failures: cap(log.failures),
        files_copied,
        bytes_copied,
    })
}

#[cfg(test)]
mod tests;
