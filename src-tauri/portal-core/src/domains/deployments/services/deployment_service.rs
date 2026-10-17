use super::cli_service::CliService;
use super::detect;
use super::docker_service::{
    find_free_port, Deployment, DeploymentStatus, DeploymentType, DockerService, EnvironmentConfig,
    PROXY_NETWORK,
};
use crate::database::DatabaseManager;
use crate::domains::deployments::repositories::deployment_repository::DeploymentRepository;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Does a `docker run` error read like the host port could not be published?
/// Covers Linux "already allocated", a lost probe-then-publish race, and the
/// Windows reserved-range case where a port binds in user mode but Docker's
/// publisher is refused — the signal to retry on the next free port.
fn is_port_unavailable(err: &str) -> bool {
    let e = err.to_lowercase();
    e.contains("port is already allocated")
        || e.contains("address already in use")
        || e.contains("forbidden by its access permissions")
        || e.contains("bind for")
        || e.contains("ports are not available")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    pub project_id: String,
    pub name: String,
    pub deployment_type: DeploymentType,
    pub sdk_version: String,
    pub project_type: String,
    pub project_path: String,
    pub environment: HashMap<String, String>,
    // Docker-specific fields
    pub exposed_port: Option<u16>,
    pub docker_image_name: Option<String>,
    pub dockerfile_path: Option<String>,
    // CLI-specific fields
    pub command: Option<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDeploymentRequest {
    pub id: String,
    pub name: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub status: Option<DeploymentStatus>,
}

pub struct DeploymentService {
    pub docker_service: DockerService,
    pub cli_service: CliService,
    repo: DeploymentRepository,
    cache: RwLock<Vec<Deployment>>,
}

impl DeploymentService {
    pub async fn new(db_manager: Arc<DatabaseManager>) -> Result<Self, String> {
        let repo = DeploymentRepository::new(db_manager);
        let deployments = repo.find_all().await?;
        Ok(Self {
            docker_service: DockerService::new(),
            cli_service: CliService::new(),
            repo,
            cache: RwLock::new(deployments),
        })
    }

    async fn persist(&self, deployment: &Deployment) -> Result<(), String> {
        self.repo.save(deployment).await?;
        let mut cache = self.cache.write().await;
        if let Some(idx) = cache.iter().position(|d| d.id == deployment.id) {
            cache[idx] = deployment.clone();
        } else {
            cache.push(deployment.clone());
        }
        Ok(())
    }

    /// Create a new deployment
    pub async fn create_deployment(
        &self,
        request: CreateDeploymentRequest,
    ) -> Result<Deployment, String> {
        let deployment_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // A missing project path otherwise fails deep in the pipeline with a
        // baffling message (empty build context, or a process spawn error).
        // Fail fast with something the user can act on.
        if !std::path::Path::new(&request.project_path).exists() {
            return Err(format!(
                "Project path not found: {}. The project may have moved — update its path and retry.",
                request.project_path
            ));
        }

        match request.deployment_type {
            DeploymentType::Docker => {
                // Check if Docker is available
                if !self.docker_service.is_docker_available().await? {
                    return Err("Docker is not available or not running".to_string());
                }

                // Detect stack, listen port, existing Dockerfile and host. A
                // committed portal.yaml overrides any inferred value.
                let detected = detect::detect(&request.project_path, &request.project_type);

                // Use an existing Dockerfile verbatim; otherwise generate one
                // whose EXPOSE matches the detected port, writing it only when
                // the project has none of its own.
                let dockerfile_path = if let Some(existing) = detected.dockerfile.clone() {
                    existing
                } else {
                    let content = self.docker_service.generate_dockerfile(
                        &detected.stack,
                        &request.sdk_version,
                        &request.project_path,
                        detected.container_port,
                    )?;
                    let path = request
                        .dockerfile_path
                        .clone()
                        .unwrap_or_else(|| format!("{}/Dockerfile", request.project_path));
                    if !std::path::Path::new(&path).exists() {
                        std::fs::write(&path, content)
                            .map_err(|e| format!("Failed to write Dockerfile: {}", e))?;
                    }
                    path
                };

                // Keep the host's node_modules/.next/.git/secrets out of the
                // build context. Written next to the build, only when the project
                // has none of its own, so a committed .dockerignore always wins.
                let dockerignore = format!("{}/.dockerignore", request.project_path);
                if !std::path::Path::new(&dockerignore).exists() {
                    let _ = std::fs::write(
                        &dockerignore,
                        self.docker_service.generate_dockerignore(),
                    );
                }

                // Build Docker image
                let image_name = request.docker_image_name.clone().unwrap_or_else(|| {
                    format!("{}-{}", detect::sanitize_name(&request.name), deployment_id)
                });

                self.docker_service
                    .build_image(&request.project_path, &image_name, Some(&dockerfile_path))
                    .await?;

                // Host to route to: portal.yaml host, else <name>.localhost.
                let host = detected
                    .host
                    .clone()
                    .unwrap_or_else(|| format!("{}.localhost", detect::sanitize_name(&request.name)));

                // Create deployment record
                let deployment = Deployment {
                    id: deployment_id.clone(),
                    project_id: request.project_id,
                    name: request.name,
                    deployment_type: DeploymentType::Docker,
                    status: DeploymentStatus::Building,
                    sdk_version: request.sdk_version,
                    environment: EnvironmentConfig {
                        variables: request.environment,
                    },
                    docker_image_name: Some(image_name),
                    container_id: None,
                    exposed_port: request.exposed_port,
                    dockerfile_path: Some(dockerfile_path),
                    container_port: Some(detected.container_port),
                    host: Some(host),
                    url: None,
                    command: None,
                    working_directory: None,
                    process_id: None,
                    logs: Vec::new(),
                    created_at: now.to_rfc3339(),
                    updated_at: now.to_rfc3339(),
                };

                self.persist(&deployment).await?;

                Ok(deployment)
            }
            DeploymentType::Cli => {
                // Validate CLI command
                let command = request
                    .command
                    .ok_or_else(|| "Command is required for CLI deployment".to_string())?;

                let working_dir = request
                    .working_directory
                    .unwrap_or_else(|| request.project_path.clone());

                // Create deployment record first (before spawning process)
                let deployment = Deployment {
                    id: deployment_id.clone(),
                    project_id: request.project_id,
                    name: request.name,
                    deployment_type: DeploymentType::Cli,
                    status: DeploymentStatus::Creating,
                    sdk_version: request.sdk_version,
                    environment: EnvironmentConfig {
                        variables: request.environment,
                    },
                    docker_image_name: None,
                    container_id: None,
                    exposed_port: None,
                    dockerfile_path: None,
                    container_port: None,
                    host: None,
                    url: None,
                    command: Some(command.clone()),
                    working_directory: Some(working_dir.clone()),
                    process_id: None,
                    logs: Vec::new(),
                    created_at: now.to_rfc3339(),
                    updated_at: now.to_rfc3339(),
                };

                self.persist(&deployment).await?;

                Ok(deployment)
            }
        }
    }

    /// Start a deployment
    pub async fn start_deployment(&self, deployment_id: &str) -> Result<Deployment, String> {
        // Get deployment info and release lock
        let deployment_type = {
            let deployments = self.cache.read().await;
            let deployment = deployments
                .iter()
                .find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Gather everything under a short read lock.
                let (
                    docker_image_name,
                    container_name,
                    container_port,
                    requested_host_port,
                    host,
                    environment_variables,
                    existing_container_id,
                ) = {
                    let deployments = self.cache.read().await;

                    let deployment = deployments
                        .iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;

                    let container_name =
                        format!("{}-{}", detect::sanitize_name(&deployment.name), deployment.id);
                    // Detected listen port — falls back to 3000 only for rows
                    // created before detection existed.
                    let container_port = deployment.container_port.unwrap_or(3000);
                    let host = deployment.host.clone().unwrap_or_else(|| {
                        format!("{}.localhost", detect::sanitize_name(&deployment.name))
                    });
                    let docker_image_name = deployment
                        .docker_image_name
                        .clone()
                        .ok_or_else(|| "Docker image name not found".to_string())?;

                    (
                        docker_image_name,
                        container_name,
                        container_port,
                        deployment.exposed_port,
                        host,
                        deployment.environment.variables.clone(),
                        deployment.container_id.clone(),
                    )
                }; // Lock is released here

                // Restart path: a container from a previous start still exists
                // (stop only stops it; it is not removed). Re-running the same
                // name would collide, so start the existing container instead.
                if let Some(cid) = existing_container_id {
                    if self.docker_service.container_exists(&cid).await {
                        self.docker_service.start_container(&cid).await?;

                        let mut deployments = self.cache.write().await;
                        let deployment = deployments
                            .iter_mut()
                            .find(|d| d.id == deployment_id)
                            .ok_or_else(|| {
                                format!("Deployment with id {} not found", deployment_id)
                            })?;
                        deployment.status = DeploymentStatus::Running;
                        deployment.updated_at = Utc::now().to_rfc3339();
                        let updated = deployment.clone();
                        drop(deployments);
                        self.persist(&updated).await?;
                        return Ok(updated);
                    }
                }

                // Fresh run. A published host port is the backbone: always
                // publish one so there is a working URL even when hostname
                // routing is unavailable. Hostname routing via the proxy is a
                // bonus layered on top, used only when its docker provider is
                // actually healthy — on some hosts (Docker Desktop for Windows)
                // the proxy container runs but its socket API is unusable, so we
                // verify instead of assuming.
                let (network, labels, proxy_port): (Option<&str>, Vec<(String, String)>, Option<u16>) =
                    match self.docker_service.ensure_proxy().await {
                        Ok(pp) if self.docker_service.proxy_provider_healthy().await => (
                            Some(PROXY_NETWORK),
                            vec![
                                ("traefik.enable".to_string(), "true".to_string()),
                                (
                                    format!("traefik.http.routers.{}.rule", container_name),
                                    format!("Host(`{}`)", host),
                                ),
                                (
                                    format!(
                                        "traefik.http.services.{}.loadbalancer.server.port",
                                        container_name
                                    ),
                                    container_port.to_string(),
                                ),
                            ],
                            Some(pp),
                        ),
                        _ => (None, Vec::new(), None),
                    };

                // No implicit volume. The old code bind-mounted `project_id` (a
                // UUID, not a path) over /app, clobbering the image's own code.
                let volumes: Vec<(String, String)> = Vec::new();

                // Publish a free host port, preferring the requested one (else the
                // container's own port). Never collide: if Docker refuses the
                // publish (reserved range, or a lost race), advance to the next
                // free port and retry.
                let mut candidate = find_free_port(requested_host_port.unwrap_or(container_port));
                let mut last_err = String::new();
                let mut run = None;
                for _ in 0..5 {
                    match self
                        .docker_service
                        .run_container(
                            &docker_image_name,
                            &container_name,
                            &[(candidate, container_port)],
                            &volumes,
                            &environment_variables,
                            network,
                            &labels,
                        )
                        .await
                    {
                        Ok(id) => {
                            run = Some((id, candidate));
                            break;
                        }
                        Err(e) if is_port_unavailable(&e) => {
                            last_err = e;
                            candidate = find_free_port(candidate.saturating_add(1));
                        }
                        Err(e) => return Err(e),
                    }
                }
                let (container_id, actual_host_port) =
                    run.ok_or_else(|| format!("Could not publish a free host port: {}", last_err))?;

                // Prefer the hostname URL when routing is healthy; otherwise the
                // published port is the reliable address.
                let url = match proxy_port {
                    Some(80) => format!("http://{}", host),
                    Some(p) => format!("http://{}:{}", host, p),
                    None => format!("http://localhost:{}", actual_host_port),
                };

                let mut deployments = self.cache.write().await;
                let deployment = deployments
                    .iter_mut()
                    .find(|d| d.id == deployment_id)
                    .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;

                deployment.container_id = Some(container_id);
                deployment.exposed_port = Some(actual_host_port);
                deployment.url = Some(url.clone());
                deployment.logs.push(format!("Available at {}", url));
                if proxy_port.is_none() {
                    deployment.logs.push(
                        "Hostname routing unavailable (local proxy not healthy); using published port"
                            .to_string(),
                    );
                }
                if requested_host_port.is_some() && requested_host_port != Some(actual_host_port) {
                    deployment.logs.push(format!(
                        "Requested host port {} was unavailable; published {} instead",
                        requested_host_port.unwrap_or(0),
                        actual_host_port
                    ));
                }
                deployment.status = DeploymentStatus::Running;
                deployment.updated_at = Utc::now().to_rfc3339();
                let updated = deployment.clone();
                drop(deployments);
                self.persist(&updated).await?;
                Ok(updated)
            }
            DeploymentType::Cli => {
                // Get deployment info and release lock
                let (command, working_dir, environment) = {
                    let deployments = self.cache.read().await;
                    let deployment = deployments
                        .iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;

                    let command = deployment
                        .command
                        .clone()
                        .ok_or_else(|| "Command not found".to_string())?;
                    let working_dir = deployment
                        .working_directory
                        .clone()
                        .unwrap_or_else(|| deployment.project_id.clone());
                    let environment = deployment.environment.variables.clone();

                    (command, working_dir, environment)
                }; // Lock is released here

                // Spawn CLI process
                let pid = self
                    .cli_service
                    .spawn_process(deployment_id, &command, Some(&working_dir), &environment)
                    .await?;

                // Update deployment with process_id
                let mut deployments = self.cache.write().await;
                let deployment = deployments
                    .iter_mut()
                    .find(|d| d.id == deployment_id)
                    .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;

                deployment.process_id = Some(pid);
                deployment.status = DeploymentStatus::Running;
                deployment.updated_at = Utc::now().to_rfc3339();
                let updated = deployment.clone();
                drop(deployments);
                self.persist(&updated).await?;
                Ok(updated)
            }
        }
    }

    /// Stop a deployment
    pub async fn stop_deployment(&self, deployment_id: &str) -> Result<Deployment, String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.cache.read().await;
            let deployment = deployments
                .iter()
                .find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and release lock
                let container_id = {
                    let deployments = self.cache.read().await;
                    deployments
                        .iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?
                        .container_id
                        .clone()
                }; // Lock is released here

                if let Some(container_id) = container_id {
                    self.docker_service.stop_container(&container_id).await?;
                }
            }
            DeploymentType::Cli => {
                // Stop CLI process
                self.cli_service.stop_process(deployment_id).await?;
            }
        }

        // Update deployment status
        let mut deployments = self.cache.write().await;
        let deployment = deployments
            .iter_mut()
            .find(|d| d.id == deployment_id)
            .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;

        deployment.status = DeploymentStatus::Stopped;
        deployment.updated_at = Utc::now().to_rfc3339();
        let updated = deployment.clone();
        drop(deployments);
        self.persist(&updated).await?;

        Ok(updated)
    }

    /// Delete a deployment
    pub async fn delete_deployment(&self, deployment_id: &str) -> Result<(), String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.cache.read().await;
            let deployment = deployments
                .iter()
                .find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and release lock
                let container_id = {
                    let deployments = self.cache.read().await;
                    deployments
                        .iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?
                        .container_id
                        .clone()
                }; // Lock is released here

                // Stop and remove container if running
                if let Some(container_id) = container_id {
                    let _ = self.docker_service.stop_container(&container_id).await;
                    let _ = self.docker_service.remove_container(&container_id).await;
                }
            }
            DeploymentType::Cli => {
                // Stop and cleanup CLI process
                let _ = self.cli_service.stop_process(deployment_id).await;
                self.cli_service.cleanup_process(deployment_id).await;
            }
        }

        self.repo.delete(deployment_id).await?;
        let mut deployments = self.cache.write().await;
        deployments.retain(|d| d.id != deployment_id);
        Ok(())
    }

    /// Get all deployments
    pub async fn get_deployments(&self) -> Result<Vec<Deployment>, String> {
        Ok(self.cache.read().await.clone())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, deployment_id: &str) -> Result<Option<Deployment>, String> {
        Ok(self
            .cache
            .read()
            .await
            .iter()
            .find(|d| d.id == deployment_id)
            .cloned())
    }

    /// Get deployment logs
    pub async fn get_deployment_logs(
        &self,
        deployment_id: &str,
        tail: Option<usize>,
    ) -> Result<Vec<String>, String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.cache.read().await;
            let deployment = deployments
                .iter()
                .find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and logs, then release lock
                let container_id = {
                    let deployments = self.cache.read().await;
                    let deployment = deployments
                        .iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    deployment.container_id.clone()
                }; // Lock is released here

                if let Some(container_id) = container_id {
                    self.docker_service
                        .get_container_logs(&container_id, tail)
                        .await
                } else {
                    Ok(Vec::new())
                }
            }
            DeploymentType::Cli => {
                // Get CLI process logs
                Ok(self.cli_service.get_process_logs(deployment_id, tail))
            }
        }
    }

    /// Update deployment
    pub async fn update_deployment(
        &self,
        request: UpdateDeploymentRequest,
    ) -> Result<Deployment, String> {
        let mut deployments = self.cache.write().await;

        if let Some(deployment) = deployments.iter_mut().find(|d| d.id == request.id) {
            if let Some(name) = request.name {
                deployment.name = name;
            }
            if let Some(environment) = request.environment {
                deployment.environment.variables = environment;
            }
            if let Some(status) = request.status {
                deployment.status = status;
            }
            deployment.updated_at = Utc::now().to_rfc3339();

            Ok(deployment.clone())
        } else {
            Err(format!("Deployment with id {} not found", request.id))
        }
    }

    /// Get container status for all deployments
    pub async fn refresh_deployment_statuses(&self) -> Result<Vec<Deployment>, String> {
        // Get all deployments with their types and relevant info
        let deployment_info: Vec<(String, DeploymentType, Option<String>, Option<u32>)> = {
            let deployments = self.cache.read().await;
            deployments
                .iter()
                .map(|d| {
                    (
                        d.id.clone(),
                        d.deployment_type.clone(),
                        d.container_id.clone(),
                        d.process_id,
                    )
                })
                .collect()
        }; // Lock is released here

        // Check status for each deployment
        let mut status_updates = Vec::new();
        for (deployment_id, deployment_type, container_id, _process_id) in deployment_info {
            match deployment_type {
                DeploymentType::Docker => {
                    if let Some(container_id) = container_id {
                        match self
                            .docker_service
                            .get_container_status(&container_id)
                            .await
                        {
                            Ok(status) => {
                                let new_status = match status.as_str() {
                                    "running" => DeploymentStatus::Running,
                                    "exited" => DeploymentStatus::Stopped,
                                    _ => DeploymentStatus::Unknown,
                                };
                                status_updates.push((deployment_id, new_status));
                            }
                            Err(_) => {
                                status_updates.push((deployment_id, DeploymentStatus::Error));
                            }
                        }
                    }
                }
                DeploymentType::Cli => {
                    match self.cli_service.is_process_running(&deployment_id).await {
                        Ok(is_running) => {
                            let new_status = if is_running {
                                DeploymentStatus::Running
                            } else {
                                DeploymentStatus::Stopped
                            };
                            status_updates.push((deployment_id, new_status));
                        }
                        Err(_) => {
                            status_updates.push((deployment_id, DeploymentStatus::Error));
                        }
                    }
                }
            }
        }

        // Update deployment statuses
        let mut deployments = self.cache.write().await;
        for (deployment_id, new_status) in status_updates {
            if let Some(deployment) = deployments.iter_mut().find(|d| d.id == deployment_id) {
                deployment.status = new_status;
                deployment.updated_at = Utc::now().to_rfc3339();
            }
        }

        Ok(deployments.clone())
    }
}
