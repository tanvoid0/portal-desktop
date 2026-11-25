//! Tauri adapter for per-project `.env` viewing and credential linking.
//!
//! `local_only`, not `cap = "projects"`: these return plaintext decrypted-
//! adjacent env values, so — like the credential vault — they are reachable
//! over IPC and loopback only, never the LAN surface.

use std::sync::Arc;

use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::credentials::services::VaultSession;
use crate::domains::projects::env::{
    EnvBackupReport, EnvBackupScan, EnvBackupService, EnvFileRef, EnvLinkView, EnvRestoreItem,
    ProjectEnv, ProjectEnvService,
};

#[portal_rpc_macro::rpc(local_only)]
pub async fn get_project_env(
    project_id: i32,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<ProjectEnv, String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .get(project_id)
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn link_project_env(
    project_id: i32,
    env_key: String,
    file_name: String,
    credential_id: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<EnvLinkView, String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .link(project_id, env_key, file_name, credential_id)
        .await
}

/// Takes `session` only to build the same `ProjectEnvService` as the other
/// commands here — the unlink itself never touches the vault.
#[portal_rpc_macro::rpc(local_only)]
pub async fn unlink_project_env(
    link_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<(), String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .unlink(link_id)
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn resync_project_env_link(
    link_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<EnvLinkView, String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .resync(link_id)
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn add_project_env_file(
    project_id: i32,
    path: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<String, String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .add_file(project_id, path)
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn remove_project_env_file(
    project_id: i32,
    rel_path: String,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<(), String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .remove_file(project_id, rel_path)
        .await
}

/// Env files on disk next to their vault copies. Read-only and never prompts for the vault:
/// with it locked, rows that exist on both sides come back `unknown`.
#[portal_rpc_macro::rpc(local_only)]
pub async fn env_backup_scan(
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<EnvBackupScan, String> {
    EnvBackupService::new(db.get_connection_clone(), session.inner().clone())
        .scan()
        .await
}

/// Disk to vault: `files = None` backs up every env file found, otherwise only those listed.
/// Prompts for the vault once, up front.
#[portal_rpc_macro::rpc(local_only)]
pub async fn env_backup_import(
    files: Option<Vec<EnvFileRef>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<EnvBackupReport, String> {
    EnvBackupService::new(db.get_connection_clone(), session.inner().clone())
        .import(files)
        .await
}

/// Vault to disk. Creates missing files; a differing file is reported as a conflict and
/// replaced only for an item sent with `overwrite: true`. Prompts for the vault once.
#[portal_rpc_macro::rpc(local_only)]
pub async fn env_backup_restore(
    items: Vec<EnvRestoreItem>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<EnvBackupReport, String> {
    EnvBackupService::new(db.get_connection_clone(), session.inner().clone())
        .restore(items)
        .await
}

#[portal_rpc_macro::rpc(local_only)]
pub async fn set_project_env_var(
    project_id: i32,
    file_name: String,
    key: String,
    value: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<(), String> {
    ProjectEnvService::new(&db, session.inner().clone())
        .set_var(project_id, file_name, key, value)
        .await
}
