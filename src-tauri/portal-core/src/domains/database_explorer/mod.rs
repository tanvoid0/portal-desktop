//! Database explorer domain.
//!
//! Read-only (editing is P2). Engines: SQLite (`sqlite`, via rusqlite) and
//! Postgres/MySQL (`sql_server`, via sqlx), behind the `DbDriver` trait. Sources
//! come from the registry (`state`) and from `discovery` (vault / SDK / Docker).
//! Mongo + Redis panes arrive in later phases; see plan `iterative-plotting-flame`.

pub mod discovery;
pub mod driver;
pub mod sql_server;
pub mod sqlite;
mod state;
mod types;

pub use driver::DbDriver;
pub use sql_server::SqlServerDriver;
pub use sqlite::SqliteDriver;
pub use state::{DbExplorerState, PasswordRef, ServerConn, Source};
pub use types::{ColumnInfo, DbSource, QueryResult, TableInfo};
