//! Postgres + MySQL driver over sqlx. Read-only ops are P1; PK detection and
//! the edit ops (update/insert/delete) are P2.
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
//! - edit params are bound by JSON-value shape (string/i64/f64/bool/null), not
//!   the target column's real type. Postgres won't implicitly cast a
//!   text-bound param into another column type (fails: "operator does not
//!   exist"), so the Postgres edit ops append a `::<type>` cast per param —
//!   introspected from `information_schema` right before executing — to the
//!   *executed* SQL only; the confirm-dialog preview (built with no DB access,
//!   so it never needs the vault unlocked) does not show the cast. The cast
//!   only changes how Postgres coerces the bound value, never the value
//!   itself, so the preview still shows exactly what gets written. MySQL and
//!   SQLite coerce text params on write and need no equivalent.

use async_trait::async_trait;
use serde_json::Value;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlRow};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgRow};
use sqlx::{Column, MySqlPool, PgPool, Row, TypeInfo};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::domains::database_explorer::driver::DbDriver;
use crate::domains::database_explorer::sqlite::ROW_CAP;
use crate::domains::database_explorer::types::{ColumnInfo, EditPreview, QueryResult, TableInfo};

/// Seconds to wait for a connection before giving up (connect + acquire).
const CONNECT_TIMEOUT_SECS: u64 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Engine {
    Postgres,
    Mysql,
}

