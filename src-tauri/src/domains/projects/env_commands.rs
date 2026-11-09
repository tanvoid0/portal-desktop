//! Tauri adapter for per-project `.env` viewing and credential linking.
//!
//! `local_only`, not `cap = "projects"`: these return plaintext decrypted-
//! adjacent env values, so — like the credential vault — they are reachable
//! over IPC and loopback only, never the LAN surface.

use std::sync::Arc;

use tauri::State;

use crate::database::DatabaseManager;
use crate::domains::credentials::services::VaultSession;
use crate::domains::projects::env::{EnvLinkView, ProjectEnv, ProjectEnvService};

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
