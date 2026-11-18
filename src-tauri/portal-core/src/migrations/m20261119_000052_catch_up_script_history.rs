//! Catch-up for the history copy in `m20261119_000051_unify_runs`. That copy is a
//! snapshot: `ScriptExecutionService` kept writing `script_executions` until it
//! became a facade over the pipeline runner, so runs made between the snapshot
//! and the cutover would be lost. Copy the table again, overwriting rows that are
//! already there (INSERT OR REPLACE), so `pipeline_executions` holds them too.
//!
//! `script_executions` is not dropped here; a later migration does that once the
//! facade has shipped. Forward-only: `down` does nothing.

use super::m20261119_000051_unify_runs::copy_script_history;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::TransactionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let txn = manager.get_connection().begin().await?;
        copy_script_history(&txn, true).await?;
        txn.commit().await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
