//! Persists a discovery run into the `projects` registry.
//!
//! Upserts are keyed on `discovery_path` (the logical id) and fall back to the
//! absolute `path` so projects added by hand before discovery existed get
//! adopted rather than duplicated.
//!
//! Anything the user owns is never overwritten: `name`, `description`, the
//! build/start/test commands, ports, `starred` and the whole triage block are
//! only ever written on insert, or when discovery has a value and the row does
//! not. Machine-owned fields (git state, activity, risk) are refreshed freely.

use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait, Unchanged,
};
use std::sync::Arc;

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::{DiscoveredProject, DiscoveryReport};
use crate::entities::project::{self, Entity as Project};

/// What a single import changed, so the UI can report it honestly.
#[derive(Debug, Clone, Default)]
pub struct ImportOutcome {
    pub inserted: Vec<String>,
    pub updated: Vec<String>,
    pub adopted: Vec<String>,
    pub missing: Vec<String>,
}

pub struct DiscoveryRepository {
    db: Arc<DatabaseManager>,
}

fn to_dt(ms: i64) -> Option<DateTime<Utc>> {
    if ms <= 0 {
        return None;
    }
    Utc.timestamp_millis_opt(ms).single()
}

fn join(values: &[String]) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values.join(","))
    }
}

impl DiscoveryRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    /// Writes the report into the registry. Returns which rows were touched.
    ///
    /// `mark_missing` flips registry rows the scan did not see to `archived`
    /// rather than deleting them: a folder can be missing because a drive is
    /// unplugged, and silently destroying the row (and its triage note) would
    /// be unrecoverable.
    pub async fn import(
        &self,
        report: &DiscoveryReport,
        mark_missing: bool,
    ) -> Result<ImportOutcome, String> {
        let conn = self.db.get_connection();
        let txn = conn.begin().await.map_err(|e| e.to_string())?;

        let existing = Project::find()
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;

        let mut outcome = ImportOutcome::default();
        let now = Utc::now();

        for p in &report.projects {
            let by_discovery = existing
                .iter()
                .find(|e| e.discovery_path.as_deref() == Some(p.path.as_str()));
            let by_path = existing.iter().find(|e| paths_equal(&e.path, &p.abs));

            match by_discovery.or(by_path) {
                Some(row) => {
                    let adopted = by_discovery.is_none();
                    let mut am: project::ActiveModel = row.clone().into();
                    am.id = Unchanged(row.id);
                    apply_machine_fields(&mut am, p, now);

                    // Adopt a hand-made row: give it an identity, and fill only
                    // the descriptive gaps the user left empty.
                    am.discovery_path = Set(Some(p.path.clone()));
                    am.path = Set(p.abs.clone());
                    if row.description.as_deref().unwrap_or("").is_empty() {
                        if let Some(d) = &p.description {
                            am.description = Set(Some(d.clone()));
                        }
                    }

                    am.update(&txn).await.map_err(|e| e.to_string())?;
                    if adopted {
                        outcome.adopted.push(p.path.clone());
                    } else {
                        outcome.updated.push(p.path.clone());
                    }
                }
                None => {
                    let mut am = project::ActiveModel {
                        name: Set(p.name.clone()),
                        description: Set(p.description.clone()),
                        path: Set(p.abs.clone()),
                        status: Set("active".to_string()),
                        starred: Set(false),
                        open_count: Set(0),
                        size: Set(0),
                        file_count: Set(0),
                        has_uncommitted_changes: Set(p.git.dirty > 0),
                        created_at: Set(Some(now.into())),
                        ..Default::default()
                    };
                    apply_machine_fields(&mut am, p, now);
                    am.discovery_path = Set(Some(p.path.clone()));
                    am.insert(&txn).await.map_err(|e| e.to_string())?;
                    outcome.inserted.push(p.path.clone());
                }
            }
        }

        if mark_missing {
            let seen: std::collections::HashSet<&str> =
                report.projects.iter().map(|p| p.path.as_str()).collect();
            for row in existing
                .iter()
                .filter(|r| r.discovery_path.is_some() && r.status != "archived")
            {
                let key = row.discovery_path.as_deref().unwrap_or_default();
                if seen.contains(key) {
                    continue;
                }
                let mut am: project::ActiveModel = row.clone().into();
                am.id = Unchanged(row.id);
                am.status = Set("archived".to_string());
                am.updated_at = Set(Some(now.into()));
                am.update(&txn).await.map_err(|e| e.to_string())?;
                outcome.missing.push(key.to_string());
            }
        }

        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(outcome)
    }

    /// Saves a triage decision. Separate from `update_project` so a triage pass
    /// can never clobber build settings.
    pub async fn set_triage(
        &self,
        id: i32,
        decision: Option<String>,
        note: Option<String>,
        successor: Option<String>,
    ) -> Result<bool, String> {
        let conn = self.db.get_connection();
        let row = Project::find_by_id(id)
            .one(conn)
            .await
            .map_err(|e| e.to_string())?;
        let Some(row) = row else {
            return Ok(false);
        };
        let mut am: project::ActiveModel = row.clone().into();
        am.id = Unchanged(row.id);
        am.triage_decision = Set(decision);
        am.triage_note = Set(note);
        am.triage_successor = Set(successor);
        am.triage_updated_at = Set(Some(Utc::now().into()));
        am.update(conn).await.map_err(|e| e.to_string())?;
        Ok(true)
    }

    /// Bulk decision for the "select 20 stale course repos, archive them" case.
    pub async fn set_triage_bulk(
        &self,
        ids: Vec<i32>,
        decision: Option<String>,
    ) -> Result<u64, String> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.db.get_connection();
        let txn = conn.begin().await.map_err(|e| e.to_string())?;
        let rows = Project::find()
            .filter(project::Column::Id.is_in(ids))
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        let now = Utc::now();
        let mut n = 0u64;
        for row in rows {
            let mut am: project::ActiveModel = row.clone().into();
            am.id = Unchanged(row.id);
            am.triage_decision = Set(decision.clone());
            am.triage_updated_at = Set(Some(now.into()));
            am.update(&txn).await.map_err(|e| e.to_string())?;
            n += 1;
        }
        txn.commit().await.map_err(|e| e.to_string())?;
        Ok(n)
    }
}

