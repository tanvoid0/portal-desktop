use sea_orm_migration::prelude::*;

/// Migration: Create device_approvals table
///
/// This migration creates the device_approvals table for managing
/// device authentication and approval for browser access.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeviceApprovals::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeviceApprovals::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::DeviceId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::DeviceName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::DeviceInfo)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::Passcode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::PasscodeExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::Approved)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::ApprovalType)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::ApprovedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(DeviceApprovals::AccessToken).string().null())
                    .col(
                        ColumnDef::new(DeviceApprovals::TokenExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DeviceApprovals::LastUsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on device_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_device_approvals_device_id")
                    .table(DeviceApprovals::Table)
                    .col(DeviceApprovals::DeviceId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Create index on access_token for faster token verification
        manager
            .create_index(
                Index::create()
                    .name("idx_device_approvals_access_token")
                    .table(DeviceApprovals::Table)
                    .col(DeviceApprovals::AccessToken)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeviceApprovals::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DeviceApprovals {
    Table,
    Id,
    DeviceId,
    DeviceName,
    DeviceInfo,
    Passcode,
    PasscodeExpiresAt,
    Approved,
    ApprovalType,
    ApprovedAt,
    ExpiresAt,
    AccessToken,
    TokenExpiresAt,
    CreatedAt,
    UpdatedAt,
    LastUsedAt,
}
