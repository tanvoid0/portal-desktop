//! `agent_session_tombstones`: jsonl session ids the `~/.claude/projects` sweep must not
//! import — ids of deleted sessions (row id and backend id), and backend ids a forking
//! `--resume` moved a session away from. Backfilled with the ids earlier forks left behind.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS agent_session_tombstones (
                session_id text NOT NULL PRIMARY KEY,
                deleted_at text NOT NULL
            )",
        )
        .await?;
        // An imported row (id = its first jsonl id) that already forked: its old file is
        // still on disk and would hit the primary key on every sweep.
        db.execute_unprepared(
            "INSERT OR IGNORE INTO agent_session_tombstones (session_id, deleted_at)
             SELECT id, strftime('%Y-%m-%dT%H:%M:%SZ', 'now') FROM agent_sessions
             WHERE backend_id = 'claude-code' AND backend_session_id IS NOT NULL AND backend_session_id <> id",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS agent_session_tombstones")
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::ConnectionTrait;

    /// Portal-launched rows (uuid id, other backend id) are tombstoned too: harmless, and
    /// their own id never names a jsonl.
    #[tokio::test]
    async fn backfill_tombstones_rows_whose_backend_id_moved() {
        let dir = tempfile::tempdir().unwrap();
        let dbm = crate::database::DatabaseManager::new(dir.path().to_path_buf()).await.unwrap();
        let db = dbm.get_connection();
        for (id, b) in [("a", "b"), ("same", "same")] {
            db.execute_unprepared(&format!(
                "INSERT INTO agent_sessions (id, backend_id, cwd, status, started_at, backend_session_id)
                 VALUES ('{id}', 'claude-code', '.', 'exited', 'x', '{b}')"
            ))
            .await
            .unwrap();
        }
        Migration.up(&SchemaManager::new(db)).await.unwrap();
        let rows = db
            .query_all(sea_orm::Statement::from_string(sea_orm::DbBackend::Sqlite, "SELECT session_id FROM agent_session_tombstones"))
            .await
            .unwrap();
        let ids: Vec<String> = rows.iter().map(|r| r.try_get_by_index(0).unwrap()).collect();
        assert_eq!(ids, ["a"]);
    }
}
