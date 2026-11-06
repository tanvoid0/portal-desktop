//! Registry rows whose folder is gone, and where that folder may have gone.
//!
//! Moving or renaming a folder leaves its row pointing at nothing. The audit
//! lists those rows with same- or similar-named folders found under the scan
//! roots, so relinking is a click instead of a hunt through Explorer.

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::fs;
use std::path::PathBuf;
use ts_rs::TS;

use super::scan::SKIP_DIRS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct UnavailableProject {
    pub id: i32,
    pub name: String,
    pub path: String,
    /// Existing folders whose name matches or resembles the missing one, best first.
    pub suggestions: Vec<String>,
}

const MAX_SUGGESTIONS: usize = 5;

/// Files whose presence says "this folder is a project root", not an asset dir
/// that happens to share the name (`public/media/job`, `lib/collections/job`).
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "Cargo.toml",
    "pubspec.yaml",
    "pom.xml",
    "build.gradle",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "composer.json",
    "Gemfile",
    "CMakeLists.txt",
];

fn looks_like_project(dir: &std::path::Path) -> bool {
    PROJECT_MARKERS.iter().any(|m| dir.join(m).exists())
}

/// Lowercase letters and digits only, so `care-flex`, `care_flex` and `CareFlex` meet.
fn key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Path components, split on either separator: a row may hold a path written on
/// another OS (`/media/...` in a Windows registry).
fn parts(path: &str) -> Vec<&str> {
    path.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
}

/// One name inside the other, and not by a sliver: `study-platform` resembles
/// `cert-study-platform`, but `spring` does not resemble `spring-professional-certification`.
fn similar(a: &str, b: &str) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    short.len() >= 4 && short.len() * 2 >= long.len() && long.contains(short)
}

/// Every directory under `roots` down to `max_depth`, minus build output, VCS
/// internals and dot-dirs. Symlinks are not followed, so junction loops end.
pub fn index_dirs(roots: &[PathBuf], max_depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = roots.iter().map(|r| (r.clone(), 0)).collect();
    while let Some((dir, depth)) = stack.pop() {
        if depth >= max_depth {
            continue;
        }
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            let p = e.path();
            out.push(p.clone());
            stack.push((p, depth + 1));
        }
    }
    out
}

/// Folders in `dirs` that could be `missing` after a move or rename. Exact name
/// matches beat similar ones; within a tier, the folder whose parents repeat
/// more of the old parents' names wins (`production/flutter/daery` moved under a
/// new root still sits in `production/flutter`), then the shorter path.
pub fn suggest(missing: &str, dirs: &[PathBuf]) -> Vec<String> {
    suggest_with(missing, dirs, looks_like_project)
}

