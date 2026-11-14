//! In-memory registry of known database sources, held as Tauri managed state.
//!
//! Two maps: `manual` (persistent — Portal's own DB + ad-hoc SQLite opens) and
//! `discovered` (recomputed fresh on every `db_list_sources` from the vault / SDK
//! services / Docker). A source carries everything needed to connect except the
//! password, which for vault entries is resolved lazily at connect time so that
//! listing never requires the vault to be unlocked.
//!
//! ponytail: no connection pool cache yet — a fresh pool is opened per call. Add
//! one when latency measurably hurts.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::domains::database_explorer::types::DbSource;

/// How a server source's password is obtained at connect time.
#[derive(Clone)]
pub enum PasswordRef {
    /// A literal password (possibly empty, e.g. a local SDK/Docker service).
    Plain(String),
    /// Decrypt this vault credential id at connect time. Keeps `db_list_sources`
    /// working while the vault is locked; the unlock error surfaces on connect.
    Vault(String),
}

/// Connection details for a Postgres/MySQL source.
#[derive(Clone)]
pub struct ServerConn {
    /// "postgres" | "mysql".
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    /// Default database to connect to (may be empty for MySQL).
    pub database: String,
    pub password: PasswordRef,
    pub label: String,
    /// Human detail shown in the UI — never includes the password.
    pub detail: String,
}

/// A connectable source. SQLite is a file; Postgres/MySQL are servers.
#[derive(Clone)]
pub enum Source {
    Sqlite { path: PathBuf, label: String },
    Server(ServerConn),
}

impl Source {
    fn descriptor(&self, id: &str) -> DbSource {
        match self {
            Source::Sqlite { path, label } => DbSource {
                id: id.to_string(),
                label: label.clone(),
                engine: "sqlite".to_string(),
                detail: path.display().to_string(),
                editable: false,
            },
            Source::Server(s) => DbSource {
                id: id.to_string(),
                label: s.label.clone(),
                engine: s.engine.clone(),
                detail: s.detail.clone(),
                editable: false,
            },
        }
    }
}

pub struct DbExplorerState {
    manual: Mutex<HashMap<String, Source>>,
    discovered: Mutex<HashMap<String, Source>>,
}

impl DbExplorerState {
    /// Seeds the registry with Portal's own database as a built-in source.
    pub fn new(portal_db_path: PathBuf) -> Self {
        let mut manual = HashMap::new();
        manual.insert(
            "portal-self".to_string(),
            Source::Sqlite {
                path: portal_db_path,
                label: "Portal (this app)".to_string(),
            },
        );
        Self {
            manual: Mutex::new(manual),
            discovered: Mutex::new(HashMap::new()),
        }
    }

    /// Registers an ad-hoc SQLite file and returns its source descriptor.
    pub fn register_sqlite(&self, path: PathBuf, label: String) -> DbSource {
        let id = uuid::Uuid::new_v4().to_string();
        let src = Source::Sqlite { path, label };
        let descriptor = src.descriptor(&id);
        self.manual.lock().unwrap().insert(id, src);
        descriptor
    }

    /// Replaces the discovered set (recomputed fresh on each list call).
    pub fn set_discovered(&self, sources: Vec<(String, Source)>) {
        let mut guard = self.discovered.lock().unwrap();
        guard.clear();
        for (id, src) in sources {
            guard.insert(id, src);
        }
    }

    /// All sources (manual + discovered), sorted by label.
    pub fn sources(&self) -> Vec<DbSource> {
        let mut out: Vec<DbSource> = Vec::new();
        for (id, s) in self.manual.lock().unwrap().iter() {
            out.push(s.descriptor(id));
        }
        for (id, s) in self.discovered.lock().unwrap().iter() {
            out.push(s.descriptor(id));
        }
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }

    /// Resolves a source by id across both maps.
    pub fn source_of(&self, id: &str) -> Option<Source> {
        if let Some(s) = self.manual.lock().unwrap().get(id) {
            return Some(s.clone());
        }
        self.discovered.lock().unwrap().get(id).cloned()
    }
}
