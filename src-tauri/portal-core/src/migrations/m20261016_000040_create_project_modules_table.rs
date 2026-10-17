//! `project_modules`: the apps, services and packages found inside a project.
//!
//! Rebuilt from disk by the module scanner, so it is a cache of the folder
//! layout, never a source of truth. No foreign key: deleting a project clears
//! its rows explicitly, and a stale row is harmless until the next scan.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectModules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectModules::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectModules::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectModules::RelPath).string().not_null())
                    .col(ColumnDef::new(ProjectModules::Name).string().not_null())
                    .col(ColumnDef::new(ProjectModules::Stack).string().not_null())
                    .col(ColumnDef::new(ProjectModules::Role).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_project_modules_project")
                    .table(ProjectModules::Table)
                    .col(ProjectModules::ProjectId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectModules::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectModules {
    Table,
    Id,
    ProjectId,
    RelPath,
    Name,
    Stack,
    Role,
}