/// [`suggest`] with the "is this a project root" probe injected, so the ranking
/// can be tested without touching the disk.
fn suggest_with(
    missing: &str,
    dirs: &[PathBuf],
    is_project: impl Fn(&std::path::Path) -> bool,
) -> Vec<String> {
    let old = parts(missing);
    let Some((base, old_parents)) = old.split_last() else {
        return Vec::new();
    };
    let want = key(base);
    if want.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(u8, bool, Reverse<usize>, usize, String)> = dirs
        .iter()
        .filter_map(|d| {
            let shown = d.display().to_string();
            let new = parts(&shown);
            let (name, new_parents) = new.split_last()?;
            let have = key(name);
            let tier = if have == want {
                0
            } else if similar(&want, &have) {
                1
            } else {
                return None;
            };
            let shared = old_parents
                .iter()
                .filter(|a| new_parents.iter().any(|b| key(a) == key(b)))
                .count();
            // `false` sorts first, so project-like folders lead their tier.
            Some((tier, !is_project(d), Reverse(shared), shown.len(), shown))
        })
        .collect();
    scored.sort();
    // When anything looks like a real project, the name-only matches are noise.
    if scored.iter().any(|s| !s.1) {
        scored.retain(|s| !s.1);
    }
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(.., p)| p)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Name-and-parents ranking only: the paths in these tests may exist on the
    /// machine running them, so the disk probe is switched off.
    fn suggest(missing: &str, dirs: &[PathBuf]) -> Vec<String> {
        suggest_with(missing, dirs, |_| false)
    }

    fn dirs(paths: &[&str]) -> Vec<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn exact_name_under_the_same_parents_ranks_first() {
        let d = dirs(&[
            r"D:\projects\practice\flutter\daery",
            r"D:\projects\production\flutter\daery",
            r"D:\projects\production\flutter\daery2",
            r"D:\projects\production\flutter\other",
        ]);
        let s = suggest(r"D:\production\flutter\daery", &d);
        assert_eq!(s[0], r"D:\projects\production\flutter\daery");
        assert_eq!(s[1], r"D:\projects\practice\flutter\daery");
        // `daery2` is longer by one char of five — similar, ranked after exact.
        assert_eq!(s[2], r"D:\projects\production\flutter\daery2");
        assert_eq!(s.len(), 3, "`other` must not be suggested: {s:?}");
    }

    #[test]
    fn similar_needs_a_real_overlap_and_separators_do_not_matter() {
        let d = dirs(&[
            r"D:\projects\practice\java\study-platform",
            r"D:\projects\practice\java\spring",
            r"D:\projects\production\aipairs\care_flex",
        ]);
        assert_eq!(
            suggest(r"D:\practice\java\cert-study-platform", &d),
            vec![r"D:\projects\practice\java\study-platform".to_string()]
        );
        assert!(suggest(r"D:\practice\java\spring-professional-certification", &d).is_empty());
        assert_eq!(
            suggest("/media/tan/Workspace/production/devstrail/care-flex", &d),
            vec![r"D:\projects\production\aipairs\care_flex".to_string()]
        );
    }

    #[test]
    fn parents_count_wherever_they_sit() {
        let d = dirs(&[
            r"D:\projects\production\javascript\next-server\media\JOB",
            r"D:\projects\production\portfolio\clean-portfolio\software-engineer\JOB",
        ]);
        assert_eq!(
            suggest(r"D:\production\portfolio\JOB", &d)[0],
            r"D:\projects\production\portfolio\clean-portfolio\software-engineer\JOB"
        );
    }

    #[test]
    fn short_names_match_exactly_only() {
        let d = dirs(&[r"D:\projects\a\jobs", r"D:\projects\b\job"]);
        assert_eq!(
            suggest(r"D:\x\JOB", &d),
            vec![r"D:\projects\b\job".to_string()]
        );
    }

    #[test]
    fn index_skips_build_output_and_dot_dirs() {
        let tmp = std::env::temp_dir().join(format!("relink-{}", std::process::id()));
        for sub in ["a/b", "a/node_modules/x", "a/.git/y", "c"] {
            fs::create_dir_all(tmp.join(sub)).unwrap();
        }
        let mut got: Vec<String> = index_dirs(&[tmp.clone()], 5)
            .iter()
            .map(|p| p.strip_prefix(&tmp).unwrap().to_string_lossy().replace('\\', "/"))
            .collect();
        got.sort();
        fs::remove_dir_all(&tmp).ok();
        assert_eq!(got, vec!["a", "a/b", "c"]);
    }

    #[test]
    fn project_roots_beat_same_named_asset_folders() {
        let base = std::env::temp_dir().join(format!("relink-test-{}", std::process::id()));
        let real = base.join("work").join("JOB");
        let asset = base.join("site").join("public").join("job");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::create_dir_all(&asset).unwrap();
        std::fs::write(real.join("package.json"), "{}").unwrap();

        let got = suggest_with(r"D:\old\JOB", &[asset.clone(), real.clone()], looks_like_project);
        std::fs::remove_dir_all(&base).ok();
        assert_eq!(got, vec![real.display().to_string()], "asset dir must be dropped");
    }
}
