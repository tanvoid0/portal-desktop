//! What "Sync all" leaves behind: the facts an AI reviews about unfinished work,
//! and the one place a plan the user approved is applied. The model never runs
//! anything — every path in a plan is checked against `git status`, and
//! secret-looking files are neither sent to the model nor committed. A project in
//! a subfolder of a larger repository only ever sees and commits its own folder.

use std::collections::{BTreeSet, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::LazyLock;

use chrono::Utc;
use regex::Regex;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, Unchanged};
use serde::{Deserialize, Serialize};

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::git::{self, GitProbe};
use crate::domains::projects::move_lock;
use crate::domains::projects::repositories::discovery_repository::apply_git_fields;
use crate::domains::projects::sync_all;
use crate::entities::project;
use crate::process_ext::NoWindowExt;

const MAX_FILES: usize = 150;
const MAX_DIFF: usize = 12_000;
/// Keeps the `git diff -- <paths>` command line under the Windows limit.
const MAX_ARG_BYTES: usize = 20_000;
const MAX_PREVIEWS: usize = 8;
const PREVIEW_BYTES: u64 = 600;
const SCAN_BYTES: u64 = 1_000_000;
const HIDDEN: &str = "[line hidden: looks like a secret]";

/// The first line of a PEM / PGP private key.
const PEM_BEGIN: &str = r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY(?: BLOCK)?-----";
static PEM_BEGIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PEM_BEGIN).expect("pem begin regex"));
static PEM_END_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----END [A-Z0-9 ]*PRIVATE KEY(?: BLOCK)?-----").expect("pem end regex")
});

/// Credentials with a shape of their own: finding one blocks a commit.
static KEY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"\b(sk-[A-Za-z0-9_-]{{20,}}|[sr]k_(?:live|test)_[A-Za-z0-9]{{16,}}|gh[pousr]_[A-Za-z0-9]{{30,}}|github_pat_[A-Za-z0-9_]{{30,}}|AKIA[0-9A-Z]{{16}}|xox[baprs]-[A-Za-z0-9-]{{10,}})|{PEM_BEGIN}|_authToken=[^\s$]+",
    ))
    .expect("key regex")
});

/// Looser: only used to blank lines before they reach a model.
static ASSIGN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(key|secret|passw(or)?d|token|authorization)[\w-]*["']?\s*[:=]\s*\S{6,}|://[^/\s:]+:[^@\s]+@"#,
    )
    .expect("assignment regex")
});

/// A `[...]` class in a `.gitignore` pattern.
static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[[!^]?\]?(\[:[a-z]+:\]|\\.|[^\]\\])*\]").expect("class regex"));

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
    /// A sync would push this repository by itself (set by `facts_for`).
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

/// The repository holding `dir`, and `dir`'s place in it: empty at the top, else
/// `sub/folder/`. Only changes under that place are reviewed and committed.
fn repo(dir: &Path) -> Result<(PathBuf, String), String> {
    if !dir.is_dir() {
        return Err("folder not on this machine".into());
    }
    move_lock::ensure_unlocked(dir)?;
    let top = git::toplevel(dir).ok_or("not a git repository")?;
    let scope = run(dir, &["rev-parse", "--show-prefix"], "")?;
    Ok((top, scope.trim_end_matches(['\r', '\n']).to_string()))
}

/// Changed files under `scope`; paths stay relative to the repository root.
fn changes(top: &Path, scope: &str) -> Result<Vec<Change>, String> {
    let mut args = vec!["status", "--porcelain=v1", "-z", "--untracked-files=all"];
    if !scope.is_empty() {
        args.extend(["--", scope]);
    }
    let out = run(top, &args, "")?;
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
        return !ends(&[
            ".example",
            ".sample",
            ".template",
            ".dist",
            ".tpl",
            ".defaults",
        ]);
    }
    ends(&[
        ".pem",
        ".key",
        ".p12",
        ".pfx",
        ".keystore",
        ".jks",
        ".ppk",
        ".kdbx",
    ]) || [
        "id_rsa",
        "id_ed25519",
        "id_ecdsa",
        ".netrc",
        ".pypirc",
        "credentials",
    ]
    .contains(&name.as_str())
        || ((name.contains("secret")
            || name.contains("credential")
            || name.starts_with("service-account"))
            && ends(&[
                ".json",
                ".yaml",
                ".yml",
                ".txt",
                ".toml",
                ".ini",
                ".properties",
            ]))
}

