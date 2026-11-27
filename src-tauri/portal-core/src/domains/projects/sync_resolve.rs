//! What "Sync all" leaves behind: the facts an AI reviews about unfinished work,
//! and the one place a plan the user approved is applied. The model never runs
//! anything — every path in a plan is checked against `git status`, and
//! secret-looking files are neither sent to the model nor committed.

use std::collections::{BTreeSet, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::LazyLock;

use chrono::Utc;
use regex::Regex;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, Unchanged};
use serde::{Deserialize, Serialize};

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::git::{self, GitProbe};
use crate::domains::projects::move_lock;
use crate::domains::projects::repositories::discovery_repository::apply_git_fields;
use crate::entities::project;
use crate::process_ext::NoWindowExt;

const MAX_FILES: usize = 150;
const MAX_DIFF: usize = 12_000;
/// Keeps the `git diff -- <paths>` command line under the Windows limit.
const MAX_ARG_BYTES: usize = 20_000;
const MAX_PREVIEWS: usize = 8;
const PREVIEW_BYTES: u64 = 600;
const SCAN_BYTES: u64 = 1_000_000;

/// Credentials with a shape of their own: finding one blocks a commit.
static KEY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(sk-[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,})|-----BEGIN [A-Z ]*PRIVATE KEY-----|_authToken=[^\s$]+",
    )
    .expect("key regex")
});

