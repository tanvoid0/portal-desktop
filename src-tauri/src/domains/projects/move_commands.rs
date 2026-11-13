//! Tauri command surface over the project-relocation engine
//! (`relocation::mover`/`relocation::usage`). Step 7 of the "safe project
//! relocation" plan.
//!
//! Every command here is `#[rpc(local_only)]`: a filesystem move, a recycle,
//! or a process kill is squarely a this-machine action, and none of it is
//! something a LAN peer talking to the daemon should ever be able to trigger
//! — there's no dialog, no user sitting at this keyboard, to have approved it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sea_orm::EntityTrait;
use tauri::{Manager, State};

use crate::database::DatabaseManager;
use crate::domains::coder::CoderService;
use crate::domains::deployments::repositories::deployment_repository::DeploymentRepository;
use crate::domains::deployments::services::deployment_service::DeploymentService;
use crate::domains::projects::pipelines::repositories::pipeline_repository::PipelineRepository;
use crate::domains::projects::pipelines::services::ExecutionService;
use crate::domains::scripts::commands::ScriptExecutionState;
use crate::domains::terminal::manager::TerminalManager;
use crate::entities::{project as project_entity, project_move};
use crate::events::{EventSinkExt, Events};
use crate::process_ext::NoWindowExt;
use portal_core::domains::projects::move_lock;
use portal_core::domains::projects::relocation::types::{
    MoveSummary, NestedProject, PreflightResult, SkippedDir,
};
use portal_core::domains::projects::relocation::{mover, usage};

const EMIT_EVERY_MS: u128 = 120;

/// Per-move "cancel requested" flags. Mirrors `DiscoveryControl`'s shape, but
/// keyed by move id since several moves can be in flight at once.
///
/// ponytail: best-effort only. The engine's copy/verify loop
/// (`mover::run_move`) has no cancellation hook — its `ProgressFn` is a plain
/// `FnMut(Progress)`, nothing to check or return — so setting this flag does
/// not interrupt a copy already running. It only makes `list_project_moves`
/// report the request and makes `resolve_interrupted_move`'s `Resume` refuse.
/// A real mid-copy abort needs a cooperative check threaded into
/// `run_move`'s loop (a new `MoveError::Cancelled`); flagged here for
/// whoever builds the move UI rather than guessed at.
#[derive(Default)]
pub struct MoveControl {
    pub cancelled: Mutex<HashSet<String>>,
    /// Project ids with a move command (`start_project_move`, or
    /// `resolve_interrupted_move`'s dispatch) actively awaiting right now —
    /// not just journaled as `copying`/`verifying`. Guarded by [`RunningGuard`]
    /// so every return path (success, error, early `?`) clears it. Without
    /// this, the interrupted-move banner and `resolve_interrupted_move`
    /// can't tell a live move from one a crash actually left mid-flight
    /// (board #207/#208 review).
    pub running: Mutex<HashSet<i32>>,
}

/// RAII marker: `project_id` counts as "running" (see [`MoveControl::running`])
/// for as long as this guard lives, cleared on drop so a failed/short-circuited
/// command never wedges a project as permanently running.
struct RunningGuard<'a> {
    running: &'a Mutex<HashSet<i32>>,
    project_id: i32,
}

impl<'a> RunningGuard<'a> {
    fn new(running: &'a Mutex<HashSet<i32>>, project_id: i32) -> Self {
        running.lock().unwrap().insert(project_id);
        Self { running, project_id }
    }
}

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        self.running.lock().unwrap().remove(&self.project_id);
    }
}

