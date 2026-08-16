use sea_orm_migration::prelude::*;

/// Migration: Create terminal_notes table
/// Persists markdown notes for each terminal tab.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TerminalNotes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TerminalNotes::TabId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TerminalNotes::Markdown)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(TerminalNotes::UpdatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TerminalNotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TerminalNotes {
    Table,
    TabId,
    Markdown,
    UpdatedAt,
}
