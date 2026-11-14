//! Postgres + MySQL read-only driver over sqlx (P1).
//!
//! One struct handles both engines, branching only where the SQL dialect differs:
//! `information_schema` introspection (schema `public` for pg, `DATABASE()` for
//! mysql), identifier quoting, and placeholder style. Row values are mapped to
//! JSON per engine by matching the column type; unknown types fall back to their
//! text form, else a `<TYPE>` tag — good enough for a read-only viewer.
//!
//! ponytail:
//! - a fresh pool is opened per call (no cache yet);
//! - Postgres introspection covers the `public` schema only;
//! - primary-key detection is deferred to P2 (editing) — `pk` is always false.

use async_trait::async_trait;
use serde_json::Value;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlRow};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgRow};
use sqlx::{Column, MySqlPool, PgPool, Row, TypeInfo};
use std::time::Duration;

use crate::domains::database_explorer::driver::DbDriver;
use crate::domains::database_explorer::sqlite::ROW_CAP;
use crate::domains::database_explorer::types::{ColumnInfo, QueryResult, TableInfo};

/// Seconds to wait for a connection before giving up (connect + acquire).
const CONNECT_TIMEOUT_SECS: u64 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Engine {
    Postgres,
    Mysql,
}

pub struct SqlServerDriver {
    engine: Engine,
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
}

impl SqlServerDriver {
    /// Build a driver from a stored engine string ("postgres" | "mysql").
    pub fn new(
        engine: &str,
        host: String,
        port: u16,
        user: String,
        password: String,
        database: String,
    ) -> Result<Self, String> {
        let engine = match engine {
            "postgres" | "postgresql" => Engine::Postgres,
            "mysql" | "mariadb" => Engine::Mysql,
            other => return Err(format!("unsupported engine: {other}")),
        };
        Ok(Self {
            engine,
            host,
            port,
            user,
            password,
            database,
        })
    }

    async fn pg_pool(&self) -> Result<PgPool, String> {
        let mut opts = PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user);
        if !self.password.is_empty() {
            opts = opts.password(&self.password);
        }
        if !self.database.is_empty() {
            opts = opts.database(&self.database);
        }
        PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .connect_with(opts)
            .await
            .map_err(|e| e.to_string())
    }

    async fn my_pool(&self) -> Result<MySqlPool, String> {
        let mut opts = MySqlConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user);
        if !self.password.is_empty() {
            opts = opts.password(&self.password);
        }
        if !self.database.is_empty() {
            opts = opts.database(&self.database);
        }
        MySqlPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .connect_with(opts)
            .await
            .map_err(|e| e.to_string())
    }
}

// ---- pure SQL builders (unit-tested; no DB needed) ----

/// Postgres: tables + views in the public schema.
const PG_LIST_TABLES: &str = "SELECT table_name::text, table_type::text \
     FROM information_schema.tables WHERE table_schema = 'public' \
     ORDER BY table_type, table_name";

/// MySQL: tables + views in the connected database.
const MY_LIST_TABLES: &str = "SELECT table_name, table_type \
     FROM information_schema.tables WHERE table_schema = DATABASE() \
     ORDER BY table_type, table_name";

const PG_DESCRIBE: &str = "SELECT column_name::text, data_type::text, is_nullable::text, column_default::text \
     FROM information_schema.columns WHERE table_schema = 'public' AND table_name = $1 \
     ORDER BY ordinal_position";

const MY_DESCRIBE: &str = "SELECT column_name, data_type, is_nullable, column_default \
     FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = ? \
     ORDER BY ordinal_position";

fn quote_pg(id: &str) -> String {
    format!("\"{}\"", id.replace('"', "\"\""))
}

fn quote_my(id: &str) -> String {
    format!("`{}`", id.replace('`', "``"))
}

/// `SELECT *` from a table, capped. `cap + 1` so we can flag truncation.
fn fetch_rows_sql(engine: Engine, table: &str, cap: usize, offset: usize) -> String {
    let quoted = match engine {
        Engine::Postgres => quote_pg(table),
        Engine::Mysql => quote_my(table),
    };
    format!("SELECT * FROM {} LIMIT {} OFFSET {}", quoted, cap + 1, offset)
}

/// Wrap an ad-hoc query in a capped subquery so a huge result never floods the
/// UI, and so one code path caps both engines. Works for any single SELECT.
fn wrap_query_sql(sql: &str, cap: usize) -> String {
    let inner = sql.trim().trim_end_matches(';').trim_end();
    format!("SELECT * FROM ({}) _q LIMIT {}", inner, cap + 1)
}

/// Map an information_schema table_type to our "table" | "view".
fn map_kind(table_type: &str) -> String {
    match table_type.to_uppercase().as_str() {
        "BASE TABLE" => "table".to_string(),
        "VIEW" => "view".to_string(),
        other => other.to_lowercase(),
    }
}

// ---- row value → JSON, per engine ----

