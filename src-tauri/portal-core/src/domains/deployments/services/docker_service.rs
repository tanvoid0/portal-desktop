use crate::process_ext::NoWindowExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpListener;
use tokio::process::Command;

/// Shared Docker network the proxy and every deployed app join, so Traefik can
/// reach apps over the network without publishing a host port.
pub const PROXY_NETWORK: &str = "portal";
/// The local reverse-proxy container name.
pub const PROXY_CONTAINER: &str = "portal-proxy";
const PROXY_IMAGE: &str = "traefik:v3.5";

/// Is `port` bindable right now? A successful bind on 0.0.0.0 (the interface
/// Docker publishes to by default) means nothing else — host process or an
/// existing container's published port — is holding it. The listener drops
/// immediately, so this only probes; it never keeps the port.
pub fn is_port_free(port: u16) -> bool {
    TcpListener::bind(("0.0.0.0", port)).is_ok()
}

/// First free host port at or above `preferred`, scanning a bounded window.
/// Falls back to an OS-assigned ephemeral port so we never return a busy one.
pub fn find_free_port(preferred: u16) -> u16 {
    for candidate in preferred..=preferred.saturating_add(200) {
        if is_port_free(candidate) {
            return candidate;
        }
    }
    // Everything in the window was taken — let the OS hand us a free port.
    TcpListener::bind(("0.0.0.0", 0))
        .ok()
        .and_then(|l| l.local_addr().ok())
        .map(|a| a.port())
        .unwrap_or(preferred)
}

