use std::sync::Arc;

use tauri::State;

use crate::database::DatabaseManager;

use super::service::GitHubService;
use super::types::{
    GitHubCloneRepositoryRequest, GitHubConnectionStatus, GitHubCreateIssueRequest,
    GitHubDeviceFlowPollResult, GitHubDeviceFlowStart, GitHubDispatchWorkflowRequest,
    GitHubIssue, GitHubLinkExistingRepositoryRequest, GitHubListIssuesRequest,
    GitHubListWorkflowRunsRequest, GitHubListWorkflowsRequest, GitHubLocalRepositoryDetection,
    GitHubProjectLink, GitHubProjectLinkResult, GitHubRepoProjects, GitHubRepository,
    GitHubUpdateIssueRequest, GitHubWorkflow, GitHubWorkflowRun, GitHubWorkflowRunDetail,
};

#[tauri::command]
pub async fn github_get_connection_status(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubConnectionStatus, String> {
    GitHubService::new(db.inner().clone())
        .get_connection_status()
        .await
}

#[tauri::command]
pub async fn github_start_device_flow(
    scope: Option<String>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubDeviceFlowStart, String> {
    GitHubService::new(db.inner().clone())
        .start_device_flow(scope)
        .await
}

#[tauri::command]
pub async fn github_poll_device_flow(
    device_code: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubDeviceFlowPollResult, String> {
    GitHubService::new(db.inner().clone())
        .poll_device_flow(&device_code)
        .await
}

#[tauri::command]
pub async fn github_disconnect(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubConnectionStatus, String> {
    GitHubService::new(db.inner().clone()).disconnect().await
}

#[tauri::command]
pub async fn github_list_repositories(
    search: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<GitHubRepository>, String> {
    GitHubService::new(db.inner().clone())
        .list_repositories(search, page, per_page)
        .await
}

#[tauri::command]
pub async fn github_list_linked_repos(
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<String>, String> {
    GitHubService::new(db.inner().clone())
        .list_linked_repo_full_names()
        .await
}

#[tauri::command]
pub async fn github_get_repository(
    owner: String,
    repo: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubRepoProjects, String> {
    GitHubService::new(db.inner().clone())
        .get_repository_with_projects(&owner, &repo)
        .await
}

#[tauri::command]
pub async fn github_list_issues(
    request: GitHubListIssuesRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<GitHubIssue>, String> {
    GitHubService::new(db.inner().clone())
        .list_issues(request)
        .await
}

#[tauri::command]
pub async fn github_get_issue(
    owner: String,
    repo: String,
    number: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubIssue, String> {
    GitHubService::new(db.inner().clone())
        .get_issue(&owner, &repo, number)
        .await
}

#[tauri::command]
pub async fn github_create_issue(
    request: GitHubCreateIssueRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubIssue, String> {
    GitHubService::new(db.inner().clone())
        .create_issue(request)
        .await
}

#[tauri::command]
pub async fn github_update_issue(
    request: GitHubUpdateIssueRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubIssue, String> {
    GitHubService::new(db.inner().clone())
        .update_issue(request)
        .await
}

#[tauri::command]
pub async fn github_clone_repository(
    request: GitHubCloneRepositoryRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubProjectLinkResult, String> {
    GitHubService::new(db.inner().clone())
        .clone_repository(request)
        .await
}

#[tauri::command]
pub async fn github_link_existing_repository(
    request: GitHubLinkExistingRepositoryRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubProjectLinkResult, String> {
    GitHubService::new(db.inner().clone())
        .link_existing_repository(request)
        .await
}

#[tauri::command]
pub async fn github_get_project_link(
    project_id: i32,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Option<GitHubProjectLink>, String> {
    GitHubService::new(db.inner().clone())
        .get_project_link(project_id)
        .await
}

#[tauri::command]
pub async fn github_detect_local_repository(
    path: String,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubLocalRepositoryDetection, String> {
    GitHubService::new(db.inner().clone())
        .detect_local_repository(&path)
        .await
}

#[tauri::command]
pub async fn github_list_workflow_runs(
    request: GitHubListWorkflowRunsRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<GitHubWorkflowRun>, String> {
    GitHubService::new(db.inner().clone())
        .list_workflow_runs(request)
        .await
}

#[tauri::command]
pub async fn github_list_workflows(
    request: GitHubListWorkflowsRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<GitHubWorkflow>, String> {
    GitHubService::new(db.inner().clone())
        .list_workflows(request)
        .await
}

#[tauri::command]
pub async fn github_dispatch_workflow(
    request: GitHubDispatchWorkflowRequest,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    GitHubService::new(db.inner().clone())
        .dispatch_workflow(request)
        .await
}

#[tauri::command]
pub async fn github_get_workflow_run(
    owner: String,
    repo: String,
    run_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<GitHubWorkflowRunDetail, String> {
    GitHubService::new(db.inner().clone())
        .get_workflow_run(&owner, &repo, run_id)
        .await
}

#[tauri::command]
pub async fn github_get_workflow_job_logs(
    owner: String,
    repo: String,
    job_id: i64,
    db: State<'_, Arc<DatabaseManager>>,
) -> Result<String, String> {
    GitHubService::new(db.inner().clone())
        .get_workflow_job_logs(&owner, &repo, job_id)
        .await
}
