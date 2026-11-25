//! Adds `push_ok` to `projects` — the per-project allowlist for "Sync all"
//! outbound pushes. Written only by `set_project_push_ok`; discovery never
//! touches it.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("projects", "push_ok").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .add_column(
                            ColumnDef::new(Projects::PushOk)
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
        if manager.has_column("projects", "push_ok").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .drop_column(Projects::PushOk)
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
    PushOk,
}
