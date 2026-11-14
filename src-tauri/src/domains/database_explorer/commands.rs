//! IPC commands for the database explorer. Read-only (editing is P2).
//!
//! Connect commands resolve a source to a `DbDriver`; vault passwords are
//! decrypted here (lazily, at connect) so listing never needs the vault unlocked.

use std::sync::Arc;
use tauri::State;

use crate::database::DatabaseManager;
use portal_core::domains::credentials::services::{CredentialService, VaultSession};
use portal_core::domains::credentials::CredentialError;
use portal_core::domains::database_explorer as dbx;

/// Resolve a source to a live driver, decrypting a vault password if needed.
async fn open_driver(
    src: dbx::Source,
    db: &Arc<DatabaseManager>,
    session: &Arc<VaultSession>,
) -> Result<Box<dyn dbx::DbDriver>, String> {
    match src {
        dbx::Source::Sqlite { path, .. } => Ok(Box::new(dbx::SqliteDriver { path })),
        dbx::Source::Server(s) => {
            let password = match s.password {
                dbx::PasswordRef::Plain(p) => p,
                dbx::PasswordRef::Vault(cid) => resolve_vault_password(&cid, db, session).await?,
            };
            let driver =
                dbx::SqlServerDriver::new(&s.engine, s.host, s.port, s.user, password, s.database)?;
            Ok(Box::new(driver))
        }
    }
}

async fn resolve_vault_password(
    credential_id: &str,
    db: &Arc<DatabaseManager>,
    session: &Arc<VaultSession>,
) -> Result<String, String> {
    let service = CredentialService::new(db.get_connection_clone(), session.clone());
    service
        .decrypt_credential(credential_id)
        .await
        .map_err(|e| match e {
            CredentialError::VaultLocked => {
                "Vault is locked — unlock it to connect to this database.".to_string()
            }
            other => other.to_string(),
        })
}

fn source_or_err(
    state: &dbx::DbExplorerState,
    source_id: &str,
) -> Result<dbx::Source, String> {
    state
        .source_of(source_id)
        .ok_or_else(|| format!("unknown source: {source_id}"))
}

#[portal_rpc_macro::rpc]
pub async fn db_list_sources(
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<Vec<dbx::DbSource>, String> {
    let discovered =
        dbx::discovery::discover(db.get_connection_clone(), session.inner().clone()).await;
    state.set_discovered(discovered);
    Ok(state.sources())
}

/// Register an ad-hoc SQLite file after confirming it opens read-only.
#[portal_rpc_macro::rpc]
pub async fn db_open_sqlite(
    path: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::DbSource, String> {
    let p = std::path::PathBuf::from(&path);
    let probe = p.clone();
    tokio::task::spawn_blocking(move || dbx::sqlite::open_ro(&probe).map(|_| ()))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let label = p
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());
    Ok(state.register_sqlite(p, label))
}

#[portal_rpc_macro::rpc]
pub async fn db_list_tables(
    source_id: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<Vec<dbx::TableInfo>, String> {
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    driver.list_tables().await
}

#[portal_rpc_macro::rpc]
pub async fn db_describe_table(
    source_id: String,
    table: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<Vec<dbx::ColumnInfo>, String> {
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    driver.describe_table(&table).await
}

#[portal_rpc_macro::rpc]
pub async fn db_fetch_rows(
    source_id: String,
    table: String,
    limit: usize,
    offset: usize,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<dbx::QueryResult, String> {
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    driver.fetch_rows(&table, limit, offset).await
}

#[portal_rpc_macro::rpc]
pub async fn db_run_query(
    source_id: String,
    sql: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<dbx::QueryResult, String> {
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    driver.run_query(&sql).await
}
