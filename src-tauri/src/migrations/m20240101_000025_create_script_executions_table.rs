use sea_orm_migration::prelude::*;

/// Migration: Create script_executions table
/// Tracks script/block execution history with status, logs, and process info
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ScriptExecutions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScriptExecutions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // Reference to block/script (can be null for ad-hoc commands)
                    .col(ColumnDef::new(ScriptExecutions::BlockId).string().null())
                    // The actual command that was executed (with parameters resolved)
                    .col(ColumnDef::new(ScriptExecutions::Command).text().not_null())
                    // Parameters that were passed (JSON object)
                    .col(
                        ColumnDef::new(ScriptExecutions::ParametersJson)
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    // Working directory for execution
                    .col(
                        ColumnDef::new(ScriptExecutions::WorkingDirectory)
                            .text()
                            .null(),
                    )
                    // Execution status: pending, running, success, failed, cancelled
                    .col(
                        ColumnDef::new(ScriptExecutions::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    // Exit code from the process
                    .col(ColumnDef::new(ScriptExecutions::ExitCode).integer().null())
                    // Process ID (for tracking running processes)
                    .col(ColumnDef::new(ScriptExecutions::Pid).integer().null())
                    // Combined stdout/stderr output
                    .col(
                        ColumnDef::new(ScriptExecutions::Output)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    // Error message if failed
                    .col(ColumnDef::new(ScriptExecutions::Error).text().null())
                    // Timestamps
                    .col(
                        ColumnDef::new(ScriptExecutions::StartedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScriptExecutions::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // Who/what triggered this execution
                    .col(
                        ColumnDef::new(ScriptExecutions::TriggeredBy)
                            .string()
                            .not_null()
                            .default("user"),
                    )
                    // Foreign key to blocks table (optional - may be a built-in block)
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_script_executions_block_id")
                            .from(ScriptExecutions::Table, ScriptExecutions::BlockId)
                            .to(Blocks::Table, Blocks::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on status for filtering running executions
        manager
            .create_index(
                Index::create()
                    .name("idx_script_executions_status")
                    .table(ScriptExecutions::Table)
                    .col(ScriptExecutions::Status)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on block_id for querying executions by block
        manager
            .create_index(
                Index::create()
                    .name("idx_script_executions_block_id")
                    .table(ScriptExecutions::Table)
                    .col(ScriptExecutions::BlockId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on started_at for ordering
        manager
            .create_index(
                Index::create()
                    .name("idx_script_executions_started_at")
                    .table(ScriptExecutions::Table)
                    .col(ScriptExecutions::StartedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ScriptExecutions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ScriptExecutions {
    Table,
    Id,
    BlockId,
    Command,
    ParametersJson,
    WorkingDirectory,
    Status,
    ExitCode,
    Pid,
    Output,
    Error,
    StartedAt,
    FinishedAt,
    TriggeredBy,
}

#[derive(DeriveIden)]
enum Blocks {
    Table,
    Id,
}