/// Everything a move audit needs before it starts, and nothing it needs
/// while running — the preflight walk/usage check is read-only. `local_only`:
/// it walks and sizes an arbitrary local directory tree, which a remote peer
/// has no business doing.
#[portal_rpc_macro::rpc(local_only)]
pub async fn preflight_project_move(
    id: i32,
    dest: String,
    include_regenerable: bool,
    db_manager: State<'_, Arc<DatabaseManager>>,
    coder: State<'_, Arc<CoderService>>,
    terminal: State<'_, TerminalManager>,
    pipelines: State<'_, Arc<ExecutionService>>,
    execution_state: State<'_, ScriptExecutionState>,
) -> Result<PreflightResult, String> {
    let project = load_project(&db_manager, id).await?;
    let source = PathBuf::from(&project.path);
    let dest_path = PathBuf::from(&dest);
    let scripts = execution_state.get_or_init(db_manager.inner()).await;
    let deployments = DeploymentRepository::new(db_manager.inner().clone());

    let report = usage::check(
        &source,
        &dest_path,
        db_manager.inner(),
        coder.inner(),
        terminal.inner(),
        pipelines.inner(),
        &scripts,
        &deployments,
        port_u16(project.dev_port),
        port_u16(project.prod_port),
    )
    .await;

    let walk = mover::preflight_walk(&source, include_regenerable)?;
    let skipped_dirs = walk
        .skipped
        .into_iter()
        .map(|(rel, bytes)| SkippedDir { rel, bytes: bytes as i64 })
        .collect();
    let nested_projects = nested_under(&db_manager, &source, id).await?;

    Ok(PreflightResult {
        blockers: report.blockers,
        warnings: report.warnings,
        skipped_dirs,
        file_count: walk.file_count as i64,
        total_bytes: walk.total_bytes as i64,
        available_bytes: available_space(&dest_path).unwrap_or(0) as i64,
        nested_projects,
    })
}

/// Starts a fresh relocation copy, streaming its progress on
/// `project-move://progress`. `local_only`: moves bytes on disk under a path
/// the caller doesn't get to pick remotely.
#[portal_rpc_macro::rpc(local_only)]
pub async fn start_project_move(
    id: i32,
    dest: String,
    include_regenerable: bool,
    acknowledged_warnings: bool,
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DatabaseManager>>,
    coder: State<'_, Arc<CoderService>>,
    terminal: State<'_, TerminalManager>,
    pipelines: State<'_, Arc<ExecutionService>>,
    execution_state: State<'_, ScriptExecutionState>,
    events: State<'_, Events>,
    control: State<'_, MoveControl>,
    deployment_service: State<'_, Arc<DeploymentService>>,
) -> Result<MoveSummary, String> {
    let project = load_project(&db_manager, id).await?;
    if control.running.lock().unwrap().contains(&project.id) {
        return Err("a move for this project is already running".into());
    }
    let _running = RunningGuard::new(&control.running, project.id);
    let scripts = execution_state.get_or_init(db_manager.inner()).await;
    let deployments = DeploymentRepository::new(db_manager.inner().clone());
    let pipelines_repo = PipelineRepository::new(db_manager.inner().clone());
    let mv = mover::Mover {
        db: db_manager.inner(),
        coder: coder.inner(),
        terminal: terminal.inner(),
        pipelines: pipelines.inner(),
        scripts: &scripts,
        deployments: &deployments,
        deployment_service: deployment_service.inner(),
        pipelines_repo: &pipelines_repo,
        moves_dir: moves_dir(&app)?,
    };
    let mut emit = progress_emitter(&events);
    // board #419: breadcrumb the long synchronous copy. If the process dies
    // silently mid-move (no panic, no handled error), this "beginning" line is
    // the last thing in the tauri-cli output and pins the abort to the copy.
    crate::log_info!(
        "Moves",
        "start_project_move: beginning copy of project {} ({}) -> {}",
        project.id,
        project.path,
        dest
    );
    let row = mv
        .start_move(&project, Path::new(&dest), include_regenerable, acknowledged_warnings, &mut emit)
        .await
        .map_err(describe_move_error)?;
    crate::log_info!(
        "Moves",
        "start_project_move: finished project {} (move {} state={})",
        project.id,
        row.id,
        row.state
    );
    Ok(MoveSummary::from_row(row, false, false))
}

