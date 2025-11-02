use sea_orm_migration::prelude::*;

/// Migration: Create ides table
///
/// This migration creates a table for storing IDE configurations:
/// - id: Primary key (auto-increment)
/// - name: IDE name
/// - executable: Path to IDE executable
/// - is_default: Whether this is the default IDE
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
                    .table(Ides::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ides::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ides::Name).string().not_null())
                    .col(ColumnDef::new(Ides::Executable).string().not_null())
                    .col(ColumnDef::new(Ides::IsDefault).boolean().not_null().default(false))
                    .col(
                        ColumnDef::new(Ides::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Ides::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on is_default for faster lookups (if it doesn't exist)
        manager
            .create_index(
                Index::create()
                    .name("idx_ides_is_default")
                    .table(Ides::Table)
                    .col(Ides::IsDefault)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ides::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Ides {
    Table,
    Id,
    Name,
    Executable,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