fn redact(text: &str) -> String {
    let last = |re: &Regex, l: &str| re.find_iter(l).last().map(|m| m.start());
    // A private key's body has no shape of its own: hidden from BEGIN through END.
    let mut in_key = false;
    text.lines()
        .map(|l| {
            let (begin, end) = (last(&PEM_BEGIN_RE, l), last(&PEM_END_RE, l));
            let hide = in_key || begin.is_some() || KEY_RE.is_match(l) || ASSIGN_RE.is_match(l);
            // Whichever marker comes last on the line says where the next one stands.
            if begin != end {
                in_key = begin > end;
            }
            if hide {
                HIDDEN
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

/// A file left out of what the model sees altogether: a diff hunk can show a
/// key's body without the BEGIN line `redact` looks for.
fn holds_private_key(path: &Path) -> bool {
    head_of(path, SCAN_BYTES).is_some_and(|t| PEM_BEGIN_RE.is_match(&t))
}

/// Whether committing `c` would add something shaped like a key. `Err` when that
/// cannot be told, which stops the commit.
fn adds_key(top: &Path, c: &Change) -> Result<bool, String> {
    // Untracked, or no commit to diff against yet: the whole file gets added.
    if c.xy == "??" || run(top, &["rev-parse", "--verify", "-q", "HEAD"], "").is_err() {
        return Ok(head_of(&top.join(&c.path), SCAN_BYTES).is_some_and(|t| KEY_RE.is_match(&t)));
    }
    // Only added lines: a key already in history is not this commit's doing.
    let diff = run(
        top,
        &[
            "diff",
            "HEAD",
            "-U0",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--",
            &c.path,
        ],
        "",
    )
    .map_err(|e| format!("cannot check {} for secrets: {e}", c.path))?;
    Ok(diff
        .lines()
        .any(|l| l.starts_with('+') && !l.starts_with("+++") && KEY_RE.is_match(l)))
}

fn fingerprint(top: &Path, scope: &str, all: &[Change]) -> String {
    let mut h = DefaultHasher::new();
    // For a subfolder, its own committed tree: a commit elsewhere in the
    // repository does not void the review.
    let at = match scope.trim_end_matches('/') {
        "" => "HEAD".to_string(),
        folder => format!("HEAD:{folder}"),
    };
    run(top, &["branch", "--show-current"], "")
        .ok()
        .hash(&mut h);
    run(top, &["rev-parse", &at], "").ok().hash(&mut h);
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
    let (top, scope) = repo(dir)?;
    let all = changes(&top, &scope)?;
    let hidden = |c: &Change| looks_secret(&c.path) || c.orig.as_deref().is_some_and(looks_secret);

    let mut bytes = 0;
    let tracked: Vec<&str> = all
        .iter()
        .filter(|c| c.xy != "??" && !hidden(c) && !holds_private_key(&top.join(&c.path)))
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
            "diff",
            "HEAD",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--",
        ];
        args.extend(&tracked);
        // Fails in a repository with no commit yet; the file list still serves.
        diff = run(&top, &args, "").unwrap_or_default();
    }
    for c in all
        .iter()
        .filter(|c| c.xy == "??" && !hidden(c) && !holds_private_key(&top.join(&c.path)))
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
        fingerprint: fingerprint(&top, &scope, &all),
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

/// `?*` and `[!x]*` hide nearly everything: a pattern has to spell out part of a name.
fn names_something(pattern: &str) -> bool {
    CLASS_RE
        .replace_all(pattern, "")
        .chars()
        .any(char::is_alphanumeric)
}

fn ignore_lines(raw: &[String], scope: &str) -> Result<Vec<String>, String> {
    raw.iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            // The model sees paths from the repository root; a folder's
            // `.gitignore` reads them from the folder.
            let line = match p.trim_start_matches('/').strip_prefix(scope) {
                Some(rest) if !scope.is_empty() => format!("/{rest}"),
                _ => p.to_string(),
            };
            if p.contains(['\n', '\r']) || p.starts_with(['!', '#']) || !names_something(&line) {
                Err(format!("not a usable ignore pattern: {p}"))
            } else {
                Ok(line)
            }
        })
        .collect()
}

/// Appends the lines not already there. Returns whether anything was written.
fn append_ignore(file: &Path, lines: &[String]) -> Result<bool, String> {
    let mut text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Cannot read .gitignore: {e}")),
    };
    let have: HashSet<String> = text.lines().map(|l| l.trim().to_string()).collect();
    let fresh: Vec<&String> = lines.iter().filter(|l| !have.contains(*l)).collect();
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
                if looks_secret(&h.path) || adds_key(top, h)? {
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
        &[
            "commit",
            "-q",
            "-m",
            &c.message,
            FROM_STDIN[0],
            FROM_STDIN[1],
        ],
        &nul_list(&c.paths),
    )
    .map(|_| ())
}