macro_rules! try_into {
    ($row:expr, $i:expr, $t:ty) => {
        match $row.try_get::<Option<$t>, _>($i) {
            Ok(Some(v)) => return v.into(),
            Ok(None) => return Value::Null,
            Err(_) => {}
        }
    };
}
macro_rules! try_str {
    ($row:expr, $i:expr, $t:ty) => {
        match $row.try_get::<Option<$t>, _>($i) {
            Ok(Some(v)) => return Value::String(v.to_string()),
            Ok(None) => return Value::Null,
            Err(_) => {}
        }
    };
}
macro_rules! try_json {
    ($row:expr, $i:expr) => {
        match $row.try_get::<Option<Value>, _>($i) {
            Ok(Some(v)) => return v,
            Ok(None) => return Value::Null,
            Err(_) => {}
        }
    };
}

fn pg_cell(row: &PgRow, i: usize) -> Value {
    let ty = row.column(i).type_info().name().to_uppercase();
    match ty.as_str() {
        "BOOL" => try_into!(row, i, bool),
        "INT2" => try_into!(row, i, i16),
        "INT4" => try_into!(row, i, i32),
        "INT8" => try_into!(row, i, i64),
        "FLOAT4" => try_into!(row, i, f32),
        "FLOAT8" => try_into!(row, i, f64),
        "NUMERIC" => try_str!(row, i, sqlx::types::BigDecimal),
        "UUID" => try_str!(row, i, sqlx::types::Uuid),
        "JSON" | "JSONB" => try_json!(row, i),
        "TIMESTAMP" => try_str!(row, i, sqlx::types::chrono::NaiveDateTime),
        "TIMESTAMPTZ" => {
            try_str!(row, i, sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>)
        }
        "DATE" => try_str!(row, i, sqlx::types::chrono::NaiveDate),
        "TIME" => try_str!(row, i, sqlx::types::chrono::NaiveTime),
        _ => {}
    }
    // Text types and anything unhandled: show the text form, else a type tag.
    match row.try_get::<Option<String>, _>(i) {
        Ok(Some(s)) => Value::String(s),
        Ok(None) => Value::Null,
        Err(_) => Value::String(format!("<{ty}>")),
    }
}

fn my_cell(row: &MySqlRow, i: usize) -> Value {
    let ty = row.column(i).type_info().name().to_uppercase();
    if ty.contains("INT") {
        // Width + signedness varies; try the plausible Rust types in order.
        try_into!(row, i, i64);
        try_into!(row, i, i32);
        try_into!(row, i, i16);
        try_into!(row, i, i8);
        try_into!(row, i, u64);
        try_into!(row, i, u32);
    } else if ty.contains("DOUBLE") || ty.contains("FLOAT") {
        try_into!(row, i, f64);
        try_into!(row, i, f32);
    } else if ty.contains("DECIMAL") || ty.contains("NUMERIC") {
        try_str!(row, i, sqlx::types::BigDecimal);
    } else if ty == "JSON" {
        try_json!(row, i);
    } else if ty.contains("DATETIME") || ty.contains("TIMESTAMP") {
        try_str!(row, i, sqlx::types::chrono::NaiveDateTime);
    } else if ty == "DATE" {
        try_str!(row, i, sqlx::types::chrono::NaiveDate);
    } else if ty == "TIME" {
        try_str!(row, i, sqlx::types::chrono::NaiveTime);
    } else if ty.contains("BLOB") || ty.contains("BINARY") {
        return Value::String("<blob>".to_string());
    }
    match row.try_get::<Option<String>, _>(i) {
        Ok(Some(s)) => Value::String(s),
        Ok(None) => Value::Null,
        Err(_) => Value::String(format!("<{ty}>")),
    }
}

fn pg_result(rows: Vec<PgRow>, cap: usize) -> QueryResult {
    let truncated = rows.len() > cap;
    let slice = if truncated { &rows[..cap] } else { &rows[..] };
    let columns: Vec<String> = slice
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
        .unwrap_or_default();
    let out: Vec<Vec<Value>> = slice
        .iter()
        .map(|r| (0..r.columns().len()).map(|i| pg_cell(r, i)).collect())
        .collect();
    QueryResult {
        row_count: out.len(),
        columns,
        rows: out,
        truncated,
    }
}

fn my_result(rows: Vec<MySqlRow>, cap: usize) -> QueryResult {
    let truncated = rows.len() > cap;
    let slice = if truncated { &rows[..cap] } else { &rows[..] };
    let columns: Vec<String> = slice
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
        .unwrap_or_default();
    let out: Vec<Vec<Value>> = slice
        .iter()
        .map(|r| (0..r.columns().len()).map(|i| my_cell(r, i)).collect())
        .collect();
    QueryResult {
        row_count: out.len(),
        columns,
        rows: out,
        truncated,
    }
}

