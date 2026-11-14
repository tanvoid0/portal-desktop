//! Database explorer domain.
//!
//! Engines: SQLite (`sqlite`, via rusqlite) and Postgres/MySQL (`sql_server`,
//! via sqlx), behind the `DbDriver` trait. Sources come from the registry
//! (`state`) and from `discovery` (vault / SDK / Docker). P2 adds editing:
//! PK detection, the write ops on `DbDriver`, and the `preview_*` dispatchers
//! below, which pick the right engine's pure SQL builder without needing a
//! live connection (so a preview never needs the vault unlocked). Mongo +
//! Redis panes arrive in later phases; see plan `iterative-plotting-flame`.

pub mod discovery;
pub mod driver;
pub mod sql_server;
pub mod sqlite;
mod state;
mod types;

use serde_json::Value;

pub use driver::DbDriver;
pub use sql_server::SqlServerDriver;
pub use sqlite::SqliteDriver;
pub use state::{DbExplorerState, PasswordRef, ServerConn, Source};
pub use types::{ColumnInfo, DbSource, EditPreview, QueryResult, TableInfo};

/// Build the preview for an UPDATE, dispatching to the source's engine.
pub fn preview_update(
    src: &Source,
    table: &str,
    pk_column: &str,
    pk_value: &Value,
    column: &str,
    new_value: &Value,
) -> Result<EditPreview, String> {
    match src {
        Source::Sqlite { .. } => Ok(sqlite::preview_update(table, pk_column, pk_value, column, new_value)),
        Source::Server(s) => Ok(sql_server::preview_update(
            sql_server::Engine::parse(&s.engine)?,
            table,
            pk_column,
            pk_value,
            column,
            new_value,
        )),
    }
}

/// Build the preview for an INSERT, dispatching to the source's engine.
pub fn preview_insert(src: &Source, table: &str, values: &[(String, Value)]) -> Result<EditPreview, String> {
    match src {
        Source::Sqlite { .. } => Ok(sqlite::preview_insert(table, values)),
        Source::Server(s) => Ok(sql_server::preview_insert(sql_server::Engine::parse(&s.engine)?, table, values)),
    }
}

/// Build the preview for a DELETE, dispatching to the source's engine.
pub fn preview_delete(
    src: &Source,
    table: &str,
    pk_column: &str,
    pk_value: &Value,
) -> Result<EditPreview, String> {
    match src {
        Source::Sqlite { .. } => Ok(sqlite::preview_delete(table, pk_column, pk_value)),
        Source::Server(s) => {
            Ok(sql_server::preview_delete(sql_server::Engine::parse(&s.engine)?, table, pk_column, pk_value))
        }
    }
}
