//! Filesystem walk: find project roots, date them, group them into projects.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use ts_rs::TS;

use super::git::{self, GitProbe};

/// Build manifests that mark a directory as a project root.
pub const MARKERS: &[&str] = &[
    "package.json",
    "pyproject.toml",
    "requirements.txt",
    "setup.py",
    "Cargo.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "pubspec.yaml",
    "composer.json",
    "Gemfile",
    "CMakeLists.txt",
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
];

/// Extensions that mark a root on their own (.NET has no fixed manifest name).
const MARKER_EXTS: &[&str] = &[".sln", ".csproj", ".fsproj", ".vbproj"];

/// Never descended into: generated output, caches, VCS internals, IDE state.
pub const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "venv",
    ".venv",
    "env",
    ".env",
    "target",
    "build",
    "dist",
    ".next",
    "out",
    "vendor",
    "__pycache__",
    ".gradle",
    ".idea",
    ".vscode",
    "Pods",
    ".dart_tool",
    "PubCache",
    ".pnpm-store",
    ".terraform",
    "obj",
    ".cache",
    "coverage",
    ".svelte-kit",
    ".nuxt",
    ".turbo",
    ".parcel-cache",
    "Debug",
    "Release",
    ".pytest_cache",
    ".mypy_cache",
    ".tox",
    "bower_components",
    ".angular",
    ".expo",
    "DerivedData",
];

/// Regenerable directories, reported so the disk domain can offer to clean them.
/// Deletion itself stays in the disk domain — this module never removes anything.
pub const REGENERABLE: &[&str] = &[
    "node_modules",
    ".venv",
    "venv",
    "target",
    "build",
    "dist",
    ".next",
    "out",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".gradle",
    ".dart_tool",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    "coverage",
    ".parcel-cache",
    "DerivedData",
];

/// Container directories whose children are separately meaningful projects.
/// Without this a monorepo collapses to a single row and its packages vanish.
const MONOREPO_DIRS: &[&str] = &[
    "apps", "packages", "services", "modules", "libs", "crates", "projects",
];

/// Manifest → display stack. Kept in this module rather than reusing the
/// `frameworks` table because discovery runs before any DB lookup.
fn stack_for(marker: &str) -> &'static str {
    match marker {
        "package.json" => "JS/TS",
        "pyproject.toml" | "requirements.txt" | "setup.py" => "Python",
        "Cargo.toml" => "Rust",
        "go.mod" => "Go",
        "pom.xml" | "build.gradle" => "Java",
        "build.gradle.kts" => "Kotlin",
        "pubspec.yaml" => "Flutter",
        "composer.json" => "PHP",
        "Gemfile" => "Ruby",
        "CMakeLists.txt" => "C/C++",
        "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => "Docker",
        "sln" => ".NET",
        _ => "Other",
    }
}

// ── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryRoot {
    /// Short label that prefixes every project id from this root (e.g. `production`).
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryConfig {
    pub roots: Vec<DiscoveryRoot>,
    /// Path components below the root that identify one project.
    /// `2` means `<root>/<category>/<project>`; everything deeper is a module.
    #[ts(type = "number")]
    pub project_depth: usize,
    #[ts(type = "number")]
    pub max_depth: usize,
    #[ts(type = "number")]
    pub active_days: i64,
    #[ts(type = "number")]
    pub recent_days: i64,
    #[ts(type = "number")]
    pub dormant_days: i64,
    /// Skip git probing — much faster, used for a quick structural refresh.
    pub skip_git: bool,
    /// Measure the regenerable directories. Off by default: sizing a few hundred
    /// `node_modules` trees costs minutes, and most passes do not need it.
    #[serde(default)]
    pub measure_sizes: bool,
    /// Archive registry rows the walk did not find. Persisted with the rest of
    /// the config so the preference survives a restart.
    #[serde(default)]
    pub mark_missing: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            project_depth: 2,
            max_depth: 7,
            active_days: 120,
            recent_days: 365,
            dormant_days: 730,
            skip_git: false,
            measure_sizes: false,
            mark_missing: false,
        }
    }
}

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveredModule {
    /// Logical id: `<root name>/<relative path>`, forward slashes on every OS.
    pub path: String,
    pub abs: String,
    pub git: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub last_commit: Option<String>,
    #[ts(type = "number")]
    pub commits: i64,
    /// Commits not on the upstream branch. `-1` when there is no upstream.
    #[ts(type = "number")]
    pub ahead: i64,
    #[ts(type = "number")]
    pub behind: i64,
    #[ts(type = "number")]
    pub stash: i64,
    /// Tracked files with changes. `-1` when not measured.
    #[ts(type = "number")]
    pub dirty: i64,
    #[ts(type = "number")]
    pub untracked: i64,
    /// Newest of (source file mtime, last commit) in epoch ms.
    #[ts(type = "number")]
    pub activity: i64,
    pub langs: Vec<String>,
    pub markers: Vec<String>,
    pub regenerable: Vec<String>,
    /// Bytes held by this module's regenerable directories. `-1` when the run
    /// did not measure — never `0`, so "not measured" cannot read as "nothing
    /// to reclaim".
    #[serde(default = "unmeasured")]
    #[ts(type = "number")]
    pub regenerable_bytes: i64,
}

