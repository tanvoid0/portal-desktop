use sea_orm_migration::prelude::*;

/// Migration: Create custom_scripts table
///
/// This migration creates a table for storing custom scripts that can be executed
/// with various parameter types (files, strings, numbers, booleans, etc.)
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CustomScripts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CustomScripts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CustomScripts::Name).string().not_null())
                    .col(ColumnDef::new(CustomScripts::Description).text().null())
                    .col(ColumnDef::new(CustomScripts::Command).text().not_null())
                    .col(ColumnDef::new(CustomScripts::ParametersJson).text().not_null())
                    .col(ColumnDef::new(CustomScripts::Category).string().null())
                    .col(ColumnDef::new(CustomScripts::Icon).string().null())
                    .col(ColumnDef::new(CustomScripts::RequiresSudo).boolean().not_null().default(false))
                    .col(ColumnDef::new(CustomScripts::IsInteractive).boolean().not_null().default(false))
                    .col(ColumnDef::new(CustomScripts::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(CustomScripts::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(CustomScripts::LastRunAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(CustomScripts::RunCount).integer().not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CustomScripts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CustomScripts {
    Table,
    Id,
    Name,
    Description,
    Command,
    ParametersJson,
    Category,
    Icon,
    RequiresSudo,
    IsInteractive,
    CreatedAt,
    UpdatedAt,
    LastRunAt,
    RunCount,
}