/// Windows paths differ only by case and separator; comparing them raw would
/// create a duplicate row for a project the user already added by hand.
fn paths_equal(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        s.replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase()
    }
    norm(a) == norm(b)
}

/// Fields discovery owns. Never touches user-authored content.
fn apply_machine_fields(
    am: &mut project::ActiveModel,
    p: &DiscoveredProject,
    now: DateTime<Utc>,
) {
    am.activity_status = Set(Some(p.status.as_str().to_string()));
    am.last_activity = Set(to_dt(p.activity).map(|d| d.into()));
    am.risk_level = Set(Some(p.risk_level.as_str().to_string()));
    am.unpushed_commits = Set(p.git.ahead.max(0) as i32);
    am.stash_count = Set(p.git.stash.max(0) as i32);
    am.has_remote = Set(p.git.remote.as_deref().map(|r| !r.is_empty()).unwrap_or(false));
    am.module_count = Set(p.module_count as i32);
    am.duplicate_group = Set(join(&p.duplicate_of));
    am.regenerable_dirs = Set(join(&p.regenerable));
    am.discovered_at = Set(Some(now.into()));

    am.git_repository = Set(p.git.remote.clone());
    am.git_branch = Set(p.git.branch.clone());
    // `dirty` is -1 when unmeasured; only a positive count means dirty.
    am.has_uncommitted_changes = Set(p.git.dirty > 0);
    am.last_commit = Set(p
        .git
        .last_commit
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc).into()));
    am.updated_at = Set(Some(now.into()));
}
