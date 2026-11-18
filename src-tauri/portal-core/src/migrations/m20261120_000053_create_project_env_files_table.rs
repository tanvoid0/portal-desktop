//! `project_env_files`: extra `.env*` files a user linked by path relative to
//! the project root (monorepo sub-apps keep theirs outside the root).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectEnvFiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectEnvFiles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvFiles::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectEnvFiles::RelPath).text().not_null())
                    .col(ColumnDef::new(ProjectEnvFiles::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_env_files_project_id")
                            .from(ProjectEnvFiles::Table, ProjectEnvFiles::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_env_files_unique")
                    .table(ProjectEnvFiles::Table)
                    .col(ProjectEnvFiles::ProjectId)
                    .col(ProjectEnvFiles::RelPath)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectEnvFiles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectEnvFiles {
    Table,
    Id,
    ProjectId,
    RelPath,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
