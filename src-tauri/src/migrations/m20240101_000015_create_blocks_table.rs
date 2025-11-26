use sea_orm_migration::prelude::*;

/// Migration: Create blocks table
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Blocks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Blocks::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Blocks::Name).string().not_null())
                    .col(ColumnDef::new(Blocks::Description).text().not_null())
                    .col(ColumnDef::new(Blocks::Category).string().not_null())
                    .col(ColumnDef::new(Blocks::Version).string().not_null())
                    .col(ColumnDef::new(Blocks::ParametersJson).text().not_null())
                    .col(ColumnDef::new(Blocks::Command).text().not_null())
                    .col(ColumnDef::new(Blocks::ExecutionType).string().not_null())
                    .col(ColumnDef::new(Blocks::DefaultConfigJson).text().not_null())
                    .col(ColumnDef::new(Blocks::TagsJson).text().not_null())
                    .col(ColumnDef::new(Blocks::Icon).string().null())
                    .col(ColumnDef::new(Blocks::Author).string().null())
                    .col(ColumnDef::new(Blocks::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Blocks::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Blocks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Blocks {
    Table,
    Id,
    Name,
    Description,
    Category,
    Version,
    ParametersJson,
    Command,
    ExecutionType,
    DefaultConfigJson,
    TagsJson,
    Icon,
    Author,
    CreatedAt,
    UpdatedAt,
}

