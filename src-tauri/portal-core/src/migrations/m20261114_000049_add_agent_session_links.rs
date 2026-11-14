//! Additive columns for the IDE-shaped session view: a display title, a link
//! back to the task that launched the run, and the backend's own session id
//! (needed to resume-in-place — see `AgentRunService::resume_session`).
//! `m20261111_000048_create_agent_tables` already shipped, so this only adds.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("agent_sessions", "title").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .add_column(ColumnDef::new(AgentSessions::Title).text().null())
                        .to_owned(),
                )
                .await?;
        }

        if !manager.has_column("agent_sessions", "task_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .add_column(ColumnDef::new(AgentSessions::TaskId).string().null())
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column("agent_sessions", "backend_session_id")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .add_column(
                            ColumnDef::new(AgentSessions::BackendSessionId)
                                .string()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager
            .has_column("agent_sessions", "backend_session_id")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .drop_column(AgentSessions::BackendSessionId)
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column("agent_sessions", "task_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .drop_column(AgentSessions::TaskId)
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column("agent_sessions", "title").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(AgentSessions::Table)
                        .drop_column(AgentSessions::Title)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AgentSessions {
    Table,
    Title,
    TaskId,
    BackendSessionId,
}
