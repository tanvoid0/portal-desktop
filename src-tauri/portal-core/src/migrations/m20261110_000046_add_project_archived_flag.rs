//! Adds `archived` to `projects` — a manual "don't consider for next work"
//! flag, set from the main list.
//!
//! Deliberately NOT the same thing as `status = "archived"`: the disk-scan
//! import (`discovery_repository.rs::import`, `mark_missing`) already writes
//! that value to mean "missing on disk this scan," and never clears it when
//! the project reappears. Reusing it for a manual flag would make "I parked
//! this" and "disk scan couldn't find this" indistinguishable. This column
//! is written only by `set_project_archived` and nothing else.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("projects", "archived").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .add_column(
                            ColumnDef::new(Projects::Archived)
                                .boolean()
                                .not_null()
                                .default(false),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_column("projects", "archived").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .drop_column(Projects::Archived)
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
    Archived,
}
