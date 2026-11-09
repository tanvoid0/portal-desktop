//! `vault`: single-row table holding the credential master key (MK),
//! wrapped under the master password (PBKDF2) and under the recovery key
//! (HKDF). MK itself never changes — see
//! `domains::credentials::services::vault_service`.
//!
//! One row only (`id = 1`, enforced by a CHECK constraint); resetting the
//! password rewrites the `pw_*` columns, never `recovery_*`, so the
//! original recovery key keeps working.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Vault::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Vault::Id).integer().not_null().primary_key())
                    .col(ColumnDef::new(Vault::PwSalt).text().not_null())
                    .col(ColumnDef::new(Vault::PwIterations).integer().not_null())
                    .col(ColumnDef::new(Vault::PwWrappedMk).text().not_null())
                    .col(ColumnDef::new(Vault::RecoverySalt).text().not_null())
                    .col(ColumnDef::new(Vault::RecoveryWrappedMk).text().not_null())
                    .col(ColumnDef::new(Vault::CreatedAt).text().not_null())
                    .col(ColumnDef::new(Vault::UpdatedAt).text().not_null())
                    .check(Expr::col(Vault::Id).eq(1))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vault::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Vault {
    Table,
    Id,
    PwSalt,
    PwIterations,
    PwWrappedMk,
    RecoverySalt,
    RecoveryWrappedMk,
    CreatedAt,
    UpdatedAt,
}
