//! The `DbDriver` abstraction. P0 had one engine and no trait; P1 adds Postgres
//! and MySQL, so the three engines now share this read-only interface. Editing
//! ops (apply_edit/insert_row/delete_row) arrive with P2.

use async_trait::async_trait;

use crate::domains::database_explorer::types::{ColumnInfo, QueryResult, TableInfo};

#[async_trait]
pub trait DbDriver: Send + Sync {
    /// Tables and views in the source's default namespace.
    async fn list_tables(&self) -> Result<Vec<TableInfo>, String>;
    /// Columns of one table.
    async fn describe_table(&self, table: &str) -> Result<Vec<ColumnInfo>, String>;
    /// A page of rows from one table (capped).
    async fn fetch_rows(
        &self,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<QueryResult, String>;
    /// An ad-hoc read-only query (capped).
    async fn run_query(&self, sql: &str) -> Result<QueryResult, String>;
}
