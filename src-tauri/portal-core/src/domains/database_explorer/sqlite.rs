//! SQLite introspection + read-only queries via rusqlite on a blocking thread.
//!
//! Connections are opened READ-ONLY, so nothing here can mutate a database — the
//! P0 safety guarantee. The pure `*(conn)` functions carry the real logic and are
//! unit-tested against an in-memory DB; the async wrappers just move the work off
//! the async runtime (rusqlite is blocking).

use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::task::spawn_blocking;

use crate::domains::database_explorer::driver::DbDriver;
use crate::domains::database_explorer::types::{ColumnInfo, EditPreview, QueryResult, TableInfo};

/// Default cap on rows returned in one call, so a huge table never floods the UI.
pub const ROW_CAP: usize = 200;

/// Open a database file read-only. Cannot create or write — the P0 guarantee.
pub fn open_ro(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
}

/// Open a database file read-write (P2). Only used for the edit ops, and only
/// on sources the caller has already confirmed are editable — `fetch_rows`/
/// `list_tables`/`describe_table`/`run_query` always stay on `open_ro`.
pub fn open_rw(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
}

/// Quote an identifier for interpolation (table names come from sqlite_master,
/// but quote anyway so a `"` in a name can't break out).
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// JS `Number` is a float64 and loses precision above this (2^53) — serialize
/// a bigger integer (an unusual but legal SQLite PK) as a string instead, so
/// round-tripping it back as an edit's PK value doesn't silently hit the
/// wrong row.
const JS_MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_992;

fn value_to_json(v: rusqlite::types::ValueRef) -> Value {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) if i.abs() > JS_MAX_SAFE_INTEGER => Value::from(i.to_string()),
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => Value::from(f),
        ValueRef::Text(t) => Value::from(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Value::from(format!("<blob: {} bytes>", b.len())),
    }
}

pub fn list_tables(conn: &Connection) -> rusqlite::Result<Vec<TableInfo>> {
    let mut stmt = conn.prepare(
        "SELECT name, type FROM sqlite_master \
         WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%' \
         ORDER BY type, name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TableInfo {
            name: r.get(0)?,
            kind: r.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn describe_table(conn: &Connection, table: &str) -> rusqlite::Result<Vec<ColumnInfo>> {
    let mut out = Vec::new();
    conn.pragma(None, "table_info", table, |r| {
        out.push(ColumnInfo {
            name: r.get("name")?,
            data_type: r.get::<_, Option<String>>("type")?.unwrap_or_default(),
            not_null: r.get::<_, i64>("notnull")? != 0,
            pk: r.get::<_, i64>("pk")? != 0,
            default_value: r.get::<_, Option<String>>("dflt_value")?,
        });
        Ok(())
    })?;
    Ok(out)
}

/// Run a prepared statement and collect up to `cap` rows as JSON, flagging
/// truncation if more were available.
fn collect(
    stmt: &mut rusqlite::Statement,
    params: impl rusqlite::Params,
    cap: usize,
) -> rusqlite::Result<QueryResult> {
    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    let n = columns.len();
    let mut rows = stmt.query(params)?;
    let mut out: Vec<Vec<Value>> = Vec::new();
    let mut truncated = false;
    while let Some(row) = rows.next()? {
        if out.len() >= cap {
            truncated = true;
            break;
        }
        let mut rec = Vec::with_capacity(n);
        for i in 0..n {
            rec.push(value_to_json(row.get_ref(i)?));
        }
        out.push(rec);
    }
    Ok(QueryResult {
        row_count: out.len(),
        columns,
        rows: out,
        truncated,
    })
}

pub fn fetch_rows(
    conn: &Connection,
    table: &str,
    limit: usize,
    offset: usize,
) -> rusqlite::Result<QueryResult> {
    let cap = limit.clamp(1, ROW_CAP);
    let sql = format!("SELECT * FROM {} LIMIT ?1 OFFSET ?2", quote_ident(table));
    let mut stmt = conn.prepare(&sql)?;
    collect(
        &mut stmt,
        rusqlite::params![cap as i64, offset as i64],
        cap,
    )
}

pub fn run_query(conn: &Connection, sql: &str, cap: usize) -> rusqlite::Result<QueryResult> {
    let mut stmt = conn.prepare(sql)?;
    collect(&mut stmt, [], cap)
}

// --- edit-SQL builders (P2): pure, no DB — reused for both the confirm-dialog
// preview and (bound, below) the actual execution. ---

fn update_sql(table: &str, pk_column: &str, column: &str) -> String {
    format!(
        "UPDATE {} SET {} = ?1 WHERE {} = ?2",
        quote_ident(table),
        quote_ident(column),
        quote_ident(pk_column)
    )
}

fn insert_sql(table: &str, columns: &[String]) -> String {
    let cols: Vec<String> = columns.iter().map(|c| quote_ident(c)).collect();
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("?{i}")).collect();
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        quote_ident(table),
        cols.join(", "),
        placeholders.join(", ")
    )
}