/// Flags a move as cancel-requested. See [`MoveControl`] for why this is
/// best-effort rather than a real abort. `local_only`: in-process state for a
/// move this app started.
#[portal_rpc_macro::rpc(local_only)]
pub async fn cancel_project_move(id: String, control: State<'_, MoveControl>) -> Result<(), String> {
    control.cancelled.lock().unwrap().insert(id);
    Ok(())
}

/// The move journal. `local_only`: paths on this machine's disk.
#[portal_rpc_macro::rpc(local_only)]
pub async fn list_project_moves(
    db_manager: State<'_, Arc<DatabaseManager>>,
    control: State<'_, MoveControl>,
) -> Result<Vec<MoveSummary>, String> {
    let rows = project_move::Entity::find()
        .all(db_manager.get_connection())
        .await
        .map_err(|e| format!("load moves: {e}"))?;
    let cancelled = control.cancelled.lock().unwrap();
    let running = control.running.lock().unwrap();
    Ok(rows
        .into_iter()
        .map(|r| {
            let cancel_requested = cancelled.contains(&r.id);
            let is_running = running.contains(&r.project_id);
            MoveSummary::from_row(r, cancel_requested, is_running)
        })
        .collect())
}

/// Dispatches the user's recovery choice for an interrupted move to the
/// matching `Mover` method. `resume` always reuses the move's own journaled
/// `include_regenerable` — the engine reads it off the row, nothing here
/// overrides it. `local_only`: resumes/finishes/rolls back a copy on this
/// machine's disk.
#[portal_rpc_macro::rpc(local_only)]
pub async fn resolve_interrupted_move(
    id: String,
    action: mover::RecoveryAction,
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DatabaseManager>>,
    coder: State<'_, Arc<CoderService>>,
    terminal: State<'_, TerminalManager>,
    pipelines: State<'_, Arc<ExecutionService>>,
    execution_state: State<'_, ScriptExecutionState>,
    events: State<'_, Events>,
    control: State<'_, MoveControl>,
    deployment_service: State<'_, Arc<DeploymentService>>,
) -> Result<Option<MoveSummary>, String> {
    if action == mover::RecoveryAction::Resume && control.cancelled.lock().unwrap().contains(&id) {
        return Err("this move was cancelled — roll back instead of resuming".into());
    }
    let project_id = project_move::Entity::find_by_id(&id)
        .one(db_manager.get_connection())
        .await
        .map_err(|e| format!("load move {id}: {e}"))?
        .ok_or_else(|| format!("no move with id {id}"))?
        .project_id;
    if control.running.lock().unwrap().contains(&project_id) {
        return Err("a move for this project is currently running — wait for it to finish".into());
    }
    let _running = RunningGuard::new(&control.running, project_id);
    let scripts = execution_state.get_or_init(db_manager.inner()).await;
    let deployments = DeploymentRepository::new(db_manager.inner().clone());
    let pipelines_repo = PipelineRepository::new(db_manager.inner().clone());
    let mv = mover::Mover {
        db: db_manager.inner(),
        coder: coder.inner(),
        terminal: terminal.inner(),
        pipelines: pipelines.inner(),
        scripts: &scripts,
        deployments: &deployments,
        deployment_service: deployment_service.inner(),
        pipelines_repo: &pipelines_repo,
        moves_dir: moves_dir(&app)?,
    };
    let mut emit = progress_emitter(&events);
    let row = mv
        .resolve_interrupted_move(&id, action, &mut emit)
        .await
        .map_err(describe_move_error)?;
    control.cancelled.lock().unwrap().remove(&id);
    Ok(row.map(|r| MoveSummary::from_row(r, false, false)))
}

