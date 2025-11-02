use sea_orm_migration::prelude::*;

/// Migration: Create saved views table
///
/// This migration creates the saved_views table for storing filter combinations:
/// - id: Primary key (auto-increment)
/// - name: View name
/// - description: View description
/// - filters: JSON object containing filter configuration
/// - is_default: Whether this is the default view
/// - created_at: Timestamp when view was created
/// - updated_at: Timestamp when view was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SavedViews::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SavedViews::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SavedViews::Name).text().not_null())
                    .col(ColumnDef::new(SavedViews::Description).text().null())
                    .col(ColumnDef::new(SavedViews::Filters).text().not_null())
                    .col(ColumnDef::new(SavedViews::IsDefault).boolean().not_null().default(false))
                    .col(ColumnDef::new(SavedViews::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(SavedViews::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SavedViews::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SavedViews {
    Table,
    Id,
    Name,
    Description,
    Filters,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}
