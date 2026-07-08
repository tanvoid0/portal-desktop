//! Project-ecosystem cleaner — walks drives/folders, finds marker files
//! (package.json, Cargo.toml, …), flags regenerable temp dirs under each root.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{
    CleanedItem, DevCleaner, DevCleanerGroup, DevCleanerItem, DevCleanerScanContext,
    DevCleanerWalkHooks, FailedCleanItem,
};
use crate::domains::disk::classify::{is_protected, Risk};
use crate::domains::disk::quarantine::{self, QuarantineItem};
use crate::domains::disk::scan::{self, FileEntry};

/// A recognized project ecosystem: how we detect its root and which of its
/// directories are regenerable (rebuilt by the toolchain — safe to reclaim).
struct Ecosystem {
    kind: &'static str,
    markers: &'static [&'static str],
    marker_exts: &'static [&'static str],
    temp_dirs: &'static [&'static str],
}

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
        markers: &[
            "pyproject.toml",
            "setup.py",
            "setup.cfg",
            "requirements.txt",
        ],
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

pub struct ProjectDevCleaner;

impl DevCleaner for ProjectDevCleaner {
    fn id(&self) -> &'static str {
        "projects"
    }

    fn label(&self) -> &'static str {
        "Projects"
    }

    fn needs_roots(&self) -> bool {
        true
    }

    fn scan(
        &self,
        ctx: &DevCleanerScanContext,
        hooks: Option<&DevCleanerWalkHooks<'_>>,
    ) -> Result<DevCleanerGroup, String> {
        if ctx.roots.is_empty() {
            return Ok(unavailable_group(
                "Pick one or more drives or folders to scan for project junk",
            ));
        }
        let Some(hooks) = hooks else {
            return Err("filesystem walk hooks required for project scan".into());
        };

        let mut items = Vec::new();
        for root in &ctx.roots {
            let raw = scan::walk(
                root,
                hooks.cancel,
                |p| (hooks.on_progress)(p),
                |_files, _total, _bytes, _elapsed| {},
            );
            if raw.cancelled {
                return Err("cancelled".into());
            }
            items.extend(detect_items(root, &raw.files, &ctx.extra_protected));
        }

        items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        let total_bytes = items.iter().map(|i| i.size_bytes).sum();
        Ok(DevCleanerGroup {
            cleaner_id: self.id().to_string(),
            label: self.label().to_string(),
            available: true,
            unavailable_reason: None,
            total_bytes,
            items,
        })
    }

    fn clean(&self, items: &[DevCleanerItem]) -> (Vec<CleanedItem>, Vec<FailedCleanItem>) {
        let batch: Vec<QuarantineItem> = items
            .iter()
            .map(|i| QuarantineItem {
                path: i.path.clone(),
                kind: i.kind.clone(),
            })
            .collect();
        let result = quarantine::quarantine(batch, |_| {});
        let cleaned = result
            .moved
            .into_iter()
            .map(|m| CleanedItem {
                path: m.path,
                kind: m.kind,
                size_bytes: m.size_bytes,
            })
            .collect();
        let failed = result
            .failed
            .into_iter()
            .map(|f| FailedCleanItem {
                path: f.path,
                kind: f.kind,
                error: f.error,
            })
            .collect();
        (cleaned, failed)
    }
}

fn unavailable_group(reason: &str) -> DevCleanerGroup {
    DevCleanerGroup {
        cleaner_id: "projects".to_string(),
        label: "Projects".to_string(),
        available: false,
        unavailable_reason: Some(reason.to_string()),
        total_bytes: 0,
        items: vec![],
    }
}

fn matches_marker(eco: &Ecosystem, name: &str) -> bool {
    eco.markers.contains(&name) || eco.marker_exts.iter().any(|e| name.ends_with(e))
}

