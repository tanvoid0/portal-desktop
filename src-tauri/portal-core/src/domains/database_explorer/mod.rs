//! Database explorer domain.
//!
//! P0: SQLite, read-only — browse schema, page table rows, run SELECTs. Server
//! engines (postgres/mysql), editing, and the Mongo/Redis panes arrive in later
//! phases; see docs plan `iterative-plotting-flame`. There is deliberately no
//! `DbDriver` trait yet: one engine does not justify an abstraction.

pub mod sqlite;
mod state;
mod types;

pub use state::DbExplorerState;
pub use types::{ColumnInfo, DbSource, QueryResult, TableInfo};
