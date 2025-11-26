use sea_orm_migration::prelude::*;

/// Migration: Create project_package_managers junction table
///
/// This migration creates a many-to-many relationship table linking projects to package managers:
/// - id: Primary key (auto-increment)
/// - project_id: Foreign key to projects table
/// - package_manager_id: Foreign key to package_managers table
/// - created_at: Timestamp when record was created
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectPackageManagers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectPackageManagers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectPackageManagers::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectPackageManagers::PackageManagerId).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectPackageManagers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_package_managers_project_id")
                            .from(ProjectPackageManagers::Table, ProjectPackageManagers::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_package_managers_package_manager_id")
                            .from(ProjectPackageManagers::Table, ProjectPackageManagers::PackageManagerId)
                            .to(PackageManagers::Table, PackageManagers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate project-package_manager pairs
        manager
            .create_index(
                Index::create()
                    .name("idx_project_package_managers_unique")
                    .table(ProjectPackageManagers::Table)
                    .col(ProjectPackageManagers::ProjectId)
                    .col(ProjectPackageManagers::PackageManagerId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on project_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_package_managers_project_id")
                    .table(ProjectPackageManagers::Table)
                    .col(ProjectPackageManagers::ProjectId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on package_manager_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_project_package_managers_package_manager_id")
                    .table(ProjectPackageManagers::Table)
                    .col(ProjectPackageManagers::PackageManagerId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectPackageManagers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectPackageManagers {
    Table,
    Id,
    ProjectId,
    PackageManagerId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum PackageManagers {
    Table,
    Id,
}

