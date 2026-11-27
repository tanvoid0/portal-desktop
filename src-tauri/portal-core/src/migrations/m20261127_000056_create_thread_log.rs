//! Server-owned Coder thread log: `threads` (one row per thread, holds the folded
//! state), `thread_events` (append-only), `thread_messages` (one row per
//! `ChatMessage`). Additive; the legacy coder/agent tables are untouched.
//! See `docs/development/CODER_THREAD_LOG_DESIGN.md` section 1.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for sql in [
            "CREATE TABLE IF NOT EXISTS threads (
                id text NOT NULL PRIMARY KEY,
                provider text NOT NULL,
                kind text NOT NULL DEFAULT 'session',
                title text NOT NULL,
                workspace_root text NOT NULL,
                project_id integer NULL,
                model text NULL,
                llm_provider text NULL,
                status text NOT NULL,
                state_json text NOT NULL,
                last_seq integer NOT NULL DEFAULT 0,
                message_count integer NOT NULL DEFAULT 0,
                created_at text NOT NULL,
                updated_at text NOT NULL
            )",
            "CREATE INDEX IF NOT EXISTS idx_threads_updated_at ON threads(updated_at)",
            "CREATE INDEX IF NOT EXISTS idx_threads_project_id ON threads(project_id)",
            "CREATE TABLE IF NOT EXISTS thread_events (
                thread_id text NOT NULL,
                seq integer NOT NULL,
                kind text NOT NULL,
                payload text NOT NULL,
                command_id text NULL,
                actor text NOT NULL,
                at text NOT NULL,
                PRIMARY KEY (thread_id, seq),
                FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE
            )",
            "CREATE INDEX IF NOT EXISTS idx_thread_events_command
                ON thread_events(thread_id, command_id) WHERE command_id IS NOT NULL",
            "CREATE TABLE IF NOT EXISTS thread_messages (
                thread_id text NOT NULL,
                idx integer NOT NULL,
                role text NOT NULL,
                message_json text NOT NULL,
                seq integer NOT NULL,
                PRIMARY KEY (thread_id, idx),
                FOREIGN KEY (thread_id) REFERENCES threads(id) ON DELETE CASCADE
            )",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    // Additive log; dropping it would lose thread history, so there is no way back.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
