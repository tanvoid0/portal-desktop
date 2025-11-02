use sea_orm_migration::prelude::*;

/// Migration: Create task attachments table
///
/// This migration creates the task_attachments table for file attachments on tasks:
/// - id: Primary key (auto-increment)
/// - task_id: Foreign key to tasks table
/// - name: Original filename
/// - url: File path or URL
/// - type: MIME type or file extension
/// - size: File size in bytes
/// - created_at: Timestamp when attachment was added
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskAttachments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskAttachments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskAttachments::TaskId).integer().not_null())
                    .col(ColumnDef::new(TaskAttachments::Name).text().not_null())
                    .col(ColumnDef::new(TaskAttachments::Url).text().not_null())
                    .col(ColumnDef::new(TaskAttachments::Type).text().not_null())
                    .col(ColumnDef::new(TaskAttachments::Size).big_integer().not_null())
                    .col(ColumnDef::new(TaskAttachments::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_attachments_task_id")
                            .from(TaskAttachments::Table, TaskAttachments::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaskAttachments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskAttachments {
    Table,
    Id,
    TaskId,
    Name,
    Url,
    Type,
    Size,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
}
