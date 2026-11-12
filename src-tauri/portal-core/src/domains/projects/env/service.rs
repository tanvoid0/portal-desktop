//! Per-project `.env` viewing with shared-credential linking.
//!
//! `.env*` files are always re-read live off disk on every [`ProjectEnvService::get`]
//! call — never cached — so staleness is purely a function of comparing the
//! current file/credential state against the snapshots taken at link time,
//! never "the app wasn't running." Portal never writes to a project's `.env`
//! file: linking only touches `project_env_links`.

use std::path::Path;
use std::sync::Arc;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::entity::{ActiveModel, Column, Entity, Model};
use crate::database::DatabaseManager;
use crate::domains::credentials::services::{CredentialService, VaultSession};
use crate::domains::deployments::services::detect::list_env_files;
use crate::domains::projects::move_lock;
use crate::domains::projects::repositories::project_repository::ProjectRepository;

/// How a linked key's current state compares to the snapshots taken when it
/// was linked. First-match-wins precedence is implemented in [`drift_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub enum DriftState {
    InSync,
    FileChanged,
    CredentialChanged,
    Differs,
    NotInFile,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvLinkView {
    #[ts(type = "number")]
    pub id: i64,
    pub credential_id: String,
    pub credential_name: String,
    pub state: DriftState,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvVarView {
    pub key: String,
    pub value: String,
    pub link: Option<EnvLinkView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct EnvFileView {
    pub name: String,
    pub vars: Vec<EnvVarView>,
}

/// A link whose `env_key` is no longer present in any `.env*` file.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct OrphanEnvLink {
    pub env_key: String,
    pub link: EnvLinkView,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct ProjectEnv {
    pub files: Vec<EnvFileView>,
    pub orphan_links: Vec<OrphanEnvLink>,
}

/// Hex SHA-256 of a `.env` value — the drift snapshot, not a security
/// boundary, so a plain unsalted hash is enough to detect "did this change".
fn hash_value(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Pure precedence chain, first match wins. Kept free of `self` so it is
/// trivially unit-testable without a database.
fn drift_state(
    stored_file_value_hash: Option<&str>,
    current_hash: Option<&str>,
    stored_credential_updated_at: &str,
    credential_updated_at: &str,
    matches: Option<bool>,
) -> DriftState {
    if current_hash.is_none() {
        return DriftState::NotInFile;
    }
    if current_hash != stored_file_value_hash {
        return DriftState::FileChanged;
    }
    if stored_credential_updated_at != credential_updated_at {
        return DriftState::CredentialChanged;
    }
    match matches {
        Some(false) => DriftState::Differs,
        None => DriftState::Unknown,
        Some(true) => DriftState::InSync,
    }
}

pub struct ProjectEnvService {
    db: DatabaseConnection,
    projects: ProjectRepository,
    credentials: CredentialService,
}

impl ProjectEnvService {
    pub fn new(db: &Arc<DatabaseManager>, session: Arc<VaultSession>) -> Self {
        Self {
            db: db.get_connection_clone(),
            projects: ProjectRepository::new(db.clone()),
            credentials: CredentialService::new(db.get_connection_clone(), session),
        }
    }

    pub async fn get(&self, project_id: i32) -> Result<ProjectEnv, String> {
        let project = self
            .projects
            .get_by_id(project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        let files = list_env_files(&project.path);
        let links = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        let mut file_views = Vec::with_capacity(files.len());
        for file in &files {
            let mut vars = Vec::with_capacity(file.vars.len());
            for var in &file.vars {
                // Scoped to this file, not just the key — the same key can be in more
                // than one file, each with its own link and its own snapshot.
                let link = match links
                    .iter()
                    .find(|l| l.file_name == file.name && l.env_key == var.key)
                {
                    Some(model) => Some(self.build_link_view(model, Some(&var.value)).await?),
                    None => None,
                };
                vars.push(EnvVarView {
                    key: var.key.clone(),
                    value: var.value.clone(),
                    link,
                });
            }
            file_views.push(EnvFileView {
                name: file.name.clone(),
                vars,
            });
        }

        let mut orphan_links = Vec::new();
        for model in &links {
            let still_present = files.iter().any(|f| {
                f.name == model.file_name && f.vars.iter().any(|v| v.key == model.env_key)
            });
            if !still_present {
                orphan_links.push(OrphanEnvLink {
                    env_key: model.env_key.clone(),
                    link: self.build_link_view(model, None).await?,
                });
            }
        }

        Ok(ProjectEnv {
            files: file_views,
            orphan_links,
        })
    }

    /// Upsert on `(project_id, env_key, file_name)` — snapshots are taken now.
    pub async fn link(
        &self,
        project_id: i32,
        env_key: String,
        file_name: String,
        credential_id: String,
    ) -> Result<EnvLinkView, String> {
        let project = self
            .projects
            .get_by_id(project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        move_lock::ensure_unlocked(Path::new(&project.path))?;
        let credential = self.credentials.get_credential(&credential_id).await?;
        let current_value = current_file_value(&project.path, &file_name, &env_key);

        Entity::delete_many()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::EnvKey.eq(env_key.clone()))
            .filter(Column::FileName.eq(file_name.clone()))
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        let model = ActiveModel {
            project_id: Set(project_id),
            env_key: Set(env_key),
            file_name: Set(file_name),
            credential_id: Set(credential_id),
            file_value_hash: Set(current_value.as_deref().map(hash_value)),
            credential_updated_at: Set(credential.updated_at.and_utc().to_rfc3339()),
            created_at: Set(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        self.build_link_view(&model, current_value.as_deref()).await
    }

    pub async fn unlink(&self, link_id: i64) -> Result<(), String> {
        Entity::delete_by_id(link_id)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Re-snapshots both hashes — "acknowledge the drift".
    pub async fn resync(&self, link_id: i64) -> Result<EnvLinkView, String> {
        let model = Entity::find_by_id(link_id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Link not found".to_string())?;
        let project = self
            .projects
            .get_by_id(model.project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        move_lock::ensure_unlocked(Path::new(&project.path))?;
        let credential = self.credentials.get_credential(&model.credential_id).await?;
        let current_value = current_file_value(&project.path, &model.file_name, &model.env_key);

        let mut active: ActiveModel = model.into();
        active.file_value_hash = Set(current_value.as_deref().map(hash_value));
        active.credential_updated_at = Set(credential.updated_at.and_utc().to_rfc3339());
        let updated = active.update(&self.db).await.map_err(|e| e.to_string())?;

        self.build_link_view(&updated, current_value.as_deref())
            .await
    }

    async fn build_link_view(
        &self,
        model: &Model,
        current_value: Option<&str>,
    ) -> Result<EnvLinkView, String> {
        let credential = self.credentials.get_credential(&model.credential_id).await?;
        let current_hash = current_value.map(hash_value);
        // Any decrypt failure on this one link (locked vault, corrupt ciphertext, whatever)
        // is "can't compare" for this link, not a reason to fail the whole env tab.
        let peeked = self
            .credentials
            .peek_value(&model.credential_id)
            .await
            .unwrap_or(None);
        let matches = peeked
            .as_ref()
            .map(|pv| current_value.map(|cv| pv == cv).unwrap_or(false));
        let credential_updated_at = credential.updated_at.and_utc().to_rfc3339();

        let state = drift_state(
            model.file_value_hash.as_deref(),
            current_hash.as_deref(),
            &model.credential_updated_at,
            &credential_updated_at,
            matches,
        );

        Ok(EnvLinkView {
            id: model.id,
            credential_id: model.credential_id.clone(),
            credential_name: credential.name,
            state,
        })
    }
}

/// The value of `env_key` in the specific `.env*` file named, live off disk.
fn current_file_value(project_path: &str, file_name: &str, env_key: &str) -> Option<String> {
    list_env_files(project_path)
        .into_iter()
        .find(|f| f.name == file_name)?
        .vars
        .into_iter()
        .find(|v| v.key == env_key)
        .map(|v| v.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_in_file_wins_first() {
        assert_eq!(
            drift_state(Some("a"), None, "t1", "t1", Some(true)),
            DriftState::NotInFile
        );
    }

    #[test]
    fn file_changed_beats_everything_below_it() {
        assert_eq!(
            drift_state(Some("a"), Some("b"), "t1", "t2", Some(false)),
            DriftState::FileChanged
        );
    }

    #[test]
    fn credential_changed_when_file_matches_but_credential_moved() {
        assert_eq!(
            drift_state(Some("a"), Some("a"), "t1", "t2", Some(false)),
            DriftState::CredentialChanged
        );
    }

    #[test]
    fn differs_when_snapshots_match_but_live_values_do_not() {
        assert_eq!(
            drift_state(Some("a"), Some("a"), "t1", "t1", Some(false)),
            DriftState::Differs
        );
    }

    #[test]
    fn unknown_when_vault_locked() {
        assert_eq!(
            drift_state(Some("a"), Some("a"), "t1", "t1", None),
            DriftState::Unknown
        );
    }

    #[test]
    fn in_sync_when_nothing_moved_and_values_match() {
        assert_eq!(
            drift_state(Some("a"), Some("a"), "t1", "t1", Some(true)),
            DriftState::InSync
        );
    }
}
