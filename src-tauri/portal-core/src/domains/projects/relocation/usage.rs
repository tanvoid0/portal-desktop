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
use ts_rs::TS;

/// One reported signal. `kind` is a short machine tag the UI maps to
/// copy/icon; `detail` is the human-readable reason.
#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
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

/// A port conflict warning: who owns the port, and enough to let the UI offer
/// "end it and continue". `killable` runs the real refusal rules (see
/// [`refuse_kill`]) — but it's still only a recommendation: the actual guard
/// before ending anything is `end_port_process`'s own pid + `start_time` +
/// port recheck, never this report.
#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct PortWarning {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    #[ts(type = "number")]
    pub start_time: u64,
    pub killable: bool,
    pub reason: String,
    pub exe: Option<String>,
    pub cmdline: Vec<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct UsageReport {
    pub blockers: Vec<Signal>,
    pub warnings: Vec<PortWarning>,
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

/// The in-use checks only — Portal's own usage plus OS processes under `path` —
/// without the start-only preconditions ([`check_preconditions`]).
///
/// Resume and source-removal re-check usage on a path that is already mid-move,
/// where the preconditions would wrongly fire: `dest_exists` (the destination
/// is supposed to exist by then), the move's own row counted as an overlapping
/// move, and free space measured against a volume that now holds the copy.
pub async fn check_in_use(
    path: &Path,
    coder: &CoderService,
    terminal: &TerminalManager,
    pipelines: &ExecutionService,
    scripts: &ScriptExecutionService,
    deployments: &DeploymentRepository,
) -> Vec<Signal> {
    let p = path.to_string_lossy().into_owned();
    let mut blockers = check_portal_usage(&p, coder, terminal, pipelines, scripts, deployments).await;
    blockers.extend(check_os_processes(&p));
    blockers
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

/// `source_path` of every move not yet done/aborted/failed. Discovery skips
/// anything under these. A query error yields none — discovery must not break.
pub async fn in_flight_move_sources(db: &DatabaseManager) -> Vec<String> {
    project_move::Entity::find()
        .all(db.get_connection())
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|r| !matches!(r.state.as_str(), "done" | "aborted" | "failed"))
        .map(|r| r.source_path)
        .collect()
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
/// it; this surfaces the pid + name plus enough (`exe`/`cmdline`/`cwd`) for
/// the UI to show what it'd actually be ending, and runs [`refuse_kill`] to
/// decide `killable` up front.
fn check_port_warnings(path: &str, dev_port: Option<u16>, prod_port: Option<u16>) -> Vec<PortWarning> {
    let mut sys = System::new_all();
    sys.refresh_processes();
    let self_chain = self_and_ancestors(&sys);
    let self_uid = sysinfo::get_current_pid()
        .ok()
        .and_then(|pid| sys.process(pid))
        .and_then(|p| p.user_id())
        .cloned();
    let windir = std::env::var("WINDIR").or_else(|_| std::env::var("SystemRoot")).ok();

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
            let process_name = proc.name().to_string();
            let exe = proc.exe().map(|e| e.to_string_lossy().into_owned());
            // ponytail: different-user detection only fires when sysinfo can read
            // both uids; it can't on every pid (e.g. protected processes), and a
            // miss here just leaves the check silently not blocking — a real gap,
            // not faked as a pass.
            let different_user = matches!(
                (proc.user_id(), self_uid.as_ref()),
                (Some(a), Some(b)) if a != b
            );
            let refusal =
                refuse_kill(pid, &process_name, exe.as_deref(), &self_chain, different_user, windir.as_deref());
            let reason = refusal.clone().unwrap_or_else(|| {
                format!("Port {port} is in use by {process_name} (pid {pid}), which isn't running from this project")
            });
            Some(PortWarning {
                port,
                pid,
                process_name,
                start_time: proc.start_time(),
                killable: refusal.is_none(),
                reason,
                exe,
                cmdline: proc.cmd().to_vec(),
                cwd: proc.cwd().map(|c| c.to_string_lossy().into_owned()),
            })
        })
        .collect()
}