impl Engine {
    /// Parses a stored engine string ("postgres" | "mysql" and aliases).
    pub fn parse(engine: &str) -> Result<Self, String> {
        match engine {
            "postgres" | "postgresql" => Ok(Engine::Postgres),
            "mysql" | "mariadb" => Ok(Engine::Mysql),
            other => Err(format!("unsupported engine: {other}")),
        }
    }
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
        Ok(Self {
            engine: Engine::parse(engine)?,
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

/// Postgres (P2): columns that are part of a PRIMARY KEY constraint on this table.
const PG_PRIMARY_KEYS: &str = "SELECT kcu.column_name::text \
     FROM information_schema.key_column_usage kcu \
     JOIN information_schema.table_constraints tc \
       ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema \
     WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = 'public' AND tc.table_name = $1";

/// MySQL (P2): `column_key = 'PRI'` already identifies PK columns directly.
const MY_PRIMARY_KEYS: &str = "SELECT column_name FROM information_schema.columns \
     WHERE table_schema = DATABASE() AND table_name = ? AND column_key = 'PRI'";

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

// ---- edit-SQL builders (P2, unit-tested; no DB needed) — reused for both the
// confirm-dialog preview and (bound, below) the actual execution. ----

fn update_sql(engine: Engine, table: &str, pk_column: &str, column: &str) -> String {
    match engine {
        Engine::Postgres => format!(
            "UPDATE {} SET {} = $1 WHERE {} = $2",
            quote_pg(table),
            quote_pg(column),
            quote_pg(pk_column)
        ),
        Engine::Mysql => format!(
            "UPDATE {} SET {} = ? WHERE {} = ?",
            quote_my(table),
            quote_my(column),
            quote_my(pk_column)
        ),
    }
}

fn insert_sql(engine: Engine, table: &str, columns: &[String]) -> String {
    match engine {
        Engine::Postgres => {
            let cols: Vec<String> = columns.iter().map(|c| quote_pg(c)).collect();
            let ph: Vec<String> = (1..=columns.len()).map(|i| format!("${i}")).collect();
            format!("INSERT INTO {} ({}) VALUES ({})", quote_pg(table), cols.join(", "), ph.join(", "))
        }
        Engine::Mysql => {
            let cols: Vec<String> = columns.iter().map(|c| quote_my(c)).collect();
            let ph: Vec<String> = columns.iter().map(|_| "?".to_string()).collect();
            format!("INSERT INTO {} ({}) VALUES ({})", quote_my(table), cols.join(", "), ph.join(", "))
        }
    }
}

fn delete_sql(engine: Engine, table: &str, pk_column: &str) -> String {
    match engine {
        Engine::Postgres => format!("DELETE FROM {} WHERE {} = $1", quote_pg(table), quote_pg(pk_column)),
        Engine::Mysql => format!("DELETE FROM {} WHERE {} = ?", quote_my(table), quote_my(pk_column)),
    }
}

/// Postgres only: `column_name -> data_type` for every column of `table`,
/// used to cast edit params to their real type at execution time. Falls back
/// to no cast (same as before this existed) for a type `information_schema`
/// can't name directly (e.g. `USER-DEFINED` enums) or on introspection failure
/// — editing such a column may still fail with the underlying Postgres error.
async fn pg_column_types(pool: &PgPool, table: &str) -> HashMap<String, String> {
    let Ok(rows) = sqlx::query(PG_DESCRIBE).bind(table).fetch_all(pool).await else {
        return HashMap::new();
    };
    rows.iter()
        .filter_map(|r| {
            let name: String = r.try_get(0).ok()?;
            let ty: String = r.try_get(1).ok()?;
            (!ty.eq_ignore_ascii_case("USER-DEFINED")).then_some((name, ty))
        })
        .collect()
}

/// `::<type>` suffix for a bound param, or empty when the column's type is
/// unknown/uncastable (see `pg_column_types`).
fn pg_cast(types: &HashMap<String, String>, column: &str) -> String {
    types.get(column).map(|t| format!("::{t}")).unwrap_or_default()
}

fn update_sql_pg_cast(table: &str, pk_column: &str, column: &str, types: &HashMap<String, String>) -> String {
    format!(
        "UPDATE {} SET {} = $1{} WHERE {} = $2{}",
        quote_pg(table),
        quote_pg(column),
        pg_cast(types, column),
        quote_pg(pk_column),
        pg_cast(types, pk_column)
    )
}

fn insert_sql_pg_cast(table: &str, columns: &[String], types: &HashMap<String, String>) -> String {
    let cols: Vec<String> = columns.iter().map(|c| quote_pg(c)).collect();
    let ph: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("${}{}", i + 1, pg_cast(types, c)))
        .collect();
    format!("INSERT INTO {} ({}) VALUES ({})", quote_pg(table), cols.join(", "), ph.join(", "))
}

fn delete_sql_pg_cast(table: &str, pk_column: &str, types: &HashMap<String, String>) -> String {
    format!(
        "DELETE FROM {} WHERE {} = $1{}",
        quote_pg(table),
        quote_pg(pk_column),
        pg_cast(types, pk_column)
    )
}

pub fn preview_update(
    engine: Engine,
    table: &str,
    pk_column: &str,
    pk_value: &Value,
    column: &str,
    new_value: &Value,
) -> EditPreview {
    EditPreview {
        sql: update_sql(engine, table, pk_column, column),
        params: vec![new_value.clone(), pk_value.clone()],
    }
}

pub fn preview_insert(engine: Engine, table: &str, values: &[(String, Value)]) -> EditPreview {
    let columns: Vec<String> = values.iter().map(|(c, _)| c.clone()).collect();
    EditPreview {
        sql: insert_sql(engine, table, &columns),
        params: values.iter().map(|(_, v)| v.clone()).collect(),
    }
}

pub fn preview_delete(engine: Engine, table: &str, pk_column: &str, pk_value: &Value) -> EditPreview {
    EditPreview {
        sql: delete_sql(engine, table, pk_column),
        params: vec![pk_value.clone()],
    }
}

/// Bind a JSON value onto a Postgres query by its JSON shape (see the
/// module-level ponytail note on the type-matching ceiling this implies).
fn bind_pg<'q>(
    mut q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    params: &[Value],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for v in params {
        q = match v {
            Value::Null => q.bind(None::<String>),
            Value::Bool(b) => q.bind(*b),
            Value::Number(n) => match n.as_i64() {
                Some(i) => q.bind(i),
                None => q.bind(n.as_f64().unwrap_or_default()),
            },
            Value::String(s) => q.bind(s.clone()),
            other => q.bind(other.to_string()),
        };
    }
    q
}

fn bind_my<'q>(
    mut q: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    params: &[Value],
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    for v in params {
        q = match v {
            Value::Null => q.bind(None::<String>),
            Value::Bool(b) => q.bind(*b),
            Value::Number(n) => match n.as_i64() {
                Some(i) => q.bind(i),
                None => q.bind(n.as_f64().unwrap_or_default()),
            },
            Value::String(s) => q.bind(s.clone()),
            other => q.bind(other.to_string()),
        };
    }
    q
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

/// JS `Number` is a float64 and loses precision above this (2^53) — serialize
/// a bigger `int8`/`bigint` as a string instead, so round-tripping it back as
/// an edit's PK value doesn't silently hit the wrong row.
const JS_MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_992;

fn pg_cell(row: &PgRow, i: usize) -> Value {
    let ty = row.column(i).type_info().name().to_uppercase();
    match ty.as_str() {
        "BOOL" => try_into!(row, i, bool),
        "INT2" => try_into!(row, i, i16),
        "INT4" => try_into!(row, i, i32),
        "INT8" => match row.try_get::<Option<i64>, _>(i) {
            Ok(Some(v)) if v.abs() > JS_MAX_SAFE_INTEGER => return Value::from(v.to_string()),
            Ok(Some(v)) => return v.into(),
            Ok(None) => return Value::Null,
            Err(_) => {}
        },
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
                let pk_rows = sqlx::query(PG_PRIMARY_KEYS)
                    .bind(table)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                let pk_names: HashSet<String> =
                    pk_rows.iter().filter_map(|r| r.try_get::<String, _>(0).ok()).collect();
                Ok(rows.iter().map(|r| pg_column(r, &pk_names)).collect())
            }
            Engine::Mysql => {
                let pool = self.my_pool().await?;
                let rows = sqlx::query(MY_DESCRIBE)
                    .bind(table)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                let pk_rows = sqlx::query(MY_PRIMARY_KEYS)
                    .bind(table)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                let pk_names: HashSet<String> =
                    pk_rows.iter().filter_map(|r| r.try_get::<String, _>(0).ok()).collect();
                Ok(rows.iter().map(|r| my_column(r, &pk_names)).collect())
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

    async fn update_cell(
        &self,
        table: &str,
        pk_column: &str,
        pk_value: Value,
        column: &str,
        new_value: Value,
    ) -> Result<QueryResult, String> {
        let params = [new_value, pk_value];
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let types = pg_column_types(&pool, table).await;
                let sql = update_sql_pg_cast(table, pk_column, column, &types);
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_pg(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_pg_tx(tx, res).await
            }
            Engine::Mysql => {
                let sql = update_sql(self.engine, table, pk_column, column);
                let pool = self.my_pool().await?;
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_my(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_my_tx(tx, res).await
            }
        }
    }

    async fn insert_row(&self, table: &str, values: Vec<(String, Value)>) -> Result<QueryResult, String> {
        let columns: Vec<String> = values.iter().map(|(c, _)| c.clone()).collect();
        let params: Vec<Value> = values.into_iter().map(|(_, v)| v).collect();
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let types = pg_column_types(&pool, table).await;
                let sql = insert_sql_pg_cast(table, &columns, &types);
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_pg(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_pg_tx(tx, res).await
            }
            Engine::Mysql => {
                let sql = insert_sql(self.engine, table, &columns);
                let pool = self.my_pool().await?;
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_my(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_my_tx(tx, res).await
            }
        }
    }

    async fn delete_row(&self, table: &str, pk_column: &str, pk_value: Value) -> Result<QueryResult, String> {
        let params = [pk_value];
        match self.engine {
            Engine::Postgres => {
                let pool = self.pg_pool().await?;
                let types = pg_column_types(&pool, table).await;
                let sql = delete_sql_pg_cast(table, pk_column, &types);
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_pg(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_pg_tx(tx, res).await
            }
            Engine::Mysql => {
                let sql = delete_sql(self.engine, table, pk_column);
                let pool = self.my_pool().await?;
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
                let res = bind_my(sqlx::query(&sql), &params).execute(&mut *tx).await;
                finish_my_tx(tx, res).await
            }
        }
    }
}

/// Commit on success, roll back unless exactly one row was affected — the
/// safety net for a PK that turns out not to be unique.
async fn finish_pg_tx(
    tx: sqlx::Transaction<'_, sqlx::Postgres>,
    res: Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
) -> Result<QueryResult, String> {
    match res {
        Ok(r) if r.rows_affected() == 1 => {
            tx.commit().await.map_err(|e| e.to_string())?;
            Ok(QueryResult::affected(1))
        }
        Ok(r) => {
            let _ = tx.rollback().await;
            Err(format!("expected to affect exactly 1 row, affected {} — rolled back", r.rows_affected()))
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e.to_string())
        }
    }
}

/// Commit on success, roll back unless exactly one row was affected — the
/// safety net for a PK that turns out not to be unique.
async fn finish_my_tx(
    tx: sqlx::Transaction<'_, sqlx::MySql>,
    res: Result<sqlx::mysql::MySqlQueryResult, sqlx::Error>,
) -> Result<QueryResult, String> {
    match res {
        Ok(r) if r.rows_affected() == 1 => {
            tx.commit().await.map_err(|e| e.to_string())?;
            Ok(QueryResult::affected(1))
        }
        Ok(r) => {
            let _ = tx.rollback().await;
            Err(format!("expected to affect exactly 1 row, affected {} — rolled back", r.rows_affected()))
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e.to_string())
        }
    }
}

fn pg_column(r: &PgRow, pk_names: &HashSet<String>) -> ColumnInfo {
    let name = r.try_get::<String, _>(0).unwrap_or_default();
    ColumnInfo {
        pk: pk_names.contains(&name),
        name,
        data_type: r.try_get::<String, _>(1).unwrap_or_default(),
        not_null: r
            .try_get::<String, _>(2)
            .map(|s| s.eq_ignore_ascii_case("NO"))
            .unwrap_or(false),
        default_value: r.try_get::<Option<String>, _>(3).ok().flatten(),
    }
}

fn my_column(r: &MySqlRow, pk_names: &HashSet<String>) -> ColumnInfo {
    let name = r.try_get::<String, _>(0).unwrap_or_default();
    ColumnInfo {
        pk: pk_names.contains(&name),
        name,
        data_type: r.try_get::<String, _>(1).unwrap_or_default(),
        not_null: r
            .try_get::<String, _>(2)
            .map(|s| s.eq_ignore_ascii_case("NO"))
            .unwrap_or(false),
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

    #[test]
    fn primary_key_queries_target_the_right_schema_scope() {
        assert!(PG_PRIMARY_KEYS.contains("constraint_type = 'PRIMARY KEY'"));
        assert!(PG_PRIMARY_KEYS.contains("table_schema = 'public'"));
        assert!(PG_PRIMARY_KEYS.contains("table_name = $1"));
        assert!(MY_PRIMARY_KEYS.contains("column_key = 'PRI'"));
        assert!(MY_PRIMARY_KEYS.contains("table_schema = DATABASE()"));
        assert!(MY_PRIMARY_KEYS.contains("table_name = ?"));
    }

    #[test]
    fn update_sql_binds_value_then_pk_per_engine() {
        assert_eq!(
            update_sql(Engine::Postgres, "orders", "id", "status"),
            "UPDATE \"orders\" SET \"status\" = $1 WHERE \"id\" = $2"
        );
        assert_eq!(
            update_sql(Engine::Mysql, "orders", "id", "status"),
            "UPDATE `orders` SET `status` = ? WHERE `id` = ?"
        );
    }

    #[test]
    fn insert_sql_lists_columns_and_placeholders_per_engine() {
        let cols = vec!["name".to_string(), "amt".to_string()];
        assert_eq!(
            insert_sql(Engine::Postgres, "orders", &cols),
            "INSERT INTO \"orders\" (\"name\", \"amt\") VALUES ($1, $2)"
        );
        assert_eq!(
            insert_sql(Engine::Mysql, "orders", &cols),
            "INSERT INTO `orders` (`name`, `amt`) VALUES (?, ?)"
        );
    }

    #[test]
    fn delete_sql_scopes_to_pk_per_engine() {
        assert_eq!(delete_sql(Engine::Postgres, "orders", "id"), "DELETE FROM \"orders\" WHERE \"id\" = $1");
        assert_eq!(delete_sql(Engine::Mysql, "orders", "id"), "DELETE FROM `orders` WHERE `id` = ?");
    }

    #[test]
    fn preview_update_carries_value_then_pk_as_params() {
        let p = preview_update(Engine::Postgres, "orders", "id", &Value::from(7), "status", &Value::from("done"));
        assert_eq!(p.sql, "UPDATE \"orders\" SET \"status\" = $1 WHERE \"id\" = $2");
        assert_eq!(p.params, vec![Value::from("done"), Value::from(7)]);
    }

    #[test]
    fn engine_parse_accepts_known_aliases_and_rejects_unknown() {
        assert_eq!(Engine::parse("postgres").unwrap(), Engine::Postgres);
        assert_eq!(Engine::parse("postgresql").unwrap(), Engine::Postgres);
        assert_eq!(Engine::parse("mysql").unwrap(), Engine::Mysql);
        assert_eq!(Engine::parse("mariadb").unwrap(), Engine::Mysql);
        assert!(Engine::parse("mongo").is_err());
    }
}
