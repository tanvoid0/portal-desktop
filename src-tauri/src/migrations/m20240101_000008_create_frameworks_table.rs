use sea_orm_migration::prelude::*;

/// Migration: Create frameworks table
///
/// This migration creates a table for storing user-defined frameworks with:
/// - id: Primary key (auto-increment)
/// - name: Framework name
/// - icon: Icon identifier (devicon or file path)
/// - icon_type: Type of icon ('devicon' or 'file')
/// - category: Category name
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
                    .table(Frameworks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Frameworks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Frameworks::Name).string().not_null())
                    .col(ColumnDef::new(Frameworks::Icon).string().not_null())
                    .col(ColumnDef::new(Frameworks::IconType).string().not_null().default("devicon"))
                    .col(ColumnDef::new(Frameworks::Category).string().not_null().default("Custom"))
                    .col(
                        ColumnDef::new(Frameworks::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Frameworks::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on name (if it doesn't exist)
        manager
            .create_index(
                Index::create()
                    .name("idx_frameworks_name")
                    .table(Frameworks::Table)
                    .col(Frameworks::Name)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Frameworks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Frameworks {
    Table,
    Id,
    Name,
    Icon,
    IconType,
    Category,
    CreatedAt,
    UpdatedAt,
}

