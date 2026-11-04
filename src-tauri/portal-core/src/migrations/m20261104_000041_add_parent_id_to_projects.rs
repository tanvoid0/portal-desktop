//! Adds `parent_id` to `projects` for manual grouping.
//!
//! When set, the project is a child of another project regardless of where it
//! sits on disk — the frontend hides it from the root list and nests it under
//! its parent. Null means "ungrouped", so it falls back to path-based nesting.
//!
//! Plain nullable integer, no foreign key: SQLite cannot add an FK via
//! `ALTER TABLE`, and a dangling parent_id is harmless — the frontend treats an
//! unknown parent as ungrouped.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("projects", "parent_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .add_column(ColumnDef::new(Projects::ParentId).integer().null())
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_projects_parent_id")
                    .table(Projects::Table)
                    .col(Projects::ParentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_projects_parent_id")
                    .table(Projects::Table)
                    .to_owned(),
            )
            .await?;

        if manager.has_column("projects", "parent_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Projects::Table)
                        .drop_column(Projects::ParentId)
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
    ParentId,
}
