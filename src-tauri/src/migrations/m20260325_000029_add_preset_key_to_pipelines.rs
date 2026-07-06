use sea_orm_migration::prelude::*;

/// Migration: Add preset_key and category columns to pipelines table
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite only supports one ALTER TABLE change per statement
        if !manager
            .has_column("pipelines", "preset_key")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(Pipelines::Table)
                        .add_column(
                            ColumnDef::new(Pipelines::PresetKey)
                                .string()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column("pipelines", "category")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(Pipelines::Table)
                        .add_column(
                            ColumnDef::new(Pipelines::Category)
                                .string()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_pipelines_project_preset_key")
                    .table(Pipelines::Table)
                    .col(Pipelines::ProjectId)
                    .col(Pipelines::PresetKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_pipelines_project_preset_key")
                    .table(Pipelines::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Pipelines::Table)
                    .drop_column(Pipelines::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Pipelines::Table)
                    .drop_column(Pipelines::PresetKey)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Pipelines {
    Table,
    ProjectId,
    PresetKey,
    Category,
}
