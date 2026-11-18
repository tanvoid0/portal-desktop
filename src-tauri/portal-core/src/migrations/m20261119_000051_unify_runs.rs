//! One run table. A single action runs as a one-step pipeline execution, so
//! `pipeline_executions` learns what `script_executions` carries: `pipeline_id`
//! / `project_id` become nullable (an action may run in a folder that belongs to
//! no project) and it gains `kind`, `command`, `working_directory`, `block_id`,
//! the workflow-run grouping and `pid`.
//!
//! History moves with it: every `script_executions` row is copied in as a
//! `kind = 'action'` row with the SAME id, so `/pipeline-runs/<id>` links and
//! run ids held elsewhere keep resolving. `script_executions` is left in place
//! and stays the full-fidelity rollback copy (and proof the copy is complete):
//! the copy here keeps only the last [`MAX_OUTPUT`] bytes of a run's output.
//! A later migration drops the table.
//!
//! The copy is a snapshot. `ScriptExecutionService` keeps writing
//! `script_executions` until C4 makes it a facade over the pipeline runner, so
//! C4 adds a catch-up migration that calls [`copy_script_history`] again with
//! `replace = true`.
//!
//! Forward-only: no `down`. All of it is one transaction, and re-running is a
//! no-op (the rebuild is skipped once `kind` exists, the copy is INSERT OR IGNORE).

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{DbBackend, Statement, TransactionTrait, Value};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Rows per page of the history copy, and per multi-row INSERT: 100 x 15 bound
/// values stays far under SQLite's variable limit.
const BATCH: usize = 100;

/// Longest `output` a copied row keeps (its tail); `logs` is built from the
/// same text, so neither can grow past this.
const MAX_OUTPUT: usize = 4 * 1024 * 1024;
const TRUNCATED: &str =
    "[earlier output truncated by migration; full text kept in script_executions]";

const RESTART_ERROR: &str = "Interrupted by app restart";

/// `pipeline_executions` as it was, plus the new columns. Nothing references
/// the table (checked: only its own foreign keys point out), so dropping it
/// cascades nowhere.
const REBUILD: [&str; 4] = [
    "CREATE TABLE pipeline_executions_new (
        id varchar NOT NULL PRIMARY KEY,
        pipeline_id integer NULL,
        project_id integer NULL,
        status varchar NOT NULL DEFAULT 'pending',
        started_at timestamp_with_timezone_text NOT NULL,
        finished_at timestamp_with_timezone_text NULL,
        triggered_by varchar NOT NULL,
        step_executions_json text NOT NULL,
        variables_json text NOT NULL,
        error text NULL,
        kind varchar NOT NULL DEFAULT 'pipeline',
        command text NULL,
        working_directory text NULL,
        block_id varchar NULL,
        workflow_run_id varchar NULL,
        workflow_id varchar NULL,
        step_id varchar NULL,
        workflow_plan_json text NULL,
        pid integer NULL,
        CONSTRAINT fk_pipeline_executions_pipeline_id FOREIGN KEY (pipeline_id) REFERENCES pipelines (id) ON DELETE CASCADE,
        CONSTRAINT fk_pipeline_executions_project_id FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
    )",
    // An id whose pipeline/project is gone (a row written while foreign keys
    // were off) becomes NULL instead of failing the insert, and startup with it.
    "INSERT INTO pipeline_executions_new
        (id, pipeline_id, project_id, status, started_at, finished_at, triggered_by,
         step_executions_json, variables_json, error)
     SELECT id,
         CASE WHEN pipeline_id IN (SELECT id FROM pipelines) THEN pipeline_id END,
         CASE WHEN project_id IN (SELECT id FROM projects) THEN project_id END,
         status, started_at, finished_at, triggered_by,
         step_executions_json, variables_json, error
     FROM pipeline_executions",
    "DROP TABLE pipeline_executions",
    "ALTER TABLE pipeline_executions_new RENAME TO pipeline_executions",
];

const INDEXES: [&str; 2] = [
    "CREATE INDEX IF NOT EXISTS idx_pipeline_executions_workflow_run_id ON pipeline_executions (workflow_run_id)",
    "CREATE INDEX IF NOT EXISTS idx_pipeline_executions_started_at ON pipeline_executions (started_at)",
];