/// Serde default for the size fields, so a report written before sizing existed
/// loads as unmeasured rather than as zero.
fn unmeasured() -> i64 {
    -1
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct ProjectGitSummary {
    #[ts(type = "number")]
    pub repos: i64,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub last_commit: Option<String>,
    #[ts(type = "number")]
    pub commits: i64,
    #[ts(type = "number")]
    pub ahead: i64,
    #[ts(type = "number")]
    pub behind: i64,
    #[ts(type = "number")]
    pub stash: i64,
    #[ts(type = "number")]
    pub dirty: i64,
    /// Repos with history but no `origin` — one disk failure from gone.
    #[ts(type = "number")]
    pub no_remote: i64,
}

impl Default for ProjectGitSummary {
    fn default() -> Self {
        Self {
            repos: 0,
            branch: None,
            remote: None,
            last_commit: None,
            commits: 0,
            ahead: 0,
            behind: 0,
            stash: 0,
            dirty: 0,
            no_remote: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum ActivityStatus {
    Active,
    Recent,
    Dormant,
    Stale,
    Unknown,
}

impl ActivityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityStatus::Active => "active",
            ActivityStatus::Recent => "recent",
            ActivityStatus::Dormant => "dormant",
            ActivityStatus::Stale => "stale",
            ActivityStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum RiskLevel {
    None,
    Low,
    Med,
    High,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::None => "none",
            RiskLevel::Low => "low",
            RiskLevel::Med => "med",
            RiskLevel::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveredProject {
    /// Stable identity across scans and moves: `<root name>/<relative path>`.
    pub path: String,
    pub abs: String,
    pub name: String,
    pub area: String,
    pub bucket: String,
    #[ts(type = "number")]
    pub module_count: usize,
    pub modules: Vec<DiscoveredModule>,
    #[ts(type = "number")]
    pub activity: i64,
    #[ts(type = "number | null")]
    pub age_days: Option<i64>,
    pub status: ActivityStatus,
    pub git: ProjectGitSummary,
    pub langs: Vec<String>,
    pub regenerable: Vec<String>,
    /// Sum over the project's modules. `-1` when unmeasured.
    #[serde(default = "unmeasured")]
    #[ts(type = "number")]
    pub regenerable_bytes: i64,
    pub description: Option<String>,
    pub risk: Vec<String>,
    pub risk_level: RiskLevel,
    /// Cluster keys this project shares with another (same remote or near-name).
    pub duplicate_of: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryProgress {
    /// `find` while walking, `meta` while probing each root.
    pub phase: String,
    #[ts(type = "number")]
    pub done: usize,
    #[ts(type = "number")]
    pub total: usize,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryStats {
    #[ts(type = "number")]
    pub projects: usize,
    #[ts(type = "number")]
    pub roots: usize,
    #[ts(type = "number")]
    pub git_repos: i64,
    #[ts(type = "number")]
    pub unpushed: i64,
    #[ts(type = "number")]
    pub no_remote: i64,
    #[ts(type = "number")]
    pub active: usize,
    #[ts(type = "number")]
    pub recent: usize,
    #[ts(type = "number")]
    pub dormant: usize,
    #[ts(type = "number")]
    pub stale: usize,
    /// Total reclaimable bytes across every project. `-1` when unmeasured.
    #[serde(default = "unmeasured")]
    #[ts(type = "number")]
    pub regenerable_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DuplicateCluster {
    /// `same-remote` or `similar-name`.
    pub kind: String,
    pub key: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryReport {
    pub generated_at: String,
    /// True when the caller stopped the walk early; `projects` is then partial.
    pub cancelled: bool,
    pub projects: Vec<DiscoveredProject>,
    pub clusters: Vec<DuplicateCluster>,
    pub stats: DiscoveryStats,
    /// Names of the roots that actually existed and were walked. A root whose
    /// drive was unplugged is absent, and nothing under it may be treated as
    /// missing — see `DiscoveryRepository::import`.
    #[serde(default)]
    pub roots_scanned: Vec<String>,
}

// ── Walk ────────────────────────────────────────────────────────────────────

struct RootHit {
    abs: String,
    rel: String,
    root_name: String,
    has_git: bool,
    markers: Vec<String>,
}

fn is_skipped(name: &str) -> bool {
    SKIP_DIRS.iter().any(|s| *s == name)
}

fn markers_in(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = MARKERS
        .iter()
        .filter(|m| dir.join(m).exists())
        .map(|m| (*m).to_string())
        .collect();
    if found.is_empty() {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    if MARKER_EXTS.iter().any(|ext| name.ends_with(ext)) {
                        found.push("sln".to_string());
                        break;
                    }
                }
            }
        }
    }
    found
}

/// Topmost project roots under `base`. Stops descending at a root, except into
/// recognised monorepo containers.
fn find_roots(base: &Path, max_depth: usize) -> Vec<(std::path::PathBuf, bool, Vec<String>)> {
    let mut out = Vec::new();
    walk_roots(base, 0, max_depth, &mut out);
    out
}

fn walk_roots(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<(std::path::PathBuf, bool, Vec<String>)>,
) {
    if depth > max_depth {
        return;
    }
    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(rd) => rd.flatten().collect(),
        Err(_) => return,
    };
    let has_git = entries.iter().any(|e| {
        e.file_name() == std::ffi::OsStr::new(".git")
            && e.file_type().map(|t| t.is_dir()).unwrap_or(false)
    });
    let markers = markers_in(dir);

    if has_git || !markers.is_empty() {
        out.push((dir.to_path_buf(), has_git, markers));
        // A monorepo's packages are separate projects; everything else below a
        // root belongs to the root and is not walked again.
        for e in &entries {
            if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if !MONOREPO_DIRS.iter().any(|m| *m == name) {
                continue;
            }
            if let Ok(rd) = fs::read_dir(e.path()) {
                for sub in rd.flatten() {
                    let sub_name = sub.file_name().to_string_lossy().to_string();
                    if sub.file_type().map(|t| t.is_dir()).unwrap_or(false)
                        && !is_skipped(&sub_name)
                        && !sub_name.starts_with('.')
                    {
                        walk_roots(&sub.path(), depth + 2, max_depth, out);
                    }
                }
            }
        }
        return;
    }

    for e in &entries {
        if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if is_skipped(&name) {
            continue;
        }
        walk_roots(&e.path(), depth + 1, max_depth, out);
    }
}

/// Newest source-file mtime in epoch ms, ignoring `.git` internals and build
/// output. Without that exclusion a `git status` or an `npm install` would make
/// an abandoned project look active.
fn newest_mtime(dir: &Path, max_depth: usize) -> i64 {
    fn walk(dir: &Path, depth: usize, max_depth: usize, newest: &mut i64) {
        if depth > max_depth {
            return;
        }
        let rd = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };
        for e in rd.flatten() {
            let ft = match e.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            if ft.is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if is_skipped(&name) {
                    continue;
                }
                walk(&e.path(), depth + 1, max_depth, newest);
            } else if ft.is_file() {
                if let Ok(meta) = e.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(d) = mtime.duration_since(UNIX_EPOCH) {
                            let ms = d.as_millis() as i64;
                            if ms > *newest {
                                *newest = ms;
                            }
                        }
                    }
                }
            }
        }
    }
    let mut newest = 0i64;
    walk(dir, 0, max_depth, &mut newest);
    newest
}

/// Bytes under `dir`, following no symlinks. Used only on regenerable
/// directories, which is where the reclaimable space actually is: sizing the
/// source tree as well would double the walk to answer a question nobody asked.
fn dir_size(dir: &Path) -> i64 {
    let mut total = 0i64;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let rd = match fs::read_dir(&next) {
            Ok(rd) => rd,
            Err(_) => continue,
        };
        for e in rd.flatten() {
            match e.file_type() {
                // `is_dir()` on a DirEntry file type is already symlink-aware:
                // a link reports as a symlink, so a loop cannot be walked into.
                Ok(ft) if ft.is_dir() => stack.push(e.path()),
                Ok(ft) if ft.is_file() => {
                    if let Ok(meta) = e.metadata() {
                        total += meta.len() as i64;
                    }
                }
                _ => {}
            }
        }
    }
    total
}

