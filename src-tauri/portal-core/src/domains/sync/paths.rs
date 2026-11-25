//! Per-device project paths (`project_device_paths`). A project's path / git / port fields are
//! machine-local, so each device keeps its own row and never touches another device's.
//! Unresolved on this machine = `projects.path = ''` and no `ignored` row for this device.
//! All functions are blocking rusqlite; they work on the live DB or a staged snapshot file.

use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

pub const RESOLVED: &str = "resolved";
pub const IGNORED: &str = "ignored";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UnresolvedProject {
    pub project_id: i64,
    pub name: String,
    pub git_remote: Option<String>,
    pub last_known_path: Option<String>,
    pub last_known_device_id: Option<String>,
}

fn has_table(c: &Connection) -> bool {
    c.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='project_device_paths'",
        [],
        |_| Ok(true),
    )
    .unwrap_or(false)
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Push side: upsert this device's row for every project that has a path here.
pub fn record_device_paths(c: &Connection, device_id: &str) -> rusqlite::Result<()> {
    if !has_table(c) {
        return Ok(());
    }
    c.execute(
        "INSERT INTO project_device_paths
           (project_id, device_id, path, git_repository, git_branch, git_commit, dev_port, discovery_path, status, updated_at)
         SELECT id, ?1, path, git_repository, git_branch, git_commit, dev_port, discovery_path, 'resolved', ?2
           FROM projects WHERE path <> ''
         ON CONFLICT(project_id, device_id) DO UPDATE SET
           path=excluded.path, git_repository=excluded.git_repository, git_branch=excluded.git_branch,
           git_commit=excluded.git_commit, dev_port=excluded.dev_port,
           discovery_path=excluded.discovery_path, status='resolved', updated_at=excluded.updated_at",
        params![device_id, now()],
    )?;
    Ok(())
}

/// Restore side, run on the staged file: apply this device's own rows; otherwise keep the
/// path only if it exists here; otherwise clear it. Returns the unresolved count.
pub fn resolve_for_device(file: &Path, device_id: &str) -> Result<u32, String> {
    let c = Connection::open(file).map_err(|e| format!("open staged snapshot: {e}"))?;
    if !has_table(&c) {
        return Ok(0);
    }
    let e = |x: rusqlite::Error| format!("resolve project paths: {x}");
    let projects: Vec<(i64, String)> = c
        .prepare("SELECT id, path FROM projects")
        .map_err(e)?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    for (id, path) in projects {
        let own: Option<String> = c
            .query_row(
                "SELECT status FROM project_device_paths WHERE project_id=?1 AND device_id=?2",
                params![id, device_id],
                |r| r.get(0),
            )
            .ok();
        match own.as_deref() {
            Some(IGNORED) => {
                c.execute("UPDATE projects SET path='' WHERE id=?1", [id]).map_err(e)?;
            }
            Some(_) => {
                c.execute(
                    "UPDATE projects SET
                       path=(SELECT path FROM project_device_paths WHERE project_id=?1 AND device_id=?2),
                       git_repository=(SELECT git_repository FROM project_device_paths WHERE project_id=?1 AND device_id=?2),
                       git_branch=(SELECT git_branch FROM project_device_paths WHERE project_id=?1 AND device_id=?2),
                       git_commit=(SELECT git_commit FROM project_device_paths WHERE project_id=?1 AND device_id=?2),
                       dev_port=(SELECT dev_port FROM project_device_paths WHERE project_id=?1 AND device_id=?2),
                       discovery_path=(SELECT discovery_path FROM project_device_paths WHERE project_id=?1 AND device_id=?2)
                     WHERE id=?1",
                    params![id, device_id],
                )
                .map_err(e)?;
            }
            None => {
                // ponytail: same-path check only; folder-name/git-remote matching against
                // discovery roots is left to the user-triggered discovery + manual resolve.
                if !path.is_empty() && Path::new(&path).is_dir() {
                    upsert_row(&c, id, device_id, RESOLVED).map_err(e)?;
                } else {
                    c.execute("UPDATE projects SET path='' WHERE id=?1", [id]).map_err(e)?;
                }
            }
        }
    }
    unresolved_count(&c, device_id).map_err(e)
}