fn delete_sql(table: &str, pk_column: &str) -> String {
    format!("DELETE FROM {} WHERE {} = ?1", quote_ident(table), quote_ident(pk_column))
}

pub fn preview_update(
    table: &str,
    pk_column: &str,
    pk_value: &Value,
    column: &str,
    new_value: &Value,
) -> EditPreview {
    EditPreview {
        sql: update_sql(table, pk_column, column),
        params: vec![new_value.clone(), pk_value.clone()],
    }
}

pub fn preview_insert(table: &str, values: &[(String, Value)]) -> EditPreview {
    let columns: Vec<String> = values.iter().map(|(c, _)| c.clone()).collect();
    EditPreview {
        sql: insert_sql(table, &columns),
        params: values.iter().map(|(_, v)| v.clone()).collect(),
    }
}

pub fn preview_delete(table: &str, pk_column: &str, pk_value: &Value) -> EditPreview {
    EditPreview {
        sql: delete_sql(table, pk_column),
        params: vec![pk_value.clone()],
    }
}

/// serde_json::Value -> a rusqlite bind param. Arrays/objects fall back to
/// their JSON text — sqlite has no native compound type anyway.
fn json_to_sql_param(v: &Value) -> rusqlite::types::Value {
    use rusqlite::types::Value as SqlValue;
    match v {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(*b as i64),
        Value::Number(n) => n
            .as_i64()
            .map(SqlValue::Integer)
            .unwrap_or_else(|| SqlValue::Real(n.as_f64().unwrap_or_default())),
        Value::String(s) => SqlValue::Text(s.clone()),
        other => SqlValue::Text(other.to_string()),
    }
}