/// Node package manager a project uses, chosen by which lockfile is present.
/// Defaults to npm when none is found.
fn node_package_manager(project_path: &str) -> &'static str {
    let dir = std::path::Path::new(project_path);
    if dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if dir.join("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose_project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose_service: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub networks: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerImageSummary {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size_bytes: u64,
    pub created_at: String,
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
    /// Port the app listens on inside the container (detected, not guessed).
    /// `#[serde(default)]` so rows written before this field load as `None`.
    #[serde(default)]
    pub container_port: Option<u16>,
    /// Hostname the local proxy routes to this app, e.g. `my-app.localhost`.
    #[serde(default)]
    pub host: Option<String>,
    /// The URL the user should open — includes the proxy's port when it is not
    /// 80, or a direct `localhost:<port>` when routing was skipped. Resolved at
    /// start time, since it depends on which ports were actually free.
    #[serde(default)]
    pub url: Option<String>,
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
                    compose_project: None,
                    compose_service: None,
                    networks: Vec::new(),
                    labels: HashMap::new(),
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

        self.enrich_containers_from_inspect(&mut containers).await?;

        Ok(containers)
    }

    /// Enrich container list with compose labels and network info from `docker inspect`
    async fn enrich_containers_from_inspect(
        &self,
        containers: &mut [DockerContainer],
    ) -> Result<(), String> {
        if containers.is_empty() {
            return Ok(());
        }

        let mut args = vec!["inspect".to_string()];
        args.extend(containers.iter().map(|c| c.id.clone()));

        let output = Command::new("docker")
            .no_window()
            .args(&args)
            .output()
            .await
            .map_err(|e| format!("Failed to inspect containers: {}", e))?;

        if !output.status.success() {
            // Non-fatal: grouping heuristics still work without inspect data
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let inspections: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();

        for (index, inspect) in inspections.iter().enumerate() {
            if index >= containers.len() {
                break;
            }

            let container = &mut containers[index];
            let labels_value = inspect
                .pointer("/Config/Labels")
                .and_then(|v| v.as_object());

            if let Some(labels_obj) = labels_value {
                for (key, value) in labels_obj {
                    if let Some(val) = value.as_str() {
                        container.labels.insert(key.clone(), val.to_string());
                    }
                }

                container.compose_project = container
                    .labels
                    .get("com.docker.compose.project")
                    .cloned()
                    .filter(|s| !s.is_empty());
                container.compose_service = container
                    .labels
                    .get("com.docker.compose.service")
                    .cloned()
                    .filter(|s| !s.is_empty());
            }

            if let Some(networks_obj) = inspect
                .pointer("/NetworkSettings/Networks")
                .and_then(|v| v.as_object())
            {
                container.networks = networks_obj.keys().cloned().collect();
            }
        }

        Ok(())
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

                let (mem_used, mem_limit) =
                    parse_io_pair(entry["MemUsage"].as_str().unwrap_or("0B / 0B"));
                let (net_rx, net_tx) = parse_io_pair(entry["NetIO"].as_str().unwrap_or("0B / 0B"));
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

    /// Ensure the local reverse proxy is running and return the host port it
    /// serves HTTP on. Idempotent: creates the shared network and the Traefik
    /// container only when missing, reuses them otherwise.
    ///
    /// Port handling is deliberate — if `:80` is taken we do NOT collide; we
    /// bind the next free port and return it so callers fold it into the URL.
    pub async fn ensure_proxy(&self) -> Result<u16, String> {
        // Network: idempotent create. A non-zero exit is almost always "already
        // exists", which is success for our purposes.
        let _ = Command::new("docker")
            .no_window()
            .args(["network", "create", PROXY_NETWORK])
            .output()
            .await;

        // Is the proxy container already there, and running?
        let state = Command::new("docker")
            .no_window()
            .args(["inspect", "-f", "{{.State.Running}}", PROXY_CONTAINER])
            .output()
            .await
            .map_err(|e| format!("Failed to inspect proxy: {}", e))?;

        if state.status.success() {
            let running = String::from_utf8_lossy(&state.stdout).trim() == "true";
            if running {
                return self.proxy_port().await;
            }
            // Exists but stopped. Its published port was fixed at creation; if
            // that port is now taken, `docker start` fails. Rather than wedge
            // every future deploy on a dead proxy, remove the stale container
            // and fall through to recreate it on a currently-free port.
            let started = Command::new("docker")
                .no_window()
                .args(["start", PROXY_CONTAINER])
                .output()
                .await;
            if started.map(|o| o.status.success()).unwrap_or(false) {
                return self.proxy_port().await;
            }
            let _ = Command::new("docker")
                .no_window()
                .args(["rm", "-f", PROXY_CONTAINER])
                .output()
                .await;
        }

        // Not present — create it on a free port, preferring 80.
        let port = find_free_port(80);
        let output = Command::new("docker")
            .no_window()
            .args([
                "run",
                "-d",
                "--name",
                PROXY_CONTAINER,
                "--network",
                PROXY_NETWORK,
                "--restart",
                "unless-stopped",
                "-p",
                &format!("{}:80", port),
                "-v",
                "/var/run/docker.sock:/var/run/docker.sock:ro",
                PROXY_IMAGE,
                "--providers.docker=true",
                "--providers.docker.exposedbydefault=false",
                "--entrypoints.web.address=:80",
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to start proxy: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Failed to start local proxy: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(port)
    }

    /// Host port the running proxy publishes for container port 80.
    async fn proxy_port(&self) -> Result<u16, String> {
        let output = Command::new("docker")
            .no_window()
            .args(["port", PROXY_CONTAINER, "80/tcp"])
            .output()
            .await
            .map_err(|e| format!("Failed to read proxy port: {}", e))?;

        // Output like `0.0.0.0:8088` (possibly several lines) — take the port
        // after the last colon of the first line.
        let text = String::from_utf8_lossy(&output.stdout);
        text.lines()
            .next()
            .and_then(|l| l.rsplit(':').next())
            .and_then(|p| p.trim().parse::<u16>().ok())
            .ok_or_else(|| "Could not determine proxy port".to_string())
    }

    /// Is the proxy's Docker provider actually working? The container can be up
    /// while its provider is dead — on Docker Desktop for Windows the mounted
    /// socket yields an unusable API, so Traefik registers no routes and every
    /// request 404s. We detect that from recent proxy logs rather than claim
    /// hostname routing works when it does not.
    pub async fn proxy_provider_healthy(&self) -> bool {
        // Give the provider a moment to make (and log) its first API call.
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let out = Command::new("docker")
            .no_window()
            .args(["logs", "--tail", "25", PROXY_CONTAINER])
            .output()
            .await;
        match out {
            Ok(o) => {
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                !(text.contains("Provider error")
                    || text.contains("Failed to retrieve information of the docker client"))
            }
            Err(_) => false,
        }
    }

    /// Run a container
    #[allow(clippy::too_many_arguments)]
    pub async fn run_container(
        &self,
        image_name: &str,
        container_name: &str,
        ports: &[(u16, u16)],         // (host_port, container_port)
        volumes: &[(String, String)], // (host_path, container_path)
        environment: &HashMap<String, String>,
        network: Option<&str>,
        labels: &[(String, String)],
    ) -> Result<String, String> {
        let mut cmd = Command::new("docker");
        cmd.no_window();
        cmd.arg("run");
        cmd.arg("-d"); // detached mode
        cmd.arg("--name").arg(container_name);

        if let Some(net) = network {
            cmd.arg("--network").arg(net);
        }

        for (key, value) in labels {
            cmd.arg("--label").arg(format!("{}={}", key, value));
        }

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

    /// Does a container with this id or name still exist (in any state)?
    pub async fn container_exists(&self, container_id: &str) -> bool {
        Command::new("docker")
            .no_window()
            .args(["inspect", container_id])
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
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

    /// List local Docker images
    pub async fn list_images(&self) -> Result<Vec<DockerImageSummary>, String> {
        let output = Command::new("docker")
            .no_window()
            .args(["images", "--format", "{{json .}}"])
            .output()
            .await
            .map_err(|e| format!("Failed to list images: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Docker images command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut images = Vec::new();

        for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                let repository = entry["Repository"].as_str().unwrap_or("").to_string();
                let tag = entry["Tag"].as_str().unwrap_or("").to_string();

                if repository == "<none>" && tag == "<none>" {
                    continue;
                }

                images.push(DockerImageSummary {
                    id: entry["ID"].as_str().unwrap_or("").to_string(),
                    repository,
                    tag,
                    size_bytes: parse_docker_bytes(entry["Size"].as_str().unwrap_or("0B")),
                    created_at: entry["CreatedAt"].as_str().unwrap_or("").to_string(),
                });
            }
        }

        Ok(images)
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

    /// Generate a Dockerfile for a project with no Dockerfile of its own.
    ///
    /// `port` is the detected listen port (see `detect`); the `EXPOSE` here must
    /// match it so routing hits the right port. Stacks without a solid template
    /// return an error asking the user to supply a Dockerfile or `portal.yaml`.
    pub fn generate_dockerfile(
        &self,
        project_type: &str,
        sdk_version: &str,
        project_path: &str,
        port: u16,
    ) -> Result<String, String> {
        match project_type.to_lowercase().as_str() {
            // Next.js: must `build` before `start`. Uses the project's own
            // package manager (pnpm/yarn/npm, by lockfile) so the lockfile is
            // honoured and the `build`/`start` scripts resolve.
            "nextjs" | "next" => {
                let (install, build, start) = match node_package_manager(project_path) {
                    // `--config.dangerouslyAllowAllBuilds=true`: pnpm 10+ refuses
                    // to run dependency build scripts (sharp, esbuild, …) without
                    // interactive approval and EXITS 1 (`ERR_PNPM_IGNORED_BUILDS`),
                    // which killed every one-click build. In a from-scratch image
                    // build there is no human to run `pnpm approve-builds`, so we
                    // opt in to running them — the pre-pnpm-10 default.
                    "pnpm" => (
                        "RUN corepack enable\nCOPY package.json pnpm-lock.yaml* ./\nRUN pnpm install --frozen-lockfile --config.dangerouslyAllowAllBuilds=true || pnpm install --config.dangerouslyAllowAllBuilds=true",
                        "RUN pnpm run build",
                        r#"CMD ["pnpm", "run", "start"]"#,
                    ),
                    "yarn" => (
                        "RUN corepack enable\nCOPY package.json yarn.lock* ./\nRUN yarn install --frozen-lockfile || yarn install",
                        "RUN yarn build",
                        r#"CMD ["yarn", "start"]"#,
                    ),
                    _ => (
                        "COPY package*.json ./\nRUN npm install",
                        "RUN npm run build",
                        r#"CMD ["npm", "run", "start"]"#,
                    ),
                };
                Ok(format!(
                    "FROM node:22-alpine\nWORKDIR /app\n{install}\nCOPY . .\n{build}\nEXPOSE {port}\n{start}"
                ))
            }
            "node" | "javascript" | "typescript" => Ok(format!(
                r#"FROM node:{sdk} AS base
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE {port}
CMD ["npm", "start"]"#,
                sdk = sdk_version
            )),
            "rust" => Ok(format!(
                r#"FROM rust:{sdk} AS base
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release
EXPOSE {port}
CMD ["./target/release/{bin}"]"#,
                sdk = sdk_version,
                bin = std::path::Path::new(project_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            )),
            "python" => Ok(format!(
                r#"FROM python:{sdk} AS base
WORKDIR /app
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY . .
EXPOSE {port}
CMD ["python", "main.py"]"#,
                sdk = sdk_version
            )),
            "go" => Ok(format!(
                r#"FROM golang:{sdk} AS base
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN go build -o main .
EXPOSE {port}
CMD ["./main"]"#,
                sdk = sdk_version
            )),
            "java" => Ok(format!(
                r#"FROM maven:3-eclipse-temurin-21 AS build
WORKDIR /app
COPY . .
RUN mvn -q -DskipTests package
FROM eclipse-temurin:21-jre
WORKDIR /app
COPY --from=build /app/target/*.jar app.jar
EXPOSE {port}
CMD ["java", "-jar", "app.jar"]"#
            )),
            "php" => Ok(format!(
                r#"FROM php:8-apache
COPY . /var/www/html/
EXPOSE {port}"#
            )),
            "ruby" => Ok(format!(
                r#"FROM ruby:{sdk} AS base
WORKDIR /app
COPY Gemfile* ./
RUN bundle install
COPY . .
EXPOSE {port}
CMD ["ruby", "app.rb"]"#,
                sdk = sdk_version
            )),
            other => Err(format!(
                "No Dockerfile template for '{other}'. Add a Dockerfile or a portal.yaml to this project."
            )),
        }
    }

    /// Default `.dockerignore` for a generated build. Without it, `COPY . .`
    /// drags the host's `node_modules` (built for the host OS, not the image),
    /// a stale `.next`/`dist`, the whole `.git`, and secrets into the image —
    /// 2.6 GB of context and a broken native-deps layer for a typical Node app.
    /// Keeping it generic (one file, all stacks) is enough: every entry is
    /// either build output or VCS/IDE cruft that no image needs.
    pub fn generate_dockerignore(&self) -> String {
        [
            "# Generated by Portal. Keeps host build artifacts out of the image.",
            "**/node_modules",
            ".next",
            "dist",
            "build",
            "out",
            "target",
            ".git",
            ".gitignore",
            ".svelte-kit",
            "*.log",
            ".DS_Store",
            ".idea",
            ".vscode",
            "Dockerfile",
            ".dockerignore",
            "",
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod itests {
    //! Live Docker integration checks — ignored by default (need a running
    //! Docker daemon). Run explicitly:
    //!   cargo test -p portal-core --lib itests -- --ignored --nocapture
    use super::*;
    use crate::domains::deployments::services::detect;

    /// HTTP status the proxy returns for `host`, hit on loopback with an
    /// explicit Host header so the check never depends on `*.localhost` DNS.
    async fn status_via_proxy(proxy_port: u16, host: &str) -> String {
        let out = Command::new("curl")
            .no_window()
            .args([
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "--max-time",
                "10",
                "-H",
                &format!("Host: {host}"),
                &format!("http://127.0.0.1:{proxy_port}/"),
            ])
            .output()
            .await;
        out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    /// Proves the pipeline — ensure_proxy + run_container with Traefik labels +
    /// routing — end to end with a known-good image, no build required.
    #[tokio::test]
    #[ignore]
    async fn verify_pipeline_routes_to_container() {
        let svc = DockerService::new();
        assert!(
            svc.is_docker_available().await.unwrap_or(false),
            "Docker must be running"
        );

        let proxy_port = svc.ensure_proxy().await.expect("proxy up");
        println!("[pipeline] proxy serving on host port {proxy_port}");

        let host = "pipeline-check.localhost";
        let cname = "portal-pipeline-check";
        let _ = svc.remove_container(cname).await;

        let labels = vec![
            ("traefik.enable".to_string(), "true".to_string()),
            (
                format!("traefik.http.routers.{cname}.rule"),
                format!("Host(`{host}`)"),
            ),
            (
                format!("traefik.http.services.{cname}.loadbalancer.server.port"),
                "80".to_string(),
            ),
        ];
        svc.run_container(
            "traefik/whoami",
            cname,
            &[],
            &[],
            &HashMap::new(),
            Some(PROXY_NETWORK),
            &labels,
        )
        .await
        .expect("run whoami");

        // Give Traefik a moment to see the new container.
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let code = status_via_proxy(proxy_port, host).await;
        println!("[pipeline] GET http://{host} -> {code}");
        let logs = svc.get_container_logs(cname, Some(5)).await.unwrap_or_default();
        println!("[pipeline] container logs:\n{}", logs.join("\n"));

        let _ = svc.remove_container(cname).await;
        assert_eq!(code, "200", "proxy did not route to the container");
    }

    /// Best-effort real deploy of the project at PORTAL_DEPLOY_PATH through the
    /// actual detect + generate + build + run + log + API path. Does not assert
    /// on build success (a real app may need env/DB); it prints every stage so
    /// the outcome is visible. The generated Dockerfile is written to a temp
    /// file, never into the target repo.
    #[tokio::test]
    #[ignore]
    async fn verify_real_project_deploy() {
        let path = std::env::var("PORTAL_DEPLOY_PATH")
            .unwrap_or_else(|_| r"D:\projects\production\portal\portal_web".to_string());
        let name = "portal-web-check";
        let svc = DockerService::new();
        assert!(
            svc.is_docker_available().await.unwrap_or(false),
            "Docker must be running"
        );

        let det = detect::detect(&path, "node");
        println!(
            "[deploy] detect: stack={} container_port={} dockerfile={:?} host={:?}",
            det.stack, det.container_port, det.dockerfile, det.host
        );

        let content = match det.dockerfile.clone() {
            Some(p) => std::fs::read_to_string(&p).unwrap_or_default(),
            None => svc
                .generate_dockerfile(&det.stack, "22", &path, det.container_port)
                .expect("generate dockerfile"),
        };
        println!("[deploy] generated Dockerfile:\n{content}\n");
        let df = std::env::temp_dir().join("portal-web-check.Dockerfile");
        std::fs::write(&df, &content).unwrap();

        let image = format!("{name}:check");
        println!("[deploy] building image (this can take several minutes)...");
        match svc
            .build_image(&path, &image, Some(df.to_str().unwrap()))
            .await
        {
            Ok(msg) => println!("[deploy] build OK: {msg}"),
            Err(e) => {
                println!("[deploy] build FAILED:\n{e}");
                return; // nothing to run; the failure is the finding
            }
        }

        // Published-port path (the reliable default; hostname routing is a
        // bonus that is dead on Docker Desktop for Windows).
        let cname = format!("{}-check", detect::sanitize_name(name));
        let _ = svc.remove_container(&cname).await;
        let host_port = find_free_port(det.container_port);
        svc.run_container(
            &image,
            &cname,
            &[(host_port, det.container_port)],
            &[],
            &HashMap::new(),
            None,
            &[],
        )
        .await
        .expect("run container");

        // Give the app time to boot (Next.js cold start can be slow).
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        let logs = svc.get_container_logs(&cname, Some(60)).await.unwrap_or_default();
        println!("[deploy] runtime logs:\n{}", logs.join("\n"));

        let out = Command::new("curl")
            .no_window()
            .args([
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "--max-time",
                "10",
                &format!("http://localhost:{host_port}/"),
            ])
            .output()
            .await;
        let code = out
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        println!("[deploy] GET http://localhost:{host_port} -> {code}");
        println!("[deploy] container={cname} port={host_port}");

        // Keep the container up for a manual/visual check when asked.
        if std::env::var("PORTAL_KEEP").is_err() {
            let _ = svc.remove_container(&cname).await;
        }
    }
}
