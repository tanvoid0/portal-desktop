use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::ProjectModel;

/// API response for projects including many-to-many relationship IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
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
    pub framework_ids: Vec<i32>,
    pub package_manager_ids: Vec<i32>,
    pub language_ids: Vec<i32>,
}

impl ProjectResponse {
    pub fn from_model(
        model: ProjectModel,
        framework_ids: Vec<i32>,
        package_manager_ids: Vec<i32>,
        language_ids: Vec<i32>,
    ) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            path: model.path,
            status: model.status,
            build_command: model.build_command,
            start_command: model.start_command,
            test_command: model.test_command,
            output_directory: model.output_directory,
            dev_port: model.dev_port,
            prod_port: model.prod_port,
            starred: model.starred,
            open_count: model.open_count,
            last_opened: model.last_opened.map(|dt| dt.into()),
            size: model.size,
            file_count: model.file_count,
            git_repository: model.git_repository,
            git_branch: model.git_branch,
            git_commit: model.git_commit,
            has_uncommitted_changes: model.has_uncommitted_changes,
            last_commit: model.last_commit.map(|dt| dt.into()),
            created_at: model.created_at.map(|dt| dt.into()),
            updated_at: model.updated_at.map(|dt| dt.into()),
            framework_ids,
            package_manager_ids,
            language_ids,
        }
    }
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
    pub frameworks: Vec<String>,       // Multiple frameworks detected
    pub languages: Vec<String>,        // Multiple languages detected
    pub package_managers: Vec<String>, // Multiple package managers detected
    pub build_command: Option<String>,
    pub start_command: Option<String>,
    pub test_command: Option<String>,
    pub output_directory: Option<String>,
    pub dev_port: Option<i32>,
    pub prod_port: Option<i32>,
}
