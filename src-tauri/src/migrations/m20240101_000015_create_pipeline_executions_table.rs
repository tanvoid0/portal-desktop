use sea_orm_migration::prelude::*;

/// Migration: Create pipeline_executions table
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PipelineExecutions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineExecutions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PipelineExecutions::PipelineId).integer().not_null())
                    .col(ColumnDef::new(PipelineExecutions::ProjectId).integer().not_null())
                    .col(ColumnDef::new(PipelineExecutions::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(PipelineExecutions::StartedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(PipelineExecutions::FinishedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(PipelineExecutions::TriggeredBy).string().not_null())
                    .col(ColumnDef::new(PipelineExecutions::StepExecutionsJson).text().not_null())
                    .col(ColumnDef::new(PipelineExecutions::VariablesJson).text().not_null())
                    .col(ColumnDef::new(PipelineExecutions::Error).text().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pipeline_executions_pipeline_id")
                            .from(PipelineExecutions::Table, PipelineExecutions::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pipeline_executions_project_id")
                            .from(PipelineExecutions::Table, PipelineExecutions::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PipelineExecutions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PipelineExecutions {
    Table,
    Id,
    PipelineId,
    ProjectId,
    Status,
    StartedAt,
    FinishedAt,
    TriggeredBy,
    StepExecutionsJson,
    VariablesJson,
    Error,
}

#[derive(DeriveIden)]
enum Pipelines {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