/// Looser: only used to blank lines before they reach a model.
static ASSIGN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(api[_-]?key|secret|passw(or)?d|token|authorization)[\w-]*["']?\s*[:=]\s*\S{6,}|://[^/\s:]+:[^@\s]+@"#,
    )
    .expect("assignment regex")
});

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncFileChange {
    /// `/`-separated, relative to the repository root.
    pub path: String,
    /// `git status --porcelain` code, trimmed: `M`, `??`, `D`, `R`…
    pub status: String,
    pub secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncRepoFacts {
    pub branch: Option<String>,
    #[ts(type = "number")]
    pub ahead: i64,
    #[ts(type = "number")]
    pub behind: i64,
    /// At most `MAX_FILES`; `files_total` is the real count.
    pub files: Vec<SyncFileChange>,
    #[ts(type = "number")]
    pub files_total: usize,
    /// Truncated, secrets blanked.
    pub diff: String,
    pub recent_subjects: Vec<String>,
    /// Changes when the working tree does; `apply` checks it.
    pub fingerprint: String,
    /// On the push allowlist (set by `facts_for`).
    pub push_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncCommitPlan {
    pub message: String,
    /// Changed paths, or a `dir/` prefix covering the changes under it.
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncResolution {
    pub commits: Vec<SyncCommitPlan>,
    /// Lines to append to `.gitignore`.
    pub ignore: Vec<String>,
    /// Replay unpushed commits on top of the upstream.
    pub rebase: bool,
    pub push: bool,
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SyncResolveResult {
    /// What happened, in order.
    pub done: Vec<String>,
    /// The step that stopped the plan; everything in `done` still happened.
    pub error: Option<String>,
    #[ts(type = "number")]
    pub ahead: i64,
    #[ts(type = "number")]
    pub behind: i64,
    #[ts(type = "number")]
    pub dirty: i64,
    #[ts(type = "number")]
    pub untracked: i64,
}

struct Change {
    xy: String,
    path: String,
    /// Rename / copy source.
    orig: Option<String>,
}

impl Change {
    fn unmerged(&self) -> bool {
        self.xy.contains('U') || self.xy == "AA" || self.xy == "DD"
    }
}

fn git_cmd(top: &Path) -> Command {
    let mut c = Command::new("git");
    // `core.fsmonitor` in a repo's own config can run an arbitrary program.
    c.args(["-c", "core.fsmonitor=", "--literal-pathspecs", "-C"])
        .arg(top)
        .env("GIT_TERMINAL_PROMPT", "0")
        .no_window();
    c
}

/// Runs git, feeding `stdin`; raw stdout (porcelain `-z` must not be trimmed).
fn run(top: &Path, args: &[&str], stdin: &str) -> Result<String, String> {
    let mut child = git_cmd(top)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    // Dropping the handle closes it, so a command that reads stdin finishes.
    if let Some(mut pipe) = child.stdin.take() {
        let _ = pipe.write_all(stdin.as_bytes());
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("Failed to run git: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        return Ok(stdout);
    }
    // Hooks and "nothing to commit" report on stdout.
    let err = String::from_utf8_lossy(&out.stderr);
    let msg = if err.trim().is_empty() {
        stdout.trim()
    } else {
        err.trim()
    };
    let skip = msg.chars().count().saturating_sub(600);
    Err(msg.chars().skip(skip).collect())
}

fn repo(dir: &Path) -> Result<PathBuf, String> {
    if !dir.is_dir() {
        return Err("folder not on this machine".into());
    }
    move_lock::ensure_unlocked(dir)?;
    git::toplevel(dir).ok_or_else(|| "not a git repository".into())
}

fn changes(top: &Path) -> Result<Vec<Change>, String> {
    let out = run(
        top,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        "",
    )?;
    let mut records = out.split('\0').filter(|s| !s.is_empty());
    let mut all = Vec::new();
    while let Some(rec) = records.next() {
        let (Some(xy), Some(path)) = (rec.get(..2), rec.get(3..)) else {
            continue;
        };
        let orig = (xy.contains('R') || xy.contains('C'))
            .then(|| records.next().map(str::to_string))
            .flatten();
        all.push(Change {
            xy: xy.to_string(),
            path: path.to_string(),
            orig,
        });
    }
    Ok(all)
}

/// A file name that normally holds credentials.
pub fn looks_secret(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    let ends = |exts: &[&str]| exts.iter().any(|e| name.ends_with(e));
    if name == ".env" || name.starts_with(".env.") || name.ends_with(".env") {
        // `.env.example` and friends are meant to be committed.
        return !ends(&[".example", ".sample", ".template", ".dist", ".tpl", ".defaults"]);
    }
    ends(&[".pem", ".key", ".p12", ".pfx", ".keystore", ".jks", ".ppk", ".kdbx"])
        || ["id_rsa", "id_ed25519", "id_ecdsa", ".netrc", ".pypirc", "credentials"]
            .contains(&name.as_str())
        || ((name.contains("secret")
            || name.contains("credential")
            || name.starts_with("service-account"))
            && ends(&[
                ".json", ".yaml", ".yml", ".txt", ".toml", ".ini", ".properties",
            ]))
}

fn redact(text: &str) -> String {
    text.lines()
        .map(|l| {
            if KEY_RE.is_match(l) || ASSIGN_RE.is_match(l) {
                "[line hidden: looks like a secret]"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Text at the head of a file; `None` for binary or unreadable ones.
fn head_of(path: &Path, max: u64) -> Option<String> {
    let mut buf = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(max)
        .read_to_end(&mut buf)
        .ok()?;
    (!buf.contains(&0)).then(|| String::from_utf8_lossy(&buf).into_owned())
}

/// Whether committing `c` would add something shaped like a key.
fn adds_key(top: &Path, c: &Change) -> bool {
    if c.xy == "??" {
        return head_of(&top.join(&c.path), SCAN_BYTES).is_some_and(|t| KEY_RE.is_match(&t));
    }
    // Only added lines: a key already in history is not this commit's doing.
    run(
        top,
        &[
            "diff", "HEAD", "-U0", "--no-ext-diff", "--no-textconv", "--no-color", "--", &c.path,
        ],
        "",
    )
    .is_ok_and(|d| {
        d.lines()
            .any(|l| l.starts_with('+') && !l.starts_with("+++") && KEY_RE.is_match(l))
    })
}

fn fingerprint(top: &Path, all: &[Change]) -> String {
    let mut h = DefaultHasher::new();
    run(top, &["rev-parse", "HEAD"], "").ok().hash(&mut h);
    for c in all {
        c.xy.hash(&mut h);
        c.path.hash(&mut h);
        if let Ok(m) = std::fs::metadata(top.join(&c.path)) {
            m.len().hash(&mut h);
            m.modified().ok().hash(&mut h);
        }
    }
    format!("{:x}", h.finish())
}

pub fn facts(dir: &Path) -> Result<SyncRepoFacts, String> {
    let top = repo(dir)?;
    let all = changes(&top)?;
    let hidden = |c: &Change| looks_secret(&c.path) || c.orig.as_deref().is_some_and(looks_secret);

    let mut bytes = 0;
    let tracked: Vec<&str> = all
        .iter()
        .filter(|c| c.xy != "??" && !hidden(c))
        .map(|c| c.path.as_str())
        .take_while(|p| {
            bytes += p.len() + 3;
            bytes < MAX_ARG_BYTES
        })
        .collect();
    let mut diff = String::new();
    // With no paths `git diff HEAD --` would diff everything, secrets included.
    if !tracked.is_empty() {
        let mut args = vec![
            "diff", "HEAD", "--no-ext-diff", "--no-textconv", "--no-color", "--",
        ];
        args.extend(&tracked);
        // Fails in a repository with no commit yet; the file list still serves.
        diff = run(&top, &args, "").unwrap_or_default();
    }
    for c in all
        .iter()
        .filter(|c| c.xy == "??" && !hidden(c))
        .take(MAX_PREVIEWS)
    {
        if let Some(text) = head_of(&top.join(&c.path), PREVIEW_BYTES) {
            diff.push_str(&format!("\n=== new file: {} ===\n{text}\n", c.path));
        }
    }

    let (ahead, behind) = git::ahead_behind(&top).unwrap_or((-1, -1));
    Ok(SyncRepoFacts {
        branch: run(&top, &["branch", "--show-current"], "")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        ahead,
        behind,
        files: all
            .iter()
            .take(MAX_FILES)
            .map(|c| SyncFileChange {
                path: c.path.clone(),
                status: c.xy.trim().to_string(),
                secret: hidden(c),
            })
            .collect(),
        files_total: all.len(),
        diff: redact(&diff).chars().take(MAX_DIFF).collect(),
        recent_subjects: run(&top, &["log", "-8", "--format=%s"], "")
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect(),
        fingerprint: fingerprint(&top, &all),
        push_ok: false,
    })
}

struct Commit {
    message: String,
    /// Need `git add` (untracked or changed in the working tree).
    add: BTreeSet<String>,
    /// The commit's pathspec, rename sources included.
    paths: BTreeSet<String>,
}

fn ignore_lines(raw: &[String]) -> Result<Vec<&str>, String> {
    raw.iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let too_broad = p.trim_matches(['*', '/', '.']).is_empty();
            if p.contains(['\n', '\r']) || p.starts_with(['!', '#']) || too_broad {
                Err(format!("not a usable ignore pattern: {p}"))
            } else {
                Ok(p)
            }
        })
        .collect()
}

/// Appends the lines not already there. Returns whether anything was written.
fn append_ignore(file: &Path, lines: &[&str]) -> Result<bool, String> {
    let mut text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Cannot read .gitignore: {e}")),
    };
    let have: HashSet<String> = text.lines().map(|l| l.trim().to_string()).collect();
    let fresh: Vec<&str> = lines.iter().copied().filter(|l| !have.contains(*l)).collect();
    if fresh.is_empty() {
        return Ok(false);
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for l in fresh {
        text.push_str(l);
        text.push('\n');
    }
    std::fs::write(file, text).map_err(|e| format!("Cannot write .gitignore: {e}"))?;
    Ok(true)
}

/// Expands and checks every commit before anything is touched.
fn plan_commits(top: &Path, all: &[Change], plan: &SyncResolution) -> Result<Vec<Commit>, String> {
    let mut used = HashSet::new();
    let mut commits = Vec::new();
    for c in &plan.commits {
        let message = c.message.trim();
        if message.is_empty() {
            return Err("Every commit needs a message".into());
        }
        let mut commit = Commit {
            message: message.to_string(),
            add: BTreeSet::new(),
            paths: BTreeSet::new(),
        };
        for want in &c.files {
            let hits: Vec<&Change> = all
                .iter()
                .filter(|h| h.path == *want || (want.ends_with('/') && h.path.starts_with(want)))
                .collect();
            if hits.is_empty() {
                return Err(format!("not a changed file: {want}"));
            }
            for h in hits {
                if h.path.ends_with('/') {
                    return Err(format!("{} is a repository inside this one", h.path));
                }
                if looks_secret(&h.path) || adds_key(top, h) {
                    return Err(format!("refusing to commit a likely secret: {}", h.path));
                }
                if !used.insert(h.path.clone()) {
                    continue;
                }
                // A change that is only staged has nothing left to add, and
                // `git add` rejects a staged deletion's path.
                if !h.xy.ends_with(' ') {
                    commit.add.insert(h.path.clone());
                }
                commit.paths.insert(h.path.clone());
                commit.paths.extend(h.orig.clone());
            }
        }
        if !commit.paths.is_empty() {
            commits.push(commit);
        }
    }
    Ok(commits)
}

fn nul_list(paths: &BTreeSet<String>) -> String {
    paths.iter().map(|p| format!("{p}\0")).collect()
}

fn commit_one(top: &Path, c: &Commit) -> Result<(), String> {
    const FROM_STDIN: [&str; 2] = ["--pathspec-from-file=-", "--pathspec-file-nul"];
    if !c.add.is_empty() {
        run(
            top,
            &["add", "-A", FROM_STDIN[0], FROM_STDIN[1]],
            &nul_list(&c.add),
        )?;
    }
    run(
        top,
        &["commit", "-q", "-m", &c.message, FROM_STDIN[0], FROM_STDIN[1]],
        &nul_list(&c.paths),
    )
    .map(|_| ())
}

/// Runs an approved plan: ignore lines, commits, rebase, push. Refuses outright
/// (`Err`) before touching anything; a failure part way is in `error`.
pub fn apply(dir: &Path, plan: &SyncResolution) -> Result<(SyncResolveResult, GitProbe), String> {
    let top = repo(dir)?;
    if let Some(why) = git::busy(&top) {
        return Err(format!("{why} — finish it first"));
    }
    let all = changes(&top)?;
    if all.iter().any(Change::unmerged) {
        return Err("unresolved merge conflicts — finish them first".into());
    }
    if let Some(seen) = &plan.fingerprint {
        if *seen != fingerprint(&top, &all) {
            return Err("Files changed since this review — review again".into());
        }
    }
    let ignore = ignore_lines(&plan.ignore)?;
    let mut commits = plan_commits(&top, &all, plan)?;

    let mut done = Vec::new();
    if !ignore.is_empty() && append_ignore(&top.join(".gitignore"), &ignore)? {
        done.push(format!("added to .gitignore: {}", ignore.join(", ")));
        let planned = commits.iter().any(|c| c.paths.contains(".gitignore"));
        if !planned && all.iter().all(|c| c.path != ".gitignore") {
            let file = BTreeSet::from([".gitignore".to_string()]);
            commits.insert(
                0,
                Commit {
                    message: "chore: update .gitignore".into(),
                    add: file.clone(),
                    paths: file,
                },
            );
        } else if !planned {
            done.push(".gitignore left uncommitted: it already had your edits".into());
        }
    }

    let mut error = None;
    for c in &commits {
        match commit_one(&top, c) {
            Ok(()) => done.push(format!(
                "committed {} file(s): {}",
                c.paths.len(),
                c.message.lines().next().unwrap_or_default()
            )),
            Err(e) => {
                error = Some(format!("commit failed: {e}"));
                break;
            }
        }
    }
    if error.is_none() && plan.rebase {
        match run(&top, &["rebase", "@{u}"], "") {
            Ok(_) => done.push("rebased onto upstream".into()),
            Err(e) => {
                // Leaves the branch where it was; a no-op if it never started.
                let _ = run(&top, &["rebase", "--abort"], "");
                error = Some(format!("rebase undone: {e}"));
            }
        }
    }
    if error.is_none() && plan.push {
        match git::git_push_net(&top, &AtomicBool::new(false)) {
            Ok(branch) => done.push(format!("pushed {branch}")),
            Err(e) => error = Some(format!("push failed: {e}")),
        }
    }

    let p = git::probe(&top);
    let result = SyncResolveResult {
        done,
        error,
        ahead: p.ahead,
        behind: p.behind,
        dirty: p.dirty,
        untracked: p.untracked,
    };
    Ok((result, p))
}

async fn project_row(db: &DatabaseManager, id: i32) -> Result<project::Model, String> {
    project::Entity::find_by_id(id)
        .one(db.get_connection())
        .await
        .map_err(|e| format!("Failed to load project: {e}"))?
        .ok_or_else(|| "Project not found".into())
}

pub async fn facts_for(db: &DatabaseManager, id: i32) -> Result<SyncRepoFacts, String> {
    let row = project_row(db, id).await?;
    let path = row.path;
    let mut found = tokio::task::spawn_blocking(move || facts(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())??;
    found.push_ok = row.push_ok;
    Ok(found)
}

pub async fn apply_for(
    db: &DatabaseManager,
    id: i32,
    plan: SyncResolution,
) -> Result<SyncResolveResult, String> {
    let path = project_row(db, id).await?.path;
    let (result, probe) = tokio::task::spawn_blocking(move || apply(Path::new(&path), &plan))
        .await
        .map_err(|e| e.to_string())??;
    let mut am = project::ActiveModel {
        id: Unchanged(id),
        ..Default::default()
    };
    apply_git_fields(&mut am, &probe);
    am.updated_at = Set(Some(Utc::now().into()));
    // The git work is done either way; a stale list badge is not worth an error.
    let _ = am.update(db.get_connection()).await;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(dir: &Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn write(dir: &Path, file: &str, text: &str) {
        let p = dir.join(file);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// One commit holding `a.txt`, `gone.txt` and `old name.txt`.
    fn fixture() -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        let d = t.path();
        sh(d, &["init", "-b", "main"]);
        for kv in [
            ["user.name", "t"],
            ["user.email", "t@t"],
            ["commit.gpgsign", "false"],
            ["core.autocrlf", "false"],
        ] {
            sh(d, &["config", kv[0], kv[1]]);
        }
        for f in ["a.txt", "gone.txt", "old name.txt"] {
            write(d, f, &format!("{f}\nline\nline\nline\n"));
        }
        sh(d, &["add", "."]);
        sh(d, &["commit", "-m", "init"]);
        t
    }

    fn commit_plan(files: &[&str]) -> SyncResolution {
        SyncResolution {
            commits: vec![SyncCommitPlan {
                message: "work".into(),
                files: files.iter().map(|f| f.to_string()).collect(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn sync_resolve_commits_only_what_the_plan_names() {
        let t = fixture();
        let d = t.path();
        write(d, "a.txt", "changed\n");
        std::fs::remove_file(d.join("gone.txt")).unwrap();
        sh(d, &["mv", "old name.txt", "new name.txt"]);
        write(d, "src/new.rs", "fn main() {}\n");
        write(d, "keep.txt", "not in the plan\n");
        write(d, "staged.txt", "staged, not in the plan\n");
        sh(d, &["add", "staged.txt"]);

        let mut plan = commit_plan(&["a.txt", "gone.txt", "new name.txt", "src/"]);
        plan.fingerprint = Some(facts(d).unwrap().fingerprint);
        let (r, _) = apply(d, &plan).unwrap();
        assert_eq!(r.error, None);

        let mut shipped: Vec<String> = sh(
            d,
            &["show", "--no-renames", "--name-only", "--format=", "HEAD"],
        )
        .lines()
        .map(str::to_string)
        .collect();
        shipped.sort();
        assert_eq!(
            shipped,
            ["a.txt", "gone.txt", "new name.txt", "old name.txt", "src/new.rs"]
        );
        let left = sh(d, &["status", "--porcelain"]);
        assert!(left.contains("A  staged.txt"), "{left}");
        assert!(left.contains("?? keep.txt"), "{left}");
    }

    #[test]
    fn sync_resolve_refuses_secrets_and_unknown_files() {
        let t = fixture();
        let d = t.path();
        write(d, "app/.env", "API_KEY=hunter2hunter2\n");
        write(d, "config.ts", &format!("const k = 'ghp_{}'\n", "a".repeat(36)));
        write(d, "a.txt", "changed\n");
        let head = sh(d, &["rev-parse", "HEAD"]);

        for bad in ["app/.env", "app/", "config.ts", "nope.txt"] {
            assert!(apply(d, &commit_plan(&["a.txt", bad])).is_err(), "{bad}");
        }
        assert_eq!(sh(d, &["rev-parse", "HEAD"]), head);
        assert_eq!(sh(d, &["diff", "--cached", "--name-only"]), "");

        let f = facts(d).unwrap();
        assert!(!f.diff.contains("hunter2"), "{}", f.diff);
        assert!(!f.diff.contains("ghp_"), "{}", f.diff);
        assert!(f.files.iter().any(|c| c.path == "app/.env" && c.secret));
        assert!(!looks_secret(".env.example"));
    }

    #[test]
    fn sync_resolve_stale_review_is_refused() {
        let t = fixture();
        let d = t.path();
        write(d, "a.txt", "one\n");
        let mut plan = commit_plan(&["a.txt"]);
        plan.fingerprint = Some(facts(d).unwrap().fingerprint);
        write(d, "a.txt", "two, edited after the review\n");
        assert!(apply(d, &plan).is_err());
    }

    #[test]
    fn sync_resolve_ignore_is_committed_and_bounded() {
        let t = fixture();
        let d = t.path();
        write(d, "build/out.bin", "x");
        let plan = SyncResolution {
            ignore: vec!["build/".into()],
            ..Default::default()
        };
        let (r, _) = apply(d, &plan).unwrap();
        assert_eq!(r.error, None);
        assert_eq!(sh(d, &["status", "--porcelain"]), "");
        for broad in ["*", "**/*", "!a.txt"] {
            let plan = SyncResolution {
                ignore: vec![broad.into()],
                ..Default::default()
            };
            assert!(apply(d, &plan).is_err(), "{broad}");
        }
    }

    #[test]
    fn sync_resolve_conflicting_rebase_is_undone() {
        let t = fixture();
        let d = t.path();
        sh(d, &["branch", "up"]);
        write(d, "a.txt", "mine\n");
        sh(d, &["commit", "-am", "mine"]);
        sh(d, &["checkout", "-q", "up"]);
        write(d, "a.txt", "theirs\n");
        sh(d, &["commit", "-am", "theirs"]);
        sh(d, &["checkout", "-q", "main"]);
        sh(d, &["branch", "--set-upstream-to=up"]);
        let head = sh(d, &["rev-parse", "HEAD"]);

        let plan = SyncResolution {
            rebase: true,
            ..Default::default()
        };
        let (r, _) = apply(d, &plan).unwrap();
        assert!(r.error.is_some());
        assert_eq!(sh(d, &["rev-parse", "HEAD"]), head);
        assert_eq!(git::busy(d), None);
    }
}