fn unresolved_count(c: &Connection, device_id: &str) -> rusqlite::Result<u32> {
    c.query_row(
        "SELECT count(*) FROM projects p WHERE p.path = '' AND NOT EXISTS
           (SELECT 1 FROM project_device_paths d WHERE d.project_id=p.id AND d.device_id=?1 AND d.status='ignored')",
        [device_id],
        |r| r.get(0),
    )
}

/// Row for this device from the project's current fields (status given).
fn upsert_row(c: &Connection, id: i64, device_id: &str, status: &str) -> rusqlite::Result<()> {
    c.execute(
        "INSERT INTO project_device_paths
           (project_id, device_id, path, git_repository, git_branch, git_commit, dev_port, discovery_path, status, updated_at)
         SELECT id, ?2, path, git_repository, git_branch, git_commit, dev_port, discovery_path, ?3, ?4
           FROM projects WHERE id=?1
         ON CONFLICT(project_id, device_id) DO UPDATE SET
           path=excluded.path, git_repository=excluded.git_repository, git_branch=excluded.git_branch,
           git_commit=excluded.git_commit, dev_port=excluded.dev_port,
           discovery_path=excluded.discovery_path, status=excluded.status, updated_at=excluded.updated_at",
        params![id, device_id, status, now()],
    )?;
    Ok(())
}

pub fn list_unresolved(live: &Path, device_id: &str) -> Result<Vec<UnresolvedProject>, String> {
    let c = Connection::open(live).map_err(|e| e.to_string())?;
    if !has_table(&c) {
        return Ok(vec![]);
    }
    let e = |x: rusqlite::Error| x.to_string();
    // Most recently updated other-device row supplies the "last known" hint.
    let mut stmt = c
        .prepare(
            "SELECT p.id, p.name, p.git_repository,
               (SELECT d.path FROM project_device_paths d WHERE d.project_id=p.id AND d.device_id<>?1 AND d.status='resolved' ORDER BY d.updated_at DESC LIMIT 1),
               (SELECT d.device_id FROM project_device_paths d WHERE d.project_id=p.id AND d.device_id<>?1 AND d.status='resolved' ORDER BY d.updated_at DESC LIMIT 1)
             FROM projects p WHERE p.path = '' AND NOT EXISTS
               (SELECT 1 FROM project_device_paths d WHERE d.project_id=p.id AND d.device_id=?1 AND d.status='ignored')
             ORDER BY p.name",
        )
        .map_err(e)?;
    let rows = stmt
        .query_map([device_id], |r| {
            Ok(UnresolvedProject {
                project_id: r.get(0)?,
                name: r.get(1)?,
                git_remote: r.get(2)?,
                last_known_path: r.get(3)?,
                last_known_device_id: r.get(4)?,
            })
        })
        .map_err(e)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(e)?;
    Ok(rows)
}

/// Point a project at a local folder: updates `projects.path` and this device's row.
pub fn resolve_project(live: &Path, device_id: &str, project_id: i64, path: &str) -> Result<(), String> {
    if !Path::new(path).is_dir() {
        return Err(format!("Not a directory: {path}"));
    }
    let c = Connection::open(live).map_err(|e| e.to_string())?;
    c.busy_timeout(std::time::Duration::from_secs(5)).ok();
    let n = c
        .execute("UPDATE projects SET path=?2 WHERE id=?1", params![project_id, path])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Project not found".into());
    }
    upsert_row(&c, project_id, device_id, RESOLVED).map_err(|e| e.to_string())
}

