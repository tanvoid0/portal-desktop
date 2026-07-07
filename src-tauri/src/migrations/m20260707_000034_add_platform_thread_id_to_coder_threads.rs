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
                    .add_column(ColumnDef::new(CoderThreads::PlatformThreadId).big_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CoderThreads::Table)
                    .drop_column(CoderThreads::PlatformThreadId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CoderThreads {
    Table,
    PlatformThreadId,
}
