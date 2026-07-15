use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::redirect::Policy;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};

use crate::database::DatabaseManager;
use crate::domains::credentials::services::credential_service::{
    CredentialCreateRequest, CredentialService, CredentialUpdateRequest,
};
use crate::domains::projects::entities::ProjectResponse;
use crate::domains::projects::services::ProjectService;
use crate::domains::settings::services::settings_service::SettingsService;
use crate::entities::github_connection as github_connection_entity;
use crate::entities::github_project_link as github_project_link_entity;
use crate::entities::project as project_entity;
use crate::log_warn;
use crate::process_ext::NoWindowExt;

use super::types::{
    GitHubAccount, GitHubCloneRepositoryRequest, GitHubConnectionStatus, GitHubCreateIssueRequest,
    GitHubDeviceFlowPollResult, GitHubDeviceFlowStart, GitHubDispatchWorkflowRequest, GitHubIssue,
    GitHubLinkExistingRepositoryRequest, GitHubListIssuesRequest, GitHubListWorkflowRunsRequest,
    GitHubListWorkflowsRequest, GitHubLocalRepositoryDetection, GitHubProjectLink,
    GitHubProjectLinkResult, GitHubRepoOwner, GitHubRepoProjects, GitHubRepository,
    GitHubUpdateIssueRequest, GitHubWorkflow, GitHubWorkflowJob, GitHubWorkflowJobStep,
    GitHubWorkflowRun, GitHubWorkflowRunDetail,
};

const GITHUB_CONNECTION_ID: &str = "github";
const GITHUB_CREDENTIAL_TYPE: &str = "github_token";
const DEFAULT_DEVICE_SCOPE: &str = "repo read:user";
const WORKFLOW_LOGS_UNAVAILABLE_MSG: &str =
    "Logs are not available yet. They usually appear shortly after the job starts producing output.";

#[derive(Clone)]
pub struct GitHubService {
    db: Arc<DatabaseManager>,
    client: reqwest::Client,
    logs_client: reqwest::Client,
}

struct EnsuredProject {
    project: ProjectResponse,
    created_project_id: Option<i32>,
}

struct LinkProjectFailure {
    message: String,
    created_project_id: Option<i32>,
}

struct PathClaim {
    exact_project: Option<project_entity::Model>,
}

