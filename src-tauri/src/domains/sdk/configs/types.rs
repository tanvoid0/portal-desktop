/**
 * SDK Configuration Types
 */
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionSource {
    Static,
    SdkManager,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentScope {
    Global,
    Project,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKManagerConfig {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub binary: String,
    pub version_command: Option<String>,
    pub supports_installation: bool,
    pub supports_version_switching: bool,
    pub install_command: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub binary: String,
    pub version_command: Option<String>,
    pub install_command: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionMethod {
    pub binary_names: Vec<String>,
    pub version_command: Option<String>,
    pub path_patterns: Vec<String>,
    pub version_file_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SDKCategory {
    Language,
    Database,
    AI,
    Server,
    Container,
    Package,
    Manager,
    Tool,
}

impl SDKCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SDKCategory::Language => "language",
            SDKCategory::Database => "database",
            SDKCategory::AI => "ai",
            SDKCategory::Server => "server",
            SDKCategory::Container => "container",
            SDKCategory::Package => "package",
            SDKCategory::Manager => "manager",
            SDKCategory::Tool => "tool",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "language" => Some(SDKCategory::Language),
            "database" => Some(SDKCategory::Database),
            "ai" => Some(SDKCategory::AI),
            "server" | "web" => Some(SDKCategory::Server),
            "container" => Some(SDKCategory::Container),
            "package" => Some(SDKCategory::Package),
            "manager" => Some(SDKCategory::Manager),
            "tool" => Some(SDKCategory::Tool),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKTabConfig {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKConfig {
    // Basic metadata
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub category: SDKCategory,

    // Tab configuration
    pub tabs: Vec<SDKTabConfig>,

    // Version sources (only for SDKs that support versioning)
    pub supported_sources: Vec<VersionSource>,
    pub default_source: Option<VersionSource>,

    // SDK Managers (for language SDKs, this is version managers like nvm, pyenv)
    pub sdk_managers: Vec<SDKManagerConfig>,

    // Package Managers (linked package managers for this SDK)
    pub package_managers: Vec<PackageManagerConfig>,

    // Detection
    pub detection: DetectionMethod,

    // Category-specific features (JSON for flexibility)
    pub category_features: Option<serde_json::Value>,

    // Environment variables
    pub environment_variables: Option<serde_json::Value>,

    // Service configuration (for SDKs that run as services)
    pub service_config: Option<serde_json::Value>,
}
