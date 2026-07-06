use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domains::disk::scan::now_secs;

/// Local durability layer: audit log, custom protected paths, and a scan cache.
/// Read-only with respect to user files — this DB only ever records metadata.
///
/// Kept on `rusqlite` (its own `disk_utility.db`) rather than the desktop's
/// sea-orm layer for a low-risk first cut. Can be ported to sea-orm entities
/// later — see `docs/development/DISK_UTILITY_MIGRATION.md`.
pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: i64,
    pub ts: i64,
    pub action: String,
    pub path: String,
    pub size_bytes: i64,
    pub kind: String,
    pub status: String,
}

impl Db {
    /// Opens (or creates) the SQLite database at the given path and applies the schema.
    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS audit (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                ts         INTEGER NOT NULL,
                action     TEXT    NOT NULL,
                path       TEXT    NOT NULL,
                size_bytes INTEGER NOT NULL,
                kind       TEXT    NOT NULL,
                status     TEXT    NOT NULL
            );
            CREATE TABLE IF NOT EXISTS protected (
                path     TEXT PRIMARY KEY,
                added_ts INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS scan_cache (
                root          TEXT PRIMARY KEY,
                ts            INTEGER NOT NULL,
                summary_json  TEXT    NOT NULL,
                status        TEXT    NOT NULL DEFAULT 'complete',
                scanned_files INTEGER NOT NULL DEFAULT 0,
                total_files   INTEGER NOT NULL DEFAULT 0,
                elapsed_ms    INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .map_err(|e| e.to_string())?;
        // Bring pre-existing scan_cache tables up to the current shape. Each
        // ALTER fails harmlessly ("duplicate column") once already applied.
        for alter in [
            "ALTER TABLE scan_cache ADD COLUMN status TEXT NOT NULL DEFAULT 'complete'",
            "ALTER TABLE scan_cache ADD COLUMN scanned_files INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE scan_cache ADD COLUMN total_files INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE scan_cache ADD COLUMN elapsed_ms INTEGER NOT NULL DEFAULT 0",
        ] {
            let _ = conn.execute(alter, []);
        }
        Ok(Db {
            conn: Mutex::new(conn),
        })
    }

    // --- Audit log -------------------------------------------------------

    /// Records one quarantine outcome. `status` is "moved" or "failed".
    pub fn log_action(
        &self,
        action: &str,
        path: &str,
        size_bytes: u64,
        kind: &str,
        status: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO audit (ts, action, path, size_bytes, kind, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                now_secs() as i64,
                action,
                path,
                size_bytes as i64,
                kind,
                status
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Returns the most recent audit entries, newest first.
    pub fn audit_log(&self, limit: u32) -> Result<Vec<AuditEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, ts, action, path, size_bytes, kind, status
                 FROM audit ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok(AuditEntry {
                    id: r.get(0)?,
                    ts: r.get(1)?,
                    action: r.get(2)?,
                    path: r.get(3)?,
                    size_bytes: r.get(4)?,
                    kind: r.get(5)?,
                    status: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    // --- Custom protected paths -----------------------------------------

    pub fn list_protected(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT path FROM protected ORDER BY path")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn add_protected(&self, path: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO protected (path, added_ts) VALUES (?1, ?2)",
            params![path, now_secs() as i64],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_protected(&self, path: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM protected WHERE path = ?1", params![path])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Scan cache ------------------------------------------------------

    /// Stores a finished scan summary for a root so it can be shown instantly
    /// on reopen without re-walking the tree. Status = `complete`.
    pub fn cache_scan(
        &self,
        root: &str,
        summary_json: &str,
        scanned_files: u64,
        total_files: u64,
        elapsed_ms: u128,
    ) -> Result<(), String> {
        self.write_cache(
            root,
            summary_json,
            "complete",
            scanned_files,
            total_files,
            elapsed_ms,
        )
    }

    /// Checkpoints a still-running (or cancelled) scan so the partial results
    /// survive an interrupt, crash, or app close. Status = `partial`.
    pub fn cache_partial(
        &self,
        root: &str,
        summary_json: &str,
        scanned_files: u64,
        total_files: u64,
        elapsed_ms: u128,
    ) -> Result<(), String> {
        self.write_cache(
            root,
            summary_json,
            "partial",
            scanned_files,
            total_files,
            elapsed_ms,
        )
    }

    fn write_cache(
        &self,
        root: &str,
        summary_json: &str,
        status: &str,
        scanned_files: u64,
        total_files: u64,
        elapsed_ms: u128,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO scan_cache
                 (root, ts, summary_json, status, scanned_files, total_files, elapsed_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                root,
                now_secs() as i64,
                summary_json,
                status,
                scanned_files as i64,
                total_files as i64,
                elapsed_ms as i64,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Returns the cached scan for a root, if present.
    pub fn cached_scan(&self, root: &str) -> Result<Option<CachedRow>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let res = conn
            .query_row(
                "SELECT ts, status, scanned_files, total_files, summary_json
                 FROM scan_cache WHERE root = ?1",
                params![root],
                |r| {
                    Ok(CachedRow {
                        ts: r.get(0)?,
                        status: r.get(1)?,
                        scanned_files: r.get(2)?,
                        total_files: r.get(3)?,
                        summary_json: r.get(4)?,
                    })
                },
            )
            .ok();
        Ok(res)
    }

    /// Drops a root's cached scan — the "Discard" action in the restore card.
    pub fn remove_cached_scan(&self, root: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM scan_cache WHERE root = ?1", params![root])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// One row of the scan cache — a finished (`complete`) or interrupted
/// (`partial`) scan that can be restored on the next run.
pub struct CachedRow {
    pub ts: i64,
    pub status: String,
    pub scanned_files: i64,
    pub total_files: i64,
    pub summary_json: String,
}
