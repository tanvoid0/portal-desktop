use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("ai_conversations", "model").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AiConversations::Table)
                        .add_column(ColumnDef::new(AiConversations::Model).string().null())
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_column("ai_conversations", "model").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AiConversations::Table)
                        .drop_column(AiConversations::Model)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AiConversations {
    Table,
    Model,
}
