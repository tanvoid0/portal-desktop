use sea_orm_migration::prelude::*;

/// Migration: Create project_languages junction table
///
/// This migration creates a many-to-many relationship table linking projects to languages:
/// - id: Primary key (auto-increment)
/// - project_id: Foreign key to projects table
/// - language_id: Foreign key to languages table
/// - created_at: Timestamp when record was created
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectLanguages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectLanguages::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectLanguages::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectLanguages::LanguageId).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectLanguages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_languages_project_id")
                            .from(ProjectLanguages::Table, ProjectLanguages::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_languages_language_id")
                            .from(ProjectLanguages::Table, ProjectLanguages::LanguageId)
                            .to(Languages::Table, Languages::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate project-language pairs
        manager
            .create_index(
                Index::create()
                    .name("idx_project_languages_unique")
                    .table(ProjectLanguages::Table)
                    .col(ProjectLanguages::ProjectId)
                    .col(ProjectLanguages::LanguageId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on project_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_languages_project_id")
                    .table(ProjectLanguages::Table)
                    .col(ProjectLanguages::ProjectId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on language_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_languages_language_id")
                    .table(ProjectLanguages::Table)
                    .col(ProjectLanguages::LanguageId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectLanguages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectLanguages {
    Table,
    Id,
    ProjectId,
    LanguageId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Languages {
    Table,
    Id,
}

