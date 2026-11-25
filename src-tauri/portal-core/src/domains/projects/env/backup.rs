//! Bulk backup and restore of project `.env*` files through the credential vault.
//!
//! One file = one `credentials` row (type [`CREDENTIAL_TYPE`]); the value is the file
//! verbatim, encrypted by [`CredentialService`] like any credential, and plaintext
//! `metadata` links it to its project. No table, no migration. Design, safety rules
//! and limits: `docs/architecture/env-vault-backup.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::domains::credentials::entities::{
    Column as CredentialColumn, Entity as CredentialEntity, Model as CredentialModel,
};
use crate::domains::credentials::services::credential_service::{
    CredentialCreateRequest, CredentialUpdateRequest,
};
use crate::domains::credentials::services::{CredentialService, VaultSession};
use crate::domains::credentials::CredentialError;
use crate::domains::projects::move_lock;
use crate::domains::sync::paths::normalize_remote;
use crate::entities::project as project_entity;

const CREDENTIAL_TYPE: &str = "env_file";
const KIND: &str = "env-file";
const TAG: &str = "env-file";
/// Bigger than any real env file; a credential value is a DB row, not blob storage.
const MAX_BYTES: u64 = 1024 * 1024;
/// Sub-folder levels walked below a project root (`apps/web/.env` is 2).
const MAX_DEPTH: usize = 4;
/// Folders never entered (compared lowercase). Mirrors what the git-sync env-backup skipped.
const PRUNE: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    "venv",
    ".venv",
    "site-packages",
    ".dart_tool",
    "pods",
    "library",
    "__pycache__",
    ".gradle",
    ".next",
    ".idea",
    ".vscode",
    "coverage",
    "vendor",
    ".claude",
];
/// `.env.<suffix>` files that are templates: committed to git, nothing to back up.
const TEMPLATE_SUFFIXES: &[&str] = &["example", "sample", "template", "dist", "tpl", "defaults"];
/// Our own temp files are `<target>.portal-tmp-<uuid>`; a leftover must never read as an env file.
const TEMP_MARK: &str = ".portal-tmp-";

/// How a file on disk compares with its vault copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum EnvFileState {
    /// On disk, not in the vault.
    New,
    InSync,
    Differs,
    /// In the vault, absent on disk: what a restore is for.
    OnlyInVault,
    /// Both exist but cannot be compared (vault locked, or the row will not decrypt).
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvBackupFile {
    pub project_id: i32,
    pub project_name: String,
    pub project_path: String,
    /// `/`-separated, relative to the project root.
    pub rel_path: String,
    pub state: EnvFileState,
    /// The vault row, when there is one.
    pub credential_id: Option<String>,
    #[ts(type = "number | null")]
    pub disk_bytes: Option<u64>,
    pub disk_modified: Option<String>,
    pub vault_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvBackupScan {
    pub files: Vec<EnvBackupFile>,
    /// Vault entries whose project is not (or no longer unambiguously) in Portal.
    pub unmatched: Vec<String>,
    /// Env files that exist but cannot be backed up, with the reason: `project/file (why)`.
    pub skipped: Vec<String>,
    /// Content could not be compared because the vault is locked.
    pub vault_locked: bool,
}

/// One file to back up, as listed by the scan. `overwrite` is the user's confirmation that
/// the vault copy may be replaced (it only matters when the vault copy differs).
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvFileRef {
    pub project_id: i32,
    pub rel_path: String,
    #[serde(default)]
    pub overwrite: bool,
}

/// One vault entry to write back. `overwrite` is the user's confirmation for exactly this row.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvRestoreItem {
    pub credential_id: String,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum EnvOutcome {
    Created,
    Updated,
    Unchanged,
    Restored,
    Overwritten,
    /// Restore found the same content already on disk.
    Identical,
    /// The other side differs and was not told to be replaced: nothing written.
    Conflict,
    /// Expected, nothing to do or nowhere to put it (see `detail`).
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvItemResult {
    /// The Portal project on this machine (the scan's `project_id`); `None` when unmatched.
    pub project_id: Option<i32>,
    pub project_name: String,
    pub rel_path: String,
    pub outcome: EnvOutcome,
    pub detail: Option<String>,
}

impl EnvItemResult {
    fn new(
        project_id: Option<i32>,
        project_name: &str,
        rel_path: &str,
        (outcome, detail): (EnvOutcome, Option<String>),
    ) -> Self {
        Self {
            project_id,
            project_name: project_name.to_string(),
            rel_path: rel_path.to_string(),
            outcome,
            detail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvBackupReport {
    pub items: Vec<EnvItemResult>,
}

/// Plaintext link stored in `credentials.metadata`. Never holds content, hashes or key names.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Meta {
    kind: String,
    project_id: i32,
    project_name: String,
    /// Normalised `host/owner/repo`, empty when the project has no remote.
    #[serde(default)]
    git_remote: String,
    rel_path: String,
}

impl Meta {
    fn to_map(&self) -> HashMap<String, serde_json::Value> {
        match serde_json::to_value(self) {
            Ok(serde_json::Value::Object(m)) => m.into_iter().collect(),
            _ => HashMap::new(),
        }
    }
}

struct Entry {
    cred: CredentialModel,
    meta: Meta,
}

struct Proj {
    id: i32,
    name: String,
    path: String,
    /// Normalised `host/owner/repo`, empty when none.
    remote: String,
}

struct DiskFile {
    rel: String,
    abs: PathBuf,
    bytes: u64,
    modified: Option<String>,
}

/// An env file that exists but cannot be a credential value.
struct Skip {
    rel: String,
    why: &'static str,
}

/// What one project folder holds.
#[derive(Default)]
struct Disk {
    files: Vec<DiskFile>,
    skipped: Vec<Skip>,
}

/// `.env` and `.env.*` except templates and our own temp files. Case-insensitive, like the
/// Windows filesystem.
fn is_env_file_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if (lower != ".env" && !lower.starts_with(".env.")) || lower.contains(TEMP_MARK) {
        return false;
    }
    let last = lower.rsplit('.').next().unwrap_or("");
    !TEMPLATE_SUFFIXES.contains(&last)
}

fn modified(meta: &std::fs::Metadata) -> Option<String> {
    meta.modified()
        .ok()
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
}

/// Env files under `root`, sorted by relative path. Symlinks are neither followed nor
/// listed. `skip` holds `move_lock::norm`-ed paths of other projects' roots, which belong to
/// those projects. Files over the size cap or not UTF-8 text are reported, not listed.
fn scan_dir(root: &Path, skip: &HashSet<String>) -> Disk {
    let mut out = Disk::default();
    walk(root, root, 0, skip, &mut out);
    out.files.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.skipped.sort_by(|a, b| a.rel.cmp(&b.rel));
    out
}

fn walk(root: &Path, dir: &Path, depth: usize, skip: &HashSet<String>, out: &mut Disk) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            if depth < MAX_DEPTH
                && !PRUNE.contains(&name.to_ascii_lowercase().as_str())
                && !skip.contains(&move_lock::norm(&path.to_string_lossy()))
            {
                walk(root, &path, depth + 1, skip, out);
            }
        } else if kind.is_file() && is_env_file_name(&name) {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let rel = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.len() > MAX_BYTES {
                out.skipped.push(Skip {
                    rel,
                    why: "over 1 MiB",
                });
                continue;
            }
            match std::fs::read(&path) {
                Ok(bytes) if std::str::from_utf8(&bytes).is_ok() => out.files.push(DiskFile {
                    rel,
                    bytes: meta.len(),
                    modified: modified(&meta),
                    abs: path,
                }),
                Ok(_) => out.skipped.push(Skip {
                    rel,
                    why: "not UTF-8 text",
                }),
                // Unreadable right now (another program holds it): not listed.
                Err(_) => {}
            }
        }
    }
}

