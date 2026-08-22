//! Persisted audit state: the roots to scan, the last report, and the history
//! of what changed between scans.
//!
//! The `projects` table is the registry — what the app tracks. This file is the
//! *audit's own* memory: the folders the user chose to watch, the last walk in
//! full, and a change log. Keeping it beside `settings.json` rather than in the
//! database is deliberate — it is one user's working state for one view, it is
//! rewritten whole on every scan, and a JSON file can be read, diffed and
//! deleted by hand when a scan goes wrong.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use ts_rs::TS;

use super::scan::{DiscoveredProject, DiscoveryConfig, DiscoveryReport};

/// Change-log entries kept. At one scan a day this is well over a year of
/// history, and it bounds a file that is otherwise append-only forever.
const MAX_CHANGES: usize = 500;

const FILE: &str = "discovery-state.json";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryChange {
    /// `generated_at` of the scan that observed it, RFC 3339.
    pub at: String,
    /// `added`, `removed` or `moved`.
    pub kind: String,
    /// Logical id the project has *now* (for `removed`, the one it had).
    pub path: String,
    /// Previous logical id, on `moved` only.
    pub from: Option<String>,
    /// Human-readable extra: the absolute path, or the old one on a move.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct DiscoveryState {
    #[serde(default)]
    pub config: DiscoveryConfig,
    /// The last completed scan, replayed on load so the view is populated
    /// without walking the disk again.
    #[serde(default)]
    pub report: Option<DiscoveryReport>,
    /// Newest first.
    #[serde(default)]
    pub changes: Vec<DiscoveryChange>,
}

fn path() -> PathBuf {
    crate::app_paths::config_dir().join(FILE)
}

/// Reads the state file. A missing or corrupt file yields defaults rather than
/// an error: the audit view must still open, and the next scan rewrites it.
pub fn load() -> DiscoveryState {
    let p = path();
    let Ok(raw) = fs::read_to_string(&p) else {
        return DiscoveryState::default();
    };
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        crate::log_warn!("Discovery", "Ignoring unreadable {}: {}", p.display(), e);
        DiscoveryState::default()
    })
}

pub fn save(state: &DiscoveryState) -> Result<(), String> {
    let p = path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&p, json).map_err(|e| format!("{}: {e}", p.display()))
}

/// Saves the config, leaving the report and history alone.
///
/// ponytail: rewrites the whole file, report included, to store a few hundred
/// bytes of config — a multi-MB write every time a checkbox is ticked. Split the
/// report into its own file if that ever shows up as lag.
pub fn save_config(config: DiscoveryConfig) -> Result<DiscoveryState, String> {
    let mut state = load();
    state.config = config;
    save(&state)?;
    Ok(state)
}

/// Records a completed scan: replaces the report and prepends what changed.
pub fn record_scan(config: &DiscoveryConfig, report: &DiscoveryReport) -> Vec<DiscoveryChange> {
    let mut state = load();
    let changes = diff(state.report.as_ref(), report);
    state.config = config.clone();
    state.report = Some(report.clone());
    let mut merged = changes.clone();
    merged.extend(state.changes.into_iter());
    merged.truncate(MAX_CHANGES);
    state.changes = merged;
    if let Err(e) = save(&state) {
        crate::log_warn!("Discovery", "Could not persist scan state: {}", e);
    }
    changes
}

fn remote_of(p: &DiscoveredProject) -> Option<String> {
    p.git
        .remote
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(|r| {
            r.to_lowercase()
                .trim_end_matches('/')
                .trim_end_matches(".git")
                .to_string()
        })
}

