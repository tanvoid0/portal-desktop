use sea_orm_migration::prelude::*;

/// Persistence for the `agents` domain (P2 of the agent-monitor epic): one
/// row per "Start run" launch, its message turns (future Conversation tab),
/// and its full raw event log in order (future Timeline tab — tool calls,
/// permission decisions, etc). The in-memory 500-event ring in
/// `AgentRunService` stays for live late-attach; this is additive.
///
/// ponytail: no separate permission-decisions table — `PermissionRequest`/
/// `PermissionResolved` already land in `agent_events` in order, which is
/// everything a Timeline tab needs; a dedicated table would just duplicate
/// that log.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentSessions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentSessions::BackendId).string().not_null())
                    .col(ColumnDef::new(AgentSessions::Model).string().null())
                    .col(ColumnDef::new(AgentSessions::Cwd).string().not_null())
                    .col(ColumnDef::new(AgentSessions::Status).string().not_null())
                    .col(ColumnDef::new(AgentSessions::StartedAt).text().not_null())
                    .col(ColumnDef::new(AgentSessions::EndedAt).text().null())
                    .col(ColumnDef::new(AgentSessions::ExitCode).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AgentMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentMessages::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AgentMessages::SessionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AgentMessages::Seq).big_integer().not_null())
                    .col(ColumnDef::new(AgentMessages::Role).string().not_null())
                    .col(ColumnDef::new(AgentMessages::Text).text().not_null())
                    .col(
                        ColumnDef::new(AgentMessages::ParentToolUseId)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(AgentMessages::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_messages_session_id")
                            .from(AgentMessages::Table, AgentMessages::SessionId)
                            .to(AgentSessions::Table, AgentSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agent_messages_session_id_seq")
                    .table(AgentMessages::Table)
                    .col(AgentMessages::SessionId)
                    .col(AgentMessages::Seq)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AgentEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentEvents::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentEvents::SessionId).string().not_null())
                    .col(ColumnDef::new(AgentEvents::Seq).big_integer().not_null())
                    // Full `AgentEvent` serialized as JSON (same approach as
                    // coder_file_changes's data_json).
                    .col(ColumnDef::new(AgentEvents::EventJson).text().not_null())
                    .col(ColumnDef::new(AgentEvents::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_agent_events_session_id")
                            .from(AgentEvents::Table, AgentEvents::SessionId)
                            .to(AgentSessions::Table, AgentSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_agent_events_session_id_seq")
                    .table(AgentEvents::Table)
                    .col(AgentEvents::SessionId)
                    .col(AgentEvents::Seq)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Backs `list_sessions`'s `ORDER BY started_at DESC`.
        manager
            .create_index(
                Index::create()
                    .name("idx_agent_sessions_started_at")
                    .table(AgentSessions::Table)
                    .col(AgentSessions::StartedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AgentMessages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AgentSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AgentSessions {
    Table,
    Id,
    BackendId,
    Model,
    Cwd,
    Status,
    StartedAt,
    EndedAt,
    ExitCode,
}

#[derive(DeriveIden)]
enum AgentMessages {
    Table,
    Id,
    SessionId,
    Seq,
    Role,
    Text,
    ParentToolUseId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AgentEvents {
    Table,
    Id,
    SessionId,
    Seq,
    EventJson,
    CreatedAt,
}
