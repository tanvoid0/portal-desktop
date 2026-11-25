//! Building, sealing, opening and staging DB snapshots. Pure file/crypto work, no
//! network and no pool: everything here is blocking and called via `spawn_blocking`.
//!
//! Wire format of a sealed snapshot: `"PSNAP1" | salt(16) | nonce(12) | AES-256-GCM(gzip(sqlite))+tag`,
//! key = PBKDF2-HMAC-SHA256(password, salt, 600k) -- the same derivation the vault uses.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use rusqlite::{Connection, OpenFlags};
use sea_orm_migration::MigratorTrait;

use super::paths;
use crate::domains::credentials::services::encryption_service::{
    EncryptionService, PBKDF2_ITERATIONS,
};
use crate::log_warn;

pub const MAGIC: &[u8; 6] = b"PSNAP1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const DB_FILE: &str = "portal_desktop.db";
const PENDING_SUFFIX: &str = "restore-pending";
const BACKUP_SUFFIX: &str = "pre-restore.bak";
const META_SUFFIX: &str = "restore-meta";

/// Sync state to commit to `sync_state.json` once the staged restore is actually live. Kept
/// beside the pending file so a pull that never reaches the swap never moves `base_version`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RestoreMeta {
    pub base_version: u32,
    pub last_synced_at: Option<String>,
}

/// Log/history/cache tables emptied in the snapshot copy (rows only; the schema stays so
/// the restored DB matches the migrations table). Big, machine-local, regenerable.
/// Everything not listed -- including `vault`, `credentials`, projects, tasks -- is kept,
/// so a table added later syncs by default.
pub const VOLATILE_TABLES: &[&str] = &[
    "agent_events",             // per-session agent event stream
    "ai_logs",                  // AI request/response log
    "ai_training_data",         // collected prompt/response pairs
    "script_executions",        // action run history + captured output
    "pipeline_executions",      // pipeline run history + captured output
    "terminal_command_history", // shell history
    "learning_events",          // learning telemetry
];

pub fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DB_FILE)
}

fn side_path(data_dir: &Path, suffix: &str) -> PathBuf {
    data_dir.join(format!("{DB_FILE}.{suffix}"))
}

pub fn pending_path(data_dir: &Path) -> PathBuf {
    side_path(data_dir, PENDING_SUFFIX)
}

fn err<E: std::fmt::Display>(ctx: &'static str) -> impl FnOnce(E) -> String {
    move |e| format!("{ctx}: {e}")
}

/// VACUUM INTO a temp copy of the live DB, record this device's project paths in the copy,
/// empty the volatile tables, re-VACUUM, gzip.
pub fn build_gzipped_copy(live: &Path, device_id: &str) -> Result<Vec<u8>, String> {
    let tmp = live.with_extension(format!("sync-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        {
            let src = Connection::open_with_flags(live, OpenFlags::SQLITE_OPEN_READ_WRITE)
                .map_err(err("open live db"))?;
            src.busy_timeout(std::time::Duration::from_secs(10)).ok();
            src.execute("VACUUM INTO ?1", [tmp.to_string_lossy().as_ref()])
                .map_err(err("snapshot live db"))?;
        }
        let copy = Connection::open(&tmp).map_err(err("open snapshot copy"))?;
        paths::record_device_paths(&copy, device_id).map_err(err("record project paths"))?;
        for t in VOLATILE_TABLES {
            let exists: bool = copy
                .query_row(
                    "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                    [t],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if exists {
                copy.execute(&format!("DELETE FROM \"{t}\""), [])
                    .map_err(err("trim snapshot"))?;
            }
        }
        copy.execute_batch("VACUUM").map_err(err("compact snapshot"))?;
        copy.close().map_err(|(_, e)| format!("close snapshot copy: {e}"))?;
        gzip(&std::fs::read(&tmp).map_err(err("read snapshot copy"))?)
    })();
    let _ = std::fs::remove_file(&tmp);
    result
}