/// Detects project roots and temp dirs from an already-walked file list.
pub fn detect_items(
    root: &str,
    files: &[FileEntry],
    extra_protected: &[String],
) -> Vec<DevCleanerItem> {
    let root_path = PathBuf::from(root);
    let project_kinds = find_project_roots(files, extra_protected);

    struct Group {
        eco_kind: &'static str,
        project_root: PathBuf,
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
            if let Some(ecos) = project_kinds.get(&parent) {
                if let Some(eco) = ecos.iter().find(|e| e.temp_dirs.contains(&name.as_ref())) {
                    let g = groups.entry(acc.clone()).or_insert(Group {
                        eco_kind: eco.kind,
                        project_root: parent.clone(),
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

    let mut items: Vec<DevCleanerItem> = groups
        .into_iter()
        .map(|(dir, g)| {
            let temp_kind = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| g.eco_kind.to_string());
            let project_root = g.project_root.display().to_string();
            let eco_label = project_kind_label(&project_kinds, &g.project_root);
            DevCleanerItem {
                id: format!("proj:{}", dir.display()),
                cleaner_id: "projects".to_string(),
                path: dir.display().to_string(),
                kind: temp_kind.clone(),
                reason: format!("regenerable {eco_label} dir ({temp_kind})"),
                size_bytes: g.size,
                file_count: g.count,
                risk: Risk::Safe,
                group_label: Some(project_root),
            }
        })
        .collect();

    items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    items
}

fn find_project_roots(
    files: &[FileEntry],
    extra_protected: &[String],
) -> HashMap<PathBuf, Vec<&'static Ecosystem>> {
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
    roots
}

fn project_kind_label(
    roots: &HashMap<PathBuf, Vec<&'static Ecosystem>>,
    root_dir: &Path,
) -> String {
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

// ── Legacy view types (Projects tab IPC) ────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemp {
    pub id: String,
    pub path: String,
    pub temp_kind: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub risk: Risk,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub root: String,
    pub kind: String,
    pub total_bytes: u64,
    pub file_count: u64,
    pub temps: Vec<ProjectTemp>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScan {
    pub root: String,
    pub project_count: usize,
    pub total_bytes: u64,
    pub projects: Vec<Project>,
}

/// Converts unified cleaner items back into the nested Projects-tab shape.
pub fn to_project_scan(primary_root: &str, group: &DevCleanerGroup) -> ProjectScan {
    let mut by_root: HashMap<String, (String, Vec<ProjectTemp>)> = HashMap::new();

    for item in &group.items {
        let project_root = item
            .group_label
            .clone()
            .unwrap_or_else(|| primary_root.to_string());
        let eco = item
            .reason
            .strip_prefix("regenerable ")
            .and_then(|r| r.split(' ').next())
            .unwrap_or("project")
            .to_string();
        let entry = by_root
            .entry(project_root.clone())
            .or_insert((eco, Vec::new()));
        entry.1.push(ProjectTemp {
            id: item.id.clone(),
            path: item.path.clone(),
            temp_kind: item.kind.clone(),
            size_bytes: item.size_bytes,
            file_count: item.file_count,
            risk: item.risk,
        });
    }

    let mut projects: Vec<Project> = by_root
        .into_iter()
        .map(|(root, (kind, mut temps))| {
            temps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            let total_bytes = temps.iter().map(|t| t.size_bytes).sum();
            let file_count = temps.iter().map(|t| t.file_count).sum();
            Project {
                root,
                kind,
                total_bytes,
                file_count,
                temps,
            }
        })
        .collect();

    projects.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    let total_bytes = projects.iter().map(|p| p.total_bytes).sum();

    ProjectScan {
        root: primary_root.to_string(),
        project_count: projects.len(),
        total_bytes,
        projects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_items_finds_node_modules() {
        let root = r"C:\code\myapp";
        let files = vec![
            FileEntry {
                path: PathBuf::from(r"C:\code\myapp\package.json"),
                size: 100,
                modified_secs: 0,
            },
            FileEntry {
                path: PathBuf::from(r"C:\code\myapp\node_modules\lodash\index.js"),
                size: 5000,
                modified_secs: 0,
            },
        ];
        let items = detect_items(root, &files, &[]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "node_modules");
        assert_eq!(items[0].size_bytes, 5000);
    }
}
