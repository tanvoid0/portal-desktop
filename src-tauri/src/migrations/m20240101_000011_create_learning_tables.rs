use sea_orm_migration::prelude::*;

/// Migration: Create learning tables
///
/// This migration creates tables for the learning system:
/// - learned_patterns: Stores learned patterns (command sequences, workflows, etc.)
/// - user_preferences: Stores user preferences learned from behavior
/// - learning_events: Stores learning events for feedback loop
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create learned_patterns table
        manager
            .create_table(
                Table::create()
                    .table(LearnedPatterns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LearnedPatterns::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LearnedPatterns::PatternType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LearnedPatterns::PatternData)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LearnedPatterns::Context).string().null())
                    .col(
                        ColumnDef::new(LearnedPatterns::Frequency)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(LearnedPatterns::LastUsed).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(LearnedPatterns::SuccessRate)
                            .double()
                            .not_null()
                            .default(1.0),
                    )
                    .col(
                        ColumnDef::new(LearnedPatterns::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on pattern_type and context for faster queries
        manager
            .create_index(
                Index::create()
                    .name("idx_learned_patterns_type_context")
                    .table(LearnedPatterns::Table)
                    .col(LearnedPatterns::PatternType)
                    .col(LearnedPatterns::Context)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on frequency for sorting
        manager
            .create_index(
                Index::create()
                    .name("idx_learned_patterns_frequency")
                    .table(LearnedPatterns::Table)
                    .col(LearnedPatterns::Frequency)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create user_preferences table
        manager
            .create_table(
                Table::create()
                    .table(UserPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPreferences::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserPreferences::PreferenceType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserPreferences::Context).string().null())
                    .col(
                        ColumnDef::new(UserPreferences::PreferenceValue)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserPreferences::Confidence)
                            .double()
                            .not_null()
                            .default(0.5),
                    )
                    .col(ColumnDef::new(UserPreferences::LearnedFrom).string().null())
                    .col(
                        ColumnDef::new(UserPreferences::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserPreferences::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on preference_type and context
        manager
            .create_index(
                Index::create()
                    .name("idx_user_preferences_type_context")
                    .table(UserPreferences::Table)
                    .col(UserPreferences::PreferenceType)
                    .col(UserPreferences::Context)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create learning_events table
        manager
            .create_table(
                Table::create()
                    .table(LearningEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LearningEvents::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LearningEvents::EventType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LearningEvents::EventData)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LearningEvents::Outcome).string().null())
                    .col(ColumnDef::new(LearningEvents::Context).string().null())
                    .col(
                        ColumnDef::new(LearningEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on event_type and created_at for querying
        manager
            .create_index(
                Index::create()
                    .name("idx_learning_events_type_created")
                    .table(LearningEvents::Table)
                    .col(LearningEvents::EventType)
                    .col(LearningEvents::CreatedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LearningEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserPreferences::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LearnedPatterns::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LearnedPatterns {
    Table,
    Id,
    PatternType,
    PatternData,
    Context,
    Frequency,
    LastUsed,
    SuccessRate,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserPreferences {
    Table,
    Id,
    PreferenceType,
    Context,
    PreferenceValue,
    Confidence,
    LearnedFrom,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum LearningEvents {
    Table,
    Id,
    EventType,
    EventData,
    Outcome,
    Context,
    CreatedAt,
}

