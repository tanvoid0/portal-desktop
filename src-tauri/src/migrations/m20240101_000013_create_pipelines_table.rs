use sea_orm_migration::prelude::*;

/// Migration: Create pipelines table
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Pipelines::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Pipelines::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Pipelines::Name).string().not_null())
                    .col(ColumnDef::new(Pipelines::Description).string().null())
                    .col(ColumnDef::new(Pipelines::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Pipelines::StepsJson).text().not_null())
                    .col(ColumnDef::new(Pipelines::VariablesJson).text().not_null())
                    .col(ColumnDef::new(Pipelines::SecretsJson).text().not_null())
                    .col(ColumnDef::new(Pipelines::ExecutionContextJson).text().not_null())
                    .col(ColumnDef::new(Pipelines::Enabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(Pipelines::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Pipelines::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pipelines_project_id")
                            .from(Pipelines::Table, Pipelines::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Pipelines::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Pipelines {
    Table,
    Id,
    Name,
    Description,
    ProjectId,
    StepsJson,
    VariablesJson,
    SecretsJson,
    ExecutionContextJson,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

