use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Deployments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Deployments::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Deployments::ProjectId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Deployments::Name).string().not_null())
                    .col(
                        ColumnDef::new(Deployments::DeploymentType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Deployments::Status).string().not_null())
                    .col(
                        ColumnDef::new(Deployments::DataJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Deployments::CreatedAt)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Deployments::UpdatedAt)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_deployments_project_id")
                    .table(Deployments::Table)
                    .col(Deployments::ProjectId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Deployments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Deployments {
    Table,
    Id,
    ProjectId,
    Name,
    DeploymentType,
    Status,
    DataJson,
    CreatedAt,
    UpdatedAt,
}
