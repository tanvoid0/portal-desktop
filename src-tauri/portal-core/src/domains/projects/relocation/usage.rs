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
            let refusal = kill_refusal(&sys, pid);
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

/// Builds whatever [`refuse_kill`] needs from a live `sys` snapshot and
/// `pid`, then runs it — the single source of truth both the UI hint
/// (`check_port_warnings`'s `killable`) and the actual kill enforcement
/// (`end_port_process`, in the Tauri command layer / a different crate) call
/// through. `None` means `pid` may be ended; `Some(reason)` is why not.
pub fn kill_refusal(sys: &System, pid: u32) -> Option<String> {
    let proc = sys.process(sysinfo::Pid::from(pid as usize))?;
    let self_chain = self_and_ancestors(sys);
    let self_uid = sysinfo::get_current_pid()
        .ok()
        .and_then(|p| sys.process(p))
        .and_then(|p| p.user_id())
        .cloned();
    let windir = std::env::var("WINDIR").or_else(|_| std::env::var("SystemRoot")).ok();
    let process_name = proc.name().to_string();
    let exe = proc.exe().map(|e| e.to_string_lossy().into_owned());
    // Fail closed: either uid unreadable (common for SYSTEM/elevated/protected
    // processes on Windows, exactly when the check matters most) now counts
    // as "different/unverifiable", not "assume same user".
    let different_user = !matches!(
        (proc.user_id(), self_uid.as_ref()),
        (Some(a), Some(b)) if a == b
    );
    refuse_kill(pid, &process_name, exe.as_deref(), &self_chain, different_user, windir.as_deref())
}

/// Docker's own port-forwarder family — on Linux the full name can also show
/// up truncated to 15 chars (`comm` via `/proc`), so this is a prefix check
/// against the basename, not an exact match.
const DOCKER_FORWARDER_PREFIX: &str = "com.docker.";
/// Other known forwarders/relays that proxy a port without being the real
/// server behind it.
const OTHER_FORWARDERS: [&str; 3] = ["wslrelay", "wslhost", "vpnkit"];

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
        return Some(
            "this process runs as a different or unverifiable OS user — ending it needs that user's permissions"
                .into(),
        );
    }
    // Fail closed: an unreadable exe means the Windows-binary rule below
    // can't fire either way, so an unknown executable is refused rather than
    // silently waved through.
    let Some(exe) = exe else {
        return Some("this process's executable path couldn't be read — refusing to end it unverified".into());
    };
    if let Some(windir) = windir {
        if is_under(exe, windir) {
            return Some("this is a core Windows system binary and shouldn't be ended".into());
        }
    }
    // Match the forwarder lists against the exe's basename, not the
    // (possibly OS-truncated) process name — over-matching here just refuses
    // a kill that might have been fine, which is the safe direction.
    let basename = exe.rsplit(['/', '\\']).next().unwrap_or(exe).to_ascii_lowercase();
    if basename.starts_with(DOCKER_FORWARDER_PREFIX) || OTHER_FORWARDERS.iter().any(|f| basename.contains(f)) {
        return Some(format!(
            "{name} only forwards this port — ending it won't free the real server behind it"
        ));
    }
    None
}

/// Every pid `LISTENING` on `port`, via `netstat -ano`. Usually one, but
/// listed per-line so [`owning_pid`] and the kill path (`end_port_process`,
/// a different crate) agree on how multiple owners are handled rather than
/// each guessing independently.
#[cfg(target_os = "windows")]
pub fn owning_pids(port: u16) -> Vec<u32> {
    let Ok(out) = Command::new("netstat").no_window().args(["-ano"]).output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let needle = format!(":{port} ");
    let mut seen = std::collections::HashSet::new();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.contains("LISTENING") && l.contains(&needle))
        .filter_map(|l| l.split_whitespace().last())
        .filter_map(|pid| pid.parse::<u32>().ok())
        .filter(|pid| seen.insert(*pid))
        .collect()
}

/// Same, via `lsof` (present on both macOS and Linux) — `-t` prints just the
/// bare pid, one per line, and a pre-fork server (gunicorn/uvicorn --reload)
/// genuinely has several worker pids sharing the inherited listening socket,
/// so all of them are returned rather than just the first.
#[cfg(not(target_os = "windows"))]
pub fn owning_pids(port: u16) -> Vec<u32> {
    let Ok(out) = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse::<u32>().ok())
        .collect()
}

/// First (or only) pid [`owning_pids`] finds — the display-only case
/// (`check_port_warnings`'s single `PortWarning` per port) doesn't need the
/// full set, the kill path does.
pub fn owning_pid(port: u16) -> Option<u32> {
    owning_pids(port).into_iter().next()
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
        let exe = Some("D:\\projects\\foo\\whatever.exe");
        assert!(refuse_kill(42, "whatever.exe", exe, &[1, 42, 99], false, None).is_some());
        assert!(refuse_kill(43, "whatever.exe", exe, &[1, 42, 99], false, None).is_none());
    }

    #[test]
    fn refuse_kill_blocks_different_user() {
        assert!(refuse_kill(42, "whatever.exe", None, &[], true, None).is_some());
    }

    #[test]
    fn refuse_kill_blocks_unknown_exe() {
        // Fail closed: an unreadable exe is refused, not waved through.
        assert!(refuse_kill(42, "node.exe", None, &[], false, None).is_some());
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
        assert!(refuse_kill(42, "wslrelay.exe", Some("C:\\x\\wslrelay.exe"), &[], false, None).is_some());
        assert!(refuse_kill(42, "com.docker.backend.exe", Some("/usr/bin/com.docker.backend"), &[], false, None).is_some());
        assert!(refuse_kill(42, "vpnkit.exe", Some("C:\\x\\vpnkit.exe"), &[], false, None).is_some());
        assert!(refuse_kill(42, "node.exe", Some("D:\\projects\\foo\\node.exe"), &[], false, None).is_none());
    }

    #[test]
    fn refuse_kill_matches_forwarder_by_exe_basename_despite_truncated_name() {
        // Linux's `/proc/<pid>/comm` truncates to 15 chars, so the process
        // name alone can't carry "com.docker.backend" — the exe basename
        // (untruncated) must still catch it.
        let reason = refuse_kill(42, "com.docker.back", Some("/usr/bin/com.docker.backend"), &[], false, None);
        assert!(reason.is_some());
    }

    #[test]
    fn kill_refusal_blocks_self() {
        let mut sys = System::new_all();
        sys.refresh_processes();
        assert!(kill_refusal(&sys, std::process::id()).is_some());
    }
}
