//! Command-facing DTOs for the move engine. Kept here (not in the `src-tauri`
//! command layer) so they can carry `#[derive(TS)]` the same way every other
//! typed command result in this domain does — `ts-rs` is a `portal-core`
//! dependency, not a `src-tauri` one.

use serde::Serialize;
use ts_rs::TS;

use super::usage;
use crate::entities::project_move;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct SkippedDir {
    pub rel: String,
    #[ts(type = "number")]
    pub bytes: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct NestedProject {
    pub id: i32,
    pub name: String,
    pub path: String,
}

/// What `preflight_project_move` reports before a move is started: everything
/// `start_project_move` would refuse or warn about, plus the sizing a user
/// needs to decide — how much would copy, what would be skipped, what else
/// sits under the source and would get silently rebased along with it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct PreflightResult {
    pub blockers: Vec<usage::Signal>,
    pub warnings: Vec<usage::PortWarning>,
    pub skipped_dirs: Vec<SkippedDir>,
    #[ts(type = "number")]
    pub file_count: i64,
    #[ts(type = "number")]
    pub total_bytes: i64,
    #[ts(type = "number")]
    pub available_bytes: i64,
    pub nested_projects: Vec<NestedProject>,
}

/// A `project_moves` journal row, the shape the UI reads. The raw
/// `project_move::Model` isn't `TS`-derived (entities stay DB-internal, see
/// `ProjectResponse` for the established pattern), so this mirrors its
/// columns plus `cancel_requested` and `running`, which live in the command
/// layer's `MoveControl`, not the row.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../../src/lib/domains/projects/generated/")]
pub struct MoveSummary {
    pub id: String,
    pub project_id: i32,
    pub source_path: String,
    pub dest_path: String,
    pub state: String,
    pub include_regenerable: bool,
    #[ts(type = "number")]
    pub file_count: i32,
    #[ts(type = "number")]
    pub total_bytes: i64,
    #[ts(type = "number")]
    pub files_done: i32,
    #[ts(type = "number")]
    pub bytes_done: i64,
    #[ts(type = "number")]
    pub resume_count: i32,
    pub unrewritten: Vec<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
    pub cancel_requested: bool,
    /// A move command for this row's project is actively awaiting right now
    /// — not just journaled mid-flight. Lets the interrupted-move banner
    /// skip a `copying`/`verifying` row that is someone's live move, not a
    /// crash left for the user to resolve (board #207/#208 review).
    pub running: bool,
}

impl MoveSummary {
    pub fn from_row(row: project_move::Model, cancel_requested: bool, running: bool) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            source_path: row.source_path,
            dest_path: row.dest_path,
            state: row.state,
            include_regenerable: row.include_regenerable,
            file_count: row.file_count,
            total_bytes: row.total_bytes,
            files_done: row.files_done,
            bytes_done: row.bytes_done,
            resume_count: row.resume_count,
            unrewritten: serde_json::from_str(&row.unrewritten).unwrap_or_default(),
            error: row.error,
            started_at: row.started_at,
            updated_at: row.updated_at,
            finished_at: row.finished_at,
            cancel_requested,
            running,
        }
    }
}
