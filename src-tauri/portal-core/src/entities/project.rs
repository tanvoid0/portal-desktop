use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub status: String,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub test_command: Option<String>,
    pub output_directory: Option<String>,
    pub dev_port: Option<i32>,
    pub prod_port: Option<i32>,
    pub starred: bool,
    pub open_count: i32,
    pub last_opened: Option<DateTimeWithTimeZone>,
    pub size: i64,
    pub file_count: i32,
    pub git_repository: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<DateTimeWithTimeZone>,
    /// Logical id from discovery: `<root>/<category>/<name>`. Survives folder
    /// moves, which is why triage decisions key off it rather than `path`.
    pub discovery_path: Option<String>,
    /// active | recent | dormant | stale
    pub activity_status: Option<String>,
    /// Newest source-file mtime or commit, whichever is later. Excludes `.git`
    /// internals and build output.
    pub last_activity: Option<DateTimeWithTimeZone>,
    /// none | low | med | high — see `discovery::analysis::score_risk`.
    pub risk_level: Option<String>,
    pub unpushed_commits: i32,
    pub stash_count: i32,
    pub has_remote: bool,
    /// Project roots folded into this project (monorepo packages, course modules).
    pub module_count: i32,
    /// Comma-separated cluster keys shared with a probable duplicate.
    pub duplicate_group: Option<String>,
    /// Comma-separated regenerable directories present (node_modules, target, …).
    pub regenerable_dirs: Option<String>,
    pub discovered_at: Option<DateTimeWithTimeZone>,
    /// keep | archive | delete | review
    pub triage_decision: Option<String>,
    pub triage_note: Option<String>,
    /// Path of the project that replaces this one.
    pub triage_successor: Option<String>,
    pub triage_updated_at: Option<DateTimeWithTimeZone>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project_framework::Entity")]
    ProjectFrameworks,

    #[sea_orm(has_many = "super::project_language::Entity")]
    ProjectLanguages,

    #[sea_orm(has_many = "super::project_package_manager::Entity")]
    ProjectPackageManagers,
}

impl Related<super::framework::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_framework::Relation::Framework.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_framework::Relation::Project.def().rev())
    }
}

impl Related<super::language::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_language::Relation::Language.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_language::Relation::Project.def().rev())
    }
}

impl Related<super::package_manager::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_package_manager::Relation::PackageManager.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::project_package_manager::Relation::Project
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