/// Run one parameterized statement in an explicit transaction, rolling back
/// unless it affects exactly one row — the safety net for a PK that turns out
/// not to be unique (e.g. a composite key reduced to one column upstream).
fn exec_in_tx(conn: &Connection, sql: &str, params: &[rusqlite::types::Value]) -> rusqlite::Result<usize> {
    conn.execute("BEGIN", [])?;
    match conn.execute(sql, rusqlite::params_from_iter(params)) {
        Ok(n) if n == 1 => {
            conn.execute("COMMIT", [])?;
            Ok(n)
        }
        Ok(n) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(rusqlite::Error::StatementChangedRows(n))
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

pub fn update_cell(
    conn: &Connection,
    table: &str,
    pk_column: &str,
    pk_value: &Value,
    column: &str,
    new_value: &Value,
) -> rusqlite::Result<usize> {
    let sql = update_sql(table, pk_column, column);
    exec_in_tx(conn, &sql, &[json_to_sql_param(new_value), json_to_sql_param(pk_value)])
}

pub fn insert_row(conn: &Connection, table: &str, values: &[(String, Value)]) -> rusqlite::Result<usize> {
    let columns: Vec<String> = values.iter().map(|(c, _)| c.clone()).collect();
    let sql = insert_sql(table, &columns);
    let params: Vec<rusqlite::types::Value> = values.iter().map(|(_, v)| json_to_sql_param(v)).collect();
    exec_in_tx(conn, &sql, &params)
}

pub fn delete_row(conn: &Connection, table: &str, pk_column: &str, pk_value: &Value) -> rusqlite::Result<usize> {
    let sql = delete_sql(table, pk_column);
    exec_in_tx(conn, &sql, &[json_to_sql_param(pk_value)])
}

// --- async wrappers: open read-only off the runtime, run the pure fn ---

pub async fn tables(path: PathBuf) -> Result<Vec<TableInfo>, String> {
    spawn_blocking(move || {
        let conn = open_ro(&path).map_err(|e| e.to_string())?;
        list_tables(&conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn columns(path: PathBuf, table: String) -> Result<Vec<ColumnInfo>, String> {
    spawn_blocking(move || {
        let conn = open_ro(&path).map_err(|e| e.to_string())?;
        describe_table(&conn, &table).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn rows(
    path: PathBuf,
    table: String,
    limit: usize,
    offset: usize,
) -> Result<QueryResult, String> {
    spawn_blocking(move || {
        let conn = open_ro(&path).map_err(|e| e.to_string())?;
        fetch_rows(&conn, &table, limit, offset).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn query(path: PathBuf, sql: String) -> Result<QueryResult, String> {
    spawn_blocking(move || {
        let conn = open_ro(&path).map_err(|e| e.to_string())?;
        run_query(&conn, &sql, ROW_CAP).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- async wrappers (P2): open read-write, run the pure edit fn ---

pub async fn apply_update(
    path: PathBuf,
    table: String,
    pk_column: String,
    pk_value: Value,
    column: String,
    new_value: Value,
) -> Result<QueryResult, String> {
    spawn_blocking(move || {
        let conn = open_rw(&path).map_err(|e| e.to_string())?;
        let n = update_cell(&conn, &table, &pk_column, &pk_value, &column, &new_value)
            .map_err(|e| e.to_string())?;
        Ok(QueryResult::affected(n))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn apply_insert(
    path: PathBuf,
    table: String,
    values: Vec<(String, Value)>,
) -> Result<QueryResult, String> {
    spawn_blocking(move || {
        let conn = open_rw(&path).map_err(|e| e.to_string())?;
        let n = insert_row(&conn, &table, &values).map_err(|e| e.to_string())?;
        Ok(QueryResult::affected(n))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn apply_delete(
    path: PathBuf,
    table: String,
    pk_column: String,
    pk_value: Value,
) -> Result<QueryResult, String> {
    spawn_blocking(move || {
        let conn = open_rw(&path).map_err(|e| e.to_string())?;
        let n = delete_row(&conn, &table, &pk_column, &pk_value).map_err(|e| e.to_string())?;
        Ok(QueryResult::affected(n))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Read-only SQLite driver: opens a fresh read-only connection per call (opens
/// are cheap) on a blocking thread.
pub struct SqliteDriver {
    pub path: PathBuf,
}

#[async_trait]
impl DbDriver for SqliteDriver {
    async fn list_tables(&self) -> Result<Vec<TableInfo>, String> {
        tables(self.path.clone()).await
    }
    async fn describe_table(&self, table: &str) -> Result<Vec<ColumnInfo>, String> {
        columns(self.path.clone(), table.to_string()).await
    }
    async fn fetch_rows(
        &self,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<QueryResult, String> {
        rows(self.path.clone(), table.to_string(), limit, offset).await
    }
    async fn run_query(&self, sql: &str) -> Result<QueryResult, String> {
        query(self.path.clone(), sql.to_string()).await
    }
    async fn update_cell(
        &self,
        table: &str,
        pk_column: &str,
        pk_value: Value,
        column: &str,
        new_value: Value,
    ) -> Result<QueryResult, String> {
        apply_update(
            self.path.clone(),
            table.to_string(),
            pk_column.to_string(),
            pk_value,
            column.to_string(),
            new_value,
        )
        .await
    }
    async fn insert_row(&self, table: &str, values: Vec<(String, Value)>) -> Result<QueryResult, String> {
        apply_insert(self.path.clone(), table.to_string(), values).await
    }
    async fn delete_row(&self, table: &str, pk_column: &str, pk_value: Value) -> Result<QueryResult, String> {
        apply_delete(self.path.clone(), table.to_string(), pk_column.to_string(), pk_value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE t(id INTEGER PRIMARY KEY, name TEXT NOT NULL, amt REAL, note TEXT);\
             CREATE VIEW v AS SELECT id FROM t;\
             INSERT INTO t(name, amt, note) VALUES ('a', 1.5, NULL), ('b', 2.0, 'hi');",
        )
        .unwrap();
        c
    }

    #[test]
    fn lists_tables_and_views_but_not_internal() {
        let c = seed();
        let t = list_tables(&c).unwrap();
        // ordered by type then name: table 't' before view 'v'
        assert_eq!(t.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["t", "v"]);
        assert_eq!(t[0].kind, "table");
        assert_eq!(t[1].kind, "view");
    }

    #[test]
    fn describes_pk_notnull_and_nullable_default() {
        let c = seed();
        let cols = describe_table(&c, "t").unwrap();
        assert_eq!(cols[0].name, "id");
        assert!(cols[0].pk);
        assert!(cols[1].not_null); // name TEXT NOT NULL
        assert!(!cols[2].not_null); // amt nullable
        assert_eq!(cols[2].data_type, "REAL");
    }

    #[test]
    fn fetches_rows_mapping_sql_types_to_json() {
        let c = seed();
        let r = fetch_rows(&c, "t", 10, 0).unwrap();
        assert_eq!(r.row_count, 2);
        assert_eq!(r.rows[0][1], Value::from("a")); // text
        assert_eq!(r.rows[0][2], Value::from(1.5)); // real
        assert_eq!(r.rows[0][3], Value::Null); // null
        assert!(!r.truncated);
    }

    #[test]
    fn fetch_respects_offset() {
        let c = seed();
        let r = fetch_rows(&c, "t", 10, 1).unwrap();
        assert_eq!(r.row_count, 1);
        assert_eq!(r.rows[0][1], Value::from("b"));
    }

    #[test]
    fn run_query_truncates_at_cap() {
        let c = seed();
        let r = run_query(&c, "SELECT 1 UNION ALL SELECT 2 UNION ALL SELECT 3", 2).unwrap();
        assert_eq!(r.rows.len(), 2);
        assert!(r.truncated);
    }

    #[test]
    fn quotes_identifiers_safely() {
        assert_eq!(quote_ident("plain"), "\"plain\"");
        assert_eq!(quote_ident("we\"ird"), "\"we\"\"ird\"");
    }

    #[test]
    fn preview_update_builds_parameterized_sql() {
        let p = preview_update("t", "id", &Value::from(2), "name", &Value::from("b2"));
        assert_eq!(p.sql, "UPDATE \"t\" SET \"name\" = ?1 WHERE \"id\" = ?2");
        assert_eq!(p.params, vec![Value::from("b2"), Value::from(2)]);
    }

    #[test]
    fn preview_insert_builds_column_and_placeholder_lists() {
        let values = vec![("name".to_string(), Value::from("c")), ("amt".to_string(), Value::from(3.0))];
        let p = preview_insert("t", &values);
        assert_eq!(p.sql, "INSERT INTO \"t\" (\"name\", \"amt\") VALUES (?1, ?2)");
        assert_eq!(p.params, vec![Value::from("c"), Value::from(3.0)]);
    }

    #[test]
    fn preview_delete_builds_pk_where_clause() {
        let p = preview_delete("t", "id", &Value::from(1));
        assert_eq!(p.sql, "DELETE FROM \"t\" WHERE \"id\" = ?1");
        assert_eq!(p.params, vec![Value::from(1)]);
    }

    #[test]
    fn update_cell_mutates_the_matched_row_only() {
        let c = seed();
        let n = update_cell(&c, "t", "id", &Value::from(1), "name", &Value::from("z")).unwrap();
        assert_eq!(n, 1);
        let r = fetch_rows(&c, "t", 10, 0).unwrap();
        assert_eq!(r.rows[0][1], Value::from("z"));
        assert_eq!(r.rows[1][1], Value::from("b"));
    }

    #[test]
    fn insert_row_adds_a_new_row() {
        let c = seed();
        let values = vec![("name".to_string(), Value::from("c")), ("amt".to_string(), Value::from(9.0))];
        let n = insert_row(&c, "t", &values).unwrap();
        assert_eq!(n, 1);
        assert_eq!(fetch_rows(&c, "t", 10, 0).unwrap().row_count, 3);
    }

    #[test]
    fn delete_row_removes_the_matched_row_only() {
        let c = seed();
        let n = delete_row(&c, "t", "id", &Value::from(1)).unwrap();
        assert_eq!(n, 1);
        let r = fetch_rows(&c, "t", 10, 0).unwrap();
        assert_eq!(r.row_count, 1);
        assert_eq!(r.rows[0][1], Value::from("b"));
    }

    #[test]
    fn exec_in_tx_rolls_back_on_error_leaving_data_untouched() {
        let c = seed();
        let err = exec_in_tx(&c, "UPDATE \"t\" SET \"nope\" = ?1 WHERE \"id\" = ?2", &[
            rusqlite::types::Value::Text("x".into()),
            rusqlite::types::Value::Integer(1),
        ]);
        assert!(err.is_err());
        // a failed tx must not leave a dangling BEGIN — a following write should still work.
        assert_eq!(update_cell(&c, "t", "id", &Value::from(1), "name", &Value::from("ok")).unwrap(), 1);
    }
}
