use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{paths, snapshot};

const STATE_FILE: &str = "sync_state.json";
const TOKEN_KEY: &str = "sync-token";

/// Serializes push/pull so two runs never stage, build or save state at once.
static SYNC_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
/// Guards sync_state.json reads/writes so parallel first calls can't mint two device ids.
static STATE_LOCK: Mutex<()> = Mutex::new(());

/// Snapshot metadata as the server reports it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub version: u32,
    pub device_id: String,
    pub client_updated_at: String,
    pub size: u64,
    pub content_hash: String,
    pub created_at: String,
}

/// Machine-local sync state, kept in `sync_state.json` next to the DB -- deliberately NOT in
/// the DB, so a restore (which replaces the DB) can never overwrite this device's identity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SyncState {
    device_id: String,
    base_version: u32,
    last_synced_at: Option<String>,
    server_url: String,
}

#[derive(Debug, Serialize)]
pub struct SyncConfig {
    pub server_url: String,
    pub has_token: bool,
}

#[derive(Debug, Serialize)]
pub struct LocalInfo {
    pub base_version: u32,
    pub local_db_modified_at: Option<String>,
    pub local_db_size: u64,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub configured: bool,
    pub device_id: String,
    pub base_version: u32,
    pub last_synced_at: Option<String>,
    pub local_db_modified_at: Option<String>,
    pub local_db_size: u64,
    pub cloud: Option<SnapshotRecord>,
    pub cloud_error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PushOutcome {
    Ok {
        record: SnapshotRecord,
    },
    Conflict {
        current: SnapshotRecord,
        local: LocalInfo,
    },
}

#[derive(Debug, Serialize)]
pub struct PullOutcome {
    /// Always `restart_required`: the pool cannot be swapped live, so the staged DB goes
    /// live at next boot.
    pub status: &'static str,
    pub record: SnapshotRecord,
    pub unresolved_count: u32,
}

pub struct SyncService {
    data_dir: PathBuf,
}

fn token_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(crate::app_paths::app_identifier(), TOKEN_KEY)
        .map_err(|e| format!("Keychain access failed: {e}"))
}

fn get_token() -> Option<String> {
    token_entry()
        .ok()?
        .get_password()
        .ok()
        .filter(|t| !t.is_empty())
}

/// Modified time (latest of db and -wal) and size (sum): WAL mode keeps recent writes in the
/// -wal file, so the main file alone looks stale.
fn db_stat(db: &Path) -> (Option<String>, u64) {
    let mut wal = db.as_os_str().to_owned();
    wal.push("-wal");
    let metas = [std::fs::metadata(db), std::fs::metadata(PathBuf::from(wal))];
    let modified = metas
        .iter()
        .flatten()
        .filter_map(|m| m.modified().ok())
        .max()
        .map(|t| DateTime::<Utc>::from(t).to_rfc3339());
    (modified, metas.iter().flatten().map(|m| m.len()).sum())
}

const UNREACHABLE: &str = "Cannot reach sync server";

impl SyncService {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    fn live(&self) -> PathBuf {
        snapshot::db_path(&self.data_dir)
    }

