/**
 * Custom Directory Manager
 * 
 * Manages custom SDK installation directories for user-specified paths
 */

use crate::domains::sdk::SDKError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDirectory {
    pub id: String,
    pub path: String,
    pub sdk_type: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub last_scan: Option<String>,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKInstallation {
    pub version: String,
    pub path: String,
    pub binary_path: String,
    pub is_valid: bool,
    pub last_modified: String,
}

pub struct CustomDirectoryManager {
    directories: HashMap<String, CustomDirectory>,
    installations: HashMap<String, Vec<SDKInstallation>>,
}

impl CustomDirectoryManager {
    pub fn new() -> Self {
        Self {
            directories: HashMap::new(),
            installations: HashMap::new(),
        }
    }

    /// Add a custom directory for SDK installations
    pub async fn add_custom_directory(
        &mut self,
        path: String,
        sdk_type: String,
        name: String,
        description: Option<String>,
    ) -> Result<CustomDirectory, SDKError> {
        let id = Uuid::new_v4().to_string();
        let path_buf = PathBuf::from(&path);
        
        // Validate the directory exists
        if !path_buf.exists() && path_buf.is_dir() {
            let mut directory = CustomDirectory {
                id: id.clone(),
                path: path.clone(),
                sdk_type: sdk_type.clone(),
                name,
                description,
                created_at: chrono::Utc::now().to_rfc3339(),
                last_scan: None,
                is_valid: true,
                error_message: None,
            };

            // Scan for SDK installations
            let installations = self.scan_directory_for_sdks(&path, &sdk_type).await;
            self.installations.insert(id.clone(), installations);
            directory.last_scan = Some(chrono::Utc::now().to_rfc3339());

            self.directories.insert(id.clone(), directory.clone());
            Ok(directory)
        } else {
            Err(SDKError::ManagerNotFound(format!("Directory {} does not exist or is not a directory", path)))
        }
    }

    /// Remove a custom directory
    pub async fn remove_custom_directory(&mut self, directory_id: &str) -> Result<(), SDKError> {
        if self.directories.remove(directory_id).is_some() {
            self.installations.remove(directory_id);
            Ok(())
        } else {
            Err(SDKError::ManagerNotFound(format!("Directory {} not found", directory_id)))
        }
    }

    /// Get all custom directories for a specific SDK type
    pub async fn get_custom_directories(&self, sdk_type: &str) -> Vec<CustomDirectory> {
        self.directories
            .values()
            .filter(|dir| dir.sdk_type == sdk_type)
            .cloned()
            .collect()
    }

    /// Get all custom directories
    pub async fn get_all_custom_directories(&self) -> Vec<CustomDirectory> {
        self.directories.values().cloned().collect()
    }

    /// Scan a directory for SDK installations
    async fn scan_directory_for_sdks(&self, path: &str, sdk_type: &str) -> Vec<SDKInstallation> {
        let mut installations = Vec::new();
        let path_buf = PathBuf::from(path);

        if let Ok(mut entries) = fs::read_dir(&path_buf).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(entry_path) = entry.path().to_str() {
                    if let Some(installation) = self.detect_sdk_installation(entry_path, sdk_type).await {
                        installations.push(installation);
                    }
                }
            }
        }

        installations
    }

    /// Detect if a directory contains a valid SDK installation
    async fn detect_sdk_installation(&self, path: &str, sdk_type: &str) -> Option<SDKInstallation> {
        let path_buf = PathBuf::from(path);
        
        if !path_buf.is_dir() {
            return None;
        }

        // Check for common SDK binary names based on type
        let binary_names = match sdk_type {
            "nodejs" => vec!["node", "npm", "npx"],
            "python" => vec!["python", "python3", "pip", "pip3"],
            "java" => vec!["java", "javac", "jar"],
            "rust" => vec!["rustc", "cargo", "rustup"],
            "go" => vec!["go"],
            "php" => vec!["php", "php-fpm"],
            "ruby" => vec!["ruby", "gem", "irb"],
            _ => return None,
        };

        // Look for binaries in the directory
        for binary_name in &binary_names {
            let binary_path = path_buf.join(binary_name);
            if binary_path.exists() && binary_path.is_file() {
                // Check if it's executable
                if self.is_executable(&binary_path).await {
                    let version = self.get_sdk_version(&binary_path, sdk_type).await.unwrap_or_else(|| "unknown".to_string());
                    let metadata = fs::metadata(&path_buf).await.ok();
                    let last_modified = metadata
                        .and_then(|m| m.modified().ok())
                        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                    return Some(SDKInstallation {
                        version,
                        path: path.to_string(),
                        binary_path: binary_path.to_string_lossy().to_string(),
                        is_valid: true,
                        last_modified,
                    });
                }
            }
        }

        None
    }

    /// Check if a file is executable
    async fn is_executable(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path).await {
                let permissions = metadata.permissions();
                permissions.mode() & 0o111 != 0 // Check if any execute bit is set
            } else {
                false
            }
        }
        #[cfg(windows)]
        {
            // On Windows, check file extension
            if let Some(extension) = path.extension() {
                matches!(extension.to_str(), Some("exe") | Some("bat") | Some("cmd") | Some("com"))
            } else {
                false
            }
        }
    }

    /// Get SDK version from binary
    async fn get_sdk_version(&self, binary_path: &Path, sdk_type: &str) -> Option<String> {
        let version_args = match sdk_type {
            "nodejs" => vec!["--version"],
            "python" => vec!["--version"],
            "java" => vec!["-version"],
            "rust" => vec!["--version"],
            "go" => vec!["version"],
            "php" => vec!["--version"],
            "ruby" => vec!["--version"],
            _ => return None,
        };

        let output = std::process::Command::new(binary_path)
            .args(&version_args)
            .output()
            .ok()?;

        if output.status.success() {
            let version_string = String::from_utf8_lossy(&output.stdout);
            // Extract version number from output
            self.extract_version_from_output(&version_string)
        } else {
            None
        }
    }

    /// Extract version number from command output
    fn extract_version_from_output(&self, output: &str) -> Option<String> {
        // Simple regex-like extraction for common version patterns
        for line in output.lines() {
            if let Some(version) = self.find_version_in_line(line) {
                return Some(version);
            }
        }
        None
    }

    /// Find version number in a line of text
    fn find_version_in_line(&self, line: &str) -> Option<String> {
        // Look for patterns like "v1.2.3", "1.2.3", "version 1.2.3", etc.
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            if word.starts_with("v") && word.len() > 1 {
                if let Some(version) = self.parse_version(&word[1..]) {
                    return Some(version);
                }
            } else if let Some(version) = self.parse_version(word) {
                return Some(version);
            }
        }
        None
    }

    /// Parse a version string
    fn parse_version(&self, s: &str) -> Option<String> {
        // Check if string looks like a version (contains dots and numbers)
        if s.chars().any(|c| c.is_ascii_digit()) && s.contains('.') {
            Some(s.to_string())
        } else {
            None
        }
    }

    /// Rescan a directory for SDK installations
    pub async fn rescan_directory(&mut self, directory_id: &str) -> Result<Vec<SDKInstallation>, SDKError> {
        if let Some(directory) = self.directories.get(directory_id) {
            let installations = self.scan_directory_for_sdks(&directory.path, &directory.sdk_type).await;
            self.installations.insert(directory_id.to_string(), installations.clone());
            
            // Update last scan time
            if let Some(dir) = self.directories.get_mut(directory_id) {
                dir.last_scan = Some(chrono::Utc::now().to_rfc3339());
            }
            
            Ok(installations)
        } else {
            Err(SDKError::ManagerNotFound(format!("Directory {} not found", directory_id)))
        }
    }

    /// Get installations for a directory
    pub async fn get_directory_installations(&self, directory_id: &str) -> Result<Vec<SDKInstallation>, SDKError> {
        self.installations
            .get(directory_id)
            .cloned()
            .ok_or_else(|| SDKError::ManagerNotFound(format!("Directory {} not found", directory_id)))
    }

    /// Validate a custom directory
    pub async fn validate_directory(&mut self, directory_id: &str) -> Result<bool, SDKError> {
        if let Some(directory) = self.directories.get_mut(directory_id) {
            let path_buf = PathBuf::from(&directory.path);
            let is_valid = path_buf.exists() && path_buf.is_dir();
            
            directory.is_valid = is_valid;
            if !is_valid {
                directory.error_message = Some("Directory does not exist or is not accessible".to_string());
            } else {
                directory.error_message = None;
            }
            
            Ok(is_valid)
        } else {
            Err(SDKError::ManagerNotFound(format!("Directory {} not found", directory_id)))
        }
    }
}
