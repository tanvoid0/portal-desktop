use super::docker_service::{DockerService, Deployment, DeploymentStatus, DeploymentType, EnvironmentConfig};
use super::cli_service::CliService;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

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
    // In a real application, this would interact with a database
    // For now, we'll use an in-memory store for demonstration
    deployments: std::sync::Mutex<Vec<Deployment>>,
}

impl DeploymentService {
    pub fn new() -> Self {
        Self {
            docker_service: DockerService::new(),
            cli_service: CliService::new(),
            deployments: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a new deployment
    pub async fn create_deployment(&self, request: CreateDeploymentRequest) -> Result<Deployment, String> {
        let deployment_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        match request.deployment_type {
            DeploymentType::Docker => {
                // Check if Docker is available
                if !self.docker_service.is_docker_available().await? {
                    return Err("Docker is not available or not running".to_string());
                }

                // Generate Dockerfile
                let dockerfile_content = self.docker_service.generate_dockerfile(
                    &request.project_type,
                    &request.sdk_version,
                    &request.project_path,
                )?;

                // Use provided dockerfile path or create one
                let dockerfile_path = request.dockerfile_path
                    .unwrap_or_else(|| format!("{}/Dockerfile", request.project_path));
                
                // Write Dockerfile if it doesn't exist
                if !std::path::Path::new(&dockerfile_path).exists() {
                    std::fs::write(&dockerfile_path, dockerfile_content)
                        .map_err(|e| format!("Failed to write Dockerfile: {}", e))?;
                }

                // Build Docker image
                let image_name = request.docker_image_name
                    .unwrap_or_else(|| format!("{}-{}", request.name.to_lowercase().replace(' ', "-"), deployment_id));
                
                self.docker_service.build_image(
                    &request.project_path,
                    &image_name,
                    Some(&dockerfile_path)
                ).await?;

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
                    command: None,
                    working_directory: None,
                    process_id: None,
                    logs: Vec::new(),
                    created_at: now.to_rfc3339(),
                    updated_at: now.to_rfc3339(),
                };

                // Store deployment
                self.deployments.lock().unwrap().push(deployment.clone());

                Ok(deployment)
            }
            DeploymentType::Cli => {
                // Validate CLI command
                let command = request.command
                    .ok_or_else(|| "Command is required for CLI deployment".to_string())?;
                
                let working_dir = request.working_directory
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
                    command: Some(command.clone()),
                    working_directory: Some(working_dir.clone()),
                    process_id: None,
                    logs: Vec::new(),
                    created_at: now.to_rfc3339(),
                    updated_at: now.to_rfc3339(),
                };

                // Store deployment
                self.deployments.lock().unwrap().push(deployment.clone());

                Ok(deployment)
            }
        }
    }