/// First lines of a README with markdown noise stripped, for the registry
/// description of projects the user never described by hand.
fn read_description(dir: &Path) -> Option<String> {
    for name in [
        "README.md",
        "readme.md",
        "README.MD",
        "README.txt",
        "README",
    ] {
        let f = dir.join(name);
        if !f.exists() {
            continue;
        }
        let raw = fs::read_to_string(&f).ok()?;
        let head: String = raw.chars().take(800).collect();
        let mut cleaned = String::with_capacity(head.len());
        let mut in_link_text = false;
        for ch in head.chars() {
            match ch {
                '#' | '*' | '`' | '>' | '_' | '[' | ']' | '(' | ')' | '!' => {
                    // Markdown punctuation and link syntax collapse to a space;
                    // link *text* survives because only the delimiters are dropped.
                    if ch == '[' {
                        in_link_text = true;
                    }
                    if ch == ']' {
                        in_link_text = false;
                    }
                    cleaned.push(' ');
                }
                '\n' | '\r' | '\t' => cleaned.push(' '),
                _ => {
                    if !in_link_text || true {
                        cleaned.push(ch);
                    }
                }
            }
        }
        let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
        let trimmed: String = collapsed.chars().take(180).collect();
        if trimmed.is_empty() {
            return None;
        }
        return Some(trimmed);
    }
    None
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn to_logical(root_name: &str, rel: &str) -> String {
    let rel = rel.replace('\\', "/");
    let rel = rel.trim_matches('/');
    if rel.is_empty() {
        root_name.to_string()
    } else {
        format!("{root_name}/{rel}")
    }
}

/// Runs the walk and per-root probing.
///
/// `on_progress` is called often enough for a progress bar and is throttled by
/// the caller, matching the disk domain. Returning `false` stops the run and
/// yields a partial report with `cancelled` set. Granularity is one root during
/// the find phase and one project root during probing, so a cancel lands within
/// a directory walk rather than instantly.
pub fn discover<F>(config: &DiscoveryConfig, mut on_progress: F) -> DiscoveryReport
where
    F: FnMut(DiscoveryProgress) -> bool,
{
    let mut cancelled = false;
    let mut hits: Vec<RootHit> = Vec::new();
    let mut roots_scanned: Vec<String> = Vec::new();

    for root in &config.roots {
        let base = Path::new(&root.path);
        if !base.exists() {
            continue;
        }
        roots_scanned.push(root.name.clone());
        if !on_progress(DiscoveryProgress {
            phase: "find".into(),
            done: 0,
            total: 0,
            path: root.path.clone(),
        }) {
            cancelled = true;
            break;
        }
        for (dir, has_git, markers) in find_roots(base, config.max_depth) {
            let rel = dir
                .strip_prefix(base)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            hits.push(RootHit {
                abs: dir.to_string_lossy().to_string(),
                rel: rel.replace('\\', "/"),
                root_name: root.name.clone(),
                has_git,
                markers,
            });
        }
    }

    let total = hits.len();
    let mut modules: Vec<DiscoveredModule> = Vec::with_capacity(total);

    for (i, hit) in hits.iter().enumerate() {
        if cancelled {
            break;
        }
        if !on_progress(DiscoveryProgress {
            phase: "meta".into(),
            done: i + 1,
            total,
            path: hit.rel.clone(),
        }) {
            cancelled = true;
            break;
        }

        let dir = Path::new(&hit.abs);
        let mtime = newest_mtime(dir, 3);
        let probe: GitProbe = if hit.has_git && !config.skip_git {
            git::probe(dir)
        } else {
            GitProbe::absent()
        };
        let commit_ms = probe.last_commit_ms();
        let activity = mtime.max(commit_ms);

        let mut regenerable: Vec<String> = REGENERABLE
            .iter()
            .filter(|r| dir.join(r).is_dir())
            .map(|r| (*r).to_string())
            .collect();
        regenerable.sort();

        let regenerable_bytes = if !config.measure_sizes {
            -1
        } else if regenerable.is_empty() {
            0
        } else {
            // Sizing walks every file in node_modules, so it gets its own tick:
            // without one the bar looks frozen on a big project.
            if !on_progress(DiscoveryProgress {
                phase: "size".into(),
                done: i + 1,
                total,
                path: hit.rel.clone(),
            }) {
                cancelled = true;
                break;
            }
            regenerable.iter().map(|r| dir_size(&dir.join(r))).sum()
        };

        let langs: BTreeSet<String> = hit
            .markers
            .iter()
            .map(|m| stack_for(m).to_string())
            .collect();

        modules.push(DiscoveredModule {
            path: to_logical(&hit.root_name, &hit.rel),
            abs: hit.abs.clone(),
            git: hit.has_git,
            branch: probe.branch.clone(),
            remote: probe.remote.clone(),
            last_commit: probe.last_commit.clone(),
            commits: probe.commits,
            ahead: probe.ahead,
            behind: probe.behind,
            stash: probe.stash,
            dirty: probe.dirty,
            untracked: probe.untracked,
            activity,
            langs: langs.into_iter().collect(),
            markers: hit.markers.clone(),
            regenerable,
            regenerable_bytes,
        });
    }

    let projects = group_into_projects(config, modules);
    let clusters = super::analysis::cluster_duplicates(&projects);
    let projects = super::analysis::apply_clusters(projects, &clusters);
    let stats = summarise(&projects, total);

    DiscoveryReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        cancelled,
        projects,
        clusters,
        stats,
        roots_scanned,
    }
}