/// Where a stored `rel_path` may be written: plain relative components only, none with a `:`
/// (drive prefix, NTFS alternate stream) or a trailing dot or space (Windows drops those),
/// ending in an env file name. A row can come from a synced snapshot, so it is not trusted
/// to stay inside the project or to name only env files.
fn safe_target(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let p = Path::new(rel);
    let plain = !rel.is_empty()
        && !rel.contains(':')
        && p.components().all(
            |c| matches!(c, Component::Normal(n) if !n.to_string_lossy().ends_with(['.', ' '])),
        );
    let env_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(is_env_file_name);
    if plain && env_name {
        Ok(root.join(p))
    } else {
        Err(format!("Refusing to write to unsafe path: {rel}"))
    }
}

/// The target's folder must exist and, after symlinks, sit inside the project root; the
/// target itself must not be a symlink. Restore creates files, never folders.
fn check_target(root: &Path, target: &Path) -> Result<(), String> {
    let parent = target.parent().unwrap_or(root);
    let parent_real = std::fs::canonicalize(parent)
        .map_err(|_| format!("Folder does not exist: {}", parent.display()))?;
    let root_real = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    if !parent_real.starts_with(&root_real) {
        return Err(format!(
            "{} is outside the project folder",
            parent.display()
        ));
    }
    if std::fs::symlink_metadata(target).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(format!(
            "{} is a symlink; not replacing it",
            target.display()
        ));
    }
    Ok(())
}

/// Creates `path` only if nothing is there (`create_new`, so no check-then-write race).
fn write_new(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    if let Err(e) = file.write_all(content.as_bytes()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(e);
    }
    Ok(())
}

