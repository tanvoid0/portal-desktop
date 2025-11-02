use sea_orm_migration::prelude::*;

/// Migration: Create tasks table
///
/// This migration creates the tasks table with the following structure:
/// - id: Primary key (auto-increment)
/// - title: Task title
/// - description: Task description (optional)
/// - status: Task status (pending, in-progress, completed, cancelled)
/// - priority: Task priority (low, medium, high)
/// - type_: Task type (Story, Bug, Note, etc.) (optional)
/// - parent_id: Parent task ID for subtasks (optional)
/// - resource_id: Linked resource ID (optional)
/// - resource_type: Type of linked resource (optional)
/// - due_date: Task due date (optional)
/// - completed_at: When task was completed (optional)
/// - created_at: Timestamp when record was created
/// - updated_at: Timestamp when record was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::Title).string().not_null())
                    .col(ColumnDef::new(Tasks::Description).string().null())
                    .col(ColumnDef::new(Tasks::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(Tasks::Priority).string().not_null().default("medium"))
                    .col(ColumnDef::new(Tasks::Type).string().null())
                    .col(ColumnDef::new(Tasks::ParentId).integer().null())
                    .col(ColumnDef::new(Tasks::ResourceId).string().null())
                    .col(ColumnDef::new(Tasks::ResourceType).string().null())
                    .col(ColumnDef::new(Tasks::DueDate).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Tasks::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Tasks::EstimatedTime).integer().null())
                    .col(ColumnDef::new(Tasks::ActualTime).integer().null())
                    .col(ColumnDef::new(Tasks::Tags).text().null())
                    .col(ColumnDef::new(Tasks::Assignee).text().null())
                    .col(ColumnDef::new(Tasks::RecurringPattern).text().null())
                    .col(ColumnDef::new(Tasks::RecurringInterval).integer().null())
                    .col(ColumnDef::new(Tasks::RecurringEndDate).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Tasks::RecurringLastGenerated).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Tasks::BlockedBy).text().null())
                    .col(ColumnDef::new(Tasks::Blocks).text().null())
                    .col(ColumnDef::new(Tasks::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Tasks::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_parent_id")
                            .from(Tasks::Table, Tasks::ParentId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Title,
    Description,
    Status,
    Priority,
    Type,
    ParentId,
    ResourceId,
    ResourceType,
    DueDate,
    CompletedAt,
    EstimatedTime,
    ActualTime,
    Tags,
    Assignee,
    RecurringPattern,
    RecurringInterval,
    RecurringEndDate,
    RecurringLastGenerated,
    BlockedBy,
    Blocks,
    CreatedAt,
    UpdatedAt,
}

