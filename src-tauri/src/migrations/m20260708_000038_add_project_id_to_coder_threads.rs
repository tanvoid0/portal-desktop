use sea_orm_migration::prelude::*;
use sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CoderThreads::Table)
                    .add_column(ColumnDef::new(CoderThreads::ProjectId).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_coder_threads_project_id")
                    .table(CoderThreads::Table)
                    .col(CoderThreads::ProjectId)
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();
        conn.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"UPDATE coder_threads
               SET project_id = (
                   SELECT id FROM projects
                   WHERE projects.path = coder_threads.workspace_root
                   LIMIT 1
               )
               WHERE project_id IS NULL"#
                .to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_coder_threads_project_id")
                    .table(CoderThreads::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CoderThreads::Table)
                    .drop_column(CoderThreads::ProjectId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CoderThreads {
    Table,
    ProjectId,
}
