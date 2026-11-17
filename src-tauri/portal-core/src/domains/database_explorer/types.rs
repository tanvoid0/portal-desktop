//! Wire types for the database explorer, shared frontend ↔ backend via ts-rs.
//!
//! P0 is SQLite, read-only. `engine` is a string ("sqlite") rather than an enum
//! so adding postgres/mysql/mongo/redis later needs no migration of this type.

use serde::{Deserialize, Serialize};
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

/// One MongoDB document as extended-JSON text (key order preserved; see
/// `mongo.rs` for why it is text, not a JSON value). `id` is the `_id` in
/// canonical extended JSON, sent back verbatim to replace/delete.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct MongoDoc {
    pub id: Option<String>,
    pub json: String,
}

/// One page of documents plus the total matching the filter.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct MongoPage {
    pub docs: Vec<MongoDoc>,
    #[ts(type = "number")]
    pub total: u64,
}

/// One logical Redis database (`db0`…) and its key count.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct RedisDb {
    pub index: u32,
    #[ts(type = "number")]
    pub keys: u64,
}

/// One SCAN page. `cursor` is a u64 as text (JS precision); "0" = done.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct RedisScan {
    pub keys: Vec<String>,
    pub cursor: String,
}

/// A key's value. `items`: hash [f,v] · list [v] · set [m] · zset [m, score].
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub struct RedisValue {
    /// "string" | "hash" | "list" | "set" | "zset" | other.
    pub kind: String,
    /// Seconds; -1 = no expiry.
    #[ts(type = "number")]
    pub ttl: i64,
    /// STRLEN / HLEN / LLEN / SCARD / ZCARD.
    #[ts(type = "number")]
    pub len: u64,
    /// String kind only.
    pub text: Option<String>,
    /// String was not UTF-8: shown lossy, not editable.
    pub binary: bool,
    pub items: Vec<Vec<String>>,
    pub truncated: bool,
}

/// A Redis write. Built into one command by `RedisWrite::args` (redis.rs),
/// which both the confirm preview and the execution use.
#[derive(Deserialize, Clone, TS)]
#[serde(tag = "op")]
#[ts(export, export_to = "../../../src/lib/domains/database/generated/")]
pub enum RedisWrite {
    #[serde(rename = "set")]
    Set {
        key: String,
        value: String,
        #[ts(type = "number | null")]
        ttl: Option<u64>,
        create: bool,
    },
    #[serde(rename = "del")]
    Del { key: String },
}
