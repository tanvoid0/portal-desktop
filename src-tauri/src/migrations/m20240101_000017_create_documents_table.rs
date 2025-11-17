use sea_orm_migration::prelude::*;

/// Migration: Create documents table
/// 
/// This migration creates the documents table with the following structure:
/// - id: Primary key (auto-increment)
/// - title: Document title
/// - content: Markdown content
/// - content_draft: Draft content for caching (like Sublime Text)
/// - is_draft: Whether the document is in draft mode
/// - tags: JSON array of tags
/// - created_at: Timestamp when record was created
/// - updated_at: Timestamp when record was last updated
/// - last_edited_at: Timestamp when content was last edited
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Documents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Documents::Title).string().not_null())
                    .col(ColumnDef::new(Documents::Content).text().not_null())
                    .col(ColumnDef::new(Documents::IsArchived).boolean().not_null().default(false))
                    .col(ColumnDef::new(Documents::ContentDraft).text().null())
                    .col(ColumnDef::new(Documents::IsDraft).boolean().not_null().default(false))
                    .col(ColumnDef::new(Documents::Tags).text().null())
                    .col(ColumnDef::new(Documents::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Documents::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Documents::LastEditedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Documents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Documents {
    Table,
    Id,
    Title,
    Content,
    IsArchived,
    ContentDraft,
    IsDraft,
    Tags,
    CreatedAt,
    UpdatedAt,
    LastEditedAt,
}

