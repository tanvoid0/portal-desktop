/**
 * Version File Manager
 * 
 * Manages version files for project-level version isolation
 * Supports multiple version file formats: .portal-version, .nvmrc, .python-version, etc.
 */

use super::ProjectEnvironment;
use crate::domains::sdk::SDKError;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFileInfo {
    pub file_path: PathBuf,
    pub sdk_type: String,
    pub version: String,
    pub format: VersionFileFormat,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionFileFormat {
    PortalVersion,  // .portal-version (TOML)
    Nvmrc,          // .nvmrc (plain text)
    PythonVersion,  // .python-version (plain text)
    RubyVersion,    // .ruby-version (plain text)
    JavaVersion,    // .java-version (plain text)
    RustToolchain,  // rust-toolchain.toml (TOML)
    PhpVersion,     // .php-version (plain text)
    GoVersion,      // go.mod (Go module)
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionFile {
    versions: HashMap<String, String>,
    metadata: VersionMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionMetadata {
    created_at: String,
    updated_at: String,
    project_name: Option<String>,
}

pub struct VersionFileManager;

impl VersionFileManager {
    /// Create a .portal-version file for a project
    pub async fn create_version_file(
        project_path: &Path,
        sdk_type: &str,
        version: &str,
    ) -> Result<(), SDKError> {
        let version_file_path = project_path.join(".portal-version");
        
        let mut versions = HashMap::new();
        versions.insert(sdk_type.to_string(), version.to_string());
        
        let version_file = VersionFile {
            versions,
            metadata: VersionMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                project_name: project_path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string()),
            },
        };

        let content = toml::to_string(&version_file)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to serialize version file: {}", e)))?;

        fs::write(&version_file_path, content).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write version file: {}", e)))?;

        Ok(())
    }

    /// Read version file from a project
    pub async fn read_version_file(project_path: &Path) -> Result<ProjectEnvironment, SDKError> {
        let version_file_path = project_path.join(".portal-version");
        
        if !version_file_path.exists() {
            return Ok(ProjectEnvironment {
                project_path: project_path.to_path_buf(),
                versions: HashMap::new(),
                environment_variables: HashMap::new(),
                shell_hooks: Vec::new(),
            });
        }

        let content = fs::read_to_string(&version_file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read version file: {}", e)))?;

        let version_file: VersionFile = toml::from_str(&content)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse version file: {}", e)))?;

        Ok(ProjectEnvironment {
            project_path: project_path.to_path_buf(),
            versions: version_file.versions,
            environment_variables: HashMap::new(),
            shell_hooks: Vec::new(),
        })
    }

    /// Update version for a specific SDK in a project
    pub async fn update_version(
        project_path: &Path,
        sdk_type: &str,
        version: &str,
    ) -> Result<(), SDKError> {
        let version_file_path = project_path.join(".portal-version");
        
        let mut version_file = if version_file_path.exists() {
            let content = fs::read_to_string(&version_file_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read version file: {}", e)))?;
            
            toml::from_str(&content)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse version file: {}", e)))?
        } else {
            VersionFile {
                versions: HashMap::new(),
                metadata: VersionMetadata {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    project_name: project_path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_string()),
                },
            }
        };

        version_file.versions.insert(sdk_type.to_string(), version.to_string());
        version_file.metadata.updated_at = chrono::Utc::now().to_rfc3339();

        let content = toml::to_string(&version_file)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to serialize version file: {}", e)))?;

        fs::write(&version_file_path, content).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write version file: {}", e)))?;

        Ok(())
    }

    /// Remove version for a specific SDK from a project
    pub async fn remove_version(project_path: &Path, sdk_type: &str) -> Result<(), SDKError> {
        let version_file_path = project_path.join(".portal-version");
        
        if !version_file_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&version_file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read version file: {}", e)))?;

        let mut version_file: VersionFile = toml::from_str(&content)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse version file: {}", e)))?;

        version_file.versions.remove(sdk_type);
        version_file.metadata.updated_at = chrono::Utc::now().to_rfc3339();

        if version_file.versions.is_empty() {
            // Remove the file if no versions left
            fs::remove_file(&version_file_path).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to remove version file: {}", e)))?;
        } else {
            let content = toml::to_string(&version_file)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to serialize version file: {}", e)))?;

            fs::write(&version_file_path, content).await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write version file: {}", e)))?;
        }

        Ok(())
    }

    /// Check if a project has a version file
    pub async fn has_version_file(project_path: &Path) -> bool {
        project_path.join(".portal-version").exists()
    }

    /// Get all projects with version files in a directory
    pub async fn find_projects_with_versions(root_path: &Path) -> Result<Vec<PathBuf>, SDKError> {
        let mut projects = Vec::new();
        
        if let Ok(mut entries) = fs::read_dir(root_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_dir() && Self::has_version_file(&path).await {
                    projects.push(path);
                }
            }
        }

        Ok(projects)
    }

    /// Detect all version files in a project directory
    pub async fn detect_version_files(project_path: &Path) -> Result<Vec<VersionFileInfo>, SDKError> {
        let mut version_files = Vec::new();
        
        // Check for .portal-version (our custom format)
        let portal_version_path = project_path.join(".portal-version");
        if portal_version_path.exists() {
            if let Ok(version_info) = Self::parse_portal_version_file(&portal_version_path).await {
                version_files.extend(version_info);
            }
        }

        // Check for standard version files
        let standard_files = vec![
            (".nvmrc", "nodejs", VersionFileFormat::Nvmrc),
            (".python-version", "python", VersionFileFormat::PythonVersion),
            (".ruby-version", "ruby", VersionFileFormat::RubyVersion),
            (".java-version", "java", VersionFileFormat::JavaVersion),
            ("rust-toolchain.toml", "rust", VersionFileFormat::RustToolchain),
            (".php-version", "php", VersionFileFormat::PhpVersion),
            ("go.mod", "go", VersionFileFormat::GoVersion),
        ];

        for (filename, sdk_type, format) in standard_files {
            let file_path = project_path.join(filename);
            if file_path.exists() {
                if let Ok(version_info) = Self::parse_standard_version_file(&file_path, sdk_type, format).await {
                    version_files.push(version_info);
                }
            }
        }

        Ok(version_files)
    }

    /// Parse .portal-version file (TOML format)
    async fn parse_portal_version_file(file_path: &Path) -> Result<Vec<VersionFileInfo>, SDKError> {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read portal version file: {}", e)))?;

        let version_file: VersionFile = toml::from_str(&content)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse portal version file: {}", e)))?;

        let metadata = fs::metadata(file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to get file metadata: {}", e)))?;
        let last_modified = metadata.modified()
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to get modification time: {}", e)))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to convert time: {}", e)))?
            .as_secs();

        let mut version_files = Vec::new();
        for (sdk_type, version) in version_file.versions {
            version_files.push(VersionFileInfo {
                file_path: file_path.to_path_buf(),
                sdk_type,
                version,
                format: VersionFileFormat::PortalVersion,
                last_modified: chrono::DateTime::<chrono::Utc>::from_timestamp(last_modified as i64, 0)
                    .unwrap_or_else(|| chrono::Utc::now())
                    .to_rfc3339(),
            });
        }

        Ok(version_files)
    }

    /// Parse standard version files (plain text or TOML)
    async fn parse_standard_version_file(
        file_path: &Path,
        sdk_type: &str,
        format: VersionFileFormat,
    ) -> Result<VersionFileInfo, SDKError> {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read version file: {}", e)))?;

        let version = match format {
            VersionFileFormat::RustToolchain => {
                // Parse rust-toolchain.toml
                Self::parse_rust_toolchain(&content)?
            }
            VersionFileFormat::GoVersion => {
                // Parse go.mod for Go version
                Self::parse_go_version(&content)?
            }
            _ => {
                // Plain text files - just trim whitespace
                content.trim().to_string()
            }
        };

        let metadata = fs::metadata(file_path).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to get file metadata: {}", e)))?;
        let last_modified = metadata.modified()
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to get modification time: {}", e)))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to convert time: {}", e)))?
            .as_secs();

        Ok(VersionFileInfo {
            file_path: file_path.to_path_buf(),
            sdk_type: sdk_type.to_string(),
            version,
            format,
            last_modified: chrono::DateTime::<chrono::Utc>::from_timestamp(last_modified as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
        })
    }

    /// Parse rust-toolchain.toml file
    fn parse_rust_toolchain(content: &str) -> Result<String, SDKError> {
        #[derive(Deserialize)]
        struct RustToolchain {
            toolchain: Option<RustToolchainInfo>,
        }

        #[derive(Deserialize)]
        struct RustToolchainInfo {
            channel: Option<String>,
            version: Option<String>,
        }

        let toolchain: RustToolchain = toml::from_str(content)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to parse rust-toolchain.toml: {}", e)))?;

        if let Some(toolchain_info) = toolchain.toolchain {
            if let Some(version) = toolchain_info.version {
                Ok(version)
            } else if let Some(channel) = toolchain_info.channel {
                Ok(channel)
            } else {
                Ok("stable".to_string())
            }
        } else {
            Ok("stable".to_string())
        }
    }

    /// Parse go.mod file for Go version
    fn parse_go_version(content: &str) -> Result<String, SDKError> {
        for line in content.lines() {
            if line.starts_with("go ") {
                let version = line.strip_prefix("go ")
                    .unwrap_or("1.21")
                    .trim();
                return Ok(version.to_string());
            }
        }
        Ok("1.21".to_string()) // Default Go version
    }

    /// Create a standard version file for a specific SDK
    pub async fn create_standard_version_file(
        project_path: &Path,
        sdk_type: &str,
        version: &str,
    ) -> Result<(), SDKError> {
        let (filename, format) = match sdk_type {
            "nodejs" => (".nvmrc", VersionFileFormat::Nvmrc),
            "python" => (".python-version", VersionFileFormat::PythonVersion),
            "ruby" => (".ruby-version", VersionFileFormat::RubyVersion),
            "java" => (".java-version", VersionFileFormat::JavaVersion),
            "rust" => ("rust-toolchain.toml", VersionFileFormat::RustToolchain),
            "php" => (".php-version", VersionFileFormat::PhpVersion),
            "go" => ("go.mod", VersionFileFormat::GoVersion),
            _ => return Err(SDKError::ManagerNotFound(format!("Unsupported SDK type: {}", sdk_type))),
        };

        let file_path = project_path.join(filename);
        let content = match format {
            VersionFileFormat::RustToolchain => {
                format!(
                    "[toolchain]\nchannel = \"{}\"\n",
                    version
                )
            }
            VersionFileFormat::GoVersion => {
                format!(
                    "module {}\n\ngo {}\n",
                    project_path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("project"),
                    version
                )
            }
            _ => version.to_string(),
        };

        fs::write(&file_path, content).await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to write version file: {}", e)))?;

        Ok(())
    }

    /// Get the recommended version file format for an SDK
    pub fn get_recommended_format(sdk_type: &str) -> (String, VersionFileFormat) {
        match sdk_type {
            "nodejs" => (".nvmrc".to_string(), VersionFileFormat::Nvmrc),
            "python" => (".python-version".to_string(), VersionFileFormat::PythonVersion),
            "ruby" => (".ruby-version".to_string(), VersionFileFormat::RubyVersion),
            "java" => (".java-version".to_string(), VersionFileFormat::JavaVersion),
            "rust" => ("rust-toolchain.toml".to_string(), VersionFileFormat::RustToolchain),
            "php" => (".php-version".to_string(), VersionFileFormat::PhpVersion),
            "go" => ("go.mod".to_string(), VersionFileFormat::GoVersion),
            _ => (".portal-version".to_string(), VersionFileFormat::PortalVersion),
        }
    }
}
