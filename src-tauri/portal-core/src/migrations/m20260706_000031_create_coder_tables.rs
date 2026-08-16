use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Coder agent threads. The full transcript is stored as a JSON blob
        // (same approach as the deployments table's data_json).
        manager
            .create_table(
                Table::create()
                    .table(CoderThreads::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CoderThreads::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CoderThreads::Title).string().not_null())
                    .col(
                        ColumnDef::new(CoderThreads::WorkspaceRoot)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CoderThreads::Model).string().null())
                    .col(ColumnDef::new(CoderThreads::MessagesJson).text().not_null())
                    .col(ColumnDef::new(CoderThreads::CreatedAt).string().not_null())
                    .col(ColumnDef::new(CoderThreads::UpdatedAt).string().not_null())
                    .to_owned(),
            )
            .await?;

        // Single-row settings: permission mode + allow/deny rules as JSON.
        manager
            .create_table(
                Table::create()
                    .table(CoderSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CoderSettings::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CoderSettings::DataJson).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CoderThreads::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CoderSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CoderThreads {
    Table,
    Id,
    Title,
    WorkspaceRoot,
    Model,
    MessagesJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CoderSettings {
    Table,
    Id,
    DataJson,
}
