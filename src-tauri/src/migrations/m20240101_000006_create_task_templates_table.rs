use sea_orm_migration::prelude::*;

/// Migration: Create task templates table
///
/// This migration creates the task_templates table for reusable task configurations:
/// - id: Primary key (auto-increment)
/// - name: Template name
/// - description: Template description
/// - default_status: Default status for new tasks
/// - default_priority: Default priority for new tasks
/// - default_type: Default type for new tasks
/// - default_tags: JSON array of default tags
/// - default_estimated_time: Default time estimate in minutes
/// - created_at: Timestamp when template was created
/// - updated_at: Timestamp when template was last updated
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskTemplates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskTemplates::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskTemplates::Name).text().not_null())
                    .col(ColumnDef::new(TaskTemplates::Description).text().null())
                    .col(ColumnDef::new(TaskTemplates::DefaultStatus).text().not_null().default("pending"))
                    .col(ColumnDef::new(TaskTemplates::DefaultPriority).text().not_null().default("medium"))
                    .col(ColumnDef::new(TaskTemplates::DefaultType).text().null())
                    .col(ColumnDef::new(TaskTemplates::DefaultTags).text().null())
                    .col(ColumnDef::new(TaskTemplates::DefaultEstimatedTime).integer().null())
                    .col(ColumnDef::new(TaskTemplates::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(TaskTemplates::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaskTemplates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskTemplates {
    Table,
    Id,
    Name,
    Description,
    DefaultStatus,
    DefaultPriority,
    DefaultType,
    DefaultTags,
    DefaultEstimatedTime,
    CreatedAt,
    UpdatedAt,
}