impl GitHubService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("portal-desktop")
            .build()
            .expect("github client");
        let logs_client = reqwest::Client::builder()
            .user_agent("portal-desktop")
            .redirect(Policy::none())
            .build()
            .expect("github logs client");
        Self {
            db,
            client,
            logs_client,
        }
    }

    pub async fn get_connection_status(&self) -> Result<GitHubConnectionStatus, String> {
        let configured = self.client_id().is_ok();
        let Some(connection) = self.get_connection_record().await? else {
            return Ok(GitHubConnectionStatus {
                connected: false,
                client_id_configured: configured,
                account: None,
            });
        };

        Ok(GitHubConnectionStatus {
            connected: true,
            client_id_configured: configured,
            account: Some(self.connection_to_account(connection)?),
        })
    }

    pub async fn start_device_flow(
        &self,
        scope: Option<String>,
    ) -> Result<GitHubDeviceFlowStart, String> {
        let client_id = self.client_id()?;
        let response = self
            .client
            .post("https://github.com/login/device/code")
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", client_id.as_str()),
                (
                    "scope",
                    scope
                        .as_deref()
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or(DEFAULT_DEVICE_SCOPE),
                ),
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to start GitHub device flow: {e}"))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Invalid GitHub device flow response: {e}"))?;

        Ok(GitHubDeviceFlowStart {
            device_code: required_string(&body, "device_code")?,
            user_code: required_string(&body, "user_code")?,
            verification_uri: required_string(&body, "verification_uri")?,
            verification_uri_complete: body
                .get("verification_uri_complete")
                .and_then(Value::as_str)
                .map(str::to_string),
            expires_in: body
                .get("expires_in")
                .and_then(Value::as_u64)
                .unwrap_or(900),
            interval: body.get("interval").and_then(Value::as_u64).unwrap_or(5),
        })
    }

    pub async fn poll_device_flow(
        &self,
        device_code: &str,
    ) -> Result<GitHubDeviceFlowPollResult, String> {
        let client_id = self.client_id()?;
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(|e| format!("Failed to poll GitHub device flow: {e}"))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| format!("Invalid GitHub token response: {e}"))?;

        if let Some(error) = body.get("error").and_then(Value::as_str) {
            let (status, retry_after_seconds) = match error {
                "authorization_pending" => ("pending", Some(5)),
                "slow_down" => ("pending", Some(10)),
                "expired_token" => ("expired", None),
                "access_denied" => ("denied", None),
                _ => ("error", None),
            };
            return Ok(GitHubDeviceFlowPollResult {
                status: status.to_string(),
                message: body
                    .get("error_description")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| Some(error.to_string())),
                retry_after_seconds,
                account: None,
            });
        }

        let token = required_string(&body, "access_token")?;
        let account = self.fetch_account(&token).await?;
        self.persist_connection(&token, &account).await?;

        Ok(GitHubDeviceFlowPollResult {
            status: "connected".to_string(),
            message: None,
            retry_after_seconds: None,
            account: Some(account),
        })
    }

    pub async fn disconnect(&self) -> Result<GitHubConnectionStatus, String> {
        let conn = self.db.get_connection();
        if let Some(existing) = self.get_connection_record().await? {
            let credential_service = CredentialService::new(self.db.get_connection_clone());
            let _ = credential_service
                .delete_credential(&existing.credential_id)
                .await;
            github_connection_entity::Entity::delete_by_id(existing.id)
                .exec(conn)
                .await
                .map_err(|e| format!("Failed to remove GitHub connection: {e}"))?;
        }
        Ok(GitHubConnectionStatus {
            connected: false,
            client_id_configured: self.client_id().is_ok(),
            account: None,
        })
    }

    pub async fn list_repositories(
        &self,
        search: Option<String>,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Vec<GitHubRepository>, String> {
        let token = self.connection_token().await?;
        let mut repos = self
            .github_get_json(
                "/user/repos",
                &token,
                &[
                    ("sort", "updated".to_string()),
                    ("page", page.unwrap_or(1).to_string()),
                    ("per_page", per_page.unwrap_or(50).min(100).to_string()),
                ],
            )
            .await?
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| self.repository_from_value(&item))
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(query) = search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
        {
            repos.retain(|repo| {
                repo.full_name.to_lowercase().contains(&query)
                    || repo.name.to_lowercase().contains(&query)
                    || repo
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query)
            });
        }

        Ok(repos)
    }

    pub async fn get_repository_with_projects(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepoProjects, String> {
        let repository = self.get_repository(owner, repo).await?;
        let linked_projects = self.find_projects_for_repo(owner, repo).await?;
        Ok(GitHubRepoProjects {
            repository,
            linked_projects,
        })
    }

    pub async fn get_repository(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepository, String> {
        let token = self.connection_token().await?;
        let value = self
            .github_get_json(&format!("/repos/{owner}/{repo}"), &token, &[])
            .await?;
        self.repository_from_value(&value)
    }

    pub async fn list_issues(
        &self,
        request: GitHubListIssuesRequest,
    ) -> Result<Vec<GitHubIssue>, String> {
        let token = self.connection_token().await?;
        let state = request.state.unwrap_or_else(|| "open".to_string());
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(50).min(100);

        let (path, extra): (String, Vec<(&str, String)>) = if let (Some(owner), Some(repo)) =
            (request.owner.as_deref(), request.repo.as_deref())
        {
            (format!("/repos/{owner}/{repo}/issues"), vec![])
        } else {
            (
                "/issues".to_string(),
                vec![(
                    "filter",
                    request.filter.unwrap_or_else(|| "assigned".to_string()),
                )],
            )
        };

        let mut query = vec![
            ("state", state),
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
        ];
        query.extend(extra);

        let items = self
            .github_get_json(&path, &token, &query)
            .await?
            .as_array()
            .cloned()
            .unwrap_or_default();

        let include_pull_requests = request.include_pull_requests.unwrap_or(false);
        let mut issues = Vec::new();
        for item in items {
            let issue = self.issue_from_value(&item)?;
            if include_pull_requests || !issue.is_pull_request {
                issues.push(issue);
            }
        }
        Ok(issues)
    }

    pub async fn list_linked_repo_full_names(&self) -> Result<Vec<String>, String> {
        let conn = self.db.get_connection();
        let links = github_project_link_entity::Entity::find()
            .all(conn)
            .await
            .map_err(|e| format!("Failed to query linked repositories: {e}"))?;
        let mut names: Vec<String> = links.into_iter().map(|l| l.repo_full_name).collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> Result<GitHubIssue, String> {
        let token = self.connection_token().await?;
        let value = self
            .github_get_json(
                &format!("/repos/{owner}/{repo}/issues/{number}"),
                &token,
                &[],
            )
            .await?;
        self.issue_from_value(&value)
    }

    pub async fn create_issue(
        &self,
        request: GitHubCreateIssueRequest,
    ) -> Result<GitHubIssue, String> {
        let token = self.connection_token().await?;
        let mut body = json!({ "title": request.title });
        if let Some(text) = request.body {
            body["body"] = json!(text);
        }
        if let Some(labels) = request.labels {
            body["labels"] = json!(labels);
        }
        let value = self
            .github_post_json(
                &format!("/repos/{}/{}/issues", request.owner, request.repo),
                &token,
                body,
            )
            .await?;
        self.issue_from_value(&value)
    }

    pub async fn update_issue(
        &self,
        request: GitHubUpdateIssueRequest,
    ) -> Result<GitHubIssue, String> {
        let token = self.connection_token().await?;
        let mut body = json!({});
        if let Some(title) = request.title {
            body["title"] = json!(title);
        }
        if let Some(text) = request.body {
            body["body"] = json!(text);
        }
        if let Some(state) = request.state {
            body["state"] = json!(state);
        }
        if let Some(labels) = request.labels {
            body["labels"] = json!(labels);
        }
        let value = self
            .github_patch_json(
                &format!(
                    "/repos/{}/{}/issues/{}",
                    request.owner, request.repo, request.number
                ),
                &token,
                body,
            )
            .await?;
        self.issue_from_value(&value)
    }

    pub async fn clone_repository(
        &self,
        request: GitHubCloneRepositoryRequest,
    ) -> Result<GitHubProjectLinkResult, String> {
        let repo = self.get_repository(&request.owner, &request.repo).await?;
        let token = self.connection_token().await?;
        let destination = PathBuf::from(request.destination_path.trim());
        if destination.as_os_str().is_empty() {
            return Err("Destination path is required.".to_string());
        }
        if destination.exists() {
            return Err(format!(
                "Destination already exists: {}",
                destination.display()
            ));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| "Destination path must include a parent directory.".to_string())?;
        if !parent.exists() {
            return Err(format!(
                "Destination parent directory does not exist: {}",
                parent.display()
            ));
        }

        self.assert_path_can_be_claimed(&destination, &repo, false)
            .await?;

        let clone_url = repo.clone_url.clone();
        let auth_clone_url = inject_token_into_clone_url(&clone_url, &token);
        run_git_clone(&auth_clone_url, &destination).map_err(|e| {
            format!("Repository clone failed before any local project was linked: {e}")
        })?;

        if auth_clone_url != clone_url {
            if let Err(rewrite_error) = self.restore_origin_url(&destination, &clone_url) {
                let cleanup_error = cleanup_cloned_directory(&destination).err();
                return Err(format_remote_rewrite_failure(
                    &destination,
                    &rewrite_error,
                    cleanup_error.as_deref(),
                ));
            }
        }

        match self.link_project_path_to_repo(&destination, &repo).await {
            Ok(result) => Ok(result),
            Err(error) => {
                let project_cleanup_error = if let Some(project_id) = error.created_project_id {
                    self.delete_project_if_present(project_id).await.err()
                } else {
                    None
                };
                let path_cleanup_error = cleanup_cloned_directory(&destination).err();
                Err(format_clone_link_failure(
                    &destination,
                    &error.message,
                    project_cleanup_error.as_deref(),
                    path_cleanup_error.as_deref(),
                ))
            }
        }
    }

    pub async fn link_existing_repository(
        &self,
        request: GitHubLinkExistingRepositoryRequest,
    ) -> Result<GitHubProjectLinkResult, String> {
        let path = PathBuf::from(request.path.trim());
        if !path.is_dir() {
            return Err(format!(
                "Repository path does not exist or is not a directory: {}",
                path.display()
            ));
        }
        if !path.join(".git").exists() {
            return Err(format!("Path is not a git repository: {}", path.display()));
        }

        let detected = self
            .detect_local_repository(path.to_string_lossy().as_ref())
            .await?;
        let owner = request.owner.or(detected.owner).ok_or_else(|| {
            "Could not determine repository owner. Provide one manually.".to_string()
        })?;
        let repo_name = request.repo.or(detected.repo).ok_or_else(|| {
            "Could not determine repository name. Provide one manually.".to_string()
        })?;
        let repo = self.get_repository(&owner, &repo_name).await?;

        self.link_project_path_to_repo(&path, &repo)
            .await
            .map_err(|error| error.message)
    }

    pub async fn get_project_link(
        &self,
        project_id: i32,
    ) -> Result<Option<GitHubProjectLink>, String> {
        let conn = self.db.get_connection();
        let row = github_project_link_entity::Entity::find()
            .filter(github_project_link_entity::Column::ProjectId.eq(project_id))
            .one(conn)
            .await
            .map_err(|e| format!("Failed to load GitHub project link: {e}"))?;
        Ok(row.map(project_link_from_model))
    }

    pub async fn list_workflows(
        &self,
        request: GitHubListWorkflowsRequest,
    ) -> Result<Vec<GitHubWorkflow>, String> {
        let token = self.connection_token().await?;
        let owner = request.owner.trim();
        let repo = request.repo.trim();
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(50).min(100);

        let value = self
            .github_get_json(
                &format!("/repos/{owner}/{repo}/actions/workflows"),
                &token,
                &[
                    ("page", page.to_string()),
                    ("per_page", per_page.to_string()),
                ],
            )
            .await?;

        value
            .get("workflows")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| self.workflow_from_value(&item))
            .collect()
    }

    pub async fn dispatch_workflow(
        &self,
        request: GitHubDispatchWorkflowRequest,
    ) -> Result<(), String> {
        let token = self.connection_token().await?;
        let owner = request.owner.trim();
        let repo = request.repo.trim();
        let workflow_id = request.workflow_id;
        let ref_name = request.ref_name.trim();
        if ref_name.is_empty() {
            return Err("ref_name is required to dispatch a workflow".to_string());
        }

        let body = json!({ "ref": ref_name });
        let path =
            format!("/repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches");
        let response = self
            .client
            .post(format!("https://api.github.com{path}"))
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        let status = response.status();
        if status.as_u16() == 204 || status.is_success() {
            return Ok(());
        }

        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(format!(
            "Failed to dispatch workflow ({status}): {text}"
        ))
    }

    pub async fn list_workflow_runs(
        &self,
        request: GitHubListWorkflowRunsRequest,
    ) -> Result<Vec<GitHubWorkflowRun>, String> {
        let token = self.connection_token().await?;
        let owner = request.owner.trim();
        let repo = request.repo.trim();
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(20).min(100);

        let mut query = vec![
            ("page", page.to_string()),
            ("per_page", per_page.to_string()),
        ];
        if let Some(branch) = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            query.push(("branch", branch.to_string()));
        }
        if let Some(status) = request
            .status
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            query.push(("status", status.to_string()));
        }

        let value = self
            .github_get_json(
                &format!("/repos/{owner}/{repo}/actions/runs"),
                &token,
                &query,
            )
            .await?;

        let runs = value
            .get("workflow_runs")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| self.workflow_run_from_value(&item))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    pub async fn get_workflow_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: i64,
    ) -> Result<GitHubWorkflowRunDetail, String> {
        let token = self.connection_token().await?;
        let run_value = self
            .github_get_json(
                &format!("/repos/{owner}/{repo}/actions/runs/{run_id}"),
                &token,
                &[],
            )
            .await?;
        let run = self.workflow_run_from_value(&run_value)?;

        let jobs_value = self
            .github_get_json(
                &format!("/repos/{owner}/{repo}/actions/runs/{run_id}/jobs"),
                &token,
                &[],
            )
            .await?;
        let jobs = jobs_value
            .get("jobs")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| self.workflow_job_from_value(&item))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(GitHubWorkflowRunDetail { run, jobs })
    }

    pub async fn get_workflow_job_logs(
        &self,
        owner: &str,
        repo: &str,
        job_id: i64,
    ) -> Result<String, String> {
        let token = self.connection_token().await?;
        let response = self
            .logs_client
            .get(format!(
                "https://api.github.com/repos/{owner}/{repo}/actions/jobs/{job_id}/logs"
            ))
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Ok(WORKFLOW_LOGS_UNAVAILABLE_MSG.to_string());
        }

        if status.is_redirection() {
            let log_url = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| "GitHub job logs redirect missing location header".to_string())?;
            return self.fetch_workflow_log_blob(log_url).await;
        }

        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("GitHub API {status}: {text}"));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read GitHub job logs: {e}"))
    }

    async fn fetch_workflow_log_blob(&self, log_url: &str) -> Result<String, String> {
        let log_response = self
            .client
            .get(log_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch GitHub job logs: {e}"))?;

        let status = log_response.status();
        if status.as_u16() == 404 {
            return Ok(WORKFLOW_LOGS_UNAVAILABLE_MSG.to_string());
        }
        if status.as_u16() == 403 {
            return Ok(
                "Log access expired or is temporarily unavailable. Refresh to fetch a new link."
                    .to_string(),
            );
        }
        if !status.is_success() {
            return Err(format!("GitHub job logs request failed: {status}"));
        }

        log_response
            .text()
            .await
            .map_err(|e| format!("Failed to read GitHub job logs: {e}"))
    }

    pub async fn detect_local_repository(
        &self,
        path: &str,
    ) -> Result<GitHubLocalRepositoryDetection, String> {
        let repo_path = PathBuf::from(path.trim());
        let remote_url = run_git_capture(
            [
                "-C",
                repo_path.to_string_lossy().as_ref(),
                "remote",
                "get-url",
                "origin",
            ]
            .as_slice(),
        )
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
        let parsed = remote_url.as_deref().and_then(parse_github_remote);
        Ok(GitHubLocalRepositoryDetection {
            path: repo_path.to_string_lossy().to_string(),
            is_git_repository: repo_path.join(".git").exists(),
            owner: parsed.as_ref().map(|(owner, _)| owner.clone()),
            repo: parsed.as_ref().map(|(_, repo)| repo.clone()),
            repo_full_name: parsed.map(|(owner, repo)| format!("{owner}/{repo}")),
            remote_url,
        })
    }

    async fn link_project_path_to_repo(
        &self,
        path: &Path,
        repo: &GitHubRepository,
    ) -> Result<GitHubProjectLinkResult, LinkProjectFailure> {
        let ensured = self.ensure_project_for_path(path, repo).await?;
        let link = self
            .upsert_project_link(ensured.project.id, repo)
            .await
            .map_err(|message| LinkProjectFailure {
                message,
                created_project_id: ensured.created_project_id,
            })?;

        Ok(GitHubProjectLinkResult {
            project: ensured.project,
            link,
            local_path: path.to_string_lossy().to_string(),
        })
    }

    async fn ensure_project_for_path(
        &self,
        path: &Path,
        repo: &GitHubRepository,
    ) -> Result<EnsuredProject, LinkProjectFailure> {
        let claim = self
            .assert_path_can_be_claimed(path, repo, true)
            .await
            .map_err(|message| LinkProjectFailure {
                message,
                created_project_id: None,
            })?;
        let service = ProjectService::new(&self.db);

        if let Some(model) = claim.exact_project {
            if let Ok(Some(existing_link)) = self.get_project_link(model.id).await {
                if existing_link.repo_full_name != repo.full_name {
                    return Err(LinkProjectFailure {
                        message: format!(
                            "Project at {} is already linked to {}, not {}.",
                            path.display(),
                            existing_link.repo_full_name,
                            repo.full_name
                        ),
                        created_project_id: None,
                    });
                }
            }

            let existing = service
                .get_project(model.id)
                .await
                .map_err(|error| LinkProjectFailure {
                    message: error.to_string(),
                    created_project_id: None,
                })?
                .ok_or_else(|| LinkProjectFailure {
                    message: format!("Project {} no longer exists.", model.id),
                    created_project_id: None,
                })?;

            let project = match service.refresh_project_metadata(model.id).await {
                Ok(Some(refreshed)) => refreshed,
                Ok(None) => existing,
                Err(refresh_error) => {
                    log_warn!(
                        "GitHub",
                        "Failed to refresh metadata for existing project {}: {}",
                        model.id,
                        refresh_error
                    );
                    existing
                }
            };

            return Ok(EnsuredProject {
                project,
                created_project_id: None,
            });
        }

        let created = service
            .create_project(
                repo.name.clone(),
                repo.description.clone(),
                path.to_string_lossy().to_string(),
                vec![],
                vec![],
                vec![],
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(|message| LinkProjectFailure {
                message,
                created_project_id: None,
            })?;
        let created_project_id = Some(created.id);

        let project = match service.refresh_project_metadata(created.id).await {
            Ok(Some(refreshed)) => refreshed,
            Ok(None) => created,
            Err(refresh_error) => {
                log_warn!(
                    "GitHub",
                    "Failed to refresh metadata for new project {}: {}",
                    created.id,
                    refresh_error
                );
                created
            }
        };

        Ok(EnsuredProject {
            project,
            created_project_id,
        })
    }

    async fn assert_path_can_be_claimed(
        &self,
        path: &Path,
        repo: &GitHubRepository,
        require_existing_path: bool,
    ) -> Result<PathClaim, String> {
        let requested = normalize_path_for_comparison(path, require_existing_path)?;
        let conn = self.db.get_connection();
        let projects = project_entity::Entity::find()
            .all(conn)
            .await
            .map_err(|e| format!("Failed to query project ownership for path checks: {e}"))?;

        let mut exact_project = None;
        for project in projects {
            let project_path = PathBuf::from(&project.path);
            let normalized_project =
                normalize_path_for_comparison(&project_path, project_path.exists())?;

            if requested == normalized_project {
                exact_project = Some(project);
                continue;
            }

            if paths_overlap(&requested, &normalized_project) {
                return Err(format!(
                    "Path conflict: {} overlaps with existing project path {}. Link or clone into a separate directory.",
                    path.display(),
                    project.path
                ));
            }
        }

        if let Some(project) = &exact_project {
            if let Some(existing_link) = self.get_project_link(project.id).await? {
                if existing_link.repo_full_name != repo.full_name {
                    return Err(format!(
                        "Path {} is already owned by project {} and linked to {}.",
                        path.display(),
                        project.id,
                        existing_link.repo_full_name
                    ));
                }
            }
        }

        Ok(PathClaim { exact_project })
    }

    async fn delete_project_if_present(&self, project_id: i32) -> Result<(), String> {
        let service = ProjectService::new(&self.db);
        let deleted = service.delete_project(project_id).await?;
        if deleted {
            Ok(())
        } else {
            Err(format!(
                "Temporary project {project_id} could not be deleted."
            ))
        }
    }

    fn restore_origin_url(&self, destination: &Path, clone_url: &str) -> Result<(), String> {
        run_git(
            [
                "-C",
                destination.to_string_lossy().as_ref(),
                "remote",
                "set-url",
                "origin",
                clone_url,
            ]
            .as_slice(),
        )
        .map_err(|e| {
            format!(
                "Clone completed at {}, but restoring the sanitized origin URL failed: {}",
                destination.display(),
                e
            )
        })
    }

    async fn upsert_project_link(
        &self,
        project_id: i32,
        repo: &GitHubRepository,
    ) -> Result<GitHubProjectLink, String> {
        let conn = self.db.get_connection();
        let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
        let active = github_project_link_entity::ActiveModel {
            project_id: Set(project_id),
            repo_owner: Set(repo.owner.login.clone()),
            repo_name: Set(repo.name.clone()),
            repo_full_name: Set(repo.full_name.clone()),
            repo_html_url: Set(Some(repo.html_url.clone())),
            repo_api_url: Set(Some(format!(
                "https://api.github.com/repos/{}/{}",
                repo.owner.login, repo.name
            ))),
            default_branch: Set(Some(repo.default_branch.clone())),
            clone_url: Set(Some(repo.clone_url.clone())),
            ssh_url: Set(repo.ssh_url.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        github_project_link_entity::Entity::insert(active)
            .on_conflict(
                OnConflict::column(github_project_link_entity::Column::ProjectId)
                    .update_columns([
                        github_project_link_entity::Column::RepoOwner,
                        github_project_link_entity::Column::RepoName,
                        github_project_link_entity::Column::RepoFullName,
                        github_project_link_entity::Column::RepoHtmlUrl,
                        github_project_link_entity::Column::RepoApiUrl,
                        github_project_link_entity::Column::DefaultBranch,
                        github_project_link_entity::Column::CloneUrl,
                        github_project_link_entity::Column::SshUrl,
                        github_project_link_entity::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to save GitHub project link: {e}"))?;

        Ok(GitHubProjectLink {
            project_id,
            repo_owner: repo.owner.login.clone(),
            repo_name: repo.name.clone(),
            repo_full_name: repo.full_name.clone(),
            repo_html_url: Some(repo.html_url.clone()),
            default_branch: Some(repo.default_branch.clone()),
            clone_url: Some(repo.clone_url.clone()),
            ssh_url: repo.ssh_url.clone(),
        })
    }

    async fn find_projects_for_repo(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<ProjectResponse>, String> {
        let full_name = format!("{owner}/{repo}");
        let conn = self.db.get_connection();
        let links = github_project_link_entity::Entity::find()
            .filter(github_project_link_entity::Column::RepoFullName.eq(full_name))
            .all(conn)
            .await
            .map_err(|e| format!("Failed to query linked projects: {e}"))?;

        let project_service = ProjectService::new(&self.db);
        let mut projects = Vec::new();
        for link in links {
            if let Some(project) = project_service.get_project(link.project_id).await? {
                projects.push(project);
            }
        }
        Ok(projects)
    }

    async fn persist_connection(&self, token: &str, account: &GitHubAccount) -> Result<(), String> {
        let credential_id = self.upsert_token(token, account).await?;
        let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
        let conn = self.db.get_connection();
        let active = github_connection_entity::ActiveModel {
            id: Set(GITHUB_CONNECTION_ID.to_string()),
            account_id: Set(account.id),
            login: Set(account.login.clone()),
            name: Set(account.name.clone()),
            avatar_url: Set(account.avatar_url.clone()),
            html_url: Set(account.html_url.clone()),
            scopes_json: Set(serde_json::to_string(&account.scopes).unwrap_or_else(|_| "[]".into())),
            credential_id: Set(credential_id),
            created_at: Set(now),
            updated_at: Set(now),
        };
        github_connection_entity::Entity::insert(active)
            .on_conflict(
                OnConflict::column(github_connection_entity::Column::Id)
                    .update_columns([
                        github_connection_entity::Column::AccountId,
                        github_connection_entity::Column::Login,
                        github_connection_entity::Column::Name,
                        github_connection_entity::Column::AvatarUrl,
                        github_connection_entity::Column::HtmlUrl,
                        github_connection_entity::Column::ScopesJson,
                        github_connection_entity::Column::CredentialId,
                        github_connection_entity::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to save GitHub connection: {e}"))?;
        Ok(())
    }

    async fn upsert_token(&self, token: &str, account: &GitHubAccount) -> Result<String, String> {
        let credential_service = CredentialService::new(self.db.get_connection_clone());
        let existing = self.get_connection_record().await?;
        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), json!("github"));
        metadata.insert("login".to_string(), json!(account.login));
        metadata.insert("account_id".to_string(), json!(account.id));

        if let Some(connection) = existing {
            credential_service
                .update_credential(
                    &connection.credential_id,
                    CredentialUpdateRequest {
                        name: Some(format!("GitHub ({})", account.login)),
                        description: Some("GitHub OAuth device flow token".to_string()),
                        tags: Some(vec!["github".to_string()]),
                        value: Some(token.to_string()),
                        fields: None,
                        metadata: Some(metadata),
                        status: Some("active".to_string()),
                        expires_at: None,
                    },
                )
                .await
                .map_err(|e| format!("Failed to update GitHub credential: {e}"))?;
            Ok(connection.credential_id)
        } else {
            let created = credential_service
                .create_credential(CredentialCreateRequest {
                    name: format!("GitHub ({})", account.login),
                    credential_type: GITHUB_CREDENTIAL_TYPE.to_string(),
                    description: Some("GitHub OAuth device flow token".to_string()),
                    tags: Some(vec!["github".to_string()]),
                    value: token.to_string(),
                    fields: None,
                    metadata: Some(metadata),
                    expires_at: None,
                })
                .await
                .map_err(|e| format!("Failed to store GitHub credential: {e}"))?;
            Ok(created.id)
        }
    }

    async fn connection_token(&self) -> Result<String, String> {
        let connection = self
            .get_connection_record()
            .await?
            .ok_or_else(|| "GitHub is not connected.".to_string())?;
        CredentialService::new(self.db.get_connection_clone())
            .decrypt_credential(&connection.credential_id)
            .await
            .map_err(|e| format!("Failed to decrypt GitHub credential: {e}"))
    }

    async fn fetch_account(&self, token: &str) -> Result<GitHubAccount, String> {
        let value = self.github_get_json("/user", token, &[]).await?;
        let scopes = value
            .get("_scopes")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(GitHubAccount {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            login: required_string(&value, "login")?,
            name: value
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string),
            avatar_url: value
                .get("avatar_url")
                .and_then(Value::as_str)
                .map(str::to_string),
            html_url: required_string(&value, "html_url")?,
            scopes,
        })
    }

    async fn github_get_json(
        &self,
        path: &str,
        token: &str,
        query: &[(&str, String)],
    ) -> Result<Value, String> {
        let mut req = self
            .client
            .get(format!("https://api.github.com{path}"))
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {token}"));
        if !query.is_empty() {
            req = req.query(query);
        }
        let response = req
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;
        self.read_json_response(response).await
    }

    async fn github_post_json(
        &self,
        path: &str,
        token: &str,
        body: Value,
    ) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("https://api.github.com{path}"))
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;
        self.read_json_response(response).await
    }

    async fn github_patch_json(
        &self,
        path: &str,
        token: &str,
        body: Value,
    ) -> Result<Value, String> {
        let response = self
            .client
            .patch(format!("https://api.github.com{path}"))
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;
        self.read_json_response(response).await
    }

    async fn read_json_response(&self, response: reqwest::Response) -> Result<Value, String> {
        let status = response.status();
        let scopes = response
            .headers()
            .get("x-oauth-scopes")
            .and_then(|v| v.to_str().ok())
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read GitHub response: {e}"))?;
        if !status.is_success() {
            return Err(format!("GitHub API {status}: {text}"));
        }
        let mut value: Value =
            serde_json::from_str(&text).map_err(|e| format!("Invalid GitHub JSON: {e}"))?;
        if !scopes.is_empty() && value.is_object() {
            value["_scopes"] = json!(scopes);
        }
        Ok(value)
    }

    async fn get_connection_record(
        &self,
    ) -> Result<Option<github_connection_entity::Model>, String> {
        github_connection_entity::Entity::find_by_id(GITHUB_CONNECTION_ID.to_string())
            .one(self.db.get_connection())
            .await
            .map_err(|e| format!("Failed to load GitHub connection: {e}"))
    }

    fn client_id(&self) -> Result<String, String> {
        if let Ok(value) = std::env::var("GITHUB_CLIENT_ID") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }

        let settings = SettingsService::new()
            .load_settings()
            .map_err(|e| format!("Failed to load app settings: {e}"))?;
        let configured = settings.app.integrations.github.client_id.trim();
        if !configured.is_empty() {
            return Ok(configured.to_string());
        }

        Err(
            "GitHub Device Flow is not configured. Add a GitHub Client ID in Settings > GitHub."
                .to_string(),
        )
    }

    fn connection_to_account(
        &self,
        model: github_connection_entity::Model,
    ) -> Result<GitHubAccount, String> {
        let scopes = serde_json::from_str::<Vec<String>>(&model.scopes_json)
            .map_err(|e| format!("Invalid GitHub scopes metadata: {e}"))?;
        Ok(GitHubAccount {
            id: model.account_id,
            login: model.login,
            name: model.name,
            avatar_url: model.avatar_url,
            html_url: model.html_url,
            scopes,
        })
    }

    fn repository_from_value(&self, value: &Value) -> Result<GitHubRepository, String> {
        let owner = value.get("owner").cloned().unwrap_or_else(|| json!({}));
        Ok(GitHubRepository {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            name: required_string(value, "name")?,
            full_name: required_string(value, "full_name")?,
            owner: GitHubRepoOwner {
                login: required_string(&owner, "login")?,
                avatar_url: owner
                    .get("avatar_url")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                html_url: owner
                    .get("html_url")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            },
            description: value
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            private: value
                .get("private")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            fork: value.get("fork").and_then(Value::as_bool).unwrap_or(false),
            html_url: required_string(value, "html_url")?,
            clone_url: required_string(value, "clone_url")?,
            ssh_url: value
                .get("ssh_url")
                .and_then(Value::as_str)
                .map(str::to_string),
            default_branch: required_string(value, "default_branch")?,
            language: value
                .get("language")
                .and_then(Value::as_str)
                .map(str::to_string),
            stargazers_count: value
                .get("stargazers_count")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            forks_count: value
                .get("forks_count")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            open_issues_count: value
                .get("open_issues_count")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            updated_at: value
                .get("updated_at")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }

    fn workflow_from_value(&self, value: &Value) -> Result<GitHubWorkflow, String> {
        Ok(GitHubWorkflow {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            name: required_string(value, "name")?,
            path: required_string(value, "path")?,
            state: required_string(value, "state")?,
            html_url: required_string(value, "html_url")?,
        })
    }

    fn workflow_run_from_value(&self, value: &Value) -> Result<GitHubWorkflowRun, String> {
        Ok(GitHubWorkflowRun {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            name: required_string(value, "name")?,
            workflow_id: value
                .get("workflow_id")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            run_number: value
                .get("run_number")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            status: required_string(value, "status")?,
            conclusion: value
                .get("conclusion")
                .and_then(Value::as_str)
                .map(str::to_string),
            event: required_string(value, "event")?,
            head_branch: value
                .get("head_branch")
                .and_then(Value::as_str)
                .map(str::to_string),
            head_sha: required_string(value, "head_sha")?,
            display_title: value
                .get("display_title")
                .and_then(Value::as_str)
                .map(str::to_string),
            html_url: required_string(value, "html_url")?,
            created_at: value
                .get("created_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            updated_at: value
                .get("updated_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            run_started_at: value
                .get("run_started_at")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }

    fn workflow_job_from_value(&self, value: &Value) -> Result<GitHubWorkflowJob, String> {
        let steps = value
            .get("steps")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|step| {
                        Ok(GitHubWorkflowJobStep {
                            name: required_string(step, "name")?,
                            status: required_string(step, "status")?,
                            conclusion: step
                                .get("conclusion")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                            number: step
                                .get("number")
                                .and_then(Value::as_i64)
                                .unwrap_or_default(),
                            started_at: step
                                .get("started_at")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                            completed_at: step
                                .get("completed_at")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(GitHubWorkflowJob {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            run_id: value
                .get("run_id")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            name: required_string(value, "name")?,
            status: required_string(value, "status")?,
            conclusion: value
                .get("conclusion")
                .and_then(Value::as_str)
                .map(str::to_string),
            html_url: required_string(value, "html_url")?,
            started_at: value
                .get("started_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            completed_at: value
                .get("completed_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            steps,
        })
    }

    fn issue_from_value(&self, value: &Value) -> Result<GitHubIssue, String> {
        let labels = value
            .get("labels")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("name").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let assignees = value
            .get("assignees")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("login").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let repo_full_name = value
            .get("repository")
            .and_then(|repo| repo.get("full_name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                value
                    .get("repository_url")
                    .and_then(Value::as_str)
                    .and_then(|url| url.strip_prefix("https://api.github.com/repos/"))
                    .map(str::to_string)
            });
        Ok(GitHubIssue {
            id: value.get("id").and_then(Value::as_i64).unwrap_or_default(),
            number: value
                .get("number")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            title: required_string(value, "title")?,
            body: value
                .get("body")
                .and_then(Value::as_str)
                .map(str::to_string),
            state: required_string(value, "state")?,
            html_url: required_string(value, "html_url")?,
            repo_full_name,
            author_login: value
                .get("user")
                .and_then(|user| user.get("login"))
                .and_then(Value::as_str)
                .map(str::to_string),
            labels,
            assignees,
            created_at: value
                .get("created_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            updated_at: value
                .get("updated_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            closed_at: value
                .get("closed_at")
                .and_then(Value::as_str)
                .map(str::to_string),
            is_pull_request: value.get("pull_request").is_some(),
        })
    }
}

fn required_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("GitHub response missing field: {key}"))
}

fn project_link_from_model(model: github_project_link_entity::Model) -> GitHubProjectLink {
    GitHubProjectLink {
        project_id: model.project_id,
        repo_owner: model.repo_owner,
        repo_name: model.repo_name,
        repo_full_name: model.repo_full_name,
        repo_html_url: model.repo_html_url,
        default_branch: model.default_branch,
        clone_url: model.clone_url,
        ssh_url: model.ssh_url,
    }
}

fn inject_token_into_clone_url(clone_url: &str, token: &str) -> String {
    if clone_url.starts_with("https://github.com/") {
        return clone_url.replacen("https://", &format!("https://x-access-token:{token}@"), 1);
    }
    clone_url.to_string()
}

fn normalize_path_for_comparison(
    path: &Path,
    require_existing_path: bool,
) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to resolve current directory: {e}"))?
            .join(path)
    };

    if require_existing_path || absolute.exists() {
        return fs::canonicalize(&absolute).map_err(|e| {
            format!(
                "Failed to resolve project path {} for conflict checks: {}",
                path.display(),
                e
            )
        });
    }

    Ok(normalize_components(&absolute))
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn cleanup_cloned_directory(path: &Path) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|e| {
        format!(
            "Failed to remove cloned directory {}: {}",
            path.display(),
            e
        )
    })
}

fn format_remote_rewrite_failure(
    path: &Path,
    rewrite_error: &str,
    cleanup_error: Option<&str>,
) -> String {
    if let Some(cleanup_error) = cleanup_error {
        format!(
            "{rewrite_error} Portal also failed to remove the cloned directory at {}. Manual cleanup is required because the repository may still contain the temporary authenticated origin URL: {}",
            path.display(),
            cleanup_error
        )
    } else {
        format!(
            "{rewrite_error} Portal removed the cloned directory at {} so the operation stays user-safe.",
            path.display()
        )
    }
}

fn format_clone_link_failure(
    path: &Path,
    link_error: &str,
    project_cleanup_error: Option<&str>,
    path_cleanup_error: Option<&str>,
) -> String {
    let mut message = format!(
        "Repository was cloned to {}, but linking it in Portal failed: {}.",
        path.display(),
        link_error
    );

    match (project_cleanup_error, path_cleanup_error) {
        (None, None) => {
            message.push_str(" Portal removed the cloned directory and rolled back any temporary project record.");
        }
        (project_error, path_error) => {
            message.push_str(" Automatic rollback was only partially successful.");
            if let Some(project_error) = project_error {
                message.push_str(&format!(" Project cleanup error: {}.", project_error));
            }
            if let Some(path_error) = path_error {
                message.push_str(&format!(" Directory cleanup error: {}.", path_error));
            }
        }
    }

    message
}

fn parse_github_remote(remote: &str) -> Option<(String, String)> {
    let trimmed = remote.trim().trim_end_matches(".git");
    if let Some(path) = trimmed.strip_prefix("https://github.com/") {
        let mut parts = path.split('/');
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        return Some((owner, repo));
    }
    if let Some(path) = trimmed.strip_prefix("git@github.com:") {
        let mut parts = path.split('/');
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        return Some((owner, repo));
    }
    None
}

fn run_git_clone(clone_url: &str, destination: &Path) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .arg("clone")
        .arg(clone_url)
        .arg(destination.as_os_str())
        .no_window()
        .output()
        .map_err(|e| format!("Failed to start git clone: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn run_git(args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .args(args)
        .no_window()
        .output()
        .map_err(|e| format!("Failed to start git: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn run_git_capture(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .no_window()
        .output()
        .map_err(|e| format!("Failed to start git: {e}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    Err(format!(
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        format_clone_link_failure, inject_token_into_clone_url, parse_github_remote, paths_overlap,
    };
    use std::path::Path;

    #[test]
    fn injects_token_only_for_https_github_urls() {
        assert_eq!(
            inject_token_into_clone_url("https://github.com/acme/repo.git", "abc"),
            "https://x-access-token:abc@github.com/acme/repo.git"
        );
        assert_eq!(
            inject_token_into_clone_url("git@github.com:acme/repo.git", "abc"),
            "git@github.com:acme/repo.git"
        );
    }

    #[test]
    fn parses_https_and_ssh_remotes() {
        assert_eq!(
            parse_github_remote("https://github.com/acme/repo.git"),
            Some(("acme".to_string(), "repo".to_string()))
        );
        assert_eq!(
            parse_github_remote("git@github.com:acme/repo.git"),
            Some(("acme".to_string(), "repo".to_string()))
        );
    }

    #[test]
    fn detects_nested_path_conflicts() {
        assert!(paths_overlap(
            Path::new("D:/code/projects/repo"),
            Path::new("D:/code/projects")
        ));
        assert!(!paths_overlap(
            Path::new("D:/code/projects/repo-a"),
            Path::new("D:/code/projects/repo-b")
        ));
    }

    #[test]
    fn clone_link_failure_mentions_rollback_status() {
        let message = format_clone_link_failure(
            Path::new("D:/tmp/repo"),
            "db write failed",
            None,
            Some("permission denied"),
        );
        assert!(message.contains("Repository was cloned"));
        assert!(message.contains("Directory cleanup error"));
    }
}
