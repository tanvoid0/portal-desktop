use sea_orm_migration::prelude::*;

/// Migration: Create SDK management tables
///
/// This migration creates tables for SDK management:
/// - sdk_custom_paths: Custom SDK installation paths
/// - sdk_environment_configs: Environment configuration per SDK
/// - sdk_environment_variables: Environment variables per SDK
/// - sdk_path_entries: PATH entries per SDK
/// - sdk_version_aliases: Version aliases per SDK
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create sdk_custom_paths table
        manager
            .create_table(
                Table::create()
                    .table(SdkCustomPaths::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SdkCustomPaths::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SdkCustomPaths::SdkType).string().not_null())
                    .col(ColumnDef::new(SdkCustomPaths::Path).string().not_null())
                    .col(ColumnDef::new(SdkCustomPaths::Version).string())
                    .col(
                        ColumnDef::new(SdkCustomPaths::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SdkCustomPaths::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkCustomPaths::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create sdk_environment_configs table
        manager
            .create_table(
                Table::create()
                    .table(SdkEnvironmentConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SdkEnvironmentConfigs::SdkType)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentConfigs::PathManagedBy)
                            .string()
                            .not_null()
                            .default("none"),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentConfigs::LastUpdated)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentConfigs::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentConfigs::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create sdk_environment_variables table
        manager
            .create_table(
                Table::create()
                    .table(SdkEnvironmentVariables::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::SdkType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::Value)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::Scope)
                            .string()
                            .not_null()
                            .default("global"),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::IsExported)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkEnvironmentVariables::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create sdk_path_entries table
        manager
            .create_table(
                Table::create()
                    .table(SdkPathEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SdkPathEntries::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SdkPathEntries::SdkType).string().not_null())
                    .col(ColumnDef::new(SdkPathEntries::Path).string().not_null())
                    .col(ColumnDef::new(SdkPathEntries::Version).string().not_null())
                    .col(
                        ColumnDef::new(SdkPathEntries::Scope)
                            .string()
                            .not_null()
                            .default("global"),
                    )
                    .col(
                        ColumnDef::new(SdkPathEntries::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SdkPathEntries::Priority)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SdkPathEntries::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkPathEntries::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create sdk_version_aliases table
        manager
            .create_table(
                Table::create()
                    .table(SdkVersionAliases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SdkVersionAliases::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SdkVersionAliases::SdkType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SdkVersionAliases::Version)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SdkVersionAliases::Alias).string().not_null())
                    .col(
                        ColumnDef::new(SdkVersionAliases::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SdkVersionAliases::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_sdk_custom_paths_sdk_type")
                    .table(SdkCustomPaths::Table)
                    .col(SdkCustomPaths::SdkType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sdk_env_vars_sdk_type")
                    .table(SdkEnvironmentVariables::Table)
                    .col(SdkEnvironmentVariables::SdkType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sdk_path_entries_sdk_type")
                    .table(SdkPathEntries::Table)
                    .col(SdkPathEntries::SdkType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sdk_version_aliases_sdk_type")
                    .table(SdkVersionAliases::Table)
                    .col(SdkVersionAliases::SdkType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sdk_version_aliases_unique")
                    .table(SdkVersionAliases::Table)
                    .col(SdkVersionAliases::SdkType)
                    .col(SdkVersionAliases::Alias)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SdkVersionAliases::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SdkPathEntries::Table).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(SdkEnvironmentVariables::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(SdkEnvironmentConfigs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SdkCustomPaths::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum SdkCustomPaths {
    Table,
    Id,
    SdkType,
    Path,
    Version,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SdkEnvironmentConfigs {
    Table,
    SdkType,
    PathManagedBy,
    LastUpdated,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SdkEnvironmentVariables {
    Table,
    Id,
    SdkType,
    Name,
    Value,
    Scope,
    IsExported,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SdkPathEntries {
    Table,
    Id,
    SdkType,
    Path,
    Version,
    Scope,
    IsActive,
    Priority,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SdkVersionAliases {
    Table,
    Id,
    SdkType,
    Version,
    Alias,
    CreatedAt,
    UpdatedAt,
}
