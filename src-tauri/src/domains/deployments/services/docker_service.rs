use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeploymentStatus {
    Building,
    Running,
    Stopped,
    Error,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildScript {
    pub script: String,
    pub language: String, // bash, powershell, n8n
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub status: DeploymentStatus,
    pub sdk_version: String,
    pub environment: EnvironmentConfig,
    pub docker_image_name: String,
    pub container_id: Option<String>,
    pub exposed_port: Option<u16>,
    pub logs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct DockerService;

impl DockerService {
    pub fn new() -> Self {
        Self
    }

    /// Check if Docker is available and running
    pub async fn is_docker_available(&self) -> Result<bool, String> {
        let output = Command::new("docker")
            .arg("version")
            .output()
            .await
            .map_err(|e| format!("Failed to execute docker command: {}", e))?;

        Ok(output.status.success())
    }

    /// List all running containers
    pub async fn list_containers(&self) -> Result<Vec<DockerContainer>, String> {
        let output = Command::new("docker")
            .args(&["ps", "-a", "--format", "json"])
            .output()
            .await
            .map_err(|e| format!("Failed to list containers: {}", e))?;

        if !output.status.success() {
            return Err(format!("Docker command failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = Vec::new();

        for line in stdout.lines() {
            if let Ok(container) = serde_json::from_str::<serde_json::Value>(line) {
                let container = DockerContainer {
                    id: container["ID"].as_str().unwrap_or("").to_string(),
                    name: container["Names"].as_str().unwrap_or("").to_string(),
                    image: container["Image"].as_str().unwrap_or("").to_string(),
                    status: container["Status"].as_str().unwrap_or("").to_string(),
                    ports: container["Ports"]
                        .as_str()
                        .unwrap_or("")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    created_at: container["CreatedAt"].as_str().unwrap_or("").to_string(),
                };
                containers.push(container);
            }
        }

        Ok(containers)
    }

    /// Build a Docker image
    pub async fn build_image(&self, context_path: &str, image_name: &str, dockerfile_path: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new("docker");
        cmd.arg("build");
        cmd.arg("-t").arg(image_name);
        
        if let Some(dockerfile) = dockerfile_path {
            cmd.arg("-f").arg(dockerfile);
        }
        
        cmd.arg(context_path);

        let output = cmd.output().await.map_err(|e| format!("Failed to build image: {}", e))?;

        if output.status.success() {
            Ok(format!("Image {} built successfully", image_name))
        } else {
            Err(format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Run a container
    pub async fn run_container(
        &self,
        image_name: &str,
        container_name: &str,
        ports: &[(u16, u16)], // (host_port, container_port)
        volumes: &[(String, String)], // (host_path, container_path)
        environment: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut cmd = Command::new("docker");
        cmd.arg("run");
        cmd.arg("-d"); // detached mode
        cmd.arg("--name").arg(container_name);

        // Port mappings
        for (host_port, container_port) in ports {
            cmd.arg("-p").arg(format!("{}:{}", host_port, container_port));
        }

        // Volume mappings
        for (host_path, container_path) in volumes {
            cmd.arg("-v").arg(format!("{}:{}", host_path, container_path));
        }

        // Environment variables
        for (key, value) in environment {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        cmd.arg(image_name);

        let output = cmd.output().await.map_err(|e| format!("Failed to run container: {}", e))?;

        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(container_id)
        } else {
            Err(format!("Failed to run container: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Stop a container
    pub async fn stop_container(&self, container_id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .arg("stop")
            .arg(container_id)
            .output()
            .await
            .map_err(|e| format!("Failed to stop container: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Failed to stop container: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .arg("rm")
            .arg("-f") // force removal
            .arg(container_id)
            .output()
            .await
            .map_err(|e| format!("Failed to remove container: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Failed to remove container: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Get container logs
    pub async fn get_container_logs(&self, container_id: &str, tail: Option<usize>) -> Result<Vec<String>, String> {
        let mut cmd = Command::new("docker");
        cmd.arg("logs");

        if let Some(tail_count) = tail {
            cmd.arg("--tail").arg(tail_count.to_string());
        }

        cmd.arg(container_id);

        let output = cmd.output().await.map_err(|e| format!("Failed to get logs: {}", e))?;

        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            Ok(logs.lines().map(|s| s.to_string()).collect())
        } else {
            Err(format!("Failed to get logs: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Get container status
    pub async fn get_container_status(&self, container_id: &str) -> Result<String, String> {
        let output = Command::new("docker")
            .args(&["inspect", "--format", "{{.State.Status}}", container_id])
            .output()
            .await
            .map_err(|e| format!("Failed to get container status: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!("Failed to get container status: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    /// Generate Dockerfile based on project type and SDK version
    pub fn generate_dockerfile(&self, project_type: &str, sdk_version: &str, project_path: &str) -> Result<String, String> {
        match project_type.to_lowercase().as_str() {
            "node" | "javascript" | "typescript" => {
                Ok(format!(
                    r#"FROM node:{} as base
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["npm", "start"]"#,
                    sdk_version
                ))
            }
            "rust" => {
                Ok(format!(
                    r#"FROM rust:{} as base
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD ["./target/release/{}"]"#,
                    sdk_version,
                    std::path::Path::new(project_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                ))
            }
            "python" => {
                Ok(format!(
                    r#"FROM python:{} as base
WORKDIR /app
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python", "main.py"]"#,
                    sdk_version
                ))
            }
            "go" => {
                Ok(format!(
                    r#"FROM golang:{} as base
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN go build -o main .
EXPOSE 8080
CMD ["./main"]"#,
                    sdk_version
                ))
            }
            _ => Err(format!("Unsupported project type: {}", project_type))
        }
    }
}
