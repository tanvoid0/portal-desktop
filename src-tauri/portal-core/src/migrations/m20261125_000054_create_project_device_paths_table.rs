//! `project_device_paths`: per-device copy of a project's machine-local fields (path,
//! git remote/branch/commit, dev port, discovery path). Lives in the DB so it travels in
//! sync snapshots; each device only ever writes its own rows.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS project_device_paths (
                    project_id integer NOT NULL,
                    device_id text NOT NULL,
                    path text NOT NULL,
                    git_repository text NULL,
                    git_branch text NULL,
                    git_commit text NULL,
                    dev_port integer NULL,
                    discovery_path text NULL,
                    status text NOT NULL DEFAULT 'resolved',
                    updated_at text NOT NULL,
                    PRIMARY KEY (project_id, device_id),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                )",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS project_device_paths")
            .await?;
        Ok(())
    }
}
