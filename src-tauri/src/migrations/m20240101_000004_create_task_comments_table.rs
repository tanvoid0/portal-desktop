use sea_orm_migration::prelude::*;

/// Migration: Create task comments table
///
/// This migration creates the task_comments table for threaded discussions on tasks:
/// - id: Primary key (auto-increment)
/// - task_id: Foreign key to tasks table
/// - content: Comment text content
/// - author: Author name/email
/// - created_at: Timestamp when comment was created
/// - updated_at: Timestamp when comment was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskComments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskComments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskComments::TaskId).integer().not_null())
                    .col(ColumnDef::new(TaskComments::Content).text().not_null())
                    .col(ColumnDef::new(TaskComments::Author).text().not_null())
                    .col(ColumnDef::new(TaskComments::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(TaskComments::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_comments_task_id")
                            .from(TaskComments::Table, TaskComments::TaskId)
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
            .drop_table(Table::drop().table(TaskComments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskComments {
    Table,
    Id,
    TaskId,
    Content,
    Author,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
}
