//! SQLite introspection + read-only queries via rusqlite on a blocking thread.
//!
//! Connections are opened READ-ONLY, so nothing here can mutate a database — the
//! P0 safety guarantee. The pure `*(conn)` functions carry the real logic and are
//! unit-tested against an in-memory DB; the async wrappers just move the work off
//! the async runtime (rusqlite is blocking).

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::task::spawn_blocking;

use crate::domains::database_explorer::types::{ColumnInfo, QueryResult, TableInfo};

/// Default cap on rows returned in one call, so a huge table never floods the UI.
pub const ROW_CAP: usize = 200;

/// Open a database file read-only. Cannot create or write — the P0 guarantee.
pub fn open_ro(path: &Path) -> rusqlite::Result<Connection> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
}

/// Quote an identifier for interpolation (table names come from sqlite_master,
/// but quote anyway so a `"` in a name can't break out).
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn value_to_json(v: rusqlite::types::ValueRef) -> Value {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => Value::Null,
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
}
