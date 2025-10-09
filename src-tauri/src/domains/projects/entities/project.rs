use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub status: String,
    pub framework: Option<String>,
    pub package_manager: Option<String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub test_command: Option<String>,
    pub output_directory: Option<String>,
    pub dev_port: Option<i32>,
    pub prod_port: Option<i32>,
    pub starred: bool,
    pub open_count: i32,
    pub last_opened: Option<DateTime<Utc>>,
    pub size: i64,
    pub file_count: i32,
    pub git_repository: Option<String>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub name: String,
    pub framework: Option<String>,
    pub package_manager: Option<String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub test_command: Option<String>,
    pub output_directory: Option<String>,
    pub dev_port: Option<i32>,
    pub prod_port: Option<i32>,
}