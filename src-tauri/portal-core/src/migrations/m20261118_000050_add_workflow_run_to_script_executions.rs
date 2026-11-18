//! Adds workflow-run grouping to `script_executions`: which run a step belongs
//! to, which workflow/step it was, and the run's plan (`[{id,name,needs}]` in
//! topo order, as JSON) so the run can be redrawn from the rows alone. All
//! null for ad-hoc runs and for rows written before this migration.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const COLUMNS: [&str; 4] = [
    "workflow_run_id",
    "workflow_id",
    "step_id",
    "workflow_plan_json",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for name in COLUMNS {
            if !manager.has_column("script_executions", name).await? {
                let mut col = ColumnDef::new(Alias::new(name));
                if name == "workflow_plan_json" {
                    col.text();
                } else {
                    col.string();
                }
                col.null();
                manager
                    .alter_table(
                        Table::alter()
                            .table(ScriptExecutions::Table)
                            .add_column(col)
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_script_executions_workflow_run_id")
                    .table(ScriptExecutions::Table)
                    .col(Alias::new("workflow_run_id"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_script_executions_workflow_run_id")
                    .table(ScriptExecutions::Table)
                    .to_owned(),
            )
            .await?;

        for name in COLUMNS.into_iter().rev() {
            if manager.has_column("script_executions", name).await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(ScriptExecutions::Table)
                            .drop_column(Alias::new(name))
                            .to_owned(),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ScriptExecutions {
    Table,
}