/// Collapses module roots into projects at `project_depth`, so a 45-module
/// course bundle is one row to triage rather than 45.
fn group_into_projects(
    config: &DiscoveryConfig,
    modules: Vec<DiscoveredModule>,
) -> Vec<DiscoveredProject> {
    let mut order: Vec<String> = Vec::new();
    let mut grouped: HashMap<String, Vec<DiscoveredModule>> = HashMap::new();

    for m in modules {
        let parts: Vec<&str> = m.path.split('/').collect();
        let take = (config.project_depth + 1).min(parts.len());
        let key = parts[..take].join("/");
        if !grouped.contains_key(&key) {
            order.push(key.clone());
        }
        grouped.entry(key).or_default().push(m);
    }

    let now = now_ms();
    let mut out = Vec::with_capacity(order.len());

    for key in order {
        let mut mods = grouped.remove(&key).unwrap_or_default();
        mods.sort_by(|a, b| a.path.cmp(&b.path));

        let area = key.split('/').next().unwrap_or("").to_string();
        let name = key.split('/').next_back().unwrap_or("").to_string();
        let bucket = key.split('/').take(2).collect::<Vec<_>>().join("/");

        let mut git = ProjectGitSummary::default();
        let mut activity = 0i64;
        let mut langs: BTreeSet<String> = BTreeSet::new();
        let mut regen: BTreeSet<String> = BTreeSet::new();
        // Stays -1 until a measured module contributes, so a project keeps
        // reading as unmeasured rather than as empty.
        let mut regen_bytes = -1i64;

        for m in &mods {
            activity = activity.max(m.activity);
            if m.regenerable_bytes >= 0 {
                regen_bytes = regen_bytes.max(0) + m.regenerable_bytes;
            }
            for l in &m.langs {
                langs.insert(l.clone());
            }
            for r in &m.regenerable {
                regen.insert(r.clone());
            }
            if m.git {
                git.repos += 1;
                git.commits += m.commits;
                if m.ahead > 0 {
                    git.ahead += m.ahead;
                }
                if m.behind > 0 {
                    git.behind += m.behind;
                }
                if m.dirty > 0 {
                    git.dirty += m.dirty;
                }
                git.stash += m.stash;
                match &m.remote {
                    Some(r) if !r.is_empty() => {
                        if git.remote.is_none() {
                            git.remote = Some(r.clone());
                        }
                    }
                    _ => git.no_remote += 1,
                }
                if git.branch.is_none() {
                    git.branch = m.branch.clone();
                }
                if git.last_commit.is_none() {
                    git.last_commit = m.last_commit.clone();
                }
            }
        }

        let age_days = if activity > 0 {
            Some((now - activity) / 86_400_000)
        } else {
            None
        };
        let status = match age_days {
            None => ActivityStatus::Unknown,
            Some(d) if d <= config.active_days => ActivityStatus::Active,
            Some(d) if d <= config.recent_days => ActivityStatus::Recent,
            Some(d) if d <= config.dormant_days => ActivityStatus::Dormant,
            Some(_) => ActivityStatus::Stale,
        };

        let abs = mods
            .iter()
            .find(|m| m.path == key)
            .map(|m| m.abs.clone())
            .unwrap_or_else(|| {
                // Root has no manifest of its own (a bare category holding
                // projects): derive it from the shallowest module.
                mods.first()
                    .map(|m| {
                        let depth_below = m.path.split('/').count() - key.split('/').count();
                        let mut p = Path::new(&m.abs).to_path_buf();
                        for _ in 0..depth_below {
                            p = p.parent().map(|x| x.to_path_buf()).unwrap_or(p);
                        }
                        p.to_string_lossy().to_string()
                    })
                    .unwrap_or_default()
            });

        let description = read_description(Path::new(&abs));

        let (risk, risk_level) = super::analysis::score_risk(&git, mods.len(), status);

        out.push(DiscoveredProject {
            path: key.clone(),
            abs,
            name,
            area,
            bucket,
            module_count: mods.len(),
            modules: mods,
            activity,
            age_days,
            status,
            git,
            langs: langs.into_iter().collect(),
            regenerable: regen.into_iter().collect(),
            regenerable_bytes: regen_bytes,
            description,
            risk,
            risk_level,
            duplicate_of: Vec::new(),
        });
    }

    out.sort_by(|a, b| b.activity.cmp(&a.activity));
    out
}

