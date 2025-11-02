use sea_orm_migration::prelude::*;

/// Migration: Create framework_ide_mappings table
///
/// This migration creates a table for linking frameworks to IDEs:
/// - id: Primary key (auto-increment)
/// - framework: Framework name (can be from suggested or user-defined)
/// - ide_id: Foreign key to ides table
/// - created_at: Timestamp when record was created
/// - updated_at: Timestamp when record was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FrameworkIdeMappings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FrameworkIdeMappings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FrameworkIdeMappings::Framework).string().not_null())
                    .col(ColumnDef::new(FrameworkIdeMappings::IdeId).integer().not_null())
                    .col(
                        ColumnDef::new(FrameworkIdeMappings::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(FrameworkIdeMappings::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_framework_ide_mappings_ide_id")
                            .from(FrameworkIdeMappings::Table, FrameworkIdeMappings::IdeId)
                            .to(Ides::Table, Ides::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on framework (one mapping per framework) (if it doesn't exist)
        manager
            .create_index(
                Index::create()
                    .name("idx_framework_ide_mappings_framework")
                    .table(FrameworkIdeMappings::Table)
                    .col(FrameworkIdeMappings::Framework)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on ide_id for faster lookups (if it doesn't exist)
        manager
            .create_index(
                Index::create()
                    .name("idx_framework_ide_mappings_ide_id")
                    .table(FrameworkIdeMappings::Table)
                    .col(FrameworkIdeMappings::IdeId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FrameworkIdeMappings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FrameworkIdeMappings {
    Table,
    Id,
    Framework,
    IdeId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Ides {
    Table,
    Id,
}