/// What changed between two scans.
///
/// Restricted to roots both runs actually walked: an unplugged drive drops out
/// of `roots_scanned`, and reporting its projects as removed would bury the real
/// changes under a hundred false ones. The first scan produces nothing — it is
/// the baseline, not a hundred additions.
pub fn diff(prev: Option<&DiscoveryReport>, next: &DiscoveryReport) -> Vec<DiscoveryChange> {
    let Some(prev) = prev else {
        return Vec::new();
    };
    let common: HashSet<&str> = prev
        .roots_scanned
        .iter()
        .map(String::as_str)
        .filter(|r| next.roots_scanned.iter().any(|n| n == r))
        .collect();

    // A logical id is `<root name>/<rest>`; the root is its first segment.
    let in_common =
        |p: &DiscoveredProject| common.contains(p.path.split('/').next().unwrap_or_default());

    let old: Vec<&DiscoveredProject> = prev.projects.iter().filter(|p| in_common(p)).collect();
    let new: Vec<&DiscoveredProject> = next.projects.iter().filter(|p| in_common(p)).collect();

    let old_paths: HashSet<&str> = old.iter().map(|p| p.path.as_str()).collect();
    let new_paths: HashSet<&str> = new.iter().map(|p| p.path.as_str()).collect();

    let mut gone: Vec<&DiscoveredProject> = old
        .iter()
        .copied()
        .filter(|p| !new_paths.contains(p.path.as_str()))
        .collect();
    let fresh: Vec<&DiscoveredProject> = new
        .iter()
        .copied()
        .filter(|p| !old_paths.contains(p.path.as_str()))
        .collect();

    let at = next.generated_at.clone();
    let mut out = Vec::new();

    // Pair a disappearance with an appearance before calling either one real:
    // moving a project is the common case, and reporting it as a delete plus an
    // unrelated add is what loses the user's trust in the log.
    for p in fresh {
        let matched = gone.iter().position(|g| {
            g.name == p.name
                && match (remote_of(g), remote_of(p)) {
                    (Some(a), Some(b)) => a == b,
                    (None, None) => true,
                    // One side has a remote and the other does not: same name is
                    // not enough to claim they are the same project.
                    _ => false,
                }
        });
        match matched {
            Some(i) => {
                let from = gone.remove(i);
                out.push(DiscoveryChange {
                    at: at.clone(),
                    kind: "moved".into(),
                    path: p.path.clone(),
                    from: Some(from.path.clone()),
                    detail: Some(from.abs.clone()),
                });
            }
            None => out.push(DiscoveryChange {
                at: at.clone(),
                kind: "added".into(),
                path: p.path.clone(),
                from: None,
                detail: Some(p.abs.clone()),
            }),
        }
    }

    for g in gone {
        out.push(DiscoveryChange {
            at: at.clone(),
            kind: "removed".into(),
            path: g.path.clone(),
            from: None,
            detail: Some(g.abs.clone()),
        });
    }

    // Same identity, different folder: a rename above the project, which the
    // path-set comparison above cannot see.
    for p in &new {
        if let Some(before) = old.iter().find(|o| o.path == p.path) {
            if !before.abs.eq_ignore_ascii_case(&p.abs) {
                out.push(DiscoveryChange {
                    at: at.clone(),
                    kind: "moved".into(),
                    path: p.path.clone(),
                    from: Some(p.path.clone()),
                    detail: Some(before.abs.clone()),
                });
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::projects::discovery::scan::{
        ActivityStatus, DiscoveryStats, ProjectGitSummary, RiskLevel,
    };

    fn project(path: &str, abs: &str, remote: Option<&str>) -> DiscoveredProject {
        DiscoveredProject {
            path: path.into(),
            abs: abs.into(),
            name: path.split('/').next_back().unwrap_or("").into(),
            area: path.split('/').next().unwrap_or("").into(),
            bucket: path.into(),
            module_count: 1,
            modules: vec![],
            activity: 1,
            age_days: Some(1),
            status: ActivityStatus::Active,
            git: ProjectGitSummary {
                remote: remote.map(str::to_string),
                ..Default::default()
            },
            langs: vec![],
            regenerable: vec![],
            regenerable_bytes: -1,
            description: None,
            risk: vec![],
            risk_level: RiskLevel::None,
            duplicate_of: vec![],
        }
    }

    fn report(roots: &[&str], projects: Vec<DiscoveredProject>) -> DiscoveryReport {
        DiscoveryReport {
            generated_at: "2026-08-22T00:00:00Z".into(),
            cancelled: false,
            projects,
            clusters: vec![],
            stats: DiscoveryStats {
                projects: 0,
                roots: roots.len(),
                git_repos: 0,
                unpushed: 0,
                no_remote: 0,
                active: 0,
                recent: 0,
                dormant: 0,
                stale: 0,
                regenerable_bytes: -1,
            },
            roots_scanned: roots.iter().map(|r| r.to_string()).collect(),
        }
    }

    #[test]
    fn first_scan_is_a_baseline_not_a_hundred_additions() {
        let next = report(&["p"], vec![project("p/web/app", "D:/p/web/app", None)]);
        assert!(diff(None, &next).is_empty());
    }

    #[test]
    fn move_is_one_change_not_a_delete_plus_an_add() {
        let prev = report(
            &["p"],
            vec![project(
                "p/web/app",
                "D:/p/web/app",
                Some("git@h:me/app.git"),
            )],
        );
        let next = report(
            &["p"],
            vec![project(
                "p/apps/app",
                "D:/p/apps/app",
                Some("git@h:me/app.git"),
            )],
        );
        let changes = diff(Some(&prev), &next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, "moved");
        assert_eq!(changes[0].from.as_deref(), Some("p/web/app"));
    }

    #[test]
    fn same_name_different_remote_is_not_a_move() {
        let prev = report(
            &["p"],
            vec![project("p/web/app", "D:/a", Some("git@h:me/a.git"))],
        );
        let next = report(
            &["p"],
            vec![project("p/apps/app", "D:/b", Some("git@h:me/b.git"))],
        );
        let changes = diff(Some(&prev), &next);
        let kinds: Vec<&str> = changes.iter().map(|c| c.kind.as_str()).collect();
        assert_eq!(kinds, vec!["added", "removed"]);
    }

    #[test]
    fn an_unplugged_root_reports_nothing() {
        let prev = report(
            &["p", "usb"],
            vec![
                project("p/web/app", "D:/p/web/app", None),
                project("usb/old/thing", "E:/old/thing", None),
            ],
        );
        // Second run: E: is not mounted, so `usb` was never walked.
        let next = report(&["p"], vec![project("p/web/app", "D:/p/web/app", None)]);
        assert!(
            diff(Some(&prev), &next).is_empty(),
            "an unplugged drive must not read as deleted projects"
        );
    }

    #[test]
    fn a_folder_rename_above_the_project_is_a_move() {
        let prev = report(&["p"], vec![project("p/web/app", "D:/p/web/app", None)]);
        let next = report(&["p"], vec![project("p/web/app", "E:/p/web/app", None)]);
        let changes = diff(Some(&prev), &next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind, "moved");
        assert_eq!(changes[0].detail.as_deref(), Some("D:/p/web/app"));
    }
}
