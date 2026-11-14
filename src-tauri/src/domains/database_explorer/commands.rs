//! IPC commands for the database explorer.
//!
//! Connect commands resolve a source to a `DbDriver`; vault passwords are
//! decrypted here (lazily, at connect) so listing never needs the vault unlocked.
//! The edit commands (P2) are gated by `require_editable` (per-source toggle,
//! read-only by default) and, for update/delete, `require_pk` (the given
//! column must really be the table's primary key) before anything executes.
//! Preview commands never open a connection — they just build the SQL text.

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

/// Edit gate #1: read-only is the default, so every edit/preview command
/// checks the per-source toggle before doing anything else.
fn require_editable(state: &dbx::DbExplorerState, source_id: &str) -> Result<(), String> {
    if state.is_editable(source_id) {
        Ok(())
    } else {
        Err("editing is disabled for this source — enable it first".to_string())
    }
}

/// Edit gate #2: update/delete need a real, single-column primary key —
/// re-describe the table and check it has exactly one `pk` column and that
/// it's the one the frontend sent, rather than trusting the client. A table
/// with a composite PK is rejected outright: a WHERE clause on only one of
/// its key columns would match every row sharing that value, not one row.
async fn require_pk(driver: &dyn dbx::DbDriver, table: &str, pk_column: &str) -> Result<(), String> {
    let cols = driver.describe_table(table).await?;
    let pk_cols: Vec<&dbx::ColumnInfo> = cols.iter().filter(|c| c.pk).collect();
    match pk_cols.as_slice() {
        [only] if only.name == pk_column => Ok(()),
        [only] => Err(format!(
            "'{pk_column}' is not '{table}''s primary key — '{}' is",
            only.name
        )),
        [] => Err(format!("'{table}' has no detectable primary key — editing needs one")),
        _ => Err(format!(
            "'{table}' has a composite primary key — editing isn't supported for multi-column keys"
        )),
    }
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

// ---- Editing (P2) — read-only is the default, toggled per source. ----

/// Enable/disable editing for a source.
#[portal_rpc_macro::rpc]
pub async fn db_set_editable(
    source_id: String,
    editable: bool,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<(), String> {
    source_or_err(&state, &source_id)?;
    state.set_editable(&source_id, editable);
    Ok(())
}

/// Preview the SQL for a single-cell UPDATE, for the confirm dialog. No DB call.
#[portal_rpc_macro::rpc]
pub async fn db_preview_update(
    source_id: String,
    table: String,
    pk_column: String,
    pk_value: serde_json::Value,
    column: String,
    new_value: serde_json::Value,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::EditPreview, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    dbx::preview_update(&src, &table, &pk_column, &pk_value, &column, &new_value)
}

/// Execute a single-cell UPDATE by primary key, in a transaction.
#[portal_rpc_macro::rpc]
#[allow(clippy::too_many_arguments)]
pub async fn db_apply_update(
    source_id: String,
    table: String,
    pk_column: String,
    pk_value: serde_json::Value,
    column: String,
    new_value: serde_json::Value,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<dbx::QueryResult, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    require_pk(driver.as_ref(), &table, &pk_column).await?;
    driver.update_cell(&table, &pk_column, pk_value, &column, new_value).await
}

/// Preview the SQL for an INSERT, for the confirm dialog. No DB call.
#[portal_rpc_macro::rpc]
pub async fn db_preview_insert(
    source_id: String,
    table: String,
    values: Vec<(String, serde_json::Value)>,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::EditPreview, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    dbx::preview_insert(&src, &table, &values)
}

/// Execute an INSERT, in a transaction.
#[portal_rpc_macro::rpc]
pub async fn db_apply_insert(
    source_id: String,
    table: String,
    values: Vec<(String, serde_json::Value)>,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<dbx::QueryResult, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    driver.insert_row(&table, values).await
}

/// Preview the SQL for a DELETE by primary key, for the confirm dialog. No DB call.
#[portal_rpc_macro::rpc]
pub async fn db_preview_delete(
    source_id: String,
    table: String,
    pk_column: String,
    pk_value: serde_json::Value,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::EditPreview, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    dbx::preview_delete(&src, &table, &pk_column, &pk_value)
}

/// Execute a DELETE by primary key, in a transaction.
#[portal_rpc_macro::rpc]
pub async fn db_apply_delete(
    source_id: String,
    table: String,
    pk_column: String,
    pk_value: serde_json::Value,
    state: State<'_, Arc<dbx::DbExplorerState>>,
    db: State<'_, Arc<DatabaseManager>>,
    session: State<'_, Arc<VaultSession>>,
) -> Result<dbx::QueryResult, String> {
    require_editable(&state, &source_id)?;
    let src = source_or_err(&state, &source_id)?;
    let driver = open_driver(src, db.inner(), session.inner()).await?;
    require_pk(driver.as_ref(), &table, &pk_column).await?;
    driver.delete_row(&table, &pk_column, pk_value).await
}
