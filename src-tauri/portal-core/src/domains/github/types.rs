use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domains::projects::entities::ProjectResponse;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubAccount {
    #[ts(type = "number")]
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub html_url: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubConnectionStatus {
    pub connected: bool,
    pub client_id_configured: bool,
    pub account: Option<GitHubAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubDeviceFlowStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    #[ts(type = "number")]
    pub expires_in: u64,
    #[ts(type = "number")]
    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubDeviceFlowPollResult {
    pub status: String,
    pub message: Option<String>,
    #[ts(type = "number")]
    pub retry_after_seconds: Option<u64>,
    pub account: Option<GitHubAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubRepoOwner {
    pub login: String,
    pub avatar_url: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubRepository {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubRepoOwner,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: Option<String>,
    pub default_branch: String,
    pub language: Option<String>,
    #[ts(type = "number")]
    pub stargazers_count: u64,
    #[ts(type = "number")]
    pub forks_count: u64,
    #[ts(type = "number")]
    pub open_issues_count: u64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubIssue {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub repo_full_name: Option<String>,
    pub author_login: Option<String>,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub closed_at: Option<String>,
    pub is_pull_request: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubListIssuesRequest {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub state: Option<String>,
    pub filter: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub include_pull_requests: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubCreateIssueRequest {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubUpdateIssueRequest {
    pub owner: String,
    pub repo: String,
    #[ts(type = "number")]
    pub number: i64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubProjectLink {
    pub project_id: i32,
    pub repo_owner: String,
    pub repo_name: String,
    pub repo_full_name: String,
    pub repo_html_url: Option<String>,
    pub default_branch: Option<String>,
    pub clone_url: Option<String>,
    pub ssh_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubRepoProjects {
    pub repository: GitHubRepository,
    pub linked_projects: Vec<ProjectResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubCloneRepositoryRequest {
    pub owner: String,
    pub repo: String,
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubLinkExistingRepositoryRequest {
    pub path: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubLocalRepositoryDetection {
    pub path: String,
    pub is_git_repository: bool,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub repo_full_name: Option<String>,
    pub remote_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubProjectLinkResult {
    pub project: ProjectResponse,
    pub link: GitHubProjectLink,
    pub local_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubWorkflowRun {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    #[ts(type = "number")]
    pub workflow_id: i64,
    #[ts(type = "number")]
    pub run_number: i64,
    pub status: String,
    pub conclusion: Option<String>,
    pub event: String,
    pub head_branch: Option<String>,
    pub head_sha: String,
    pub display_title: Option<String>,
    pub html_url: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub run_started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubWorkflowJobStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    #[ts(type = "number")]
    pub number: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubWorkflowJob {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub run_id: i64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub steps: Vec<GitHubWorkflowJobStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubWorkflowRunDetail {
    pub run: GitHubWorkflowRun,
    pub jobs: Vec<GitHubWorkflowJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubListWorkflowRunsRequest {
    pub owner: String,
    pub repo: String,
    pub branch: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../src/lib/domains/github/generated/")]
pub struct GitHubWorkflow {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub path: String,
    pub state: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubListWorkflowsRequest {
    pub owner: String,
    pub repo: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(
    export,
    export_to = "../../../src/lib/domains/github/generated/",
    optional_fields
)]
pub struct GitHubDispatchWorkflowRequest {
    pub owner: String,
    pub repo: String,
    #[ts(type = "number")]
    pub workflow_id: i64,
    pub ref_name: String,
}
