//! Wire types for the database explorer, shared frontend ↔ backend via ts-rs.
//!
//! P0 is SQLite, read-only. `engine` is a string ("sqlite") rather than an enum
//! so adding postgres/mysql/mongo/redis later needs no migration of this type.

use serde::Serialize;
use ts_rs::TS;

/// A connectable database the explorer knows about. P0: SQLite files only
/// (Portal's own DB + ad-hoc opens). Later phases add server + discovered sources.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct DbSource {
    pub id: String,
    pub label: String,
    /// "sqlite" for now.
    pub engine: String,
    /// Human detail, e.g. the file path.
    pub detail: String,
    /// P0 is read-only everywhere; editing arrives in P2.
    pub editable: bool,
}

/// A table or view in a SQL source.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct TableInfo {
    pub name: String,
    /// "table" | "view".
    pub kind: String,
}

/// One column of a table.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub pk: bool,
    pub default_value: Option<String>,
}

/// Result of a table fetch or an ad-hoc query. `rows` are column-ordered, each
/// cell a JSON value (null/number/string; blobs become a "<blob: N bytes>" label).
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    /// True when the row cap was hit and more rows exist.
    pub truncated: bool,
}

impl QueryResult {
    /// A result carrying only an affected-row count (UPDATE/INSERT/DELETE) —
    /// no columns/rows to display.
    pub fn affected(n: usize) -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: n,
            truncated: false,
        }
    }
}

/// The parameterized SQL + bound values for one edit (P2). Pure to build — the
/// same text is shown to the user as the confirm-dialog preview and used
/// (with real bound params) to execute; a value is never string-interpolated
/// into SQL that actually runs. Postgres execution appends a per-param
/// `::<type>` cast that this preview text omits (see `sql_server.rs`) — it
/// only affects type coercion, never the bound value.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct EditPreview {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}
