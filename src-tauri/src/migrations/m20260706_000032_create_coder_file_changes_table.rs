use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CoderFileChanges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CoderFileChanges::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CoderFileChanges::ThreadId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CoderFileChanges::Path).string().not_null())
                    .col(ColumnDef::new(CoderFileChanges::Status).string().not_null())
                    // Full FileChange serialized as JSON (before, hunks, ...).
                    .col(
                        ColumnDef::new(CoderFileChanges::DataJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CoderFileChanges::CreatedAt)
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
                    .name("idx_coder_file_changes_thread_id")
                    .table(CoderFileChanges::Table)
                    .col(CoderFileChanges::ThreadId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CoderFileChanges::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CoderFileChanges {
    Table,
    Id,
    ThreadId,
    Path,
    Status,
    DataJson,
    CreatedAt,
}
