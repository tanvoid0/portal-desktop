use sea_orm_migration::prelude::*;

/// Migration: Create project_frameworks junction table
///
/// This migration creates a many-to-many relationship table linking projects to frameworks:
/// - id: Primary key (auto-increment)
/// - project_id: Foreign key to projects table
/// - framework_id: Foreign key to frameworks table
/// - created_at: Timestamp when record was created
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectFrameworks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectFrameworks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectFrameworks::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectFrameworks::FrameworkId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectFrameworks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_frameworks_project_id")
                            .from(ProjectFrameworks::Table, ProjectFrameworks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_frameworks_framework_id")
                            .from(ProjectFrameworks::Table, ProjectFrameworks::FrameworkId)
                            .to(Frameworks::Table, Frameworks::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate project-framework pairs
        manager
            .create_index(
                Index::create()
                    .name("idx_project_frameworks_unique")
                    .table(ProjectFrameworks::Table)
                    .col(ProjectFrameworks::ProjectId)
                    .col(ProjectFrameworks::FrameworkId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on project_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_frameworks_project_id")
                    .table(ProjectFrameworks::Table)
                    .col(ProjectFrameworks::ProjectId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on framework_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_frameworks_framework_id")
                    .table(ProjectFrameworks::Table)
                    .col(ProjectFrameworks::FrameworkId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectFrameworks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectFrameworks {
    Table,
    Id,
    ProjectId,
    FrameworkId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Frameworks {
    Table,
    Id,
}