#[async_trait]
impl DbDriver for SqlServerDriver {
    async fn list_tables(&self) -> Result<Vec<TableInfo>, String> {
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let rows = sqlx::query(PG_LIST_TABLES)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(rows
                    .iter()
                    .map(|r| TableInfo {
                        name: r.try_get::<String, _>(0).unwrap_or_default(),
                        kind: map_kind(&r.try_get::<String, _>(1).unwrap_or_default()),
                    })
                    .collect())
            }
            Engine::Mysql => {
                let pool = self.my_pool().await?;
                let rows = sqlx::query(MY_LIST_TABLES)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(rows
                    .iter()
                    .map(|r| TableInfo {
                        name: r.try_get::<String, _>(0).unwrap_or_default(),
                        kind: map_kind(&r.try_get::<String, _>(1).unwrap_or_default()),
                    })
                    .collect())
            }
        }
    }

    async fn describe_table(&self, table: &str) -> Result<Vec<ColumnInfo>, String> {
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let rows = sqlx::query(PG_DESCRIBE)
                    .bind(table)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(rows.iter().map(pg_column).collect())
            }
            Engine::Mysql => {
                let pool = self.my_pool().await?;
                let rows = sqlx::query(MY_DESCRIBE)
                    .bind(table)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(rows.iter().map(my_column).collect())
            }
        }
    }

    async fn fetch_rows(
        &self,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<QueryResult, String> {
        let cap = limit.clamp(1, ROW_CAP);
        let sql = fetch_rows_sql(self.engine, table, cap, offset);
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let rows = sqlx::query(&sql)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(pg_result(rows, cap))
            }
            Engine::Mysql => {
                let pool = self.my_pool().await?;
                let rows = sqlx::query(&sql)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(my_result(rows, cap))
            }
        }
    }

    async fn run_query(&self, sql: &str) -> Result<QueryResult, String> {
        let cap = ROW_CAP;
        let wrapped = wrap_query_sql(sql, cap);
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let rows = sqlx::query(&wrapped)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(pg_result(rows, cap))
            }
            Engine::Mysql => {
                let pool = self.my_pool().await?;
                let rows = sqlx::query(&wrapped)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(my_result(rows, cap))
            }
        }
    }
}

fn pg_column(r: &PgRow) -> ColumnInfo {
    ColumnInfo {
        name: r.try_get::<String, _>(0).unwrap_or_default(),
        data_type: r.try_get::<String, _>(1).unwrap_or_default(),
        not_null: r
            .try_get::<String, _>(2)
            .map(|s| s.eq_ignore_ascii_case("NO"))
            .unwrap_or(false),
        pk: false,
        default_value: r.try_get::<Option<String>, _>(3).ok().flatten(),
    }
}

fn my_column(r: &MySqlRow) -> ColumnInfo {
    ColumnInfo {
        name: r.try_get::<String, _>(0).unwrap_or_default(),
        data_type: r.try_get::<String, _>(1).unwrap_or_default(),
        not_null: r
            .try_get::<String, _>(2)
            .map(|s| s.eq_ignore_ascii_case("NO"))
            .unwrap_or(false),
        pk: false,
        default_value: r.try_get::<Option<String>, _>(3).ok().flatten(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_identifiers_per_engine() {
        assert_eq!(quote_pg("users"), "\"users\"");
        assert_eq!(quote_pg("we\"ird"), "\"we\"\"ird\"");
        assert_eq!(quote_my("users"), "`users`");
        assert_eq!(quote_my("we`ird"), "`we``ird`");
    }

    #[test]
    fn introspection_targets_the_right_schema_scope() {
        // pg scopes to the public schema; mysql to the connected DB.
        assert!(PG_LIST_TABLES.contains("information_schema.tables"));
        assert!(PG_LIST_TABLES.contains("table_schema = 'public'"));
        assert!(MY_LIST_TABLES.contains("table_schema = DATABASE()"));
        // describe binds the table name and orders by position.
        assert!(PG_DESCRIBE.contains("table_name = $1"));
        assert!(PG_DESCRIBE.contains("ORDER BY ordinal_position"));
        assert!(MY_DESCRIBE.contains("table_name = ?"));
        assert!(MY_DESCRIBE.contains("ORDER BY ordinal_position"));
    }

    #[test]
    fn fetch_rows_sql_quotes_and_caps_plus_one() {
        assert_eq!(
            fetch_rows_sql(Engine::Postgres, "orders", 100, 50),
            "SELECT * FROM \"orders\" LIMIT 101 OFFSET 50"
        );
        assert_eq!(
            fetch_rows_sql(Engine::Mysql, "orders", 100, 0),
            "SELECT * FROM `orders` LIMIT 101 OFFSET 0"
        );
    }

    #[test]
    fn wrap_query_strips_trailing_semicolon_and_caps() {
        assert_eq!(
            wrap_query_sql("SELECT 1 ;  ", 200),
            "SELECT * FROM (SELECT 1) _q LIMIT 201"
        );
    }

    #[test]
    fn maps_table_type_to_kind() {
        assert_eq!(map_kind("BASE TABLE"), "table");
        assert_eq!(map_kind("VIEW"), "view");
        assert_eq!(map_kind("SYSTEM VIEW"), "system view");
    }
}