/// Runs an approved plan: ignore lines, commits, rebase, push. Refuses outright
/// (`Err`) before touching anything; a failure part way is in `error`.
pub fn apply(dir: &Path, plan: &SyncResolution) -> Result<(SyncResolveResult, GitProbe), String> {
    let (top, scope) = repo(dir)?;
    if let Some(why) = git::busy(&top) {
        return Err(format!("{why} — finish it first"));
    }
    // Repository-wide: git commits nothing while any path is unmerged.
    if !run(&top, &["ls-files", "--unmerged"], "")?.is_empty() {
        return Err("unresolved merge conflicts — finish them first".into());
    }
    let all = changes(&top, &scope)?;
    match &plan.fingerprint {
        Some(seen) if *seen != fingerprint(&top, &scope, &all) => {
            return Err("Files changed since this review — review again".into());
        }
        // Rebase and push move commits that exist; anything new needs a review.
        None if !plan.commits.is_empty() || !plan.ignore.is_empty() => {
            return Err("These changes have not been reviewed — review first".into());
        }
        _ => {}
    }
    let ignore = ignore_lines(&plan.ignore, &scope)?;
    let mut commits = plan_commits(&top, &all, plan)?;

    let mut done = Vec::new();
    let ignore_file = format!("{scope}.gitignore");
    if !ignore.is_empty() && append_ignore(&top.join(&ignore_file), &ignore)? {
        done.push(format!("added to .gitignore: {}", ignore.join(", ")));
        let planned = commits.iter().any(|c| c.paths.contains(&ignore_file));
        if !planned && all.iter().all(|c| c.path != ignore_file) {
            let file = BTreeSet::from([ignore_file]);
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
    let path = project_row(db, id).await?.path;
    // The rows a sync groups: who shares the repository decides the push.
    let rows = project::Entity::find()
        .filter(project::Column::Archived.eq(false))
        .all(db.get_connection())
        .await
        .map_err(|e| format!("Failed to load projects: {e}"))?;
    tokio::task::spawn_blocking(move || {
        let mut found = facts(Path::new(&path))?;
        let paths: Vec<String> = rows.iter().map(|r| r.path.clone()).collect();
        let allowed: Vec<bool> = rows.iter().map(|r| r.push_ok).collect();
        found.push_ok = rows
            .iter()
            .position(|r| r.id == id)
            .is_some_and(|i| sync_all::push_allowed(&paths, &allowed, i));
        Ok(found)
    })
    .await
    .map_err(|e| e.to_string())?
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

    /// A repository with no commit yet.
    fn unborn() -> tempfile::TempDir {
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
        t
    }

    /// One commit holding `a.txt`, `gone.txt` and `old name.txt`.
    fn fixture() -> tempfile::TempDir {
        let t = unborn();
        let d = t.path();
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

    /// The plan as the UI sends it: carrying the fingerprint of the review.
    fn reviewed(dir: &Path, mut plan: SyncResolution) -> SyncResolution {
        plan.fingerprint = Some(facts(dir).unwrap().fingerprint);
        plan
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

        let plan = reviewed(
            d,
            commit_plan(&["a.txt", "gone.txt", "new name.txt", "src/"]),
        );
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
            [
                "a.txt",
                "gone.txt",
                "new name.txt",
                "old name.txt",
                "src/new.rs"
            ]
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
        write(
            d,
            "config.ts",
            &format!("const k = 'ghp_{}'\n", "a".repeat(36)),
        );
        write(d, "a.txt", "changed\n");
        let head = sh(d, &["rev-parse", "HEAD"]);

        for bad in ["app/.env", "app/", "config.ts", "nope.txt"] {
            let plan = reviewed(d, commit_plan(&["a.txt", bad]));
            assert!(apply(d, &plan).is_err(), "{bad}");
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
        let plan = reviewed(d, commit_plan(&["a.txt"]));
        write(d, "a.txt", "two, edited after the review\n");
        assert!(apply(d, &plan).is_err());

        // No fingerprint at all: nothing may be committed or ignored.
        assert!(apply(d, &commit_plan(&["a.txt"])).is_err());
        let ignore = SyncResolution {
            ignore: vec!["build/".into()],
            ..Default::default()
        };
        assert!(apply(d, &ignore).is_err());
        assert!(!d.join(".gitignore").exists());
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
        let (r, _) = apply(d, &reviewed(d, plan)).unwrap();
        assert_eq!(r.error, None);
        assert_eq!(sh(d, &["status", "--porcelain"]), "");
        for broad in [
            "*",
            "**/*",
            "!a.txt",
            ".*",
            "?*",
            "**/?*",
            "[!x]*",
            "[!]x]*",
            "[[:alpha:]]*",
        ] {
            let plan = SyncResolution {
                ignore: vec![broad.into()],
                ..Default::default()
            };
            let refused = apply(d, &reviewed(d, plan)).unwrap_err();
            assert!(refused.contains("ignore pattern"), "{broad}: {refused}");
        }
        for fine in ["*.log", "[Bb]in/", "__pycache__"] {
            assert!(names_something(fine), "{fine}");
        }
    }

    #[test]
    fn sync_resolve_private_key_body_never_reaches_the_model() {
        // Assembled, so this file does not itself look like it holds a key.
        let pem = |edge: &str| format!("-----{edge} {} KEY-----", "RSA PRIVATE");
        let (begin, end) = (pem("BEGIN"), pem("END"));

        let text = format!("a\n+{begin}\n+MIIBODY\n+{end}\nafter");
        assert_eq!(
            redact(&text),
            format!("a\n{HIDDEN}\n{HIDDEN}\n{HIDDEN}\nafter")
        );
        let cut_off = redact(&format!("{begin}\nMIIBODY\nMIIMORE"));
        assert!(!cut_off.contains("MII"), "{cut_off}");
        let one_line = redact(&format!("\"{begin}\\nMIIBODY\\n{end}\"\nafter"));
        assert_eq!(one_line, format!("{HIDDEN}\nafter"));

        let t = fixture();
        let d = t.path();
        let body = "MIIBODY1\nMIIBODY2\nMIIBODY3\nMIIBODY4\nMIIBODY5";
        write(d, "deploy.sh", &format!("{begin}\n{body}\n{end}\nrun\n"));
        sh(d, &["add", "."]);
        sh(d, &["commit", "-m", "deploy"]);
        // The hunk around this edit holds the end of the key but not its BEGIN.
        write(
            d,
            "deploy.sh",
            &format!("{begin}\n{body}\n{end}\nrun --fast\n"),
        );
        write(d, "notes.txt", &format!("{begin}\nMIINEW\n"));
        let f = facts(d).unwrap();
        assert!(!f.diff.contains("MII"), "{}", f.diff);
        assert!(!f.diff.contains("notes.txt"), "{}", f.diff);
    }

    #[test]
    fn sync_resolve_stripe_keys_and_key_assignments() {
        for prefix in ["sk_live_", "rk_live_", "sk_test_", "rk_test_"] {
            let key = format!("{prefix}{}", "a1".repeat(12));
            assert!(KEY_RE.is_match(&key), "{prefix}");
        }
        assert!(!KEY_RE.is_match("task_test_helpers_for_the_runner"));
        for line in [
            "STRIPE_KEY=abc123abc123",
            "const stripeKey = 'abc123abc123';",
        ] {
            assert_eq!(redact(line), HIDDEN, "{line}");
        }
    }

    #[test]
    fn sync_resolve_subfolder_project_stays_in_its_folder() {
        let t = fixture();
        let d = t.path();
        let app = d.join("app");
        write(d, "app/main.rs", "fn main() {}\n");
        write(d, "app/dist/out.js", "x");
        write(d, "a.txt", "changed elsewhere\n");
        write(d, "other/new.txt", "elsewhere\n");

        let f = facts(&app).unwrap();
        let listed: Vec<&str> = f.files.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(listed, ["app/dist/out.js", "app/main.rs"]);
        assert_eq!(f.files_total, 2);
        assert!(!f.diff.contains("elsewhere"), "{}", f.diff);
        let outside = reviewed(&app, commit_plan(&["a.txt"]));
        assert!(apply(&app, &outside).is_err());

        // A sibling's commit does not make this review stale.
        sh(d, &["commit", "-m", "sibling", "a.txt"]);
        write(d, "a.txt", "still uncommitted\n");
        let mut plan = commit_plan(&["app/main.rs"]);
        plan.ignore = vec!["app/dist/".into()];
        plan.fingerprint = Some(f.fingerprint);
        let (r, _) = apply(&app, &plan).unwrap();
        assert_eq!(r.error, None);

        assert_eq!(
            std::fs::read_to_string(app.join(".gitignore")).unwrap(),
            "/dist/\n"
        );
        assert!(!d.join(".gitignore").exists());
        let shipped = sh(d, &["log", "-2", "--name-only", "--format="]);
        let mut shipped: Vec<&str> = shipped.lines().filter(|l| !l.is_empty()).collect();
        shipped.sort();
        assert_eq!(shipped, ["app/.gitignore", "app/main.rs"]);
        let left = sh(d, &["status", "--porcelain", "--untracked-files=all"]);
        assert!(left.contains("M a.txt"), "{left}");
        assert!(left.contains("?? other/new.txt"), "{left}");
        assert!(!left.contains("app/"), "{left}");
    }

    #[test]
    fn sync_resolve_first_commit_is_scanned_for_keys() {
        let t = unborn();
        let d = t.path();
        write(
            d,
            "config.ts",
            &format!("const k = 'ghp_{}'\n", "a".repeat(36)),
        );
        sh(d, &["add", "."]);
        let refused = apply(d, &reviewed(d, commit_plan(&["config.ts"])));
        assert!(refused.unwrap_err().contains("secret"));
    }

    #[test]
    fn sync_resolve_refuses_during_a_cherry_pick() {
        let t = fixture();
        let d = t.path();
        sh(d, &["checkout", "-qb", "other"]);
        write(d, "a.txt", "theirs\n");
        sh(d, &["commit", "-am", "theirs"]);
        sh(d, &["checkout", "-q", "main"]);
        write(d, "a.txt", "mine\n");
        sh(d, &["commit", "-am", "mine"]);
        // Conflicts (so `sh` would panic); resolved but not yet continued.
        Command::new("git")
            .arg("-C")
            .arg(d)
            .args(["cherry-pick", "other"])
            .output()
            .unwrap();
        write(d, "a.txt", "resolved\n");
        sh(d, &["add", "a.txt"]);

        let plan = reviewed(d, commit_plan(&["a.txt"]));
        assert!(apply(d, &plan).unwrap_err().contains("cherry-pick"));
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