/// This process's own pid plus every ancestor's, by walking `parent()` in
/// `sys`. Capped so a pid-reuse cycle can't loop forever.
fn self_and_ancestors(sys: &System) -> Vec<u32> {
    let mut chain = Vec::new();
    let Ok(mut pid) = sysinfo::get_current_pid() else {
        return chain;
    };
    for _ in 0..64 {
        chain.push(pid.as_u32());
        match sys.process(pid).and_then(|p| p.parent()) {
            Some(parent) if !chain.contains(&parent.as_u32()) => pid = parent,
            _ => break,
        }
    }
    chain
}

/// Port-forwarders that proxy a port without being the real server behind
/// it — ending one of these doesn't free the port's actual owner, it just
/// kills the relay and confuses the user. Matched by name substring,
/// case-insensitive, since the on-disk name varies (`wslrelay.exe` etc).
const PORT_FORWARDERS: [&str; 3] = ["com.docker.backend", "wslrelay", "wslhost"];

/// Decides whether `end_port_process` should ever be allowed to try `pid`.
/// Pure on plain data (no sysinfo lookups) so it's unit-testable; `None`
/// means killable, `Some(reason)` is why not.
fn refuse_kill(
    pid: u32,
    name: &str,
    exe: Option<&str>,
    self_and_ancestors: &[u32],
    different_user: bool,
    windir: Option<&str>,
) -> Option<String> {
    if pid == 0 || pid == 4 {
        return Some("this is a core Windows system process and can never be ended".into());
    }
    if self_and_ancestors.contains(&pid) {
        return Some(
            "this is Portal itself, or a process that started it — ending it would take Portal down too".into(),
        );
    }
    if different_user {
        return Some("this process runs as a different OS user — ending it needs that user's permissions".into());
    }
    if let (Some(exe), Some(windir)) = (exe, windir) {
        if is_under(exe, windir) {
            return Some("this is a core Windows system binary and shouldn't be ended".into());
        }
    }
    let lower = name.to_ascii_lowercase();
    if PORT_FORWARDERS.iter().any(|f| lower.contains(f)) {
        return Some(format!(
            "{name} only forwards this port — ending it won't free the real server behind it"
        ));
    }
    None
}

/// PID of whatever is `LISTENING` on `port`, via `netstat -ano`. sysinfo has
/// no per-socket API, so this is the pragmatic way to get from a port to a
/// process on Windows. `pub`: also the recheck `end_port_process` (the Tauri
/// command layer, a different crate) uses before it kills anything.
#[cfg(target_os = "windows")]
pub fn owning_pid(port: u16) -> Option<u32> {
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

/// Same, via `lsof` (present on both macOS and Linux) — `-t` prints just the
/// bare pid, one per line if several processes share the listening socket;
/// the first is as good a guess as any.
#[cfg(not(target_os = "windows"))]
pub fn owning_pid(port: u16) -> Option<u32> {
    let out = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .and_then(|l| l.trim().parse::<u32>().ok())
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

    #[test]
    fn refuse_kill_blocks_system_pids() {
        assert!(refuse_kill(0, "System Idle Process", None, &[], false, None).is_some());
        assert!(refuse_kill(4, "System", None, &[], false, None).is_some());
    }

    #[test]
    fn refuse_kill_blocks_self_and_ancestors() {
        assert!(refuse_kill(42, "whatever.exe", None, &[1, 42, 99], false, None).is_some());
        assert!(refuse_kill(43, "whatever.exe", None, &[1, 42, 99], false, None).is_none());
    }

    #[test]
    fn refuse_kill_blocks_different_user() {
        assert!(refuse_kill(42, "whatever.exe", None, &[], true, None).is_some());
    }

    #[test]
    fn refuse_kill_blocks_windows_system_binaries() {
        let reason = refuse_kill(42, "svchost.exe", Some("C:\\Windows\\System32\\svchost.exe"), &[], false, Some("C:\\Windows"));
        assert!(reason.is_some());
        let ok = refuse_kill(42, "node.exe", Some("D:\\projects\\foo\\node.exe"), &[], false, Some("C:\\Windows"));
        assert!(ok.is_none());
    }

    #[test]
    fn refuse_kill_blocks_known_forwarders() {
        assert!(refuse_kill(42, "wslrelay.exe", None, &[], false, None).is_some());
        assert!(refuse_kill(42, "com.docker.backend.exe", None, &[], false, None).is_some());
        assert!(refuse_kill(42, "node.exe", None, &[], false, None).is_none());
    }
}
