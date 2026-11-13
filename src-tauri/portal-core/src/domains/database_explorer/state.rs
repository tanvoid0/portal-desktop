//! In-memory registry of open SQLite sources, held as Tauri managed state.
//!
//! P0 stores only a file path per source and opens a fresh read-only connection
//! per query (SQLite opens are cheap). A real connection pool is deferred until a
//! server engine (P1) makes it worth it — ponytail: no pool before it earns one.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::domains::database_explorer::types::DbSource;

struct SqliteSource {
    path: PathBuf,
    label: String,
}

pub struct DbExplorerState {
    sources: Mutex<HashMap<String, SqliteSource>>,
}

impl DbExplorerState {
    /// Seeds the registry with Portal's own database as a built-in source.
    pub fn new(portal_db_path: PathBuf) -> Self {
        let mut sources = HashMap::new();
        sources.insert(
            "portal-self".to_string(),
            SqliteSource {
                path: portal_db_path,
                label: "Portal (this app)".to_string(),
            },
        );
        Self {
            sources: Mutex::new(sources),
        }
    }

    /// Registers an ad-hoc SQLite file and returns its source descriptor.
    pub fn register_sqlite(&self, path: PathBuf, label: String) -> DbSource {
        let id = uuid::Uuid::new_v4().to_string();
        let detail = path.display().to_string();
        self.sources
            .lock()
            .unwrap()
            .insert(id.clone(), SqliteSource { path, label: label.clone() });
        DbSource {
            id,
            label,
            engine: "sqlite".to_string(),
            detail,
            editable: false,
        }
    }

    pub fn sources(&self) -> Vec<DbSource> {
        let guard = self.sources.lock().unwrap();
        let mut out: Vec<DbSource> = guard
            .iter()
            .map(|(id, s)| DbSource {
                id: id.clone(),
                label: s.label.clone(),
                engine: "sqlite".to_string(),
                detail: s.path.display().to_string(),
                editable: false,
            })
            .collect();
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }

    pub fn path_of(&self, id: &str) -> Option<PathBuf> {
        self.sources.lock().unwrap().get(id).map(|s| s.path.clone())
    }
}
