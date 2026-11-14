//! The `DbDriver` abstraction. P0 had one engine and no trait; P1 adds Postgres
//! and MySQL, so the three engines now share this read-only interface. P2 adds
//! the three write ops below — gated by the caller on the editable toggle and a
//! detectable PK (see `commands.rs`), never by the driver itself.

use async_trait::async_trait;
use serde_json::Value;

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

    /// UPDATE one cell by primary key, in a transaction. Parameterized —
    /// `new_value`/`pk_value` are bound, never interpolated into the SQL text.
    async fn update_cell(
        &self,
        table: &str,
        pk_column: &str,
        pk_value: Value,
        column: &str,
        new_value: Value,
    ) -> Result<QueryResult, String>;
    /// INSERT one row, in a transaction.
    async fn insert_row(&self, table: &str, values: Vec<(String, Value)>) -> Result<QueryResult, String>;
    /// DELETE one row by primary key, in a transaction.
    async fn delete_row(&self, table: &str, pk_column: &str, pk_value: Value) -> Result<QueryResult, String>;
}
