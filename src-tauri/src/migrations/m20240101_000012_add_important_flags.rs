use sea_orm_migration::prelude::*;
use sea_orm::DbErr;

/// Migration: Add important flags to learning tables
///
/// This migration adds flags to mark patterns and preferences as important,
/// preventing automatic cleanup of user-valuable data.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Try to add is_important flag to learned_patterns
        // If column already exists, the error will be ignored
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(LearnedPatterns::Table)
                    .add_column(
                        ColumnDef::new(LearnedPatterns::IsImportant)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await;

        // Try to add is_important flag to user_preferences
        // If column already exists, the error will be ignored
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(UserPreferences::Table)
                    .add_column(
                        ColumnDef::new(UserPreferences::IsImportant)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await;

        // Create index on is_important for faster queries
        manager
            .create_index(
                Index::create()
                    .name("idx_learned_patterns_important")
                    .table(LearnedPatterns::Table)
                    .col(LearnedPatterns::IsImportant)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_preferences_important")
                    .table(UserPreferences::Table)
                    .col(UserPreferences::IsImportant)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_learned_patterns_important")
                    .table(LearnedPatterns::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_preferences_important")
                    .table(UserPreferences::Table)
                    .to_owned(),
            )
            .await?;

        // Remove columns
        manager
            .alter_table(
                Table::alter()
                    .table(LearnedPatterns::Table)
                    .drop_column(LearnedPatterns::IsImportant)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserPreferences::Table)
                    .drop_column(UserPreferences::IsImportant)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LearnedPatterns {
    Table,
    IsImportant,
}

#[derive(DeriveIden)]
enum UserPreferences {
    Table,
    IsImportant,
}