/// Replaces `path`: a fresh uniquely named temp file beside it (`create_new`, never an
/// existing file), flushed, then renamed over it.
fn replace_file(path: &Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!("{TEMP_MARK}{}", uuid::Uuid::new_v4().simple()));
    let tmp = PathBuf::from(tmp);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)?;
    let written = file
        .write_all(content.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| std::fs::rename(&tmp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// The Portal project a vault entry belongs to, as an index into `projects`: its id while the
/// name or remote still agrees, else the one project with the same git remote. A name alone
/// never matches (`web` is a common folder name, and a wrong guess puts a secret file in the
/// wrong project), and two candidates are no match. `projects.id` is AUTOINCREMENT, so within
/// one database an id never means two projects; the agreement check only guards against ids
/// from another database.
fn match_project(meta: &Meta, projects: &[Proj]) -> Option<usize> {
    let has_remote = !meta.git_remote.is_empty();
    if let Some(i) = projects.iter().position(|p| {
        p.id == meta.project_id
            && (p.name == meta.project_name || (has_remote && p.remote == meta.git_remote))
    }) {
        return Some(i);
    }
    if !has_remote {
        return None;
    }
    let mut hits = projects
        .iter()
        .enumerate()
        .filter(|(_, p)| p.remote == meta.git_remote)
        .map(|(i, _)| i);
    let first = hits.next()?;
    hits.next().is_none().then_some(first)
}

fn done(outcome: EnvOutcome) -> (EnvOutcome, Option<String>) {
    (outcome, None)
}

fn skipped(why: impl ToString) -> (EnvOutcome, Option<String>) {
    (EnvOutcome::Skipped, Some(why.to_string()))
}

fn failed(why: impl ToString) -> (EnvOutcome, Option<String>) {
    (EnvOutcome::Failed, Some(why.to_string()))
}

fn conflict(why: &str) -> (EnvOutcome, Option<String>) {
    (EnvOutcome::Conflict, Some(why.to_string()))
}

pub struct EnvBackupService {
    db: DatabaseConnection,
    credentials: CredentialService,
    session: Arc<VaultSession>,
}

impl EnvBackupService {
    pub fn new(db: DatabaseConnection, session: Arc<VaultSession>) -> Self {
        Self {
            credentials: CredentialService::new(db.clone(), session.clone()),
            db,
            session,
        }
    }

    /// Env files on disk next to their vault copies. Never prompts and never writes:
    /// with the vault locked, rows that exist on both sides are `Unknown`.
    pub async fn scan(&self) -> Result<EnvBackupScan, String> {
        let projects = self.load_projects().await?;
        let entries = self.load_entries().await?;
        let disk = disk_files(&projects).await?;

        let mut mine: Vec<Vec<&Entry>> = projects.iter().map(|_| Vec::new()).collect();
        let mut unmatched = Vec::new();
        for entry in &entries {
            match match_project(&entry.meta, &projects) {
                Some(i) => mine[i].push(entry),
                None => unmatched.push(entry.cred.name.clone()),
            }
        }

        let mut files = Vec::new();
        let mut skipped = Vec::new();
        for (i, p) in projects.iter().enumerate() {
            skipped.extend(
                disk[i]
                    .skipped
                    .iter()
                    .map(|s| format!("{}/{} ({})", p.name, s.rel, s.why)),
            );
            // `entries` is oldest first, so a later duplicate wins.
            let vault: BTreeMap<&str, &Entry> = mine[i]
                .iter()
                .map(|e| (e.meta.rel_path.as_str(), *e))
                .collect();
            let on_disk: BTreeMap<&str, &DiskFile> =
                disk[i].files.iter().map(|f| (f.rel.as_str(), f)).collect();
            let rels: BTreeSet<&str> = vault.keys().chain(on_disk.keys()).copied().collect();
            for rel in rels {
                let (d, v) = (on_disk.get(rel).copied(), vault.get(rel).copied());
                let state = match (d, v) {
                    (Some(_), None) => EnvFileState::New,
                    (None, Some(_)) => EnvFileState::OnlyInVault,
                    (Some(d), Some(v)) => self.compare(d, v).await,
                    (None, None) => continue,
                };
                files.push(EnvBackupFile {
                    project_id: p.id,
                    project_name: p.name.clone(),
                    project_path: p.path.clone(),
                    rel_path: rel.to_string(),
                    state,
                    credential_id: v.map(|e| e.cred.id.clone()),
                    disk_bytes: d.map(|f| f.bytes),
                    disk_modified: d.and_then(|f| f.modified.clone()),
                    vault_updated_at: v.map(|e| e.cred.updated_at.and_utc().to_rfc3339()),
                });
            }
        }
        Ok(EnvBackupScan {
            files,
            unmatched,
            skipped,
            vault_locked: !self.session.is_unlocked(),
        })
    }

    async fn compare(&self, disk: &DiskFile, vault: &Entry) -> EnvFileState {
        // `peek_value` never prompts and never bumps `last_used`; locked is `Ok(None)`.
        let Ok(Some(stored)) = self.credentials.peek_value(&vault.cred.id).await else {
            return EnvFileState::Unknown;
        };
        match tokio::fs::read(&disk.abs).await {
            Ok(now) if now == stored.as_bytes() => EnvFileState::InSync,
            Ok(_) => EnvFileState::Differs,
            Err(_) => EnvFileState::Unknown,
        }
    }

    /// Disk to vault. `selection = None` backs up every file found; otherwise only the listed
    /// ones. A file with no vault copy is created. A vault copy with different content is
    /// kept (`Conflict`) unless that file's ref carries `overwrite`. The vault key is asked
    /// for once up front: locked and not unlocked by the user is `Vault is locked` with
    /// nothing written; locked again mid-run stops the run and reports what was done.
    /// Never touches the disk.
    pub async fn import(
        &self,
        selection: Option<Vec<EnvFileRef>>,
    ) -> Result<EnvBackupReport, String> {
        self.session
            .require_key()
            .await
            .map_err(|e| e.to_string())?;
        let projects = self.load_projects().await?;
        let entries = self.load_entries().await?;
        let disk = disk_files(&projects).await?;

        let mut existing: HashMap<(usize, &str), &Entry> = HashMap::new();
        for entry in &entries {
            if let Some(i) = match_project(&entry.meta, &projects) {
                existing.insert((i, entry.meta.rel_path.as_str()), entry);
            }
        }

        let mut items = Vec::new();
        'run: for (i, p) in projects.iter().enumerate() {
            for file in &disk[i].files {
                let overwrite = match &selection {
                    None => false,
                    Some(sel) => match sel
                        .iter()
                        .find(|r| r.project_id == p.id && r.rel_path == file.rel)
                    {
                        Some(r) => r.overwrite,
                        None => continue,
                    },
                };
                let existing = existing.get(&(i, file.rel.as_str())).copied();
                let result = match self.import_one(p, file, existing, overwrite).await {
                    Ok(r) => r,
                    Err(CredentialError::VaultLocked) => {
                        let stop = failed("Vault is locked; stopped here");
                        items.push(EnvItemResult::new(Some(p.id), &p.name, &file.rel, stop));
                        break 'run;
                    }
                    Err(e) => failed(e),
                };
                items.push(EnvItemResult::new(Some(p.id), &p.name, &file.rel, result));
            }
        }
        Ok(EnvBackupReport { items })
    }

    async fn import_one(
        &self,
        project: &Proj,
        file: &DiskFile,
        existing: Option<&Entry>,
        overwrite: bool,
    ) -> Result<(EnvOutcome, Option<String>), CredentialError> {
        // New and existing rows alike: unlocked, or ask the user again.
        self.session.require_key().await?;
        let content = match tokio::fs::read(&file.abs).await.map(String::from_utf8) {
            Ok(Ok(text)) => text,
            Ok(Err(_)) => return Ok(skipped("Not valid UTF-8")),
            Err(e) => return Ok(failed(e)),
        };
        let meta = Meta {
            kind: KIND.to_string(),
            project_id: project.id,
            project_name: project.name.clone(),
            git_remote: project.remote.clone(),
            rel_path: file.rel.clone(),
        };
        let name = format!("{}/{}", project.name, file.rel);
        let Some(existing) = existing else {
            self.credentials
                .create_credential(CredentialCreateRequest {
                    name,
                    credential_type: CREDENTIAL_TYPE.to_string(),
                    description: None,
                    tags: Some(vec![TAG.to_string()]),
                    value: content,
                    fields: None,
                    metadata: Some(meta.to_map()),
                    expires_at: None,
                })
                .await?;
            return Ok(done(EnvOutcome::Created));
        };
        // Locked mid-run reads as `None`: stop instead of calling it a difference.
        let stored = self
            .credentials
            .peek_value(&existing.cred.id)
            .await?
            .ok_or(CredentialError::VaultLocked)?;
        if stored == content {
            return Ok(done(EnvOutcome::Unchanged));
        }
        if !overwrite {
            return Ok(conflict("The vault copy differs from this file"));
        }
        self.credentials
            .update_credential(
                &existing.cred.id,
                CredentialUpdateRequest {
                    name: Some(name),
                    description: None,
                    tags: None,
                    value: Some(content),
                    fields: None,
                    metadata: Some(meta.to_map()),
                    status: None,
                    expires_at: None,
                },
            )
            .await?;
        Ok(done(EnvOutcome::Updated))
    }

    /// Vault to disk. A file that is missing is created; one with the same content is left
    /// alone; one with different content is a `Conflict` and is replaced only when that item
    /// carries `overwrite`. Never creates folders, never writes through a symlink.
    pub async fn restore(&self, requests: Vec<EnvRestoreItem>) -> Result<EnvBackupReport, String> {
        self.session
            .require_key()
            .await
            .map_err(|e| e.to_string())?;
        let projects = self.load_projects().await?;
        let entries = self.load_entries().await?;

        let mut items = Vec::new();
        for request in requests {
            let Some(entry) = entries.iter().find(|e| e.cred.id == request.credential_id) else {
                let result = skipped("Not an env file entry");
                items.push(EnvItemResult::new(None, "", &request.credential_id, result));
                continue;
            };
            let project = match_project(&entry.meta, &projects).map(|i| &projects[i]);
            let (id, name) = match project {
                Some(p) => (Some(p.id), p.name.as_str()),
                None => (None, entry.meta.project_name.as_str()),
            };
            let result = match self.restore_one(entry, project, request.overwrite).await {
                Ok(r) => r,
                Err(CredentialError::VaultLocked) => {
                    let stop = failed("Vault is locked; stopped here");
                    items.push(EnvItemResult::new(id, name, &entry.meta.rel_path, stop));
                    break;
                }
                Err(e) => failed(e),
            };
            items.push(EnvItemResult::new(id, name, &entry.meta.rel_path, result));
        }
        Ok(EnvBackupReport { items })
    }

    async fn restore_one(
        &self,
        entry: &Entry,
        project: Option<&Proj>,
        overwrite: bool,
    ) -> Result<(EnvOutcome, Option<String>), CredentialError> {
        let Some(project) = project else {
            return Ok(skipped("No matching project in Portal"));
        };
        let root = Path::new(&project.path);
        if project.path.is_empty() || !root.is_dir() {
            return Ok(skipped("Project folder not found on this machine"));
        }
        if let Err(e) = move_lock::ensure_unlocked(root) {
            return Ok(skipped(e));
        }
        let target = match safe_target(root, &entry.meta.rel_path) {
            Ok(t) => t,
            Err(e) => return Ok(skipped(e)),
        };
        if let Err(e) = check_target(root, &target) {
            return Ok(skipped(e));
        }

        let value = self.credentials.decrypt_credential(&entry.cred.id).await?;
        let differs = || conflict("A different file already exists");
        Ok(match tokio::fs::read(&target).await {
            Ok(now) if now == value.as_bytes() => done(EnvOutcome::Identical),
            Ok(_) if !overwrite => differs(),
            Ok(_) => match replace_file(&target, &value) {
                Ok(()) => done(EnvOutcome::Overwritten),
                Err(e) => failed(e),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                match write_new(&target, &value) {
                    Ok(()) => done(EnvOutcome::Restored),
                    // Appeared since the read: still not ours to overwrite.
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => differs(),
                    Err(e) => failed(e),
                }
            }
            Err(e) => failed(e),
        })
    }

    async fn load_projects(&self) -> Result<Vec<Proj>, String> {
        let mut projects: Vec<Proj> = project_entity::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| format!("Failed to load projects: {e}"))?
            .into_iter()
            .map(|m| Proj {
                id: m.id,
                remote: normalize_remote(m.git_repository.as_deref().unwrap_or("")),
                name: m.name,
                path: m.path,
            })
            .collect();
        projects.sort_by_key(|p| p.name.to_lowercase());
        Ok(projects)
    }

    /// Env-file rows, oldest first.
    async fn load_entries(&self) -> Result<Vec<Entry>, String> {
        let rows = CredentialEntity::find()
            .filter(CredentialColumn::CredentialType.eq(CREDENTIAL_TYPE))
            .order_by_asc(CredentialColumn::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| format!("Failed to load credentials: {e}"))?;
        Ok(rows
            .into_iter()
            .filter_map(|cred| {
                let meta = serde_json::from_str::<Meta>(&cred.metadata)
                    .ok()
                    .filter(|m| m.kind == KIND)?;
                Some(Entry { cred, meta })
            })
            .collect())
    }
}

