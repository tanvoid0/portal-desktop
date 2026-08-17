//! Adds discovery + triage columns to `projects`.
//!
//! `discovery_path` is the logical id (`<root>/<category>/<name>`) rather than
//! the absolute path, so a project keeps its identity — and its triage decision —
//! when the folder moves to another drive or is nested under a new parent.
//!
//! Every column is nullable or defaulted, so projects created by hand before this
//! migration stay valid and simply carry no discovery data until the first scan.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// `(column name, definition)` — checked individually so a partially applied
/// migration can be re-run without erroring.
fn columns() -> Vec<(&'static str, ColumnDef)> {
    vec![
        (
            "discovery_path",
            ColumnDef::new(Projects::DiscoveryPath).string().null().to_owned(),
        ),
        (
            "activity_status",
            ColumnDef::new(Projects::ActivityStatus).string().null().to_owned(),
        ),
        (
            "last_activity",
            ColumnDef::new(Projects::LastActivity)
                .timestamp_with_time_zone()
                .null()
                .to_owned(),
        ),
        (
            "risk_level",
            ColumnDef::new(Projects::RiskLevel).string().null().to_owned(),
        ),
        (
            "unpushed_commits",
            ColumnDef::new(Projects::UnpushedCommits)
                .integer()
                .not_null()
                .default(0)
                .to_owned(),
        ),
        (
            "stash_count",
            ColumnDef::new(Projects::StashCount)
                .integer()
                .not_null()
                .default(0)
                .to_owned(),
        ),
        (
            "has_remote",
            ColumnDef::new(Projects::HasRemote)
                .boolean()
                .not_null()
                .default(false)
                .to_owned(),
        ),
        (
            "module_count",
            ColumnDef::new(Projects::ModuleCount)
                .integer()
                .not_null()
                .default(1)
                .to_owned(),
        ),
        (
            "duplicate_group",
            ColumnDef::new(Projects::DuplicateGroup).string().null().to_owned(),
        ),
        (
            "regenerable_dirs",
            ColumnDef::new(Projects::RegenerableDirs).string().null().to_owned(),
        ),
        (
            "discovered_at",
            ColumnDef::new(Projects::DiscoveredAt)
                .timestamp_with_time_zone()
                .null()
                .to_owned(),
        ),
        (
            "triage_decision",
            ColumnDef::new(Projects::TriageDecision).string().null().to_owned(),
        ),
        (
            "triage_note",
            ColumnDef::new(Projects::TriageNote).text().null().to_owned(),
        ),
        (
            "triage_successor",
            ColumnDef::new(Projects::TriageSuccessor).string().null().to_owned(),
        ),
        (
            "triage_updated_at",
            ColumnDef::new(Projects::TriageUpdatedAt)
                .timestamp_with_time_zone()
                .null()
                .to_owned(),
        ),
    ]
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for (name, mut def) in columns() {
            if !manager.has_column("projects", name).await? {
                manager
                    .alter_table(
                        Table::alter()
                            .table(Projects::Table)
                            .add_column(&mut def)
                            .to_owned(),
                    )
                    .await?;
            }
        }

        // Every scan looks projects up by this column.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_projects_discovery_path")
                    .table(Projects::Table)
                    .col(Projects::DiscoveryPath)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_projects_discovery_path")
                    .table(Projects::Table)
                    .to_owned(),
            )
            .await?;

        for (name, _) in columns() {
            if manager.has_column("projects", name).await? {
                let col = Alias::new(name);
                manager
                    .alter_table(
                        Table::alter()
                            .table(Projects::Table)
                            .drop_column(col)
                            .to_owned(),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    DiscoveryPath,
    ActivityStatus,
    LastActivity,
    RiskLevel,
    UnpushedCommits,
    StashCount,
    HasRemote,
    ModuleCount,
    DuplicateGroup,
    RegenerableDirs,
    DiscoveredAt,
    TriageDecision,
    TriageNote,
    TriageSuccessor,
    TriageUpdatedAt,
}
