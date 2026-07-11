/**
 * SDK Configuration Service
 *
 * Processes SDK configurations (all categories) and returns formatted data for frontend display.
 * Frontend receives ready-to-display data without needing to know configuration details.
 */
use crate::domains::sdk::configs::{get_all_sdk_configs, get_sdk_config, SDKCategory, SDKConfig};
use crate::domains::sdk::commands::manager_commands::get_sdk_manager_workflow_support;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSDKConfig {
    // Basic metadata
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub category: String, // SDKCategory as string

    // Tab configuration (only enabled tabs)
    pub tabs: Vec<ProcessedTabConfig>,

    // Version sources (as strings for frontend)
    pub supported_sources: Vec<String>,
    pub default_source: String,

    // SDK Managers (with installation status)
    pub sdk_managers: Vec<ProcessedSDKManager>,

    // Package Managers (with installation status)
    pub package_managers: Vec<ProcessedPackageManager>,

    // Detection info
    pub detection: ProcessedDetectionMethod,

    // Category-specific features
    pub category_features: Option<serde_json::Value>,

    // Environment variables
    pub environment_variables: Option<serde_json::Value>,

    // Service configuration
    pub service_config: Option<serde_json::Value>,

    // SDK installation status (whether the SDK itself is installed, not just managers)
    pub sdk_installed: bool,
    pub sdk_version: Option<String>,

    // Service status (for database/server SDKs)
    pub service_running: Option<bool>,
    pub service_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedTabConfig {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSDKManager {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub supports_installation: bool,
    pub supports_version_switching: bool,
    pub install_command: Option<String>,
    pub website: Option<String>,
    pub install_available: bool,
    pub install_unavailable_reason: Option<String>,
    pub uninstall_available: bool,
    pub uninstall_unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedPackageManager {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_command: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDetectionMethod {
    pub binary_names: Vec<String>,
    pub version_command: Option<String>,
    pub path_patterns: Vec<String>,
    pub version_file_patterns: Vec<String>,
}

pub struct SDKConfigService;

impl SDKConfigService {
    /// Get processed configuration for an SDK
    pub async fn get_sdk_config(sdk_id: &str) -> Result<Option<ProcessedSDKConfig>, String> {
        let config = match get_sdk_config(sdk_id) {
            Some(c) => c,
            None => return Ok(None),
        };

        Ok(Some(Self::process_config(config).await))
    }

    /// Get all processed SDK configurations
    pub async fn get_all_sdk_configs() -> Result<Vec<ProcessedSDKConfig>, String> {
        let configs = get_all_sdk_configs();
        let mut processed = Vec::new();

        for config in configs {
            processed.push(Self::process_config(config).await);
        }

        Ok(processed)
    }

    /// Get SDKs by category
    pub async fn get_sdks_by_category(
        category: SDKCategory,
    ) -> Result<Vec<ProcessedSDKConfig>, String> {
        let configs = crate::domains::sdk::configs::get_sdks_by_category(category);
        let mut processed = Vec::new();

        for config in configs {
            processed.push(Self::process_config(config).await);
        }

        Ok(processed)
    }

    /// Get all SDK managers from all SDK configs
    /// Returns a deduplicated list of all SDK managers across all SDKs
    pub async fn get_all_sdk_managers() -> Result<Vec<ProcessedSDKManager>, String> {
        let configs = get_all_sdk_configs();
        let mut managers_map: std::collections::HashMap<String, ProcessedSDKManager> =
            std::collections::HashMap::new();

        // Collect all managers from all configs
        for config in configs {
            for m in config.sdk_managers {
                // TEMPORARY: SDKMAN is disabled right now because candidate/version support
                // (Java, Maven, Liquibase, etc.) requires a more careful PTY + parsing + UI
                // model. Keep it out of the sidebar/manager lists until we revisit.
                if m.id == "sdkman" {
                    continue;
                }

                // Use manager ID as key to deduplicate
                if !managers_map.contains_key(&m.id) {
                    let (installed, version) = Self::check_manager_status(&m.binary).await;
                    let workflow_support = get_sdk_manager_workflow_support(&m.id, installed);
                    managers_map.insert(
                        m.id.clone(),
                        ProcessedSDKManager {
                            id: m.id,
                            name: m.name,
                            display_name: m.display_name,
                            installed,
                            version,
                            supports_installation: m.supports_installation,
                            supports_version_switching: m.supports_version_switching,
                            install_command: m.install_command,
                            website: m.website,
                            install_available: workflow_support.install_available,
                            install_unavailable_reason: workflow_support.install_unavailable_reason,
                            uninstall_available: workflow_support.uninstall_available,
                            uninstall_unavailable_reason: workflow_support
                                .uninstall_unavailable_reason,
                        },
                    );
                }
            }
        }

        // Convert to vector and sort by display name
        let mut managers: Vec<ProcessedSDKManager> = managers_map.into_values().collect();
        managers.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        Ok(managers)
    }

    /// Process a raw config into frontend-ready format
    async fn process_config(config: SDKConfig) -> ProcessedSDKConfig {
        // Filter to only enabled tabs
        let tabs: Vec<ProcessedTabConfig> = config
            .tabs
            .into_iter()
            .filter(|t| t.enabled)
            .map(|t| ProcessedTabConfig {
                id: t.id,
                label: t.label,
            })
            .collect();

        // Convert version sources to strings
        let supported_sources: Vec<String> = config
            .supported_sources
            .into_iter()
            .map(|s| match s {
                crate::domains::sdk::configs::types::VersionSource::Static => "static".to_string(),
                crate::domains::sdk::configs::types::VersionSource::SdkManager => {
                    "sdk_manager".to_string()
                }
                crate::domains::sdk::configs::types::VersionSource::System => "system".to_string(),
                crate::domains::sdk::configs::types::VersionSource::Custom => "custom".to_string(),
            })
            .collect();

        let default_source = match config.default_source {
            Some(crate::domains::sdk::configs::types::VersionSource::Static) => {
                "static".to_string()
            }
            Some(crate::domains::sdk::configs::types::VersionSource::SdkManager) => {
                "sdk_manager".to_string()
            }
            Some(crate::domains::sdk::configs::types::VersionSource::System) => {
                "system".to_string()
            }
            Some(crate::domains::sdk::configs::types::VersionSource::Custom) => {
                "custom".to_string()
            }
            None => "system".to_string(),
        };

        // Process SDK managers (check installation status)
        let mut sdk_managers = Vec::new();
        for m in config.sdk_managers {
            let (installed, version) = Self::check_manager_status(&m.binary).await;
            let workflow_support = get_sdk_manager_workflow_support(&m.id, installed);
            sdk_managers.push(ProcessedSDKManager {
                id: m.id,
                name: m.name,
                display_name: m.display_name,
                installed,
                version,
                supports_installation: m.supports_installation,
                supports_version_switching: m.supports_version_switching,
                install_command: m.install_command,
                website: m.website,
                install_available: workflow_support.install_available,
                install_unavailable_reason: workflow_support.install_unavailable_reason,
                uninstall_available: workflow_support.uninstall_available,
                uninstall_unavailable_reason: workflow_support.uninstall_unavailable_reason,
            });
        }

        // Process package managers (check installation status)
        let mut package_managers = Vec::new();
        for p in config.package_managers {
            let (installed, version) = Self::check_manager_status(&p.binary).await;
            package_managers.push(ProcessedPackageManager {
                id: p.id,
                name: p.name,
                display_name: p.display_name,
                installed,
                version,
                install_command: p.install_command,
                website: p.website,
            });
        }

        // Check if the SDK itself is installed (not just managers) - do this before moving detection
        let (sdk_installed, sdk_version) = Self::check_sdk_installation(&config.detection).await;

        // Check service status for SDKs that declare a service configuration.
        // This keeps UI behavior data-driven: languages/framework SDKs typically have no `service_config`.
        let (service_running, service_port) = if config.service_config.is_some() {
            Self::check_service_status(&config.service_config).await
        } else {
            (None, None)
        };

        // Extract detection data
        let detection = ProcessedDetectionMethod {
            binary_names: config.detection.binary_names,
            version_command: config.detection.version_command,
            path_patterns: config.detection.path_patterns,
            version_file_patterns: config.detection.version_file_patterns,
        };

        ProcessedSDKConfig {
            id: config.id,
            name: config.name,
            display_name: config.display_name,
            description: config.description,
            icon: config.icon,
            category: config.category.as_str().to_string(),
            tabs,
            supported_sources,
            default_source,
            sdk_managers,
            package_managers,
            detection,
            category_features: config.category_features,
            environment_variables: config.environment_variables,
            service_config: config.service_config,
            sdk_installed,
            sdk_version,
            service_running,
            service_port,
        }
    }

    /// Check if a manager/binary is installed and get its version
    /// For known SDK managers (nvm, sdkman, etc.), use their specific detection methods
    /// For others, fall back to simple binary check
    async fn check_manager_status(binary: &str) -> (bool, Option<String>) {
        use crate::process_ext::NoWindowExt;
        use std::process::Command;

        // Map of known manager binaries to their factory names
        // Note: SDKMAN uses 'sdk' as the binary name, but the factory uses 'sdk' as the manager name
        let manager_map: std::collections::HashMap<&str, &str> = [
            ("nvm", "nvm"),
            ("sdk", "sdk"),    // SDKMAN uses 'sdk' command, factory name is also 'sdk'
            ("sdkman", "sdk"), // Also check for 'sdkman' as alias
        ]
        .iter()
        .cloned()
        .collect();

        // Check if this is a known manager that needs special handling
        if let Some(manager_name) = manager_map.get(binary) {
            let factory = crate::domains::sdk::factory::SDKManagerFactory::new();
            if let Some(manager) = factory.get_manager(manager_name) {
                match manager.is_installed().await {
                    Ok(true) => {
                        // Get version if available
                        match manager.get_manager_version().await {
                            Ok(version) => return (true, Some(version)),
                            Err(_) => return (true, None),
                        }
                    }
                    Ok(false) => return (false, None),
                    Err(_) => {
                        // Fall through to binary check
                    }
                }
            }
        }

        // Fallback: Try to run version command
        if let Ok(output) = Command::new(binary).no_window().arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Also check stderr (some commands output version there)
                if version.is_empty() {
                    let stderr_version = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if !stderr_version.is_empty() {
                        return (true, Some(stderr_version));
                    }
                } else {
                    return (true, Some(version));
                }
            }
        }

        // Try to check if binary exists in PATH
        if let Ok(output) = Command::new("which").arg(binary).output() {
            if output.status.success() {
                return (true, None);
            }
        }

        // For conda, also check common installation paths
        if binary == "conda" {
            let home = std::env::var("HOME").ok();
            if let Some(home) = home {
                let conda_paths = [
                    format!("{}/anaconda3/bin/conda", home),
                    format!("{}/miniconda3/bin/conda", home),
                    format!("{}/.conda/bin/conda", home),
                ];

                for path in &conda_paths {
                    if std::path::Path::new(path).exists() {
                        // Try to get version
                        if let Ok(output) = Command::new(path).no_window().arg("--version").output()
                        {
                            if output.status.success() {
                                let version =
                                    String::from_utf8_lossy(&output.stdout).trim().to_string();
                                return (true, Some(version));
                            }
                        }
                        return (true, None);
                    }
                }
            }
        }

        (false, None)
    }

    /// Check if the SDK itself is installed (using detection methods)
    async fn check_sdk_installation(
        detection: &crate::domains::sdk::configs::types::DetectionMethod,
    ) -> (bool, Option<String>) {
        use crate::process_ext::NoWindowExt;
        use std::process::Command;

        // First, try the version command if available
        if let Some(ref version_cmd) = detection.version_command {
            // Parse command and args (simple split for now)
            let parts: Vec<&str> = version_cmd.split_whitespace().collect();
            if !parts.is_empty() {
                let binary = parts[0];
                let args: Vec<&str> = parts[1..].to_vec();

                if let Ok(output) = Command::new(binary).no_window().args(&args).output() {
                    if output.status.success() {
                        // Some commands (like docker --version) output to stderr instead of stdout
                        let raw_output = if !output.stdout.is_empty() {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        } else if !output.stderr.is_empty() {
                            String::from_utf8_lossy(&output.stderr).trim().to_string()
                        } else {
                            "installed".to_string()
                        };

                        // Parse version for specific SDKs that output verbose information
                        let version = Self::parse_version_output(binary, &raw_output);
                        return (true, Some(version));
                    }
                }
            }
        }

        // Fallback: check if any of the binary names exist
        for binary_name in &detection.binary_names {
            if let Ok(output) = Command::new("which").arg(binary_name).output() {
                if output.status.success() {
                    // Try to get version
                    if let Ok(version_output) = Command::new(binary_name)
                        .no_window()
                        .arg("--version")
                        .output()
                    {
                        if version_output.status.success() {
                            // Some commands output to stderr instead of stdout
                            let raw_output = if !version_output.stdout.is_empty() {
                                String::from_utf8_lossy(&version_output.stdout)
                                    .trim()
                                    .to_string()
                            } else if !version_output.stderr.is_empty() {
                                String::from_utf8_lossy(&version_output.stderr)
                                    .trim()
                                    .to_string()
                            } else {
                                "installed".to_string()
                            };
                            let version = Self::parse_version_output(binary_name, &raw_output);
                            return (true, Some(version));
                        }
                    }
                    return (true, None);
                }
            }
        }

        (false, None)
    }

    /// Check if a database/service is running by checking if its port is listening
    async fn check_service_status(
        service_config: &Option<serde_json::Value>,
    ) -> (Option<bool>, Option<u16>) {
        // Extract port from service_config
        let port = if let Some(ref config) = service_config {
            config
                .get("port")
                .and_then(|v| v.as_u64())
                .map(|p| p as u16)
        } else {
            None
        };

        if let Some(port_num) = port {
            // Check if port is listening (in use)
            let is_running = Self::is_port_listening(port_num).await;
            (Some(is_running), Some(port_num))
        } else {
            (None, None)
        }
    }

    /// Check if a port is listening (in use) - opposite of is_port_available
    async fn is_port_listening(port: u16) -> bool {
        use std::net::SocketAddr;
        use std::str::FromStr;
        use std::time::Duration;

        // Try to connect to the port - if successful, it means something is listening
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(socket_addr) = SocketAddr::from_str(&addr) {
            // Try to connect with a short timeout
            match tokio::time::timeout(
                Duration::from_millis(100),
                tokio::net::TcpStream::connect(&socket_addr),
            )
            .await
            {
                Ok(Ok(_)) => return true, // Connection successful = port is listening
                _ => return false,        // Connection failed = port is not listening
            }
        }

        false
    }

    /// Parse version output to extract clean version string
    /// Handles SDKs that output verbose multi-line information
    fn parse_version_output(binary: &str, raw_output: &str) -> String {
        match binary {
            "java" => {
                // Java outputs to stderr with format like:
                // openjdk version "17.0.16" 2025-07-15 LTS
                // OpenJDK Runtime Environment Corretto-17.0.16.8.1 (build 17.0.16+8-LTS)
                // OpenJDK 64-Bit Server VM Corretto-17.0.16.8.1 (build 17.0.16+8-LTS, mixed mode, sharing)
                // Extract just the version number from the first line
                if let Some(first_line) = raw_output.lines().next() {
                    // Look for pattern: openjdk version "X.Y.Z"
                    if let Some(start) = first_line.find("version \"") {
                        let start = start + 9; // length of "version \""
                        if let Some(end) = first_line[start..].find('"') {
                            return first_line[start..start + end].to_string();
                        }
                    }
                    // Fallback: try to extract version number pattern
                    if let Some(captures) =
                        regex::Regex::new(r#"version\s+"?([0-9]+\.[0-9]+\.[0-9]+)"#)
                            .ok()
                            .and_then(|re| re.captures(first_line))
                    {
                        if let Some(version) = captures.get(1) {
                            return version.as_str().to_string();
                        }
                    }
                }
                // If parsing fails, return first line trimmed
                raw_output
                    .lines()
                    .next()
                    .unwrap_or(raw_output)
                    .trim()
                    .to_string()
            }
            _ => {
                // For other commands, return first line or full output if single line
                let lines: Vec<&str> = raw_output.lines().collect();
                if lines.len() > 1 {
                    lines[0].trim().to_string()
                } else {
                    raw_output.trim().to_string()
                }
            }
        }
    }
}
