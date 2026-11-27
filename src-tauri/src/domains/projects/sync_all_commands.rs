//! Tauri adapter for "Sync all projects". Progress goes out on
//! `sync-all://progress`, unthrottled. The daemon's `event_cap` has no
//! capability for the `sync-all` domain, so it is never forwarded to a peer.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::credentials::services::VaultSession;
use crate::domains::projects::sync_all::{SyncAllOptions, SyncAllReport, SyncAllService};
use crate::domains::projects::sync_resolve::{
    self, SyncRepoFacts, SyncResolution, SyncResolveResult,
};
use crate::events::{EventSinkExt, Events};

#[derive(Default)]
pub struct SyncAllControl {
    pub cancel: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
}

/// Clears `running` however `run` ends.
struct RunningGuard(Arc<AtomicBool>);

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

// local_only: this pushes to remotes and uploads a snapshot; no LAN device may trigger it.
#[portal_rpc_macro::rpc(local_only)]
pub async fn sync_all_projects(
    options: SyncAllOptions,
    events: State<'_, Events>,
    control: State<'_, SyncAllControl>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<SyncAllReport, String> {
    if control.running.swap(true, Ordering::SeqCst) {
        return Err("A sync is already running".into());
    }
    let _guard = RunningGuard(control.running.clone());
    control.cancel.store(false, Ordering::SeqCst);

    let sink = events.sink();
    let emit = move |p| {
        let _ = sink.emit("sync-all://progress", p);
    };
    SyncAllService::new(db.inner().clone(), session.inner().clone())
        .run(options, control.cancel.clone(), Arc::new(emit))
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn cancel_sync_all(control: State<'_, SyncAllControl>) -> Result<(), String> {
    control.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

// local_only: carries diffs of uncommitted work.
#[portal_rpc_macro::rpc(local_only)]
pub async fn sync_repo_facts(
    project_id: i32,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<SyncRepoFacts, String> {
    sync_resolve::facts_for(&db, project_id).await
}

// local_only: commits and pushes. Shares the running flag so it never overlaps
// a sync (or another apply) on the same working trees.
#[portal_rpc_macro::rpc(local_only)]
pub async fn sync_apply_resolution(
    project_id: i32,
    plan: SyncResolution,
    control: State<'_, SyncAllControl>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<SyncResolveResult, String> {
    if control.running.swap(true, Ordering::SeqCst) {
        return Err("A sync is already running".into());
    }
    let _guard = RunningGuard(control.running.clone());
    sync_resolve::apply_for(&db, project_id, plan).await
}