    /// A device id is minted only when the file is absent (or has no id yet). An unreadable
    /// or corrupt file is an error: silently minting would orphan this device's identity.
    fn load(&self) -> Result<SyncState, String> {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut s: SyncState = match std::fs::read_to_string(self.data_dir.join(STATE_FILE)) {
            Ok(t) => serde_json::from_str(&t)
                .map_err(|e| format!("{STATE_FILE} is corrupt ({e}); fix or delete it"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => SyncState::default(),
            Err(e) => return Err(format!("Cannot read {STATE_FILE}: {e}")),
        };
        if s.device_id.is_empty() {
            s.device_id = uuid::Uuid::new_v4().to_string();
            self.write(&s)?;
        }
        Ok(s)
    }

    fn save(&self, s: &SyncState) -> Result<(), String> {
        let _g = STATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        self.write(s)
    }

    /// Temp file + rename, so a crash never leaves a half-written state file.
    fn write(&self, s: &SyncState) -> Result<(), String> {
        let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
        let tmp = self
            .data_dir
            .join(format!("{STATE_FILE}.{}.tmp", uuid::Uuid::new_v4()));
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, self.data_dir.join(STATE_FILE)).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })
    }

    /// Blocking (keychain read).
    pub fn get_config(&self) -> Result<SyncConfig, String> {
        Ok(SyncConfig {
            server_url: self.load()?.server_url,
            has_token: get_token().is_some(),
        })
    }

    /// `token: None` or empty keeps the stored token.
    pub fn set_config(&self, server_url: String, token: Option<String>) -> Result<(), String> {
        let mut s = self.load()?;
        s.server_url = server_url.trim().trim_end_matches('/').to_string();
        self.save(&s)?;
        if let Some(t) = token.filter(|t| !t.is_empty()) {
            token_entry()?
                .set_password(&t)
                .map_err(|e| format!("Failed to store token: {e}"))?;
        }
        Ok(())
    }

    fn local_info(&self, s: &SyncState) -> LocalInfo {
        let (m, size) = db_stat(&self.live());
        LocalInfo {
            base_version: s.base_version,
            local_db_modified_at: m,
            local_db_size: size,
        }
    }

    async fn request(
        &self,
        s: &SyncState,
        method: reqwest::Method,
        path: &str,
    ) -> Result<reqwest::RequestBuilder, String> {
        let token = tokio::task::spawn_blocking(get_token)
            .await
            .map_err(|e| e.to_string())?;
        if s.server_url.is_empty() || token.is_none() {
            return Err("Sync is not configured: set the server URL and token first".into());
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(client
            .request(method, format!("{}{}", s.server_url, path))
            .bearer_auth(token.unwrap_or_default()))
    }

    async fn latest(&self, s: &SyncState) -> Result<Option<SnapshotRecord>, String> {
        let r = self
            .request(s, reqwest::Method::GET, "/api/sync/snapshot/latest")
            .await?
            .send()
            .await
            .map_err(|e| format!("{UNREACHABLE}: {e}"))?;
        match r.status().as_u16() {
            404 => Ok(None),
            200 => Ok(Some(r.json().await.map_err(|e| e.to_string())?)),
            c => Err(format!("Sync server error {c}")),
        }
    }

    pub async fn status(&self) -> Result<SyncStatus, String> {
        let this = Self::new(&self.data_dir);
        let (s, has_token, (m, size)) = tokio::task::spawn_blocking(move || {
            Ok::<_, String>((this.load()?, get_token().is_some(), db_stat(&this.live())))
        })
        .await
        .map_err(|e| e.to_string())??;
        let configured = !s.server_url.is_empty() && has_token;
        let (cloud, cloud_error) = if configured {
            match self.latest(&s).await {
                Ok(c) => (c, None),
                Err(e) => (None, Some(e)),
            }
        } else {
            (None, None)
        };
        Ok(SyncStatus {
            configured,
            device_id: s.device_id,
            base_version: s.base_version,
            last_synced_at: s.last_synced_at,
            local_db_modified_at: m,
            local_db_size: size,
            cloud,
            cloud_error,
        })
    }

    pub async fn list_versions(&self) -> Result<Vec<SnapshotRecord>, String> {
        #[derive(Deserialize)]
        struct V {
            versions: Vec<SnapshotRecord>,
        }
        let s = self.load()?;
        let r = self
            .request(&s, reqwest::Method::GET, "/api/sync/snapshot/versions")
            .await?
            .send()
            .await
            .map_err(|e| format!("{UNREACHABLE}: {e}"))?;
        if !r.status().is_success() {
            return Err(format!("Sync server error {}", r.status().as_u16()));
        }
        Ok(r.json::<V>().await.map_err(|e| e.to_string())?.versions)
    }

    pub async fn push(&self, password: String, force: bool) -> Result<PushOutcome, String> {
        if password.is_empty() {
            return Err("A sync password is required".into());
        }
        if snapshot::pending_path(&self.data_dir).exists() {
            return Err("Restart Portal to finish the pending restore first".into());
        }
        let _guard = SYNC_LOCK.lock().await;
        let mut s = self.load()?;
        let (live, device_id) = (self.live(), s.device_id.clone());
        let blob = tokio::task::spawn_blocking(move || {
            snapshot::build_snapshot(&live, &device_id, &password)
        })
        .await
        .map_err(|e| e.to_string())??;
        let hash = hex::encode(Sha256::digest(&blob));
        let (modified, _) = db_stat(&self.live());
        let mut q = vec![
            ("base_version", s.base_version.to_string()),
            ("device_id", s.device_id.clone()),
            (
                "client_updated_at",
                modified.unwrap_or_else(|| Utc::now().to_rfc3339()),
            ),
            ("content_hash", hash),
        ];
        if force {
            q.push(("force", "true".into()));
        }
        let r = self
            .request(&s, reqwest::Method::POST, "/api/sync/snapshot")
            .await?
            .query(&q)
            .header("content-type", "application/octet-stream")
            .body(blob)
            .send()
            .await
            .map_err(|e| format!("{UNREACHABLE}: {e}"))?;
        match r.status().as_u16() {
            200 | 201 => {
                let record: SnapshotRecord = r.json().await.map_err(|e| e.to_string())?;
                s.base_version = record.version;
                s.last_synced_at = Some(Utc::now().to_rfc3339());
                self.save(&s)?;
                Ok(PushOutcome::Ok { record })
            }
            409 => {
                #[derive(Deserialize)]
                struct C {
                    current: SnapshotRecord,
                }
                let c: C = r.json().await.map_err(|e| e.to_string())?;
                Ok(PushOutcome::Conflict {
                    current: c.current,
                    local: self.local_info(&s),
                })
            }
            413 => Err("Snapshot is too large for the sync server".into()),
            503 => Err("Sync server storage is not configured".into()),
            c => Err(format!("Sync server error {c}")),
        }
    }

    pub async fn pull(
        &self,
        password: String,
        version: Option<u32>,
    ) -> Result<PullOutcome, String> {
        if password.is_empty() {
            return Err("A sync password is required".into());
        }
        let _guard = SYNC_LOCK.lock().await;
        let s = self.load()?;
        let record = match version {
            None => self
                .latest(&s)
                .await?
                .ok_or("No snapshot on the server yet")?,
            Some(v) => self
                .list_versions()
                .await?
                .into_iter()
                .find(|r| r.version == v)
                .ok_or_else(|| format!("Snapshot version {v} not found"))?,
        };
        let r = self
            .request(
                &s,
                reqwest::Method::GET,
                &format!("/api/sync/snapshot/{}/download", record.version),
            )
            .await?
            .send()
            .await
            .map_err(|e| format!("{UNREACHABLE}: {e}"))?;
        if !r.status().is_success() {
            return Err(format!("Sync server error {}", r.status().as_u16()));
        }
        let blob = r.bytes().await.map_err(|e| e.to_string())?.to_vec();
        if hex::encode(Sha256::digest(&blob)) != record.content_hash.to_lowercase() {
            return Err("Downloaded snapshot failed its integrity check (hash mismatch)".into());
        }
        let (dir, device_id) = (self.data_dir.clone(), s.device_id.clone());
        // Not saved to sync_state.json yet: the old DB stays live until restart, so the new
        // base_version is committed by `apply_pending_restore` after the swap succeeds.
        let meta = snapshot::RestoreMeta {
            base_version: record.version,
            last_synced_at: Some(Utc::now().to_rfc3339()),
        };
        let unresolved_count = tokio::task::spawn_blocking(move || {
            snapshot::stage_restore(&dir, &blob, &password, &device_id, &meta)
        })
        .await
        .map_err(|e| e.to_string())??;
        Ok(PullOutcome {
            status: "restart_required",
            record,
            unresolved_count,
        })
    }

    pub fn list_unresolved(&self) -> Result<Vec<paths::UnresolvedProject>, String> {
        paths::list_unresolved(&self.live(), &self.load()?.device_id)
    }

    pub fn resolve_project(&self, project_id: i64, path: &str) -> Result<(), String> {
        paths::resolve_project(&self.live(), &self.load()?.device_id, project_id, path)
    }

    pub fn ignore_project(&self, project_id: i64) -> Result<(), String> {
        paths::ignore_project(&self.live(), &self.load()?.device_id, project_id)
    }
}

