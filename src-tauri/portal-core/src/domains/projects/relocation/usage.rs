//! Everything that is, or might be, using a project's path right now — the
//! gate the move engine (step 4+ of the "safe project relocation" plan) runs
//! before it starts. A blocker stops the move outright; a warning is shown
//! but doesn't.
//!
//! This module only reads state. Nothing here locks a path (see
//! `super::move_lock`) or touches the filesystem beyond reading it.

use crate::database::DatabaseManager;
use crate::domains::coder::CoderService;
use crate::domains::deployments::repositories::deployment_repository::DeploymentRepository;
use crate::domains::deployments::services::docker_service::DeploymentStatus;
use crate::domains::projects::move_lock::is_under;
use crate::domains::projects::pipelines::services::ExecutionService;
use crate::domains::scripts::ScriptExecutionService;
use crate::domains::terminal::manager::TerminalManager;
use crate::entities::project_move;
use crate::process_ext::NoWindowExt;
use sea_orm::EntityTrait;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

/// One reported signal. `kind` is a short machine tag the UI maps to
/// copy/icon; `detail` is the human-readable reason.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Signal {
    pub kind: String,
    pub detail: String,
}

fn signal(kind: &str, detail: impl Into<String>) -> Signal {
    Signal {
        kind: kind.to_string(),
        detail: detail.into(),
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct UsageReport {
    pub blockers: Vec<Signal>,
    pub warnings: Vec<Signal>,
}

/// Run every check and combine them into one report. `dev_port`/`prod_port`
/// come from the project row — this module doesn't load it itself so it
/// doesn't have to guess which repository call does that.
#[allow(clippy::too_many_arguments)]
pub async fn check(
    source: &Path,
    dest: &Path,
    db: &DatabaseManager,
    coder: &CoderService,
    terminal: &TerminalManager,
    pipelines: &ExecutionService,
    scripts: &ScriptExecutionService,
    deployments: &DeploymentRepository,
    dev_port: Option<u16>,
    prod_port: Option<u16>,
) -> UsageReport {
    let path = source.to_string_lossy().into_owned();

    let mut blockers = check_preconditions(db, source, dest).await;
    blockers.extend(check_portal_usage(&path, coder, terminal, pipelines, scripts, deployments).await);
    blockers.extend(check_os_processes(&path));

    let warnings = check_port_warnings(&path, dev_port, prod_port);

    UsageReport { blockers, warnings }
}

/// Hard preconditions: paths and filesystem state, nothing about who's using
/// them. All blockers — there's no "warn and proceed" version of these.
pub async fn check_preconditions(db: &DatabaseManager, source: &Path, dest: &Path) -> Vec<Signal> {
    let mut blockers = Vec::new();

    if dest.exists() {
        blockers.push(signal(
            "dest_exists",
            format!("{} already exists", dest.display()),
        ));
    }

    match dest.parent() {
        Some(parent) if parent.exists() => {}
        _ => blockers.push(signal(
            "dest_parent_missing",
            format!("{}'s parent directory doesn't exist", dest.display()),
        )),
    }

    let (src_s, dst_s) = (source.to_string_lossy(), dest.to_string_lossy());
    if is_under(&src_s, &dst_s) {
        blockers.push(signal(
            "nested_paths",
            "Source and destination are nested inside each other",
        ));
    }

    let links = find_symlinks(source);
    if !links.is_empty() {
        blockers.push(signal(
            "symlinks",
            format!(
                "Source contains symlinks/junctions, which the copy can't follow safely: {}",
                links
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }

    if let Some(s) = check_free_space(source, dest) {
        blockers.push(s);
    }

    if let Some(s) = check_overlapping_moves(db, source, dest).await {
        blockers.push(s);
    }

    blockers
}

/// Any symlink or junction under `root` — both show up as `is_symlink()` on
/// Windows (a junction is a reparse point, same as a symlink).
fn find_symlinks(root: &Path) -> Vec<PathBuf> {
    jwalk::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_symlink())
        .map(|e| e.path())
        .collect()
}

fn check_free_space(source: &Path, dest: &Path) -> Option<Signal> {
    let total: u64 = jwalk::WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum();
    let needed = (total as f64 * 1.1) as u64;

    let dest_s = dest.to_string_lossy();
    let disks = sysinfo::Disks::new_with_refreshed_list();
    // Most specific (longest) matching mount point wins.
    let avail = disks
        .iter()
        .filter(|d| is_under(&d.mount_point().to_string_lossy(), &dest_s))
        .max_by_key(|d| d.mount_point().to_string_lossy().len())
        .map(|d| d.available_space())?;

    if avail < needed {
        Some(signal(
            "low_space",
            format!("Destination has {avail} bytes free, need ~{needed} (1.1x the {total}-byte source)"),
        ))
    } else {
        None
    }
}

/// Any other `project_moves` row, still in flight, whose source or dest
/// overlaps either path in this move.
async fn check_overlapping_moves(db: &DatabaseManager, source: &Path, dest: &Path) -> Option<Signal> {
    let rows = project_move::Entity::find()
        .all(db.get_connection())
        .await
        .ok()?;
    let (src_s, dst_s) = (source.to_string_lossy(), dest.to_string_lossy());
    let hit = rows.into_iter().find(|r| {
        !matches!(r.state.as_str(), "done" | "aborted" | "failed")
            && (is_under(&r.source_path, &src_s)
                || is_under(&r.source_path, &dst_s)
                || is_under(&r.dest_path, &src_s)
                || is_under(&r.dest_path, &dst_s))
    })?;
    Some(signal(
        "move_in_progress",
        format!("Move {} is already in progress for an overlapping path", hit.id),
    ))
}

/// Everything Portal itself knows is touching `path` right now.
async fn check_portal_usage(
    path: &str,
    coder: &CoderService,
    terminal: &TerminalManager,
    pipelines: &ExecutionService,
    scripts: &ScriptExecutionService,
    deployments: &DeploymentRepository,
) -> Vec<Signal> {
    let mut blockers = Vec::new();

    if coder.uses_under(path).await {
        blockers.push(signal(
            "coder",
            "An agent run, pending command, or sub-agent is active in this project",
        ));
    }

    if let Ok(processes) = terminal.get_all_processes().await {
        let hit = processes.iter().any(|p| {
            matches!(p.status.as_str(), "starting" | "running") && is_under(&p.working_directory, path)
        });
        if hit {
            blockers.push(signal("terminal", "A terminal process is running in this project"));
        }
    }

    if pipelines.running_under(path) {
        blockers.push(signal("pipeline", "A pipeline is running in this project"));
    }

    if scripts.running_under(path) {
        blockers.push(signal("script", "A script is running in this project"));
    }

    if let Ok(all) = deployments.find_all().await {
        let hit = all.iter().any(|d| {
            matches!(d.status, DeploymentStatus::Running)
                && d.process_id.is_some()
                && d
                    .working_directory
                    .as_deref()
                    .is_some_and(|wd| is_under(wd, path))
        });
        if hit {
            blockers.push(signal("deployment", "A deployment is live in this project"));
        }
    }

    if worktree_count(Path::new(path)).unwrap_or(0) > 1 {
        blockers.push(signal(
            "worktree",
            "More than one git worktree is checked out — a multitask sub-agent is likely still working",
        ));
    }

    blockers
}

/// `git worktree list` entry count, or `None` if `path` isn't a git repo or
/// git isn't on PATH.
fn worktree_count(path: &Path) -> Option<usize> {
    let mut cmd = Command::new("git");
    cmd.current_dir(path).no_window();
    cmd.args(["worktree", "list"]);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count(),
    )
}

/// Any OS process — tracked by Portal or not — with a cwd or exe under
/// `path`.
fn check_os_processes(path: &str) -> Vec<Signal> {
    let mut sys = System::new_all();
    sys.refresh_processes();
    sys.processes()
        .values()
        .filter(|p| {
            p.cwd().is_some_and(|c| is_under(&c.to_string_lossy(), path))
                || p.exe().is_some_and(|e| is_under(&e.to_string_lossy(), path))
        })
        .map(|p| {
            signal(
                "process",
                format!("{} (pid {}) is running from this project", p.name(), p.pid()),
            )
        })
        .collect()
}

/// Point-in-time "is anything already holding a file open for write" check —
/// independent of, and much weaker than, the persistent share-deny hold a
/// later step (#217) takes during the actual copy.
// TODO(#217): a real probe needs `CreateFileW` with a deny-write share mode,
// which means a `windows`/`winapi` dependency this repo doesn't have. Rather
// than guess at that API, this stays a stub: always "unknown", never blocks.
#[allow(dead_code)]
fn check_open_for_write(_path: &Path) -> Option<Signal> {
    None
}

/// Warning (not blocker): something is listening on `dev_port`/`prod_port`
/// but its owner isn't running from this project. A later step offers to end
/// it; this just surfaces the pid + name.
fn check_port_warnings(path: &str, dev_port: Option<u16>, prod_port: Option<u16>) -> Vec<Signal> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    [dev_port, prod_port]
        .into_iter()
        .flatten()
        .filter(|port| std::net::TcpListener::bind(("127.0.0.1", *port)).is_err())
        .filter_map(|port| owning_pid(port).map(|pid| (port, pid)))
        .filter_map(|(port, pid)| {
            let proc = sys.process(sysinfo::Pid::from(pid as usize))?;
            let under = proc.cwd().is_some_and(|c| is_under(&c.to_string_lossy(), path))
                || proc.exe().is_some_and(|e| is_under(&e.to_string_lossy(), path));
            if under {
                return None;
            }
            let name = proc.name().to_string();
            Some(signal(
                "port_owner_outside_project",
                format!("Port {port} is in use by {name} (pid {pid}), which isn't running from this project"),
            ))
        })
        .collect()
}

/// PID of whatever is `LISTENING` on `port`, via `netstat -ano`. sysinfo has
/// no per-socket API, so this is the pragmatic way to get from a port to a
/// process on Windows.
#[cfg(target_os = "windows")]
fn owning_pid(port: u16) -> Option<u32> {
    let out = Command::new("netstat").no_window().args(["-ano"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let needle = format!(":{port} ");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|l| l.contains("LISTENING") && l.contains(&needle))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|pid| pid.parse::<u32>().ok())
}

#[cfg(not(target_os = "windows"))]
fn owning_pid(_port: u16) -> Option<u32> {
    None // ponytail: netstat-based lookup is Windows-only for now; add a /proc or lsof path if this ships cross-platform.
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn find_symlinks_empty_for_plain_dir() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hi").unwrap();
        assert!(find_symlinks(dir.path()).is_empty());
    }

    #[test]
    fn worktree_count_none_outside_a_repo() {
        let dir = tempdir().unwrap();
        assert_eq!(worktree_count(dir.path()), None);
    }

    #[test]
    fn open_for_write_stub_never_reports() {
        assert!(check_open_for_write(Path::new("C:\\whatever")).is_none());
    }
}
