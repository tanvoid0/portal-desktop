use sea_orm_migration::prelude::*;

/// Migration: Create terminal_sessions table
/// Persists PTY scrollback + cursor/session metadata for each tab.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TerminalSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TerminalSessions::TabId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::WorkingDirectory)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::EnvironmentJson)
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::ScrollbackBufferJson)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::CursorX)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::CursorY)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::TerminalCols)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::TerminalRows)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalSessions::LastActivity)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TerminalSessions::ProcessId).text().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TerminalSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TerminalSessions {
    Table,
    TabId,
    WorkingDirectory,
    EnvironmentJson,
    ScrollbackBufferJson,
    CursorX,
    CursorY,
    TerminalCols,
    TerminalRows,
    LastActivity,
    ProcessId,
}