/// Boot-time hook: record the sync state of a restore that just went live.
pub(super) fn commit_restore_state(
    data_dir: &Path,
    meta: &snapshot::RestoreMeta,
) -> Result<(), String> {
    let svc = SyncService::new(data_dir);
    let mut s = svc.load()?;
    s.base_version = meta.base_version;
    s.last_synced_at = meta.last_synced_at.clone();
    svc.save(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn push_is_refused_while_a_restore_is_pending() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(snapshot::pending_path(dir.path()), b"x").unwrap();
        let e = SyncService::new(dir.path())
            .push("pw".into(), false)
            .await
            .unwrap_err();
        assert!(e.contains("Restart Portal"), "{e}");
    }

    #[test]
    fn state_file_mints_once_and_corrupt_is_an_error_not_a_new_id() {
        let dir = tempfile::tempdir().unwrap();
        let svc = SyncService::new(dir.path());
        let id = svc.load().unwrap().device_id;
        assert_eq!(svc.load().unwrap().device_id, id);

        std::fs::write(dir.path().join(STATE_FILE), "{ not json").unwrap();
        assert!(svc.load().unwrap_err().contains("corrupt"));
        // Still the corrupt bytes: nothing overwrote them with a fresh id.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(STATE_FILE)).unwrap(),
            "{ not json"
        );
    }
}
