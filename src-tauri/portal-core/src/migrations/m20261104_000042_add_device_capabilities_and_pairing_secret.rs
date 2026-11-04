//! Adds the two columns phase 4's pairing needs to `device_approvals`.
//!
//! - `capabilities`: JSON array of the capability names a paired device was
//!   granted at approval (e.g. `["coder","terminal"]`). NOT NULL, default `[]`
//!   so pre-existing rows read as "nothing granted".
//! - `pairing_secret`: SHA-256 of the per-device pairing secret. Gates
//!   `/pair/status` so that knowing a device_id is not enough to collect its
//!   token. Nullable — legacy rows carry none and simply can't pass the gate.
//!
//! Forward-only in spirit; the `down` is here only to keep the migration
//! reversible in tests, guarded so a re-run is harmless.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager
            .has_column("device_approvals", "capabilities")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(DeviceApprovals::Table)
                        .add_column(
                            ColumnDef::new(DeviceApprovals::Capabilities)
                                .text()
                                .not_null()
                                .default("[]"),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column("device_approvals", "pairing_secret")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(DeviceApprovals::Table)
                        .add_column(ColumnDef::new(DeviceApprovals::PairingSecret).text().null())
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager
            .has_column("device_approvals", "pairing_secret")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(DeviceApprovals::Table)
                        .drop_column(DeviceApprovals::PairingSecret)
                        .to_owned(),
                )
                .await?;
        }

        if manager
            .has_column("device_approvals", "capabilities")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(DeviceApprovals::Table)
                        .drop_column(DeviceApprovals::Capabilities)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum DeviceApprovals {
    Table,
    Capabilities,
    PairingSecret,
}