fn sql(s: &str) -> Statement {
    Statement::from_string(DbBackend::Sqlite, s.to_string())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Checked before the transaction opens: it holds the pool's connection.
        let rebuild = !manager.has_column("pipeline_executions", "kind").await?;
        let txn = manager.get_connection().begin().await?;

        if rebuild {
            // A half-done earlier attempt rolled back with its transaction, so
            // the scratch table can only be a stray from outside.
            txn.execute(sql("DROP TABLE IF EXISTS pipeline_executions_new"))
                .await?;
            for stmt in REBUILD {
                txn.execute(sql(stmt)).await?;
            }
        }
        for stmt in INDEXES {
            txn.execute(sql(stmt)).await?;
        }

        copy_script_history(&txn, false).await?;

        txn.commit().await
    }
}

/// The part of `output` a copied row keeps: all of it, or its last `max` bytes
/// (cut on a char boundary) behind a marker line.
fn tail(output: String, max: usize) -> String {
    if output.len() <= max {
        return output;
    }
    let mut from = output.len() - max;
    while !output.is_char_boundary(from) {
        from += 1;
    }
    format!("{TRUNCATED}\n{}", &output[from..])
}

/// Copy every `script_executions` row into `pipeline_executions` as a
/// `kind = 'action'` row with the same id (see the module docs for the mapping).
/// `replace = false` leaves rows that are already there alone (INSERT OR
/// IGNORE); `replace = true` overwrites them (INSERT OR REPLACE), for a
/// catch-up migration that re-snapshots the table. Pages by `rowid`, so no more
/// than [`BATCH`] rows are in memory at once.
pub(crate) async fn copy_script_history(
    db: &impl ConnectionTrait,
    replace: bool,
) -> Result<(), DbErr> {
    let select = format!(
        "SELECT rowid AS rid, id, block_id, command, parameters_json, working_directory, status,
                exit_code, pid, output, error,
                CAST(started_at AS TEXT) AS started_at,
                CAST(finished_at AS TEXT) AS finished_at,
                triggered_by, workflow_run_id, workflow_id, step_id, workflow_plan_json
         FROM script_executions WHERE rowid > ? ORDER BY rowid LIMIT {BATCH}"
    );
    let verb = if replace { "OR REPLACE" } else { "OR IGNORE" };
    let mut after = 0i64;
    loop {
        let page = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &select,
                [after.into()],
            ))
            .await?;
        let Some(last) = page.last() else {
            return Ok(());
        };
        after = last.try_get("", "rid")?;

        let mut values: Vec<Value> = Vec::with_capacity(page.len() * 15);
        for r in &page {
            let s = |c: &str| r.try_get::<String>("", c);
            let os = |c: &str| r.try_get::<Option<String>>("", c);
            let id = s("id")?;
            let command = s("command")?;
            let output = tail(s("output")?, MAX_OUTPUT);
            let started = s("started_at")?;
            let finished = os("finished_at")?;
            let exit_code = r.try_get::<Option<i32>>("", "exit_code")?;

            let mut status = s("status")?;
            let mut error = os("error")?;
            if status == "running" || status == "pending" {
                status = "failed".to_string();
                error = Some(RESTART_ERROR.to_string());
            }
            let duration = match (
                chrono::DateTime::parse_from_rfc3339(&started),
                finished.as_deref().map(chrono::DateTime::parse_from_rfc3339),
            ) {
                (Ok(a), Some(Ok(b))) => Some((b - a).num_milliseconds()),
                _ => None,
            };
            let step_id = os("step_id")?;
            let mut step = serde_json::json!({
                // The step state the runner reads needs `id` and `retryCount`.
                "id": id,
                "stepId": step_id.as_deref().unwrap_or("run"),
                "stepName": command,
                "status": status,
                "startedAt": started,
                "output": output,
                "logs": output.lines().collect::<Vec<_>>(),
                "retryCount": 0,
            });
            for (k, v) in [
                ("finishedAt", serde_json::json!(finished)),
                ("exitCode", serde_json::json!(exit_code)),
                ("error", serde_json::json!(error)),
                ("duration", serde_json::json!(duration)),
            ] {
                if !matches!(v, serde_json::Value::Null) {
                    step[k] = v;
                }
            }

            values.extend([
                id.into(),
                status.into(),
                started.into(),
                finished.into(),
                s("triggered_by")?.into(),
                serde_json::json!([step]).to_string().into(),
                s("parameters_json")?.into(),
                error.into(),
                command.into(),
                os("working_directory")?.into(),
                os("block_id")?.into(),
                os("workflow_run_id")?.into(),
                os("workflow_id")?.into(),
                step_id.into(),
                os("workflow_plan_json")?.into(),
            ]);
            values.push(r.try_get::<Option<i32>>("", "pid")?.into());
        }
        let row = "(?, NULL, NULL, ?, ?, ?, ?, ?, ?, ?, 'action', ?, ?, ?, ?, ?, ?, ?, ?)";
        let stmt = format!(
            "INSERT {verb} INTO pipeline_executions
             (id, pipeline_id, project_id, status, started_at, finished_at, triggered_by,
              step_executions_json, variables_json, error, kind, command, working_directory,
              block_id, workflow_run_id, workflow_id, step_id, workflow_plan_json, pid)
             VALUES {}",
            vec![row; page.len()].join(", ")
        );
        db.execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            stmt,
            values,
        ))
        .await?;
    }
}