/// What each project folder holds, in `projects` order. Off the async threads: it is a
/// directory walk plus a read of every env file found.
async fn disk_files(projects: &[Proj]) -> Result<Vec<Disk>, String> {
    let roots: Vec<String> = projects.iter().map(|p| p.path.clone()).collect();
    let skip: HashSet<String> = roots
        .iter()
        .filter(|r| !r.is_empty())
        .map(|r| move_lock::norm(r))
        .collect();
    tokio::task::spawn_blocking(move || {
        roots
            .iter()
            .map(|r| {
                let root = Path::new(r);
                if r.is_empty() || !root.is_dir() {
                    Disk::default()
                } else {
                    scan_dir(root, &skip)
                }
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::credentials::services::VaultService;
    use crate::migrations::runner::run_migrations;
    use sea_orm::sea_query::Expr;
    use sea_orm::{ActiveModelTrait, Database, Set};
    use std::fs;

    const SECRET: &str = "TOKEN=s3cr3t-value-123";

    struct Fixture {
        svc: EnvBackupService,
        db: DatabaseConnection,
        session: Arc<VaultSession>,
        tmp: tempfile::TempDir,
    }

    /// Throwaway everything: in-memory DB, vault set up with a fixed test key (never the OS
    /// keychain), temp project folders.
    async fn fixture() -> Fixture {
        let db = Database::connect("sqlite::memory:").await.expect("connect");
        run_migrations(&db).await.expect("migrate");
        let session = Arc::new(VaultSession::default());
        VaultService::new(db.clone(), session.clone())
            .setup_with_master_key([7u8; 32], "password-one")
            .await
            .expect("setup vault");
        Fixture {
            svc: EnvBackupService::new(db.clone(), session.clone()),
            db,
            session,
            tmp: tempfile::tempdir().expect("tempdir"),
        }
    }

    async fn add_project(
        db: &DatabaseConnection,
        name: &str,
        path: &Path,
        remote: Option<&str>,
    ) -> i32 {
        project_entity::ActiveModel {
            name: Set(name.to_string()),
            path: Set(path.to_string_lossy().into_owned()),
            status: Set("active".to_string()),
            starred: Set(false),
            open_count: Set(0),
            size: Set(0),
            file_count: Set(0),
            has_uncommitted_changes: Set(false),
            unpushed_commits: Set(0),
            stash_count: Set(0),
            has_remote: Set(false),
            module_count: Set(0),
            archived: Set(false),
            git_repository: Set(remote.map(str::to_string)),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("insert project")
        .id
    }

    /// A vault row exactly as `import` writes it, for tests that need a row without a file.
    async fn add_row(f: &Fixture, meta: Meta) -> String {
        CredentialService::new(f.db.clone(), f.session.clone())
            .create_credential(CredentialCreateRequest {
                name: format!("{}/{}", meta.project_name, meta.rel_path),
                credential_type: CREDENTIAL_TYPE.into(),
                description: None,
                tags: None,
                value: SECRET.into(),
                fields: None,
                metadata: Some(meta.to_map()),
                expires_at: None,
            })
            .await
            .expect("create row")
            .id
    }

    fn count(report: &EnvBackupReport, outcome: EnvOutcome) -> usize {
        report.items.iter().filter(|i| i.outcome == outcome).count()
    }

    fn refs(project_id: i32, overwrite: bool, rels: &[&str]) -> Vec<EnvFileRef> {
        rels.iter()
            .map(|r| EnvFileRef {
                project_id,
                rel_path: (*r).to_string(),
                overwrite,
            })
            .collect()
    }

    fn proj(id: i32, name: &str, remote: &str) -> Proj {
        Proj {
            id,
            name: name.into(),
            path: String::new(),
            remote: remote.into(),
        }
    }

    fn meta(id: i32, name: &str, remote: &str) -> Meta {
        Meta {
            kind: KIND.into(),
            project_id: id,
            project_name: name.into(),
            git_remote: remote.into(),
            rel_path: ".env".into(),
        }
    }

    #[test]
    fn env_file_names_exclude_templates_and_our_temp_files() {
        for yes in [
            ".env",
            ".ENV",
            ".env.local",
            ".env.production",
            ".env.example.local",
        ] {
            assert!(is_env_file_name(yes), "{yes}");
        }
        for no in [
            ".env.example",
            ".env.sample",
            ".env.template",
            ".env.dist",
            ".env.tpl",
            ".env.defaults",
            ".envrc",
            "env",
            "app.env",
            ".environment",
            "x.env.local",
            ".env.portal-tmp-0123456789abcdef",
        ] {
            assert!(!is_env_file_name(no), "{no}");
        }
    }

    #[test]
    fn safe_target_only_allows_plain_relative_env_paths() {
        let root = Path::new("proj");
        assert!(safe_target(root, ".env").is_ok());
        assert!(safe_target(root, "backend/.env.local").is_ok());
        for bad in [
            "",
            "../.env",
            "a/../../.env",
            "/etc/.env",
            "./.env",
            "src/main.rs",
            ".bashrc",
            "a/.env/b",
            ".env.example",
            // NTFS alternate data stream, and names Windows silently trims.
            ".env.local:payload",
            "sub/.env:x",
            ".env.",
            "a /.env",
            "a./.env",
        ] {
            assert!(safe_target(root, bad).is_err(), "{bad:?} must be refused");
        }
        // Drive prefixes are only a prefix on Windows; elsewhere they are a plain folder name.
        #[cfg(windows)]
        for bad in ["C:/x/.env", "C:.env", "\\\\server\\share\\.env"] {
            assert!(safe_target(root, bad).is_err(), "{bad:?} must be refused");
        }
    }

    #[test]
    fn scan_dir_finds_nested_env_files_and_reports_the_ones_it_cannot_use() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for (rel, body) in [
            (".env", "A=1"),
            (".env.example", "A="),
            ("backend/.env.local", "B=2"),
            ("a/b/c/d/.env", "deep ok"),
            ("a/b/c/d/e/.env", "too deep"),
            ("node_modules/pkg/.env", "pruned"),
            (".claude/worktrees/w/.env", "pruned"),
            ("child/.env", "belongs to the child project"),
            ("big/.env.big", ""),
            ("big/.env.utf16", ""),
        ] {
            let p = root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, body).unwrap();
        }
        fs::write(
            root.join("big/.env.big"),
            vec![b'x'; MAX_BYTES as usize + 1],
        )
        .unwrap();
        // What PowerShell 5's `>` writes: UTF-16 LE with a BOM.
        fs::write(
            root.join("big/.env.utf16"),
            [0xFF, 0xFE, b'A', 0, b'=', 0, b'1', 0],
        )
        .unwrap();

        let skip: HashSet<String> = [move_lock::norm(&root.join("child").to_string_lossy())].into();
        let disk = scan_dir(root, &skip);
        let found: Vec<&str> = disk.files.iter().map(|f| f.rel.as_str()).collect();
        assert_eq!(found, [".env", "a/b/c/d/.env", "backend/.env.local"]);
        let unusable: Vec<(&str, &str)> = disk
            .skipped
            .iter()
            .map(|s| (s.rel.as_str(), s.why))
            .collect();
        assert_eq!(
            unusable,
            [
                ("big/.env.big", "over 1 MiB"),
                ("big/.env.utf16", "not UTF-8 text")
            ]
        );
    }

    #[test]
    fn match_project_needs_corroboration_and_never_guesses() {
        let projects = [
            proj(1, "app", "github.com/o/app"),
            proj(2, "web", "github.com/b/web"),
            proj(3, "web", ""),
        ];
        // Id plus name, or id plus remote (a renamed project, a remote that moved).
        assert_eq!(match_project(&meta(1, "app", ""), &projects), Some(0));
        assert_eq!(
            match_project(&meta(1, "renamed", "github.com/o/app"), &projects),
            Some(0)
        );
        assert_eq!(
            match_project(&meta(1, "app", "github.com/o/moved"), &projects),
            Some(0)
        );
        // Same id, different project everywhere else: another database's id, not a match.
        assert_eq!(
            match_project(&meta(1, "other", "github.com/x/y"), &projects),
            None
        );
        // Unknown id (project deleted and re-added): a unique remote finds it again.
        assert_eq!(
            match_project(&meta(9, "gone", "github.com/o/app"), &projects),
            Some(0)
        );
        // A name alone never matches: this "web" was another repo, whose project is gone.
        assert_eq!(
            match_project(&meta(9, "web", "github.com/a/web"), &projects),
            None
        );
        assert_eq!(match_project(&meta(9, "app", ""), &projects), None);
        // Two projects with the same remote are two candidates: no match.
        let clones = [
            proj(1, "a", "github.com/o/r"),
            proj(2, "b", "github.com/o/r"),
        ];
        assert_eq!(
            match_project(&meta(9, "c", "github.com/o/r"), &clones),
            None
        );
    }

    #[tokio::test]
    async fn backup_then_restore_round_trip() {
        let f = fixture().await;
        let app = f.tmp.path().join("app");
        fs::create_dir_all(app.join("backend")).unwrap();
        // CRLF, quotes and a comment must survive byte for byte.
        let root_env = format!("# prod\r\nA=1\r\nB=\"x y\"\r\n{SECRET}\r\n");
        fs::write(app.join(".env"), &root_env).unwrap();
        fs::write(app.join("backend/.env.local"), "DB=postgres://u:p@h/d\n").unwrap();
        fs::write(app.join(".env.example"), "A=\n").unwrap();
        fs::create_dir_all(app.join("node_modules/x")).unwrap();
        fs::write(app.join("node_modules/x/.env"), "Z=1").unwrap();
        let id = add_project(&f.db, "app", &app, Some("https://github.com/o/app.git")).await;

        // Nothing backed up yet.
        let scan = f.svc.scan().await.unwrap();
        assert!(!scan.vault_locked && scan.skipped.is_empty());
        let seen: Vec<_> = scan
            .files
            .iter()
            .map(|x| (x.rel_path.as_str(), x.state))
            .collect();
        assert_eq!(
            seen,
            [
                (".env", EnvFileState::New),
                ("backend/.env.local", EnvFileState::New)
            ]
        );

        let report = f.svc.import(None).await.unwrap();
        assert_eq!(
            (count(&report, EnvOutcome::Created), report.items.len()),
            (2, 2)
        );
        assert!(report.items.iter().all(|i| i.project_id == Some(id)));

        // Encrypted at rest; nothing secret in any plaintext column.
        let rows = CredentialEntity::find().all(&f.db).await.unwrap();
        assert_eq!(rows.len(), 2);
        for row in &rows {
            assert_eq!(row.credential_type, "env_file");
            for plain in [&row.encrypted_value, &row.metadata, &row.name, &row.tags] {
                assert!(!plain.contains("s3cr3t"), "secret leaked into {plain}");
                assert!(!plain.contains("postgres"), "secret leaked into {plain}");
            }
        }
        let m: Meta = serde_json::from_str(&rows[0].metadata).unwrap();
        assert_eq!(
            (m.project_id, m.git_remote.as_str()),
            (id, "github.com/o/app")
        );

        // Re-running changes nothing; everything reads as in sync.
        assert_eq!(
            count(&f.svc.import(None).await.unwrap(), EnvOutcome::Unchanged),
            2
        );
        let scan = f.svc.scan().await.unwrap();
        assert!(scan.files.iter().all(|x| x.state == EnvFileState::InSync));
        assert!(scan
            .files
            .iter()
            .all(|x| x.credential_id.is_some() && x.disk_bytes.is_some()));

        // Edited on disk: differs. Back up keeps the vault copy until that file is confirmed.
        fs::write(app.join(".env"), "A=2\n").unwrap();
        fs::write(app.join("backend/.env.local"), "DB=changed\n").unwrap();
        let differs = |scan: &EnvBackupScan| {
            scan.files
                .iter()
                .filter(|x| x.state == EnvFileState::Differs)
                .count()
        };
        assert_eq!(differs(&f.svc.scan().await.unwrap()), 2);
        assert_eq!(
            count(&f.svc.import(None).await.unwrap(), EnvOutcome::Conflict),
            2
        );
        let report = f
            .svc
            .import(Some(refs(id, false, &[".env"])))
            .await
            .unwrap();
        assert_eq!(
            (count(&report, EnvOutcome::Conflict), report.items.len()),
            (1, 1)
        );
        assert_eq!(
            differs(&f.svc.scan().await.unwrap()),
            2,
            "nothing was written"
        );
        let report = f.svc.import(Some(refs(id, true, &[".env"]))).await.unwrap();
        assert_eq!(
            (count(&report, EnvOutcome::Updated), report.items.len()),
            (1, 1)
        );
        assert_eq!(differs(&f.svc.scan().await.unwrap()), 1);

        // Back to the real content in the vault, then lose the file: only in the vault.
        fs::write(app.join(".env"), &root_env).unwrap();
        f.svc.import(Some(refs(id, true, &[".env"]))).await.unwrap();
        fs::remove_file(app.join(".env")).unwrap();
        let scan = f.svc.scan().await.unwrap();
        let lost = scan.files.iter().find(|x| x.rel_path == ".env").unwrap();
        assert_eq!(lost.state, EnvFileState::OnlyInVault);
        assert!(lost.disk_bytes.is_none());
        let restore = |overwrite| {
            vec![EnvRestoreItem {
                credential_id: lost.credential_id.clone().unwrap(),
                overwrite,
            }]
        };
        // Restore writes it back byte for byte.
        let report = f.svc.restore(restore(false)).await.unwrap();
        assert_eq!(count(&report, EnvOutcome::Restored), 1);
        assert_eq!(report.items[0].project_id, Some(id));
        assert_eq!(fs::read(app.join(".env")).unwrap(), root_env.as_bytes());
        assert_eq!(
            count(
                &f.svc.restore(restore(false)).await.unwrap(),
                EnvOutcome::Identical
            ),
            1
        );

        // A different file is never replaced without the flag, and is with it.
        fs::write(app.join(".env"), "MINE=do-not-lose\n").unwrap();
        let report = f.svc.restore(restore(false)).await.unwrap();
        assert_eq!(count(&report, EnvOutcome::Conflict), 1);
        assert_eq!(
            fs::read_to_string(app.join(".env")).unwrap(),
            "MINE=do-not-lose\n"
        );
        let report = f.svc.restore(restore(true)).await.unwrap();
        assert_eq!(count(&report, EnvOutcome::Overwritten), 1);
        assert_eq!(fs::read(app.join(".env")).unwrap(), root_env.as_bytes());
        // No temp file left behind.
        for entry in fs::read_dir(&app).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            assert!(!name.contains("portal-tmp"), "stray {name}");
        }
    }

    #[tokio::test]
    async fn scan_says_which_env_files_it_cannot_back_up() {
        let f = fixture().await;
        let app = f.tmp.path().join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join(".env"), "A=1").unwrap();
        fs::write(app.join(".env.local"), [0xFF, 0xFE, b'A', 0]).unwrap();
        add_project(&f.db, "app", &app, None).await;

        let scan = f.svc.scan().await.unwrap();
        assert_eq!(scan.files.len(), 1);
        assert_eq!(scan.skipped, ["app/.env.local (not UTF-8 text)"]);
        assert_eq!(
            count(&f.svc.import(None).await.unwrap(), EnvOutcome::Created),
            1
        );
    }

    #[tokio::test]
    async fn locked_vault_writes_nothing_and_scan_never_prompts() {
        let f = fixture().await;
        let app = f.tmp.path().join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join(".env"), SECRET).unwrap();
        add_project(&f.db, "app", &app, None).await;
        f.svc.import(None).await.unwrap();

        f.session.lock();
        let scan = f.svc.scan().await.unwrap();
        assert!(scan.vault_locked);
        assert_eq!(scan.files[0].state, EnvFileState::Unknown);
        // No prompter is wired in tests, so this is the "user cancelled" path.
        assert_eq!(f.svc.import(None).await.unwrap_err(), "Vault is locked");
        let id = scan.files[0].credential_id.clone().unwrap();
        fs::remove_file(app.join(".env")).unwrap();
        let err = f
            .svc
            .restore(vec![EnvRestoreItem {
                credential_id: id,
                overwrite: true,
            }])
            .await
            .unwrap_err();
        assert_eq!(err, "Vault is locked");
        assert!(!app.join(".env").exists());
    }

    #[tokio::test]
    async fn restore_refuses_tampered_rows_and_missing_folders() {
        let f = fixture().await;
        let app = f.tmp.path().join("app");
        fs::create_dir_all(&app).unwrap();
        let id = add_project(&f.db, "app", &app, None).await;
        let outside = f.tmp.path().join("outside");
        fs::create_dir_all(&outside).unwrap();

        let mut planted = Vec::new();
        for rel in [
            "../outside/.env",
            "src/main.rs",
            "missing/.env",
            "/abs/.env",
            ".env.local:payload",
            ".env.",
        ] {
            let row = Meta {
                rel_path: rel.into(),
                ..meta(id, "app", "")
            };
            planted.push(EnvRestoreItem {
                credential_id: add_row(&f, row).await,
                overwrite: true,
            });
        }
        // A row for a project that is not in Portal.
        planted.push(EnvRestoreItem {
            credential_id: add_row(&f, meta(99, "ghost", "")).await,
            overwrite: true,
        });

        let report = f.svc.restore(planted).await.unwrap();
        assert_eq!(
            (count(&report, EnvOutcome::Skipped), report.items.len()),
            (7, 7)
        );
        assert!(!outside.join(".env").exists());
        assert!(!app.join("src").exists() && !app.join("missing").exists());
        assert_eq!(fs::read_dir(&app).unwrap().count(), 0);
        assert_eq!(f.svc.scan().await.unwrap().unmatched, ["ghost/.env"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn restore_never_writes_through_a_symlink() {
        let f = fixture().await;
        let app = f.tmp.path().join("app");
        fs::create_dir_all(&app).unwrap();
        let shared = f.tmp.path().join("shared.env");
        fs::write(&shared, "KEEP").unwrap();
        std::os::unix::fs::symlink(&shared, app.join(".env")).unwrap();
        let id = add_project(&f.db, "app", &app, None).await;

        let item = EnvRestoreItem {
            credential_id: add_row(&f, meta(id, "app", "")).await,
            overwrite: true,
        };
        let report = f.svc.restore(vec![item]).await.unwrap();
        assert_eq!(count(&report, EnvOutcome::Skipped), 1);
        assert_eq!(fs::read_to_string(&shared).unwrap(), "KEEP");
        assert!(fs::symlink_metadata(app.join(".env"))
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[tokio::test]
    async fn a_nested_project_keeps_its_own_env_files() {
        let f = fixture().await;
        let parent = f.tmp.path().join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.join(".env"), "P=1").unwrap();
        fs::write(child.join(".env"), "C=1").unwrap();
        add_project(&f.db, "parent", &parent, None).await;
        add_project(&f.db, "child", &child, None).await;

        let report = f.svc.import(None).await.unwrap();
        let got: Vec<_> = report
            .items
            .iter()
            .map(|i| (i.project_name.as_str(), i.rel_path.as_str()))
            .collect();
        assert_eq!(got, [("child", ".env"), ("parent", ".env")]);
    }

    #[tokio::test]
    async fn restore_finds_its_project_after_ids_change() {
        // Another database's ids: same project, different id, and a different folder.
        let f = fixture().await;
        let old = f.tmp.path().join("old");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join(".env"), SECRET).unwrap();
        add_project(&f.db, "app", &old, Some("git@github.com:o/app.git")).await;
        f.svc.import(None).await.unwrap();

        let new = f.tmp.path().join("new");
        fs::create_dir_all(&new).unwrap();
        project_entity::Entity::update_many()
            .col_expr(project_entity::Column::Id, Expr::value(41))
            .col_expr(
                project_entity::Column::Path,
                Expr::value(new.to_string_lossy().into_owned()),
            )
            .exec(&f.db)
            .await
            .unwrap();
        let scan = f.svc.scan().await.unwrap();
        assert_eq!(scan.files.len(), 1);
        assert_eq!(scan.files[0].state, EnvFileState::OnlyInVault);
        assert!(scan.unmatched.is_empty());

        let id = scan.files[0].credential_id.clone().unwrap();
        let report = f
            .svc
            .restore(vec![EnvRestoreItem {
                credential_id: id,
                overwrite: false,
            }])
            .await
            .unwrap();
        assert_eq!(count(&report, EnvOutcome::Restored), 1);
        assert_eq!(report.items[0].project_id, Some(41));
        assert_eq!(fs::read_to_string(new.join(".env")).unwrap(), SECRET);
    }

    #[tokio::test]
    async fn a_deleted_projects_rows_never_land_in_a_same_named_project() {
        // "web" for repo A is deleted; a different "web" (repo B) is registered later.
        let f = fixture().await;
        let a = f.tmp.path().join("a-web");
        fs::create_dir_all(&a).unwrap();
        fs::write(a.join(".env"), SECRET).unwrap();
        let a_id = add_project(&f.db, "web", &a, Some("https://github.com/a/web.git")).await;
        f.svc.import(None).await.unwrap();
        project_entity::Entity::delete_by_id(a_id)
            .exec(&f.db)
            .await
            .unwrap();

        let b = f.tmp.path().join("b-web");
        fs::create_dir_all(&b).unwrap();
        add_project(&f.db, "web", &b, Some("https://github.com/b/web.git")).await;

        let scan = f.svc.scan().await.unwrap();
        assert_eq!(scan.unmatched, ["web/.env"]);
        assert!(scan.files.is_empty());
        let row = CredentialEntity::find().one(&f.db).await.unwrap().unwrap();
        let report = f
            .svc
            .restore(vec![EnvRestoreItem {
                credential_id: row.id,
                overwrite: true,
            }])
            .await
            .unwrap();
        assert_eq!(count(&report, EnvOutcome::Skipped), 1);
        assert!(!b.join(".env").exists());
    }
}
