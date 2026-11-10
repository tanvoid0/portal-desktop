//! `project_moves`: one row per project-relocation attempt — the journal
//! that lets the move engine (step 4+ of the "safe project relocation" plan)
//! resume a copy/verify/commit after a crash instead of restarting from
//! zero. This migration only creates the table; nothing writes to it yet.
//!
//! Forward-only: there is no rename fast-path elsewhere that reads these
//! columns, so there is nothing to keep in sync if the shape changes later —
//! get it right here.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_table("project_moves").await? {
            return Ok(());
        }

        manager
            .create_table(
                Table::create()
                    .table(ProjectMoves::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectMoves::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectMoves::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectMoves::SourcePath).text().not_null())
                    .col(ColumnDef::new(ProjectMoves::DestPath).text().not_null())
                    .col(ColumnDef::new(ProjectMoves::StagingPath).text().not_null())
                    // copying|verifying|verified|committed|done|aborted|failed
                    .col(ColumnDef::new(ProjectMoves::State).text().not_null())
                    .col(
                        ColumnDef::new(ProjectMoves::IncludeRegenerable)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // JSON array of dirs skipped during the copy (e.g. node_modules).
                    .col(
                        ColumnDef::new(ProjectMoves::SkippedDirs)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    // Manifest + progress file aren't written until the copy runs.
                    .col(ColumnDef::new(ProjectMoves::ManifestPath).text().null())
                    .col(ColumnDef::new(ProjectMoves::ManifestSha256).text().null())
                    .col(ColumnDef::new(ProjectMoves::ManifestVersion).integer().null())
                    .col(ColumnDef::new(ProjectMoves::ProgressPath).text().null())
                    .col(
                        ColumnDef::new(ProjectMoves::FileCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProjectMoves::TotalBytes)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProjectMoves::FilesDone)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProjectMoves::BytesDone)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ProjectMoves::ResumeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // JSON array of paths the post-move rewrite pass couldn't fix up.
                    .col(
                        ColumnDef::new(ProjectMoves::Unrewritten)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(ProjectMoves::Error).text().null())
                    .col(ColumnDef::new(ProjectMoves::StartedAt).text().not_null())
                    .col(ColumnDef::new(ProjectMoves::UpdatedAt).text().not_null())
                    .col(ColumnDef::new(ProjectMoves::FinishedAt).text().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_moves_project_id")
                            .from(ProjectMoves::Table, ProjectMoves::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectMoves::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectMoves {
    Table,
    Id,
    ProjectId,
    SourcePath,
    DestPath,
    StagingPath,
    State,
    IncludeRegenerable,
    SkippedDirs,
    ManifestPath,
    ManifestSha256,
    ManifestVersion,
    ProgressPath,
    FileCount,
    TotalBytes,
    FilesDone,
    BytesDone,
    ResumeCount,
    Unrewritten,
    Error,
    StartedAt,
    UpdatedAt,
    FinishedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