fn gzip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).map_err(err("gzip"))?;
    enc.finish().map_err(err("gzip"))
}

fn key_for(password: &str, salt: &[u8]) -> Result<LessSafeKey, String> {
    let k = EncryptionService::new()
        .derive_key(password, salt, PBKDF2_ITERATIONS)
        .map_err(|e| e.to_string())?;
    let k = UnboundKey::new(&AES_256_GCM, &k).map_err(|_| "bad key".to_string())?;
    Ok(LessSafeKey::new(k))
}

pub fn seal(plain: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let svc = EncryptionService::new();
    let salt = svc.generate_bytes(SALT_LEN).map_err(|e| e.to_string())?;
    let nonce = svc.generate_bytes(NONCE_LEN).map_err(|e| e.to_string())?;
    let key = key_for(password, &salt)?;
    let mut buf = plain.to_vec();
    key.seal_in_place_append_tag(
        Nonce::try_assume_unique_for_key(&nonce).map_err(|_| "bad nonce".to_string())?,
        Aad::from(MAGIC),
        &mut buf,
    )
    .map_err(|_| "encryption failed".to_string())?;
    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + buf.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&buf);
    Ok(out)
}

pub fn open(blob: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let head = MAGIC.len() + SALT_LEN + NONCE_LEN;
    if blob.len() < head + 16 || &blob[..MAGIC.len()] != MAGIC {
        return Err("Not a Portal snapshot (unrecognised format)".into());
    }
    let salt = &blob[MAGIC.len()..MAGIC.len() + SALT_LEN];
    let nonce = Nonce::try_assume_unique_for_key(&blob[MAGIC.len() + SALT_LEN..head])
        .map_err(|_| "bad nonce".to_string())?;
    let mut buf = blob[head..].to_vec();
    let plain = key_for(password, salt)?
        .open_in_place(nonce, Aad::from(MAGIC), &mut buf)
        .map_err(|_| "Wrong password, or the snapshot is corrupted".to_string())?;
    Ok(plain.to_vec())
}

/// Full push-side pipeline: live db -> trimmed copy -> gzip -> sealed bytes.
pub fn build_snapshot(live: &Path, device_id: &str, password: &str) -> Result<Vec<u8>, String> {
    seal(&build_gzipped_copy(live, device_id)?, password)
}

/// Check a (decrypted, gunzipped) SQLite file is a restorable Portal DB.
pub fn sanity_check(path: &Path) -> Result<(), String> {
    let c = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(err("snapshot is not a SQLite database"))?;
    let verdict: String = c
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(err("snapshot is not a SQLite database"))?;
    if verdict != "ok" {
        return Err(format!("Snapshot failed integrity check: {verdict}"));
    }
    let has = |t: &str| -> bool {
        c.query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
            [t],
            |_| Ok(true),
        )
        .unwrap_or(false)
    };
    if !has("vault") {
        return Err("Snapshot has no vault table".into());
    }
    if !has("seaql_migrations") {
        return Err("Snapshot has no migrations table".into());
    }
    // A snapshot from a newer app would make boot-time migrations fail on a missing file.
    let known: Vec<String> = crate::migrations::Migrator::migrations()
        .iter()
        .map(|m| m.name().to_string())
        .collect();
    let mut stmt = c
        .prepare("SELECT version FROM seaql_migrations")
        .map_err(err("read migrations"))?;
    let versions = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(err("read migrations"))?;
    for v in versions {
        let v = v.map_err(err("read migrations"))?;
        if !known.contains(&v) {
            return Err(format!(
                "Snapshot is from a newer Portal version (migration {v}); update this app first"
            ));
        }
    }
    Ok(())
}

