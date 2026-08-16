use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CoderThreads::Table)
                    .add_column(
                        ColumnDef::new(CoderThreads::ThreadKind)
                            .string()
                            .not_null()
                            .default("session"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CoderSubAgents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CoderSubAgents::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CoderSubAgents::CoordinatorThreadId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CoderSubAgents::ChildThreadId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CoderSubAgents::Title).string().not_null())
                    .col(
                        ColumnDef::new(CoderSubAgents::WorkspaceRoot)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CoderSubAgents::Branch).string().not_null())
                    .col(ColumnDef::new(CoderSubAgents::Status).string().not_null())
                    .col(ColumnDef::new(CoderSubAgents::GithubOwner).string().null())
                    .col(ColumnDef::new(CoderSubAgents::GithubRepo).string().null())
                    .col(
                        ColumnDef::new(CoderSubAgents::GithubIssueNumber)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CoderSubAgents::GithubIssueUrl)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(CoderSubAgents::ResultSummary).text().null())
                    .col(ColumnDef::new(CoderSubAgents::Error).text().null())
                    .col(
                        ColumnDef::new(CoderSubAgents::CreatedAt)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CoderSubAgents::UpdatedAt)
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
                    .name("idx_coder_sub_agents_coordinator_thread_id")
                    .table(CoderSubAgents::Table)
                    .col(CoderSubAgents::CoordinatorThreadId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_coder_sub_agents_child_thread_id")
                    .table(CoderSubAgents::Table)
                    .col(CoderSubAgents::ChildThreadId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CoderSubAgents::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CoderThreads::Table)
                    .drop_column(CoderThreads::ThreadKind)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CoderThreads {
    Table,
    ThreadKind,
}

#[derive(DeriveIden)]
enum CoderSubAgents {
    Table,
    Id,
    CoordinatorThreadId,
    ChildThreadId,
    Title,
    WorkspaceRoot,
    Branch,
    Status,
    GithubOwner,
    GithubRepo,
    GithubIssueNumber,
    GithubIssueUrl,
    ResultSummary,
    Error,
    CreatedAt,
    UpdatedAt,
}
