use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::domains::disk::classify::{is_protected, Risk};
use crate::domains::disk::scan::FileEntry;

/// A recognized project ecosystem: how we detect its root and which of its
/// directories are regenerable (rebuilt by the toolchain — safe to reclaim).
struct Ecosystem {
    kind: &'static str,
    /// Exact filenames whose presence marks a project root.
    markers: &'static [&'static str],
    /// Filename extensions (with dot) that mark a project root, e.g. ".csproj".
    marker_exts: &'static [&'static str],
    /// Directory names, *directly under the project root*, safe to delete.
    temp_dirs: &'static [&'static str],
}

/// Marker → temp-dir table. Ambiguous names (`target`, `build`, `dist`, `bin`)
/// are only ever flagged when they sit under a matching project root, which is
/// what makes this scanner safer than a blind name match.
const ECOSYSTEMS: &[Ecosystem] = &[
    Ecosystem {
        kind: "node",
        markers: &["package.json"],
        marker_exts: &[],
        temp_dirs: &[
            "node_modules",
            ".next",
            ".nuxt",
            "dist",
            ".turbo",
            ".parcel-cache",
            ".svelte-kit",
            ".angular",
            "coverage",
        ],
    },
    Ecosystem {
        kind: "rust",
        markers: &["Cargo.toml"],
        marker_exts: &[],
        temp_dirs: &["target"],
    },
    Ecosystem {
        kind: "maven",
        markers: &["pom.xml"],
        marker_exts: &[],
        temp_dirs: &["target"],
    },
    Ecosystem {
        kind: "gradle",
        markers: &[
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
        marker_exts: &[],
        temp_dirs: &["build", ".gradle"],
    },
    Ecosystem {
        kind: "python",
        markers: &["pyproject.toml", "setup.py", "setup.cfg", "requirements.txt"],
        marker_exts: &[],
        temp_dirs: &[
            "__pycache__",
            ".venv",
            "venv",
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".tox",
            "build",
            "dist",
            ".eggs",
        ],
    },
    Ecosystem {
        kind: "dotnet",
        markers: &[],
        marker_exts: &[".csproj", ".fsproj", ".vbproj", ".sln"],
        temp_dirs: &["bin", "obj"],
    },
    Ecosystem {
        kind: "go",
        markers: &["go.mod"],
        marker_exts: &[],
        temp_dirs: &["vendor"],
    },
    Ecosystem {
        kind: "php",
        markers: &["composer.json"],
        marker_exts: &[],
        temp_dirs: &["vendor"],
    },
];

/// One regenerable directory belonging to a detected project.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemp {
    pub id: String,
    pub path: String,
    pub temp_kind: String, // directory name, e.g. "node_modules"
    pub size_bytes: u64,
    pub file_count: u64,
    pub risk: Risk, // always Safe — regenerable by the toolchain
}

/// A detected project root plus its reclaimable temp directories.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub root: String,
    pub kind: String, // ecosystem(s), e.g. "rust" or "maven+node"
    pub total_bytes: u64,
    pub file_count: u64,
    pub temps: Vec<ProjectTemp>,
}

/// Result of a project-aware scan: projects, each with their temp dirs.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScan {
    pub root: String,
    pub project_count: usize,
    pub total_bytes: u64, // total reclaimable across all projects
    pub projects: Vec<Project>,
}

fn matches_marker(eco: &Ecosystem, name: &str) -> bool {
    eco.markers.contains(&name) || eco.marker_exts.iter().any(|e| name.ends_with(e))
}

/// Detects project roots by their marker files, then attributes every file to
/// the nearest temp directory sitting directly under a matching root. Groups by
/// project so the UI can offer per-project cleanup. Never reads or mutates disk
/// — works purely off the already-collected file list.
pub fn detect(root: &str, files: &[FileEntry], extra_protected: &[String]) -> ProjectScan {
    let root_path = PathBuf::from(root);

    // Pass 1: find project roots. A directory is a root for every ecosystem
    // whose marker it contains.
    let mut roots: HashMap<PathBuf, Vec<&'static Ecosystem>> = HashMap::new();
    for f in files {
        if is_protected(&f.path, extra_protected) {
            continue;
        }
        let (Some(parent), Some(name)) = (f.path.parent(), f.path.file_name()) else {
            continue;
        };
        let name = name.to_string_lossy();
        for eco in ECOSYSTEMS {
            if matches_marker(eco, &name) {
                let e = roots.entry(parent.to_path_buf()).or_default();
                if !e.iter().any(|x| x.kind == eco.kind) {
                    e.push(eco);
                }
            }
        }
    }

    // Pass 2: aggregate each file into the nearest temp dir under a known root.
    struct Group {
        kind: &'static str,
        root: PathBuf,
        size: u64,
        count: u64,
    }
    let mut groups: HashMap<PathBuf, Group> = HashMap::new();

    for f in files {
        if is_protected(&f.path, extra_protected) {
            continue;
        }
        let Ok(rel) = f.path.strip_prefix(&root_path) else {
            continue;
        };
        let mut acc = root_path.clone();
        for comp in rel.components() {
            let parent = acc.clone();
            acc.push(comp.as_os_str());
            let name = comp.as_os_str().to_string_lossy();
            // Qualifies only if the parent dir is a project root whose ecosystem
            // lists this dir name as regenerable. First (shallowest) match wins,
            // so nested node_modules roll up into the top-level one.
            if let Some(ecos) = roots.get(&parent) {
                if let Some(eco) = ecos.iter().find(|e| e.temp_dirs.contains(&name.as_ref())) {
                    let g = groups.entry(acc.clone()).or_insert(Group {
                        kind: eco.kind,
                        root: parent.clone(),
                        size: 0,
                        count: 0,
                    });
                    g.size += f.size;
                    g.count += 1;
                    break;
                }
            }
        }
    }

    // Assemble: temp groups → per-project buckets.
    let mut by_root: HashMap<PathBuf, Vec<ProjectTemp>> = HashMap::new();
    for (dir, g) in groups {
        let temp_kind = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| g.kind.to_string());
        by_root.entry(g.root).or_default().push(ProjectTemp {
            id: format!("proj:{}", dir.display()),
            path: dir.display().to_string(),
            temp_kind,
            size_bytes: g.size,
            file_count: g.count,
            risk: Risk::Safe,
        });
    }

    let mut projects: Vec<Project> = by_root
        .into_iter()
        .map(|(root_dir, mut temps)| {
            temps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            let total_bytes = temps.iter().map(|t| t.size_bytes).sum();
            let file_count = temps.iter().map(|t| t.file_count).sum();
            Project {
                kind: project_kind(&roots, &root_dir),
                root: root_dir.display().to_string(),
                total_bytes,
                file_count,
                temps,
            }
        })
        .collect();

    // Biggest projects first.
    projects.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    let total_bytes = projects.iter().map(|p| p.total_bytes).sum();

    ProjectScan {
        root: root.to_string(),
        project_count: projects.len(),
        total_bytes,
        projects,
    }
}

/// Distinct ecosystem label(s) detected at a root, e.g. "maven+node".
fn project_kind(roots: &HashMap<PathBuf, Vec<&'static Ecosystem>>, root_dir: &Path) -> String {
    let kinds: BTreeSet<&str> = roots
        .get(root_dir)
        .map(|ecos| ecos.iter().map(|e| e.kind).collect())
        .unwrap_or_default();
    if kinds.is_empty() {
        "project".to_string()
    } else {
        kinds.into_iter().collect::<Vec<_>>().join("+")
    }
}