/// Ends whatever's bound to `port`, after re-verifying `pid` is still the
/// same process `start_time` recorded (the classic PID-reuse guard), is
/// still among the pids actually listening on `port` — the pid a caller
/// holds could be stale by the time they act on it — and passes
/// [`usage::kill_refusal`], the same refusal rules that produced
/// `PortWarning.killable`, now actually enforced rather than just advised.
/// Kills every pid `usage::owning_pids` reports for `port`, not just `pid`:
/// a pre-fork server (gunicorn/uvicorn --reload) shares its listening
/// socket across a master + worker pids, and ending only one would leave
/// the port held. Via `taskkill /T /F` (Windows) or `kill -9` (unix) per
/// pid, mirroring `kill_process_tree` in
/// `pipelines::services::execution_service`. Then polls up to 3s for the
/// port to actually free, so the caller's next preflight (it always re-runs
/// one after this resolves) sees settled state instead of racing the OS's
/// own cleanup. `local_only`: killing an arbitrary local process by pid is
/// squarely a this-machine action.
#[portal_rpc_macro::rpc(local_only)]
pub async fn end_port_process(port: u16, pid: u32, start_time: u64) -> Result<(), String> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes();
    verify_pid(&sys, pid, start_time)?;

    let owners = usage::owning_pids(port);
    if !owners.contains(&pid) {
        return Err(format!("pid {pid} is no longer the process listening on port {port}"));
    }

    // Targeted re-check right before the kill, minimizing the gap between
    // the snapshot above and the actual kill (vs. reusing a `System` that
    // could be hundreds of ms stale by now).
    sys.refresh_process(sysinfo::Pid::from(pid as usize));
    verify_pid(&sys, pid, start_time)?;

    for &owner in &owners {
        if let Some(reason) = usage::kill_refusal(&sys, owner) {
            return Err(reason);
        }
    }

    for owner in owners {
        let out = kill_command(owner)
            .output()
            .map_err(|e| format!("failed to run the kill command for pid {owner}: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("failed to end pid {owner}: {}", stderr.trim()));
        }
    }

    for _ in 0..15 {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    Ok(())
}

fn verify_pid(sys: &sysinfo::System, pid: u32, start_time: u64) -> Result<(), String> {
    let proc = sys
        .process(sysinfo::Pid::from(pid as usize))
        .ok_or_else(|| format!("pid {pid} is no longer running"))?;
    if proc.start_time() != start_time {
        return Err(format!(
            "pid {pid} belongs to a different process now (start time changed) — refusing to end it"
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn kill_command(pid: u32) -> std::process::Command {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.no_window().args(["/PID", &pid.to_string(), "/T", "/F"]);
    cmd
}

#[cfg(not(target_os = "windows"))]
fn kill_command(pid: u32) -> std::process::Command {
    let mut cmd = std::process::Command::new("kill");
    cmd.args(["-9", &pid.to_string()]);
    cmd
}

/// Deletes a committed move's source tree (to the Recycle Bin). `local_only`:
/// deletes a directory on this machine's disk.
#[portal_rpc_macro::rpc(local_only)]
pub async fn remove_moved_source(
    id: String,
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DatabaseManager>>,
    coder: State<'_, Arc<CoderService>>,
    terminal: State<'_, TerminalManager>,
    pipelines: State<'_, Arc<ExecutionService>>,
    execution_state: State<'_, ScriptExecutionState>,
    deployment_service: State<'_, Arc<DeploymentService>>,
) -> Result<(), String> {
    let scripts = execution_state.get_or_init(db_manager.inner()).await;
    let deployments = DeploymentRepository::new(db_manager.inner().clone());
    let pipelines_repo = PipelineRepository::new(db_manager.inner().clone());
    let mv = mover::Mover {
        db: db_manager.inner(),
        coder: coder.inner(),
        terminal: terminal.inner(),
        pipelines: pipelines.inner(),
        scripts: &scripts,
        deployments: &deployments,
        deployment_service: deployment_service.inner(),
        pipelines_repo: &pipelines_repo,
        moves_dir: moves_dir(&app)?,
    };
    mv.remove_moved_source(&id).await.map_err(describe_move_error)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

async fn load_project(
    db_manager: &DatabaseManager,
    id: i32,
) -> Result<project_entity::Model, String> {
    project_entity::Entity::find_by_id(id)
        .one(db_manager.get_connection())
        .await
        .map_err(|e| format!("load project {id}: {e}"))?
        .ok_or_else(|| format!("no project with id {id}"))
}

/// Other registered projects whose path sits under `source` — the commit step
/// rebases every one of these onto the destination along with the moved
/// project itself, so preflight surfaces them up front.
async fn nested_under(
    db_manager: &DatabaseManager,
    source: &Path,
    project_id: i32,
) -> Result<Vec<NestedProject>, String> {
    let source_s = source.to_string_lossy();
    let rows = project_entity::Entity::find()
        .all(db_manager.get_connection())
        .await
        .map_err(|e| format!("load projects: {e}"))?;
    Ok(rows
        .into_iter()
        .filter(|p| p.id != project_id && move_lock::is_under(&p.path, &source_s))
        .map(|p| NestedProject { id: p.id, name: p.name, path: p.path })
        .collect())
}

/// Free space on the volume `dest` would land on — display-only, the actual
/// go/no-go on space is `usage::check`'s own (independent) `low_space`
/// blocker. Mirrors that check's disk lookup (longest matching mount point).
fn available_space(dest: &Path) -> Option<u64> {
    let dest_s = dest.to_string_lossy();
    let disks = sysinfo::Disks::new_with_refreshed_list();
    disks
        .iter()
        .filter(|d| move_lock::is_under(&d.mount_point().to_string_lossy(), &dest_s))
        .max_by_key(|d| d.mount_point().to_string_lossy().len())
        .map(|d| d.available_space())
}

fn describe_move_error(e: mover::MoveError) -> String {
    match e {
        mover::MoveError::Blocked(report) => {
            let mut reasons: Vec<String> = report.blockers.into_iter().map(|s| s.detail).collect();
            reasons.extend(report.warnings.into_iter().map(|w| w.reason));
            if reasons.is_empty() {
                "blocked".into()
            } else {
                reasons.join("; ")
            }
        }
        mover::MoveError::Failed(msg) => msg,
    }
}

fn port_u16(p: Option<i32>) -> Option<u16> {
    p.and_then(|v| u16::try_from(v).ok())
}

/// Throttled forwarder for `mover::Progress` onto `project-move://progress`,
/// mirroring `discovery_commands`'s throttle: always the first tick and a
/// phase change (copying -> verifying), otherwise at most every
/// `EMIT_EVERY_MS`.
fn progress_emitter(events: &Events) -> impl FnMut(mover::Progress) + Send + '_ {
    let sink = events.sink();
    let mut last = std::time::Instant::now();
    let mut last_phase = String::new();
    let mut first = true;
    move |p: mover::Progress| {
        let phase_changed = p.phase != last_phase;
        if first || phase_changed || last.elapsed().as_millis() >= EMIT_EVERY_MS {
            last = std::time::Instant::now();
            first = false;
            last_phase = p.phase.clone();
            let _ = sink.emit("project-move://progress", p);
        }
    }
}

/// Where this run's manifests/progress logs live — alongside the rest of its
/// (possibly instance-isolated, see `instance_data_dir`) app data, so a dev
/// instance and the release never trip over each other's move journals.
fn moves_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir: {e}"))?;
    Ok(crate::instance_data_dir(base).join("moves"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// board #216 HIGH: `end_port_process` must refuse to end the caller's
    /// own process even when every pid/start_time/port-ownership check it
    /// runs would otherwise pass. Binds a real listener so this process is
    /// genuinely the OS-reported owner of `port`, then calls the command
    /// end-to-end exactly as the command layer would.
    #[tokio::test]
    async fn end_port_process_refuses_to_kill_self() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let pid = std::process::id();

        let mut sys = sysinfo::System::new_all();
        sys.refresh_processes();
        let start_time = sys
            .process(sysinfo::Pid::from(pid as usize))
            .expect("self pid must be visible to sysinfo")
            .start_time();

        let result = end_port_process(port, pid, start_time).await;
        drop(listener);

        assert!(result.is_err(), "must refuse to kill the caller's own process, got {result:?}");
    }
}
