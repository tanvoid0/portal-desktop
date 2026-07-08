use crate::process_ext::NoWindowExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeploymentType {
    Docker,
    Cli,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeploymentStatus {
    Building,
    Running,
    Stopped,
    Error,
    Unknown,
    Creating,
    Failed,
    Restarting,
    Removing,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerStatus {
    pub running: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ContainerResourceStats {
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: Vec<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_stats: Option<ContainerResourceStats>,
}

fn parse_percent(value: &str) -> f64 {
    value.trim().trim_end_matches('%').parse().unwrap_or(0.0)
}

fn parse_docker_bytes(value: &str) -> u64 {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" || trimmed == "0B" {
        return 0;
    }

    let num_end = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .count();
    let num_str = &trimmed[..num_end];
    let unit = trimmed[num_end..].trim();

    let num: f64 = num_str.parse().unwrap_or(0.0);
    let multiplier: f64 = match unit.to_uppercase().as_str() {
        "B" => 1.0,
        "KB" | "KIB" => 1024.0,
        "MB" | "MIB" => 1024.0 * 1024.0,
        "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (num * multiplier) as u64
}

fn parse_io_pair(value: &str) -> (u64, u64) {
    let parts: Vec<&str> = value.split('/').map(str::trim).collect();
    if parts.len() == 2 {
        (parse_docker_bytes(parts[0]), parse_docker_bytes(parts[1]))
    } else {
        (0, 0)
    }
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
    pub deployment_type: DeploymentType,
    pub status: DeploymentStatus,
    pub sdk_version: String,
    pub environment: EnvironmentConfig,
    // Docker-specific fields
    pub docker_image_name: Option<String>,
    pub container_id: Option<String>,
    pub exposed_port: Option<u16>,
    pub dockerfile_path: Option<String>,
    // CLI-specific fields
    pub command: Option<String>,
    pub working_directory: Option<String>,
    pub process_id: Option<u32>,
    // Shared fields
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
        Ok(self.get_docker_status().await?.running)
    }

    /// Get structured Docker daemon status
    pub async fn get_docker_status(&self) -> Result<DockerStatus, String> {
        let version_output = Command::new("docker")
            .no_window()
            .args(["version", "--format", "{{.Client.Version}}"])
            .output()
            .await;

        let installed = match &version_output {
            Ok(output) => output.status.success() || !output.stderr.is_empty(),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(DockerStatus {
                        running: false,
                        installed: false,
                        version: None,
                        message: Some(
                            "Docker is not installed. Install Docker Desktop to use containers."
                                .to_string(),
                        ),
                    });
                }
                return Err(format!("Failed to execute docker command: {}", e));
            }
        };

        let info_output = Command::new("docker")
            .no_window()
            .arg("info")
            .output()
            .await
            .map_err(|e| format!("Failed to execute docker command: {}", e))?;

        if info_output.status.success() {
            let version = version_output.ok().and_then(|output| {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            });

            return Ok(DockerStatus {
                running: true,
                installed,
                version,
                message: None,
            });
        }

        let stderr = String::from_utf8_lossy(&info_output.stderr)
            .trim()
            .to_string();
        let message = if stderr.contains("dockerDesktopLinuxEngine")
            || stderr.contains("Cannot connect to the Docker daemon")
            || stderr.contains("failed to connect to the docker API")
            || stderr.contains("The system cannot find the file specified")
        {
            "Docker is installed but not running. Start Docker Desktop to use containers."
                .to_string()
        } else if stderr.is_empty() {
            "Docker is not running.".to_string()
        } else {
            stderr
        };

        Ok(DockerStatus {
            running: false,
            installed,
            version: None,
            message: Some(message),
        })
    }

    /// Attempt to start the Docker daemon / Docker Desktop
    pub async fn start_docker(&self) -> Result<String, String> {
        if self.get_docker_status().await?.running {
            return Ok("Docker is already running".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            use std::path::Path;
            use std::process::Stdio;

            let candidates = [r"C:\Program Files\Docker\Docker\Docker Desktop.exe"];

            for path in candidates {
                if Path::new(path).exists() {
                    Command::new("cmd")
                        .no_window()
                        .args(["/C", "start", "", path])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|e| format!("Failed to start Docker Desktop: {}", e))?;

                    return Ok("Starting Docker Desktop. This may take a minute.".to_string());
                }
            }

            return Err(
                "Docker Desktop was not found. Install Docker Desktop or start it manually."
                    .to_string(),
            );
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("open")
                .args(["-a", "Docker"])
                .output()
                .await
                .map_err(|e| format!("Failed to start Docker Desktop: {}", e))?;

            if output.status.success() {
                return Ok("Starting Docker Desktop. This may take a minute.".to_string());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to start Docker Desktop: {}", stderr));
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let output = Command::new("systemctl")
                .args(["start", "docker"])
                .output()
                .await
                .map_err(|e| format!("Failed to start Docker service: {}", e))?;

            if output.status.success() {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                return Ok("Docker service started successfully".to_string());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to start Docker service. Ensure Docker is installed and you have permission to start system services. {}",
                stderr
            ));
        }
    }

    /// List all running containers
    pub async fn list_containers(&self) -> Result<Vec<DockerContainer>, String> {
        let output = Command::new("docker")
            .no_window()
            .args(&["ps", "-a", "--format", "json"])
            .output()
            .await
            .map_err(|e| format!("Failed to list containers: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Docker command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
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
                    resource_stats: None,
                };
                containers.push(container);
            }
        }

        // Attach live resource stats for running containers
        if let Ok(stats_map) = self.list_container_stats().await {
            for container in &mut containers {
                if let Some(stats) = stats_map.get(&container.id) {
                    container.resource_stats = Some(stats.clone());
                } else if let Some(stats) = stats_map.get(&container.name) {
                    container.resource_stats = Some(stats.clone());
                }
            }
        }

        Ok(containers)
    }

    /// Fetch live resource usage for all running containers via `docker stats`
    pub async fn list_container_stats(
        &self,
    ) -> Result<std::collections::HashMap<String, ContainerResourceStats>, String> {
        let output = Command::new("docker")
            .no_window()
            .args(["stats", "--no-stream", "--format", "{{json .}}"])
            .output()
            .await
            .map_err(|e| format!("Failed to get container stats: {}", e))?;

        if !output.status.success() {
            // No running containers or stats unavailable — return empty map
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No such container") || stderr.is_empty() {
                return Ok(std::collections::HashMap::new());
            }
            return Err(format!("Docker stats failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut stats_map = std::collections::HashMap::new();

        for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                let id = entry["ID"].as_str().unwrap_or("").to_string();
                let name = entry["Name"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches('/')
                    .to_string();

                let (mem_used, mem_limit) = parse_io_pair(
                    entry["MemUsage"].as_str().unwrap_or("0B / 0B"),
                );
                let (net_rx, net_tx) =
                    parse_io_pair(entry["NetIO"].as_str().unwrap_or("0B / 0B"));
                let (block_read, block_write) =
                    parse_io_pair(entry["BlockIO"].as_str().unwrap_or("0B / 0B"));

                let stats = ContainerResourceStats {
                    cpu_percent: parse_percent(entry["CPUPerc"].as_str().unwrap_or("0%")),
                    memory_bytes: mem_used,
                    memory_limit_bytes: mem_limit,
                    memory_percent: parse_percent(entry["MemPerc"].as_str().unwrap_or("0%")),
                    network_rx_bytes: net_rx,
                    network_tx_bytes: net_tx,
                    block_read_bytes: block_read,
                    block_write_bytes: block_write,
                };

                if !id.is_empty() {
                    stats_map.insert(id.clone(), stats.clone());
                }
                if !name.is_empty() {
                    stats_map.insert(name, stats);
                }
            }
        }

        Ok(stats_map)
    }

    /// Build a Docker image with progress tracking
    pub async fn build_image(
        &self,
        context_path: &str,
        image_name: &str,
        dockerfile_path: Option<&str>,
    ) -> Result<String, String> {
        let mut cmd = Command::new("docker");
        cmd.no_window();
        cmd.arg("build");
        cmd.arg("--progress=plain"); // Plain progress for easier parsing
        cmd.arg("-t").arg(image_name);

        if let Some(dockerfile) = dockerfile_path {
            cmd.arg("-f").arg(dockerfile);
        }

        cmd.arg(context_path);

        // Capture output for progress tracking
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to build image: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract image ID if available
            let image_id = stdout
                .lines()
                .rev()
                .find_map(|line| {
                    if line.contains("Successfully built") || line.contains("Successfully tagged") {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("Image {} built successfully", image_name));

            Ok(image_id)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!("Build failed: {}\n{}", stderr, stdout))
        }
    }

    /// Build a Docker image and stream output (for real-time progress)
    pub async fn build_image_stream(
        &self,
        context_path: &str,
        image_name: &str,
        dockerfile_path: Option<&str>,
    ) -> Result<tokio::process::Child, String> {
        let mut cmd = Command::new("docker");
        cmd.no_window();
        cmd.arg("build");
        cmd.arg("--progress=plain");
        cmd.arg("-t").arg(image_name);

        if let Some(dockerfile) = dockerfile_path {
            cmd.arg("-f").arg(dockerfile);
        }

        cmd.arg(context_path);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn docker build process: {}", e))
    }

    /// Run a container
    pub async fn run_container(
        &self,
        image_name: &str,
        container_name: &str,
        ports: &[(u16, u16)],         // (host_port, container_port)
        volumes: &[(String, String)], // (host_path, container_path)
        environment: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut cmd = Command::new("docker");
        cmd.no_window();
        cmd.arg("run");
        cmd.arg("-d"); // detached mode
        cmd.arg("--name").arg(container_name);

        // Port mappings
        for (host_port, container_port) in ports {
            cmd.arg("-p")
                .arg(format!("{}:{}", host_port, container_port));
        }

        // Volume mappings
        for (host_path, container_path) in volumes {
            cmd.arg("-v")
                .arg(format!("{}:{}", host_path, container_path));
        }

        // Environment variables
        for (key, value) in environment {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        cmd.arg(image_name);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to run container: {}", e))?;

        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(container_id)
        } else {
            Err(format!(
                "Failed to run container: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Start a container by ID or name
    pub async fn start_container(&self, container_id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .no_window()
            .arg("start")
            .arg(container_id)
            .output()
            .await
            .map_err(|e| format!("Failed to start container: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to start container: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Stop a container
    pub async fn stop_container(&self, container_id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .no_window()
            .arg("stop")
            .arg(container_id)
            .output()
            .await
            .map_err(|e| format!("Failed to stop container: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to stop container: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Remove a container
    pub async fn remove_container(&self, container_id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .no_window()
            .arg("rm")
            .arg("-f") // force removal
            .arg(container_id)
            .output()
            .await
            .map_err(|e| format!("Failed to remove container: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to remove container: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Get container logs
    pub async fn get_container_logs(
        &self,
        container_id: &str,
        tail: Option<usize>,
    ) -> Result<Vec<String>, String> {
        let mut cmd = Command::new("docker");
        cmd.no_window();
        cmd.arg("logs");

        if let Some(tail_count) = tail {
            cmd.arg("--tail").arg(tail_count.to_string());
        }

        cmd.arg(container_id);

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to get logs: {}", e))?;

        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            Ok(logs.lines().map(|s| s.to_string()).collect())
        } else {
            Err(format!(
                "Failed to get logs: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Get container status
    pub async fn get_container_status(&self, container_id: &str) -> Result<String, String> {
        let output = Command::new("docker")
            .no_window()
            .args(&["inspect", "--format", "{{.State.Status}}", container_id])
            .output()
            .await
            .map_err(|e| format!("Failed to get container status: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!(
                "Failed to get container status: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Generate Dockerfile based on project type and SDK version
    pub fn generate_dockerfile(
        &self,
        project_type: &str,
        sdk_version: &str,
        project_path: &str,
    ) -> Result<String, String> {
        match project_type.to_lowercase().as_str() {
            "node" | "javascript" | "typescript" => Ok(format!(
                r#"FROM node:{} as base
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["npm", "start"]"#,
                sdk_version
            )),
            "rust" => Ok(format!(
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
            )),
            "python" => Ok(format!(
                r#"FROM python:{} as base
WORKDIR /app
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python", "main.py"]"#,
                sdk_version
            )),
            "go" => Ok(format!(
                r#"FROM golang:{} as base
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN go build -o main .
EXPOSE 8080
CMD ["./main"]"#,
                sdk_version
            )),
            _ => Err(format!("Unsupported project type: {}", project_type)),
        }
    }
}
