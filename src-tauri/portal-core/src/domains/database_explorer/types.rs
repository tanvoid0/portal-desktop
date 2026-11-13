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
