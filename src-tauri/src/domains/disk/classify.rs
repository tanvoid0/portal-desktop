use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domains::disk::scan::{now_secs, FileEntry};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Risk {
    Safe,
    Review,
    Danger,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub reason: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub risk: Risk,
}

/// Directory names that are almost always safe to regenerate/redownload.
const CACHE_DIRS: &[(&str, &str)] = &[
    ("node_modules", "node_modules"),
    ("target", "rust-target"),
    (".gradle", "gradle-cache"),
    ("__pycache__", "python-cache"),
    (".cache", "cache"),
    ("Cache", "cache"),
    ("Cache_Data", "cache"),
    ("tmp", "temp"),
    ("temp", "temp"),
    ("Temp", "temp"),
];

/// Never propose anything inside these roots — OS / installed programs.
const PROTECTED: &[&str] = &[
    "windows",
    "program files",
    "program files (x86)",
    "system volume information",
    "$recycle.bin",
];

const LARGE_FILE_BYTES: u64 = 200 * 1024 * 1024; // 200 MB
const OLD_SECS: u64 = 180 * 24 * 60 * 60; // 180 days

pub(crate) fn is_protected(path: &Path, extra: &[String]) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    if extra
        .iter()
        .any(|p| !p.is_empty() && lower.starts_with(&p.to_lowercase()))
    {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy().to_lowercase();
        PROTECTED.contains(&s.as_str())
    })
}

/// Returns the nearest ancestor directory of `path` (within `root`) whose name
/// matches a cache pattern, along with that pattern's kind label.
fn cache_ancestor(path: &Path, root: &Path) -> Option<(PathBuf, &'static str)> {
    let mut acc = root.to_path_buf();
    for comp in path.strip_prefix(root).ok()?.components() {
        acc.push(comp);
        let name = comp.as_os_str().to_string_lossy();
        if let Some((_, kind)) = CACHE_DIRS.iter().find(|(n, _)| *n == name.as_ref()) {
            return Some((acc.clone(), *kind));
        }
    }
    None
}

/// Turns raw file entries into reviewable proposals via deterministic heuristics.
/// AI classification (future) plugs in here for the ambiguous remainder.
pub fn classify(root: &str, files: &[FileEntry], extra_protected: &[String]) -> Vec<Proposal> {
    let root_path = PathBuf::from(root);
    let now = now_secs();

    // Aggregate cache/temp directories.
    struct Group {
        kind: &'static str,
        size: u64,
        count: u64,
    }
    let mut groups: HashMap<PathBuf, Group> = HashMap::new();
    let mut large_old: Vec<&FileEntry> = Vec::new();

    for f in files {
        if is_protected(&f.path, extra_protected) {
            continue;
        }
        if let Some((dir, kind)) = cache_ancestor(&f.path, &root_path) {
            let g = groups.entry(dir).or_insert(Group {
                kind,
                size: 0,
                count: 0,
            });
            g.size += f.size;
            g.count += 1;
        } else if f.size >= LARGE_FILE_BYTES && now.saturating_sub(f.modified_secs) >= OLD_SECS {
            large_old.push(f);
        }
    }

    let mut out = Vec::new();

    for (dir, g) in groups {
        out.push(Proposal {
            id: format!("dir:{}", dir.display()),
            path: dir.display().to_string(),
            kind: g.kind.to_string(),
            reason: format!(
                "Regenerable {} directory · {} files. Safe to delete; rebuilt on demand.",
                g.kind, g.count
            ),
            size_bytes: g.size,
            file_count: g.count,
            risk: Risk::Safe,
        });
    }

    for f in large_old {
        let age_days = now.saturating_sub(f.modified_secs) / (24 * 60 * 60);
        out.push(Proposal {
            id: format!("file:{}", f.path.display()),
            path: f.path.display().to_string(),
            kind: "old-large".to_string(),
            reason: format!("Large file untouched for {age_days} days. Review before removing."),
            size_bytes: f.size,
            file_count: 1,
            risk: Risk::Review,
        });
    }

    // Biggest wins first.
    out.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    out
}