/// Decrypt + gunzip + sanity-check + resolve project paths for `device_id`, then stage as
/// `<db>.restore-pending` (plus its `meta`) for the next boot. Returns the unresolved count.
pub fn stage_restore(
    data_dir: &Path,
    blob: &[u8],
    password: &str,
    device_id: &str,
    meta: &RestoreMeta,
) -> Result<u32, String> {
    let gz = open(blob, password)?;
    let mut raw = Vec::new();
    GzDecoder::new(&gz[..])
        .read_to_end(&mut raw)
        .map_err(err("gunzip"))?;
    let tmp = side_path(data_dir, &format!("restore-incoming-{}", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, &raw).map_err(err("write staged snapshot"))?;
    let staged = sanity_check(&tmp).and_then(|_| paths::resolve_for_device(&tmp, device_id));
    match staged {
        Ok(unresolved) => {
            let meta_path = side_path(data_dir, META_SUFFIX);
            let staged = serde_json::to_string(meta)
                .map_err(err("restore marker"))
                .and_then(|j| std::fs::write(&meta_path, j).map_err(err("write restore marker")))
                .and_then(|_| {
                    std::fs::rename(&tmp, pending_path(data_dir)).map_err(err("stage snapshot"))
                });
            if let Err(e) = staged {
                let _ = std::fs::remove_file(&tmp);
                let _ = std::fs::remove_file(&meta_path);
                return Err(e);
            }
            Ok(unresolved)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Boot-time swap, before any DB pool exists. Old DB (and its -wal/-shm) is kept as
/// `<db>.pre-restore.bak[-wal|-shm]`. Only a successful swap commits the staged sync state.
pub fn apply_pending_restore(data_dir: &Path) {
    let pending = pending_path(data_dir);
    if !pending.exists() {
        return;
    }
    let live = db_path(data_dir);
    let bak = side_path(data_dir, BACKUP_SUFFIX);
    let had_live = live.exists();
    // (live-side, backup-side) for the WAL companions.
    let companions: Vec<(PathBuf, PathBuf)> = ["-wal", "-shm"]
        .iter()
        .map(|x| {
            (
                data_dir.join(format!("{DB_FILE}{x}")),
                side_path(data_dir, &format!("{BACKUP_SUFFIX}{x}")),
            )
        })
        .collect();
    if had_live {
        let _ = std::fs::remove_file(&bak);
        if let Err(e) = std::fs::rename(&live, &bak) {
            log_warn!("Sync", "restore skipped, cannot move live db aside: {}", e);
            return;
        }
    }
    for (l, b) in &companions {
        let _ = std::fs::remove_file(b);
        if l.exists() && std::fs::rename(l, b).is_err() {
            // A WAL left next to the new DB would corrupt it; drop it as a last resort.
            let _ = std::fs::remove_file(l);
        }
    }
    if let Err(e) = std::fs::rename(&pending, &live) {
        log_warn!("Sync", "restore failed, rolling back: {}", e);
        if had_live {
            let _ = std::fs::rename(&bak, &live);
            for (l, b) in &companions {
                let _ = std::fs::rename(b, l);
            }
        }
        return;
    }
    // The restored vault wraps another device's master key; make Hello unlock refuse until
    // a password unlock re-seeds this device's keychain.
    let _ = std::fs::write(
        data_dir.join(crate::domains::credentials::services::vault_service::KEYCHAIN_STALE_FLAG),
        b"1",
    );
    let meta_path = side_path(data_dir, META_SUFFIX);
    if let Some(meta) = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|t| serde_json::from_str::<RestoreMeta>(&t).ok())
    {
        match super::service::commit_restore_state(data_dir, &meta) {
            Ok(()) => {
                let _ = std::fs::remove_file(&meta_path);
            }
            Err(e) => log_warn!("Sync", "restored, but could not record sync state: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_db(path: &Path) {
        let c = Connection::open(path).unwrap();
        c.execute_batch(
            "CREATE TABLE vault (id INTEGER PRIMARY KEY, blob TEXT);
             INSERT INTO vault VALUES (1, 'secret');
             CREATE TABLE seaql_migrations (version TEXT PRIMARY KEY, applied_at INTEGER);
             CREATE TABLE terminal_command_history (id INTEGER PRIMARY KEY, cmd TEXT);
             INSERT INTO terminal_command_history VALUES (1, 'ls');
             CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT);
             INSERT INTO projects VALUES (1, 'p');",
        )
        .unwrap();
    }

    #[test]
    fn roundtrip_wrong_password_and_tamper() {
        let sealed = seal(b"hello", "pw").unwrap();
        assert_eq!(open(&sealed, "pw").unwrap(), b"hello");
        assert!(open(&sealed, "nope").unwrap_err().contains("Wrong password"));
        let mut bad = sealed.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert!(open(&bad, "pw").is_err());
        assert!(open(b"short", "pw").is_err());
    }

    #[test]
    fn snapshot_drops_log_rows_keeps_vault() {
        let dir = tempfile::tempdir().unwrap();
        let live = db_path(dir.path());
        fake_db(&live);
        let blob = build_snapshot(&live, "dev", "pw").unwrap();

        let gz = open(&blob, "pw").unwrap();
        let mut raw = Vec::new();
        GzDecoder::new(&gz[..]).read_to_end(&mut raw).unwrap();
        let out = dir.path().join("out.db");
        std::fs::write(&out, raw).unwrap();
        let c = Connection::open(&out).unwrap();
        let n = |t: &str| -> i64 {
            c.query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(n("terminal_command_history"), 0);
        assert_eq!(n("vault"), 1);
        assert_eq!(n("projects"), 1);
        // Live DB untouched, temp copy cleaned up.
        let live_c = Connection::open(&live).unwrap();
        let cnt: i64 = live_c
            .query_row("SELECT count(*) FROM terminal_command_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cnt, 1);
        assert!(std::fs::read_dir(dir.path())
            .unwrap()
            .all(|e| !e.unwrap().file_name().to_string_lossy().contains("sync-")));
    }

    #[test]
    fn sanity_rejects_missing_vault_and_pending_swap_keeps_backup() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.db");
        Connection::open(&bad)
            .unwrap()
            .execute_batch("CREATE TABLE x(a)")
            .unwrap();
        assert!(sanity_check(&bad).unwrap_err().contains("vault"));

        std::fs::write(db_path(dir.path()), b"old").unwrap();
        std::fs::write(pending_path(dir.path()), b"new").unwrap();
        let wal = |x: &str| dir.path().join(format!("{DB_FILE}{x}"));
        std::fs::write(wal("-wal"), b"oldwal").unwrap();
        std::fs::write(wal("-shm"), b"oldshm").unwrap();
        std::fs::write(
            side_path(dir.path(), META_SUFFIX),
            r#"{"base_version":7,"last_synced_at":"t"}"#,
        )
        .unwrap();
        apply_pending_restore(dir.path());
        // Old WAL/SHM are kept beside the backup, not deleted and not left next to the new db.
        assert!(!wal("-wal").exists() && !wal("-shm").exists());
        assert_eq!(
            std::fs::read(side_path(dir.path(), "pre-restore.bak-wal")).unwrap(),
            b"oldwal"
        );
        assert_eq!(
            std::fs::read(side_path(dir.path(), "pre-restore.bak-shm")).unwrap(),
            b"oldshm"
        );
        // Sync state is committed only now that the swap happened.
        let st = std::fs::read_to_string(dir.path().join("sync_state.json")).unwrap();
        assert!(st.contains("\"base_version\": 7"), "{st}");
        assert!(!side_path(dir.path(), META_SUFFIX).exists());
        assert_eq!(std::fs::read(db_path(dir.path())).unwrap(), b"new");
        assert_eq!(
            std::fs::read(side_path(dir.path(), BACKUP_SUFFIX)).unwrap(),
            b"old"
        );
        assert!(!pending_path(dir.path()).exists());
    }
}