/// Mark unresolved-but-silent on this device (the project keeps its empty path).
pub fn ignore_project(live: &Path, device_id: &str, project_id: i64) -> Result<(), String> {
    let c = Connection::open(live).map_err(|e| e.to_string())?;
    c.busy_timeout(std::time::Duration::from_secs(5)).ok();
    c.execute(
        "INSERT INTO project_device_paths (project_id, device_id, path, status, updated_at)
         VALUES (?1, ?2, '', 'ignored', ?3)
         ON CONFLICT(project_id, device_id) DO UPDATE SET path='', status='ignored', updated_at=excluded.updated_at",
        params![project_id, device_id, now()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(file: &Path, path: &str) {
        let c = Connection::open(file).unwrap();
        c.execute_batch(
            "CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT, path TEXT NOT NULL, git_repository TEXT, git_branch TEXT, git_commit TEXT, dev_port INTEGER, discovery_path TEXT);
             CREATE TABLE project_device_paths (project_id INTEGER NOT NULL, device_id TEXT NOT NULL, path TEXT NOT NULL, git_repository TEXT, git_branch TEXT, git_commit TEXT, dev_port INTEGER, discovery_path TEXT, status TEXT NOT NULL DEFAULT 'resolved', updated_at TEXT NOT NULL, PRIMARY KEY (project_id, device_id));",
        )
        .unwrap();
        c.execute("INSERT INTO projects (id,name,path,dev_port) VALUES (1,'p',?1,3000)", [path]).unwrap();
    }

    fn path_of(file: &Path) -> String {
        Connection::open(file).unwrap().query_row("SELECT path FROM projects WHERE id=1", [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn round_trip_a_b_a_keeps_a_and_b_resolve_does_not_touch_a() {
        let dir = tempfile::tempdir().unwrap();
        let a_dir = dir.path().join("a_proj");
        let b_dir = dir.path().join("b_proj");
        std::fs::create_dir(&b_dir).unwrap();
        let db = dir.path().join("x.db");
        // Device A's project lives at a path that does not exist on "B" (this test machine).
        seed(&db, a_dir.to_str().unwrap());
        record_device_paths(&Connection::open(&db).unwrap(), "A").unwrap();

        // Restore on B: A's path not here -> cleared, listed with A as last known.
        assert_eq!(resolve_for_device(&db, "B").unwrap(), 1);
        assert_eq!(path_of(&db), "");
        let un = list_unresolved(&db, "B").unwrap();
        assert_eq!(un[0].last_known_path.as_deref(), a_dir.to_str());
        assert_eq!(un[0].last_known_device_id.as_deref(), Some("A"));

        // B resolves; B pushes; A's row is untouched.
        resolve_project(&db, "B", 1, b_dir.to_str().unwrap()).unwrap();
        record_device_paths(&Connection::open(&db).unwrap(), "B").unwrap();
        let a_row: String = Connection::open(&db).unwrap()
            .query_row("SELECT path FROM project_device_paths WHERE device_id='A'", [], |r| r.get(0)).unwrap();
        assert_eq!(a_row, a_dir.to_str().unwrap());

        // Back on A: own row applies exactly.
        assert_eq!(resolve_for_device(&db, "A").unwrap(), 0);
        assert_eq!(path_of(&db), a_dir.to_str().unwrap());
    }

    #[test]
    fn ignore_persists_and_silences() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("x.db");
        seed(&db, "/nonexistent/zzz");
        assert_eq!(resolve_for_device(&db, "B").unwrap(), 1);
        ignore_project(&db, "B", 1).unwrap();
        assert!(list_unresolved(&db, "B").unwrap().is_empty());
        assert_eq!(resolve_for_device(&db, "B").unwrap(), 0);
        assert_eq!(path_of(&db), "");
        // push never writes a row for an empty path or overwrites an ignored one
        record_device_paths(&Connection::open(&db).unwrap(), "B").unwrap();
        let st: String = Connection::open(&db).unwrap()
            .query_row("SELECT status FROM project_device_paths WHERE device_id='B'", [], |r| r.get(0)).unwrap();
        assert_eq!(st, "ignored");
    }

    #[test]
    fn push_replaces_an_ignored_row_when_the_project_has_a_real_path_again() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("x.db");
        seed(&db, "/nonexistent/zzz");
        resolve_for_device(&db, "B").unwrap();
        ignore_project(&db, "B", 1).unwrap();
        Connection::open(&db).unwrap().execute("UPDATE projects SET path='/real'", []).unwrap();
        record_device_paths(&Connection::open(&db).unwrap(), "B").unwrap();
        let (st, p): (String, String) = Connection::open(&db).unwrap()
            .query_row("SELECT status, path FROM project_device_paths WHERE device_id='B'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!((st.as_str(), p.as_str()), ("resolved", "/real"));
    }
}