    /// Start a deployment
    pub async fn start_deployment(&self, deployment_id: &str) -> Result<Deployment, String> {
        // Get deployment info and release lock
        let deployment_type = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments.iter().find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get deployment info and release lock
                let (docker_image_name, container_name, ports, volumes, environment_variables) = {
                    let deployments = self.deployments.lock().unwrap();
                    
                    let deployment = deployments.iter().find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    
                    let container_name = format!("{}-{}", deployment.name.to_lowercase().replace(' ', "-"), deployment.id);
                    let ports = if let Some(port) = deployment.exposed_port {
                        vec![(port, 3000)]
                    } else {
                        vec![(3000, 3000)]
                    };
                    let volumes = vec![
                        (deployment.project_id.clone(), "/app".to_string())
                    ];
                    let docker_image_name = deployment.docker_image_name.clone()
                        .ok_or_else(|| "Docker image name not found".to_string())?;
                    let environment_variables = deployment.environment.variables.clone();
                    
                    (docker_image_name, container_name, ports, volumes, environment_variables)
                }; // Lock is released here

                let container_id = self.docker_service.run_container(
                    &docker_image_name,
                    &container_name,
                    &ports,
                    &volumes,
                    &environment_variables,
                ).await?;

                // Update deployment with container_id
                let mut deployments = self.deployments.lock().unwrap();
                let deployment = deployments.iter_mut().find(|d| d.id == deployment_id)
                    .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    
                deployment.container_id = Some(container_id);
                deployment.status = DeploymentStatus::Running;
                deployment.updated_at = Utc::now().to_rfc3339();

                Ok(deployment.clone())
            }
            DeploymentType::Cli => {
                // Get deployment info and release lock
                let (command, working_dir, environment) = {
                    let deployments = self.deployments.lock().unwrap();
                    let deployment = deployments.iter().find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    
                    let command = deployment.command.clone()
                        .ok_or_else(|| "Command not found".to_string())?;
                    let working_dir = deployment.working_directory.clone()
                        .unwrap_or_else(|| deployment.project_id.clone());
                    let environment = deployment.environment.variables.clone();
                    
                    (command, working_dir, environment)
                }; // Lock is released here

                // Spawn CLI process
                let pid = self.cli_service.spawn_process(
                    deployment_id,
                    &command,
                    Some(&working_dir),
                    &environment,
                ).await?;

                // Update deployment with process_id
                let mut deployments = self.deployments.lock().unwrap();
                let deployment = deployments.iter_mut().find(|d| d.id == deployment_id)
                    .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    
                deployment.process_id = Some(pid);
                deployment.status = DeploymentStatus::Running;
                deployment.updated_at = Utc::now().to_rfc3339();

                Ok(deployment.clone())
            }
        }
    }

    /// Stop a deployment
    pub async fn stop_deployment(&self, deployment_id: &str) -> Result<Deployment, String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments.iter().find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and release lock
                let container_id = {
                    let deployments = self.deployments.lock().unwrap();
                    deployments.iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?
                        .container_id.clone()
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
        let mut deployments = self.deployments.lock().unwrap();
        let deployment = deployments.iter_mut()
            .find(|d| d.id == deployment_id)
            .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            
        deployment.status = DeploymentStatus::Stopped;
        deployment.updated_at = Utc::now().to_rfc3339();

        Ok(deployment.clone())
    }

    /// Delete a deployment
    pub async fn delete_deployment(&self, deployment_id: &str) -> Result<(), String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments.iter().find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and release lock
                let container_id = {
                    let deployments = self.deployments.lock().unwrap();
                    deployments.iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?
                        .container_id.clone()
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

        // Remove from deployments list
        let mut deployments = self.deployments.lock().unwrap();
        deployments.retain(|d| d.id != deployment_id);
        Ok(())
    }

    /// Get all deployments
    pub fn get_deployments(&self) -> Result<Vec<Deployment>, String> {
        Ok(self.deployments.lock().unwrap().clone())
    }

    /// Get deployment by ID
    pub fn get_deployment(&self, deployment_id: &str) -> Result<Option<Deployment>, String> {
        Ok(self.deployments.lock().unwrap().iter().find(|d| d.id == deployment_id).cloned())
    }

    /// Get deployment logs
    pub async fn get_deployment_logs(&self, deployment_id: &str, tail: Option<usize>) -> Result<Vec<String>, String> {
        // Get deployment type and relevant info
        let deployment_type = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments.iter().find(|d| d.id == deployment_id)
                .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
            deployment.deployment_type.clone()
        }; // Lock is released here

        match deployment_type {
            DeploymentType::Docker => {
                // Get container_id and logs, then release lock
                let container_id = {
                    let deployments = self.deployments.lock().unwrap();
                    let deployment = deployments.iter()
                        .find(|d| d.id == deployment_id)
                        .ok_or_else(|| format!("Deployment with id {} not found", deployment_id))?;
                    deployment.container_id.clone()
                }; // Lock is released here
                
                if let Some(container_id) = container_id {
                    self.docker_service.get_container_logs(&container_id, tail).await
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
    pub async fn update_deployment(&self, request: UpdateDeploymentRequest) -> Result<Deployment, String> {
        let mut deployments = self.deployments.lock().unwrap();
        
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
            let deployments = self.deployments.lock().unwrap();
            deployments
                .iter()
                .map(|d| (d.id.clone(), d.deployment_type.clone(), d.container_id.clone(), d.process_id))
                .collect()
        }; // Lock is released here
        
        // Check status for each deployment
        let mut status_updates = Vec::new();
        for (deployment_id, deployment_type, container_id, _process_id) in deployment_info {
            match deployment_type {
                DeploymentType::Docker => {
                    if let Some(container_id) = container_id {
                        match self.docker_service.get_container_status(&container_id).await {
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
        let mut deployments = self.deployments.lock().unwrap();
        for (deployment_id, new_status) in status_updates {
            if let Some(deployment) = deployments.iter_mut().find(|d| d.id == deployment_id) {
                deployment.status = new_status;
                deployment.updated_at = Utc::now().to_rfc3339();
            }
        }

        Ok(deployments.clone())
    }
}