fn summarise(projects: &[DiscoveredProject], root_count: usize) -> DiscoveryStats {
    let mut s = DiscoveryStats {
        projects: projects.len(),
        roots: root_count,
        git_repos: 0,
        unpushed: 0,
        no_remote: 0,
        active: 0,
        recent: 0,
        dormant: 0,
        stale: 0,
        regenerable_bytes: -1,
    };
    for p in projects {
        s.git_repos += p.git.repos;
        s.unpushed += p.git.ahead;
        s.no_remote += p.git.no_remote;
        if p.regenerable_bytes >= 0 {
            s.regenerable_bytes = s.regenerable_bytes.max(0) + p.regenerable_bytes;
        }
        match p.status {
            ActivityStatus::Active => s.active += 1,
            ActivityStatus::Recent => s.recent += 1,
            ActivityStatus::Dormant => s.dormant += 1,
            ActivityStatus::Stale => s.stale += 1,
            ActivityStatus::Unknown => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Per-test directory: the suite runs in parallel, so a shared name would
    /// let one test's cleanup delete another's fixtures.
    fn tmp(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("disco-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn finds_roots_and_stops_descending() {
        let base = tmp("roots");
        let a = base.join("web").join("app-a");
        fs::create_dir_all(a.join("node_modules").join("dep")).unwrap();
        fs::write(a.join("package.json"), "{}").unwrap();
        // A manifest inside node_modules must not create a second root.
        fs::write(
            a.join("node_modules").join("dep").join("package.json"),
            "{}",
        )
        .unwrap();

        let roots = find_roots(&base, 7);
        assert_eq!(roots.len(), 1, "node_modules must be pruned");
        assert!(roots[0].0.ends_with("app-a"));
    }

    #[test]
    fn descends_into_monorepo_packages() {
        let base = tmp("mono");
        let mono = base.join("mono");
        fs::create_dir_all(mono.join("apps").join("gym")).unwrap();
        fs::create_dir_all(mono.join("apps").join("finance")).unwrap();
        fs::write(mono.join("package.json"), "{}").unwrap();
        fs::write(mono.join("apps").join("gym").join("package.json"), "{}").unwrap();
        fs::write(mono.join("apps").join("finance").join("package.json"), "{}").unwrap();

        let roots = find_roots(&base, 7);
        assert_eq!(roots.len(), 3, "monorepo packages are separate roots");
    }

    #[test]
    fn groups_modules_under_project_depth() {
        let cfg = DiscoveryConfig {
            project_depth: 2,
            ..Default::default()
        };
        let mk = |p: &str| DiscoveredModule {
            path: p.into(),
            abs: p.into(),
            git: false,
            branch: None,
            remote: None,
            last_commit: None,
            commits: 0,
            ahead: -1,
            behind: -1,
            stash: 0,
            dirty: -1,
            untracked: -1,
            activity: 1,
            langs: vec![],
            markers: vec![],
            regenerable: vec![],
            regenerable_bytes: -1,
        };
        let projects = group_into_projects(
            &cfg,
            vec![
                mk("production/portal/pocket_portal"),
                mk("production/portal/pocket_portal/apps/gym"),
                mk("production/portal/portal_web"),
            ],
        );
        assert_eq!(projects.len(), 2);
        let pocket = projects
            .iter()
            .find(|p| p.name == "pocket_portal")
            .expect("pocket_portal present");
        assert_eq!(pocket.module_count, 2, "apps/gym folds into its parent");
    }

    #[test]
    fn sizes_only_regenerable_dirs_and_only_when_asked() {
        let base = tmp("size");
        let p = base.join("proj");
        fs::create_dir_all(p.join("node_modules").join("dep")).unwrap();
        fs::write(p.join("package.json"), "{}").unwrap();
        fs::write(p.join("src.js"), "0123456789").unwrap();
        fs::write(p.join("node_modules").join("dep").join("a.js"), "0123").unwrap();

        let root = vec![DiscoveryRoot {
            name: "t".into(),
            path: base.to_string_lossy().to_string(),
        }];
        let off = discover(
            &DiscoveryConfig {
                roots: root.clone(),
                skip_git: true,
                ..Default::default()
            },
            |_| true,
        );
        assert_eq!(
            off.projects[0].regenerable_bytes, -1,
            "unmeasured must not read as zero"
        );
        assert_eq!(off.roots_scanned, vec!["t".to_string()]);

        let on = discover(
            &DiscoveryConfig {
                roots: root,
                skip_git: true,
                measure_sizes: true,
                ..Default::default()
            },
            |_| true,
        );
        assert_eq!(
            on.projects[0].regenerable_bytes, 4,
            "source files must not be counted, only node_modules"
        );
        assert_eq!(on.stats.regenerable_bytes, 4);
    }

    #[test]
    fn build_output_does_not_count_as_activity() {
        let base = tmp("mtime");
        let p = base.join("proj");
        fs::create_dir_all(p.join("node_modules")).unwrap();
        fs::write(p.join("src.rs"), "old").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(p.join("node_modules").join("fresh.js"), "new").unwrap();

        let with_skip = newest_mtime(&p, 3);
        let src_only = fs::metadata(p.join("src.rs"))
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert_eq!(
            with_skip, src_only,
            "a file in node_modules must not date the project"
        );
    }
}