#[cfg(test)]
mod tests {
    use super::{copy_script_history, tail, MAX_OUTPUT, TRUNCATED};
    use crate::migrations::Migrator;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
    use sea_orm_migration::MigratorTrait;
    use serde_json::Value;

    async fn run(db: &DatabaseConnection, sql: &str) {
        db.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await
            .unwrap();
    }

    async fn rows(db: &DatabaseConnection, sql: &str) -> Vec<sea_orm::QueryResult> {
        db.query_all(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await
            .unwrap()
    }

    async fn count(db: &DatabaseConnection, table: &str) -> i64 {
        rows(db, &format!("SELECT COUNT(*) AS n FROM {table}"))
            .await[0]
            .try_get("", "n")
            .unwrap()
    }

    /// A database migrated up to just before this migration, with a project +
    /// pipeline + execution and three legacy script runs seeded.
    async fn legacy_db() -> (DatabaseConnection, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("t.db").display());
        let db = Database::connect(&url).await.unwrap();
        let all = Migrator::migrations().len() as u32;
        Migrator::up(&db, Some(all - 1)).await.unwrap();

        run(&db, "INSERT INTO projects (name, path, created_at, updated_at)
                  VALUES ('p', '/p', '2026-01-01T00:00:00+00:00', '2026-01-01T00:00:00+00:00')").await;
        run(&db, "INSERT INTO pipelines (name, project_id, steps_json, variables_json, secrets_json, execution_context_json, enabled)
                  VALUES ('pl', 1, '[]', '[]', '[]', '{}', 1)").await;
        run(&db, "INSERT INTO pipeline_executions (id, pipeline_id, project_id, status, started_at, triggered_by, step_executions_json, variables_json)
                  VALUES ('pe', 1, 1, 'success', '2026-01-02T00:00:00+00:00', 'user', '[]', '{}')").await;
        // done, with output and workflow ids
        run(&db, "INSERT INTO script_executions (id, command, parameters_json, working_directory, status, exit_code, output, started_at, finished_at, triggered_by, workflow_run_id, workflow_id, step_id, workflow_plan_json)
                  VALUES ('s-done', 'echo hi', '{\"a\":\"1\"}', '/w', 'success', 0, 'one\ntwo\n', '2026-01-03T00:00:00+00:00', '2026-01-03T00:00:02+00:00', 'user', 'wr', 'wf', 'st', '[{\"id\":\"st\"}]')").await;
        // mid-run when the app died
        run(&db, "INSERT INTO script_executions (id, command, parameters_json, status, pid, output, started_at, triggered_by)
                  VALUES ('s-run', 'sleep 9', '{}', 'running', 42, 'partial\n', '2026-01-04T00:00:00+00:00', 'user')").await;
        run(&db, "INSERT INTO script_executions (id, command, parameters_json, status, output, started_at, triggered_by)
                  VALUES ('s-pend', 'ls', '{}', 'pending', '', '2026-01-05T00:00:00+00:00', 'user')").await;
        (db, dir)
    }

    #[tokio::test]
    async fn legacy_history_moves_into_pipeline_executions() {
        let (db, _dir) = legacy_db().await;
        Migrator::up(&db, None).await.unwrap();

        assert_eq!(count(&db, "script_executions").await, 3, "legacy table kept");
        assert_eq!(count(&db, "pipeline_executions").await, 4);

        let pe = &rows(&db, "SELECT * FROM pipeline_executions WHERE id = 'pe'").await[0];
        assert_eq!(pe.try_get::<String>("", "kind").unwrap(), "pipeline");
        assert_eq!(pe.try_get::<Option<i32>>("", "pipeline_id").unwrap(), Some(1));
        assert_eq!(pe.try_get::<Option<i32>>("", "project_id").unwrap(), Some(1));
        assert_eq!(pe.try_get::<String>("", "status").unwrap(), "success");

        let done = &rows(&db, "SELECT * FROM pipeline_executions WHERE id = 's-done'").await[0];
        assert_eq!(done.try_get::<String>("", "kind").unwrap(), "action");
        assert_eq!(done.try_get::<Option<i32>>("", "pipeline_id").unwrap(), None);
        assert_eq!(done.try_get::<Option<i32>>("", "project_id").unwrap(), None);
        assert_eq!(done.try_get::<String>("", "variables_json").unwrap(), r#"{"a":"1"}"#);
        assert_eq!(done.try_get::<String>("", "workflow_run_id").unwrap(), "wr");
        assert_eq!(done.try_get::<String>("", "working_directory").unwrap(), "/w");
        let steps: Vec<Value> =
            serde_json::from_str(&done.try_get::<String>("", "step_executions_json").unwrap()).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["stepId"], "st");
        assert_eq!(steps[0]["stepName"], "echo hi");
        assert_eq!(steps[0]["status"], "success");
        assert_eq!(steps[0]["exitCode"], 0);
        assert_eq!(steps[0]["duration"], 2000);
        assert_eq!(steps[0]["output"], "one\ntwo\n");
        assert_eq!(steps[0]["logs"], serde_json::json!(["one", "two"]));

        let run_row = &rows(&db, "SELECT * FROM pipeline_executions WHERE id = 's-run'").await[0];
        assert_eq!(run_row.try_get::<String>("", "status").unwrap(), "failed");
        assert_eq!(run_row.try_get::<String>("", "error").unwrap(), "Interrupted by app restart");
        assert_eq!(run_row.try_get::<Option<i32>>("", "pid").unwrap(), Some(42));
        let steps: Vec<Value> =
            serde_json::from_str(&run_row.try_get::<String>("", "step_executions_json").unwrap()).unwrap();
        assert_eq!(steps[0]["stepId"], "run");
        assert_eq!(steps[0]["status"], "failed");
        assert_eq!(steps[0]["logs"], serde_json::json!(["partial"]));

        let pend = &rows(&db, "SELECT status FROM pipeline_executions WHERE id = 's-pend'").await[0];
        assert_eq!(pend.try_get::<String>("", "status").unwrap(), "failed");
    }

    #[tokio::test]
    async fn running_the_migration_twice_is_a_no_op() {
        let (db, _dir) = legacy_db().await;
        Migrator::up(&db, None).await.unwrap();
        let before = rows(&db, "SELECT * FROM pipeline_executions ORDER BY id").await.len();
        run(&db, "DELETE FROM seaql_migrations WHERE version LIKE '%000051%'").await;
        Migrator::up(&db, None).await.unwrap();
        assert_eq!(rows(&db, "SELECT * FROM pipeline_executions ORDER BY id").await.len(), before);
        assert_eq!(count(&db, "pipeline_executions").await, 4);
    }

    #[test]
    fn tail_keeps_the_end_and_cuts_on_a_char_boundary() {
        assert_eq!(tail("short".into(), 10), "short");
        // "é" is two bytes: byte 3 of "éééé" is mid-char, so the cut moves to 4.
        assert_eq!(tail("éééé".into(), 5), format!("{TRUNCATED}\néé"));
    }

    #[tokio::test]
    async fn a_dangling_pipeline_or_project_id_becomes_null_instead_of_failing_startup() {
        let (db, _dir) = legacy_db().await;
        // A row written while foreign keys were off. The pool holds one
        // connection, so the pragma sticks until it is turned back on.
        run(&db, "PRAGMA foreign_keys = OFF").await;
        run(&db, "INSERT INTO pipeline_executions (id, pipeline_id, project_id, status, started_at, triggered_by, step_executions_json, variables_json)
                  VALUES ('orphan', 99, 98, 'failed', '2026-01-02T00:00:00+00:00', 'user', '[]', '{}')").await;
        run(&db, "PRAGMA foreign_keys = ON").await;

        Migrator::up(&db, None).await.unwrap();

        let o = &rows(&db, "SELECT * FROM pipeline_executions WHERE id = 'orphan'").await[0];
        assert_eq!(o.try_get::<Option<i32>>("", "pipeline_id").unwrap(), None);
        assert_eq!(o.try_get::<Option<i32>>("", "project_id").unwrap(), None);
        assert_eq!(o.try_get::<String>("", "kind").unwrap(), "pipeline");
        // A healthy row keeps its ids.
        let pe = &rows(&db, "SELECT pipeline_id FROM pipeline_executions WHERE id = 'pe'").await[0];
        assert_eq!(pe.try_get::<Option<i32>>("", "pipeline_id").unwrap(), Some(1));
    }

    #[tokio::test]
    async fn big_output_keeps_its_tail_and_the_legacy_row_stays_whole() {
        let (db, _dir) = legacy_db().await;
        // 5,000,000 characters on one line.
        run(&db, "INSERT INTO script_executions (id, command, parameters_json, status, output, started_at, triggered_by)
                  VALUES ('s-big', 'yes', '{}', 'success', hex(zeroblob(2500000)), '2026-01-06T00:00:00+00:00', 'user')").await;
        Migrator::up(&db, None).await.unwrap();

        let big = &rows(&db, "SELECT step_executions_json FROM pipeline_executions WHERE id = 's-big'").await[0];
        let steps: Vec<Value> =
            serde_json::from_str(&big.try_get::<String>("", "step_executions_json").unwrap()).unwrap();
        let output = steps[0]["output"].as_str().unwrap();
        assert!(output.starts_with(TRUNCATED));
        assert_eq!(output.len(), TRUNCATED.len() + 1 + MAX_OUTPUT);
        // `logs` comes from the trimmed text: the marker line and the one long line.
        assert_eq!(steps[0]["logs"].as_array().unwrap().len(), 2);

        let kept = &rows(&db, "SELECT length(output) AS n FROM script_executions WHERE id = 's-big'").await[0];
        assert_eq!(kept.try_get::<i64>("", "n").unwrap(), 5_000_000, "rollback copy untouched");
    }

    #[tokio::test]
    async fn history_copy_pages_through_every_row_and_replace_mode_refreshes() {
        let (db, _dir) = legacy_db().await;
        run(&db, "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 250)
                  INSERT INTO script_executions (id, command, parameters_json, status, output, started_at, triggered_by)
                  SELECT 'bulk-' || i, 'echo', '{}', 'success', 'x', '2026-02-01T00:00:00+00:00', 'user' FROM n").await;
        Migrator::up(&db, None).await.unwrap();
        assert_eq!(count(&db, "script_executions").await, 253);
        assert_eq!(count(&db, "pipeline_executions").await, 254, "253 actions + the pipeline row");

        // The legacy table moves on after the snapshot (SES writes it until C4).
        run(&db, "UPDATE script_executions SET status = 'failed' WHERE id = 'bulk-7'").await;
        run(&db, "INSERT INTO script_executions (id, command, parameters_json, status, output, started_at, triggered_by)
                  VALUES ('late', 'ls', '{}', 'success', 'y', '2026-02-02T00:00:00+00:00', 'user')").await;

        copy_script_history(&db, false).await.unwrap();
        assert_eq!(count(&db, "pipeline_executions").await, 255, "the new row arrives");
        let s = &rows(&db, "SELECT status FROM pipeline_executions WHERE id = 'bulk-7'").await[0];
        assert_eq!(s.try_get::<String>("", "status").unwrap(), "success", "ignore mode keeps the old row");

        copy_script_history(&db, true).await.unwrap();
        assert_eq!(count(&db, "pipeline_executions").await, 255);
        let s = &rows(&db, "SELECT status FROM pipeline_executions WHERE id = 'bulk-7'").await[0];
        assert_eq!(s.try_get::<String>("", "status").unwrap(), "failed", "replace mode refreshes it");
    }

    async fn ids(db: &DatabaseConnection, sql: &str) -> std::collections::BTreeSet<String> {
        rows(db, sql)
            .await
            .iter()
            .map(|r| r.try_get("", "id").unwrap())
            .collect()
    }

    /// Run by hand against a COPY of a real `portal_desktop.db` (never the live
    /// file; this works on a further copy in a temp dir, so the file the
    /// variable names is not written either):
    /// `PORTAL_MIGRATION_DB_COPY=<path> cargo test real_db_copy_migrates -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "needs PORTAL_MIGRATION_DB_COPY: the path of a copy of a real portal_desktop.db"]
    async fn real_db_copy_migrates() {
        use crate::domains::projects::pipelines::services::execution_service::StepExecutionState;

        let src = std::path::PathBuf::from(
            std::env::var("PORTAL_MIGRATION_DB_COPY")
                .expect("set PORTAL_MIGRATION_DB_COPY to the path of a COPY of portal_desktop.db"),
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("copy.db");
        std::fs::copy(&src, &path).expect("copy the database into a temp dir");
        for suffix in ["-wal", "-shm"] {
            let side = format!("{}{suffix}", src.display());
            if std::path::Path::new(&side).exists() {
                std::fs::copy(&side, format!("{}{suffix}", path.display())).unwrap();
            }
        }
        let db = Database::connect(format!("sqlite://{}?mode=rw", path.display()))
            .await
            .unwrap();

        let pipelines_before = ids(&db, "SELECT id FROM pipeline_executions").await;
        let scripts = ids(&db, "SELECT id FROM script_executions").await;
        let open = ids(
            &db,
            "SELECT id FROM script_executions WHERE status IN ('running', 'pending')",
        )
        .await;
        let fk_before = rows(&db, "PRAGMA foreign_key_check").await.len();

        Migrator::up(&db, None).await.unwrap();

        let pipelines = ids(&db, "SELECT id FROM pipeline_executions WHERE kind = 'pipeline'").await;
        let actions = ids(&db, "SELECT id FROM pipeline_executions WHERE kind = 'action'").await;
        let not_failed = ids(
            &db,
            "SELECT id FROM pipeline_executions WHERE kind = 'action' AND status != 'failed'
             AND id IN (SELECT id FROM script_executions WHERE status IN ('running', 'pending'))",
        )
        .await;
        let fk_after = rows(&db, "PRAGMA foreign_key_check").await;
        let mut unreadable = Vec::new();
        for r in rows(&db, "SELECT id, step_executions_json FROM pipeline_executions").await {
            let json: String = r.try_get("", "step_executions_json").unwrap();
            if let Err(e) = serde_json::from_str::<Vec<StepExecutionState>>(&json) {
                unreadable.push(format!("{}: {e}", r.try_get::<String>("", "id").unwrap()));
            }
        }

        println!("pipeline_executions before: {}", pipelines_before.len());
        println!("  kind='pipeline' after: {}", pipelines.len());
        println!("script_executions: {}", scripts.len());
        println!("  kind='action' after: {}", actions.len());
        println!("  running/pending made failed: {} (not failed: {})", open.len(), not_failed.len());
        println!("foreign_key_check violations: before {fk_before}, after {}", fk_after.len());
        println!("step_executions_json unreadable: {}", unreadable.len());

        assert_eq!(pipelines, pipelines_before, "pipeline rows unchanged");
        assert_eq!(actions, scripts, "one action row per script row, same ids");
        assert!(not_failed.is_empty(), "running/pending rows left alive: {not_failed:?}");
        assert!(fk_after.is_empty(), "foreign_key_check found {} violations", fk_after.len());
        assert!(unreadable.is_empty(), "StepExecutionState cannot read: {unreadable:?}");
    }
}
