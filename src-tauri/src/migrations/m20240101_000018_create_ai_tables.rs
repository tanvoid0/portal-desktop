use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiConversations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiConversations::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AiConversations::Title).string().not_null())
                    .col(ColumnDef::new(AiConversations::Provider).string().not_null())
                    .col(ColumnDef::new(AiConversations::CreatedAt).string().not_null())
                    .col(ColumnDef::new(AiConversations::UpdatedAt).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_conversations_provider")
                    .table(AiConversations::Table)
                    .col(AiConversations::Provider)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_conversations_updated_at")
                    .table(AiConversations::Table)
                    .col(AiConversations::UpdatedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AiConversationMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiConversationMessages::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AiConversationMessages::ConversationId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AiConversationMessages::Role).string().not_null())
                    .col(ColumnDef::new(AiConversationMessages::Content).text().not_null())
                    .col(ColumnDef::new(AiConversationMessages::Timestamp).string().not_null())
                    .col(ColumnDef::new(AiConversationMessages::Sequence).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_conversation_messages_conversation_id")
                            .from(AiConversationMessages::Table, AiConversationMessages::ConversationId)
                            .to(AiConversations::Table, AiConversations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_conversation_messages_conversation_id")
                    .table(AiConversationMessages::Table)
                    .col(AiConversationMessages::ConversationId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_conversation_messages_sequence")
                    .table(AiConversationMessages::Table)
                    .col(AiConversationMessages::ConversationId)
                    .col(AiConversationMessages::Sequence)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AiLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiLogs::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AiLogs::Provider).string().not_null())
                    .col(ColumnDef::new(AiLogs::LogType).string().not_null())
                    .col(ColumnDef::new(AiLogs::RequestData).text().null())
                    .col(ColumnDef::new(AiLogs::ResponseData).text().null())
                    .col(ColumnDef::new(AiLogs::ErrorMessage).text().null())
                    .col(ColumnDef::new(AiLogs::Timestamp).string().not_null())
                    .col(ColumnDef::new(AiLogs::ConversationId).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ai_logs_conversation_id")
                            .from(AiLogs::Table, AiLogs::ConversationId)
                            .to(AiConversations::Table, AiConversations::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_logs_provider")
                    .table(AiLogs::Table)
                    .col(AiLogs::Provider)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_logs_log_type")
                    .table(AiLogs::Table)
                    .col(AiLogs::LogType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_logs_timestamp")
                    .table(AiLogs::Table)
                    .col(AiLogs::Timestamp)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_logs_conversation_id")
                    .table(AiLogs::Table)
                    .col(AiLogs::ConversationId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AiTrainingData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiTrainingData::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AiTrainingData::Name).string().not_null())
                    .col(ColumnDef::new(AiTrainingData::Type).string().not_null())
                    .col(ColumnDef::new(AiTrainingData::Content).text().not_null())
                    .col(ColumnDef::new(AiTrainingData::Metadata).text().null())
                    .col(ColumnDef::new(AiTrainingData::CreatedAt).string().not_null())
                    .col(ColumnDef::new(AiTrainingData::UpdatedAt).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_training_data_type")
                    .table(AiTrainingData::Table)
                    .col(AiTrainingData::Type)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiTrainingData::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AiLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AiConversationMessages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AiConversations::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AiConversations {
    Table,
    Id,
    Title,
    Provider,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AiConversationMessages {
    Table,
    Id,
    ConversationId,
    Role,
    Content,
    Timestamp,
    Sequence,
}

#[derive(DeriveIden)]
enum AiLogs {
    Table,
    Id,
    Provider,
    LogType,
    RequestData,
    ResponseData,
    ErrorMessage,
    Timestamp,
    ConversationId,
}

#[derive(DeriveIden)]
enum AiTrainingData {
    Table,
    Id,
    Name,
    Type,
    Content,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

