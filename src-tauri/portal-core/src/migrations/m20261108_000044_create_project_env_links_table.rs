//! `project_env_links`: one row per `(project_id, env_key)` the user has linked
//! to a credential, with the file-value hash and credential `updated_at` at
//! link time — the snapshots the drift check in `domains::projects::env`
//! compares live state against. Portal never writes to a project's `.env`
//! file; linking only touches this table.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectEnvLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectEnvLinks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvLinks::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectEnvLinks::EnvKey).text().not_null())
                    .col(
                        ColumnDef::new(ProjectEnvLinks::CredentialId)
                            .string()
                            .not_null(),
                    )
                    // Which `.env*` file the snapshot below was taken against — a key can be
                    // defined in more than one file, and drift must compare against that one,
                    // not whichever file `get()` happens to iterate to first.
                    .col(ColumnDef::new(ProjectEnvLinks::FileName).text().not_null())
                    .col(ColumnDef::new(ProjectEnvLinks::FileValueHash).text().null())
                    .col(
                        ColumnDef::new(ProjectEnvLinks::CredentialUpdatedAt)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectEnvLinks::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_env_links_project_id")
                            .from(ProjectEnvLinks::Table, ProjectEnvLinks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_env_links_credential_id")
                            .from(ProjectEnvLinks::Table, ProjectEnvLinks::CredentialId)
                            .to(Credentials::Table, Credentials::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_project_env_links_unique")
                    .table(ProjectEnvLinks::Table)
                    .col(ProjectEnvLinks::ProjectId)
                    .col(ProjectEnvLinks::EnvKey)
                    .col(ProjectEnvLinks::FileName)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectEnvLinks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectEnvLinks {
    Table,
    Id,
    ProjectId,
    EnvKey,
    CredentialId,
    FileName,
    FileValueHash,
    CredentialUpdatedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Credentials {
    Table,
    Id,
}
