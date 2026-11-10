//! Adds `stage` to `projects` for manual SDLC-maturity tracking.
//!
//! Independent of the `archived` flag (see the next migration) and of
//! `status` (active/archived/deleted, which the disk-scan import also
//! writes for an unrelated reason): a project can be `stable` and archived
//! at once. Null means "not set" — every project starts unset until the
//! owner picks one from the list.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("projects", "stage").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .add_column(ColumnDef::new(Projects::Stage).string().null())
                        .to_owned(),
                )
                .await?;
        }

        if !manager.has_column("projects", "stage_updated_at").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .add_column(
                            ColumnDef::new(Projects::StageUpdatedAt)
                                .timestamp_with_time_zone()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_column("projects", "stage_updated_at").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .drop_column(Projects::StageUpdatedAt)
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column("projects", "stage").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .drop_column(Projects::Stage)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Stage,
    StageUpdatedAt,
}
