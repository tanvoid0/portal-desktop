//! IPC commands for the database explorer (P0: SQLite, read-only).

use std::sync::Arc;
use tauri::State;

use portal_core::domains::database_explorer as dbx;

#[portal_rpc_macro::rpc]
pub async fn db_list_sources(
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<Vec<dbx::DbSource>, String> {
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
) -> Result<Vec<dbx::TableInfo>, String> {
    let path = state
        .path_of(&source_id)
        .ok_or_else(|| format!("unknown source: {source_id}"))?;
    dbx::sqlite::tables(path).await
}

#[portal_rpc_macro::rpc]
pub async fn db_describe_table(
    source_id: String,
    table: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<Vec<dbx::ColumnInfo>, String> {
    let path = state
        .path_of(&source_id)
        .ok_or_else(|| format!("unknown source: {source_id}"))?;
    dbx::sqlite::columns(path, table).await
}

#[portal_rpc_macro::rpc]
pub async fn db_fetch_rows(
    source_id: String,
    table: String,
    limit: usize,
    offset: usize,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::QueryResult, String> {
    let path = state
        .path_of(&source_id)
        .ok_or_else(|| format!("unknown source: {source_id}"))?;
    dbx::sqlite::rows(path, table, limit, offset).await
}

#[portal_rpc_macro::rpc]
pub async fn db_run_query(
    source_id: String,
    sql: String,
    state: State<'_, Arc<dbx::DbExplorerState>>,
) -> Result<dbx::QueryResult, String> {
    let path = state
        .path_of(&source_id)
        .ok_or_else(|| format!("unknown source: {source_id}"))?;
    dbx::sqlite::query(path, sql).await
}
