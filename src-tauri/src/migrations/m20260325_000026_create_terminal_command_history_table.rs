use sea_orm_migration::prelude::*;

/// Migration: Create terminal_command_history table
/// Persists terminal command execution history for each tab.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TerminalCommandHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TerminalCommandHistory::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::TabId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::Command)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::Output)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    // Stored as RFC3339 string for easy interchange with frontend
                    .col(
                        ColumnDef::new(TerminalCommandHistory::Timestamp)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::ExitCode)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::DurationMs)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TerminalCommandHistory::Intercepted)
                            .boolean()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_terminal_command_history_tab_id")
                    .table(TerminalCommandHistory::Table)
                    .col(TerminalCommandHistory::TabId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_terminal_command_history_tab_id_timestamp")
                    .table(TerminalCommandHistory::Table)
                    .col(TerminalCommandHistory::TabId)
                    .col(TerminalCommandHistory::Timestamp)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(TerminalCommandHistory::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum TerminalCommandHistory {
    Table,
    Id,
    TabId,
    Command,
    Output,
    Timestamp,
    ExitCode,
    DurationMs,
    Intercepted,
}
