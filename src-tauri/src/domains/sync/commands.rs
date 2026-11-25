use std::sync::Arc;

use portal_core::database::DatabaseManager;
use portal_core::domains::sync::paths::UnresolvedProject;
use portal_core::domains::sync::service::{
    PullOutcome, PushOutcome, SnapshotRecord, SyncConfig, SyncService, SyncStatus,
};
use tauri::State;

fn svc(db: &State<'_, Arc<DatabaseManager>>) -> SyncService {
    SyncService::new(db.data_dir())
}

/// Runs blocking work (rusqlite, keychain, state file) off the async runtime threads.
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[portal_rpc_macro::rpc]
pub async fn sync_get_config(db: State<'_, Arc<DatabaseManager>>) -> Result<SyncConfig, String> {
    let s = svc(&db);
    blocking(move || s.get_config()).await
}

/// `token` empty/absent keeps the stored one.
#[portal_rpc_macro::rpc]
pub async fn sync_set_config(
    server_url: String,
    token: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let s = svc(&db);
    blocking(move || s.set_config(server_url, token)).await
}

#[portal_rpc_macro::rpc]
pub async fn sync_status(db: State<'_, Arc<DatabaseManager>>) -> Result<SyncStatus, String> {
    svc(&db).status().await
}

#[portal_rpc_macro::rpc]
pub async fn sync_list_versions(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<SnapshotRecord>, String> {
    svc(&db).list_versions().await
}

#[portal_rpc_macro::rpc]
pub async fn sync_push(
    password: String,
    force: bool,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<PushOutcome, String> {
    svc(&db).push(password, force).await
}

#[portal_rpc_macro::rpc]
pub async fn sync_pull(
    password: String,
    version: Option<u32>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<PullOutcome, String> {
    svc(&db).pull(password, version).await
}

#[portal_rpc_macro::rpc]
pub async fn sync_list_unresolved_projects(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<UnresolvedProject>, String> {
    let s = svc(&db);
    blocking(move || s.list_unresolved()).await
}

#[portal_rpc_macro::rpc]
pub async fn sync_resolve_project_path(
    project_id: i64,
    path: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let s = svc(&db);
    blocking(move || s.resolve_project(project_id, &path)).await
}

#[portal_rpc_macro::rpc]
pub async fn sync_ignore_project(
    project_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    let s = svc(&db);
    blocking(move || s.ignore_project(project_id)).await
}
