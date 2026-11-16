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
use std::path::Path;
use std::sync::Arc;

use crate::database::DatabaseManager;
use crate::domains::projects::discovery::{DiscoveredProject, DiscoveryReport};
use crate::domains::projects::move_lock;
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
        let locked_sources =
            crate::domains::projects::relocation::usage::in_flight_move_sources(&self.db).await;
        // A project mid-move is neither re-imported at its old path nor archived as missing.
        let moving = |abs: &str| {
            move_lock::is_locked(Path::new(abs))
                || locked_sources.iter().any(|s| move_lock::is_under(abs, s))
        };

        let conn = self.db.get_connection();
        let txn = conn.begin().await.map_err(|e| e.to_string())?;

        let existing = Project::find().all(&txn).await.map_err(|e| e.to_string())?;

        let mut outcome = ImportOutcome::default();
        let now = Utc::now();

        for p in &report.projects {
            if moving(&p.abs) {
                continue;
            }
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
            // Re-read rather than reuse `existing`: the upsert loop above may have
            // just adopted a row — repointing a relinked or moved project's
            // `discovery_path` onto its new location. Checking that against the
            // pre-upsert snapshot would still see the old key, which this scan
            // can't have found there, and archive a row that is alive one step up.
            let existing = Project::find().all(&txn).await.map_err(|e| e.to_string())?;
            let seen: std::collections::HashSet<&str> =
                report.projects.iter().map(|p| p.path.as_str()).collect();
            // Only roots this run actually walked. A root whose drive was not
            // mounted is absent from `roots_scanned`, and archiving everything
            // under it would punish the user for unplugging a disk.
            let walked: std::collections::HashSet<&str> =
                report.roots_scanned.iter().map(String::as_str).collect();
            for row in existing.iter().filter(|r| {
                r.discovery_path.is_some() && r.status != "archived" && !moving(&r.path)
            }) {
                let key = row.discovery_path.as_deref().unwrap_or_default();
                if seen.contains(key) {
                    continue;
                }
                if !walked.contains(key.split('/').next().unwrap_or_default()) {
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

    /// Id, name and path of every registry row — all the unavailable check needs.
    pub async fn list_paths(&self) -> Result<Vec<(i32, String, String)>, String> {
        let rows = Project::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|r| (r.id, r.name, r.path)).collect())
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
    use crate::domains::projects::move_lock::norm;
    norm(a) == norm(b)
}

/// Fields discovery owns. Never touches user-authored content.
fn apply_machine_fields(am: &mut project::ActiveModel, p: &DiscoveredProject, now: DateTime<Utc>) {
    am.activity_status = Set(Some(p.status.as_str().to_string()));
    am.last_activity = Set(to_dt(p.activity).map(|d| d.into()));
    am.risk_level = Set(Some(p.risk_level.as_str().to_string()));
    am.unpushed_commits = Set(p.git.ahead.max(0) as i32);
    am.stash_count = Set(p.git.stash.max(0) as i32);
    am.has_remote = Set(p
        .git
        .remote
        .as_deref()
        .map(|r| !r.is_empty())
        .unwrap_or(false));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::projects::discovery::{
        ActivityStatus, DiscoveryStats, ProjectGitSummary, RiskLevel,
    };
    use crate::domains::projects::services::project_service::ProjectService;

    async fn test_db() -> (Arc<DatabaseManager>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = DatabaseManager::new(dir.path().to_path_buf())
            .await
            .expect("db init");
        (Arc::new(db), dir)
    }

    /// A report with one project found under `roots_scanned`, for the
    /// `mark_missing` path.
    fn report(path: &str, abs: &str, roots_scanned: &[&str]) -> DiscoveryReport {
        DiscoveryReport {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            cancelled: false,
            projects: vec![DiscoveredProject {
                path: path.to_string(),
                abs: abs.to_string(),
                name: "foo".to_string(),
                area: "apps".to_string(),
                bucket: "demo".to_string(),
                module_count: 0,
                modules: vec![],
                activity: 0,
                age_days: None,
                status: ActivityStatus::Active,
                git: ProjectGitSummary::default(),
                langs: vec![],
                regenerable: vec![],
                regenerable_bytes: -1,
                description: None,
                risk: vec![],
                risk_level: RiskLevel::None,
                duplicate_of: vec![],
            }],
            clusters: vec![],
            stats: DiscoveryStats {
                projects: 1,
                roots: 1,
                git_repos: 0,
                unpushed: 0,
                no_remote: 0,
                active: 1,
                recent: 0,
                dormant: 0,
                stale: 0,
                regenerable_bytes: -1,
            },
            roots_scanned: roots_scanned.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// board #210: relink (`UnavailableProjects.svelte`'s `update_project` call)
    /// repoints `path` but leaves `discovery_path` stale — exactly what
    /// `ProjectService::update_project` does below. `mark_missing` used to read
    /// that staleness off the snapshot taken *before* this same `import()`
    /// call's upsert loop had already adopted the row onto its new key, and
    /// archived a project this very scan had just found alive.
    #[tokio::test]
    async fn relinked_project_found_again_is_not_archived() {
        let (db, _dir) = test_db().await;
        let conn = db.get_connection();

        // Seed a row the way discovery itself would have, the first time it
        // found this project.
        let now = Utc::now();
        let seed = project::ActiveModel {
            name: Set("foo".to_string()),
            path: Set("C:/old/foo".to_string()),
            status: Set("active".to_string()),
            starred: Set(false),
            open_count: Set(0),
            size: Set(0),
            file_count: Set(0),
            has_uncommitted_changes: Set(false),
            discovery_path: Set(Some("roots/apps/foo".to_string())),
            created_at: Set(Some(now.into())),
            ..Default::default()
        };
        let row = seed.insert(conn).await.expect("seed row");

        // Relink to where the folder went — the real command, not a stand-in.
        let service = ProjectService::new(&db);
        service
            .update_project(
                row.id,
                None,                           // name
                None,                           // description
                Some("C:/new/foo".to_string()), // path
                None,                           // status
                None,                           // framework_ids
                None,                           // package_manager_ids
                None,                           // language_ids
                None,                           // build_command
                None,                           // start_command
                None,                           // test_command
                None,                           // output_directory
                None,                           // dev_port
                None,                           // prod_port
            )
            .await
            .expect("relink");

        // The next scan walks the same root and finds the project alive at its
        // new path, under a fresh key (a rename, not just a drive letter change).
        let rpt = report("roots/apps/foo-renamed", "C:/new/foo", &["roots"]);

        let repo = DiscoveryRepository::new(db.clone());
        let outcome = repo.import(&rpt, true).await.expect("import");

        assert_eq!(outcome.adopted, vec!["roots/apps/foo-renamed".to_string()]);
        assert!(
            outcome.missing.is_empty(),
            "wrongly archived: {:?}",
            outcome.missing
        );

        let reloaded = Project::find_by_id(row.id)
            .one(conn)
            .await
            .expect("reload")
            .expect("row still exists");
        assert_eq!(reloaded.status, "active");
        assert_eq!(
            reloaded.discovery_path,
            Some("roots/apps/foo-renamed".to_string())
        );
    }
}
