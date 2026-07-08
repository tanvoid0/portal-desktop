use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GithubConnections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GithubConnections::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GithubConnections::AccountId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GithubConnections::Login).string().not_null())
                    .col(ColumnDef::new(GithubConnections::Name).string().null())
                    .col(ColumnDef::new(GithubConnections::AvatarUrl).string().null())
                    .col(
                        ColumnDef::new(GithubConnections::HtmlUrl)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubConnections::ScopesJson)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(GithubConnections::CredentialId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubConnections::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubConnections::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GithubProjectLinks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GithubProjectLinks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::RepoOwner)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::RepoName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::RepoFullName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::RepoHtmlUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::RepoApiUrl)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::DefaultBranch)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(GithubProjectLinks::CloneUrl).string().null())
                    .col(ColumnDef::new(GithubProjectLinks::SshUrl).string().null())
                    .col(
                        ColumnDef::new(GithubProjectLinks::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GithubProjectLinks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-github_project_links-project_id")
                            .from(GithubProjectLinks::Table, GithubProjectLinks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-github_project_links-project_id")
                    .table(GithubProjectLinks::Table)
                    .col(GithubProjectLinks::ProjectId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-github_project_links-repo_full_name")
                    .table(GithubProjectLinks::Table)
                    .col(GithubProjectLinks::RepoFullName)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GithubProjectLinks::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GithubConnections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GithubConnections {
    Table,
    Id,
    AccountId,
    Login,
    Name,
    AvatarUrl,
    HtmlUrl,
    ScopesJson,
    CredentialId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GithubProjectLinks {
    Table,
    Id,
    ProjectId,
    RepoOwner,
    RepoName,
    RepoFullName,
    RepoHtmlUrl,
    RepoApiUrl,
    DefaultBranch,
    CloneUrl,
    SshUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
