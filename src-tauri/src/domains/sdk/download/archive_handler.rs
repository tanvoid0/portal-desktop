/**
 * Archive Handler
 * 
 * Handles extraction of various archive formats (tar.gz, tar.xz, zip)
 */

use super::{InstallProgress, InstallStage};
use crate::domains::sdk::SDKError;
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct ArchiveHandler;

impl ArchiveHandler {
    /// Extract archive to destination directory
    pub async fn extract_archive(
        &self,
        archive_path: &PathBuf,
        destination: &PathBuf,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<(), SDKError> {
        // Create destination directory
        tokio::fs::create_dir_all(destination)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to create destination directory: {}", e)))?;

        // Determine archive type and extract
        let extension = archive_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "gz" | "tgz" => self.extract_tar_gz(archive_path, destination, progress_sender).await,
            "xz" => self.extract_tar_xz(archive_path, destination, progress_sender).await,
            "zip" => self.extract_zip(archive_path, destination, progress_sender).await,
            _ => Err(SDKError::ManagerNotFound(format!("Unsupported archive format: {}", extension))),
        }
    }

    /// Extract tar.gz archive
    async fn extract_tar_gz(
        &self,
        archive_path: &PathBuf,
        destination: &PathBuf,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<(), SDKError> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        progress_sender.send(InstallProgress {
            stage: InstallStage::Extracting,
            progress: super::DownloadProgress {
                total_bytes: 0,
                downloaded_bytes: 0,
                percentage: 0.0,
                speed: 0,
                eta: None,
            },
            message: "Extracting tar.gz archive...".to_string(),
        }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

        let file = std::fs::File::open(archive_path)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to open archive: {}", e)))?;

        let gz_decoder = GzDecoder::new(file);
        let mut archive = Archive::new(gz_decoder);

        archive
            .unpack(destination)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to extract archive: {}", e)))?;

        // Move child directory to parent if needed (common with SDK archives)
        self.move_child_dir_to_parent(destination).await?;

        Ok(())
    }

    /// Extract tar.xz archive
    async fn extract_tar_xz(
        &self,
        archive_path: &PathBuf,
        destination: &PathBuf,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<(), SDKError> {
        use xz2::read::XzDecoder;
        use tar::Archive;

        progress_sender.send(InstallProgress {
            stage: InstallStage::Extracting,
            progress: super::DownloadProgress {
                total_bytes: 0,
                downloaded_bytes: 0,
                percentage: 0.0,
                speed: 0,
                eta: None,
            },
            message: "Extracting tar.xz archive...".to_string(),
        }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

        let file = std::fs::File::open(archive_path)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to open archive: {}", e)))?;

        let xz_decoder = XzDecoder::new(file);
        let mut archive = Archive::new(xz_decoder);

        archive
            .unpack(destination)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to extract archive: {}", e)))?;

        // Move child directory to parent if needed
        self.move_child_dir_to_parent(destination).await?;

        Ok(())
    }

    /// Extract zip archive
    async fn extract_zip(
        &self,
        archive_path: &PathBuf,
        destination: &PathBuf,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<(), SDKError> {
        use zip::ZipArchive;

        progress_sender.send(InstallProgress {
            stage: InstallStage::Extracting,
            progress: super::DownloadProgress {
                total_bytes: 0,
                downloaded_bytes: 0,
                percentage: 0.0,
                speed: 0,
                eta: None,
            },
            message: "Extracting zip archive...".to_string(),
        }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

        let file = std::fs::File::open(archive_path)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to open archive: {}", e)))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to open zip archive: {}", e)))?;

        archive
            .extract(destination)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to extract zip archive: {}", e)))?;

        // Move child directory to parent if needed
        self.move_child_dir_to_parent(destination).await?;

        Ok(())
    }

    /// Move child directory contents to parent directory
    /// This is common with SDK archives that have a single root directory
    async fn move_child_dir_to_parent(&self, destination: &PathBuf) -> Result<(), SDKError> {
        let mut entries = tokio::fs::read_dir(destination)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read destination directory: {}", e)))?;

        let mut child_dirs = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read directory entry: {}", e)))? {
            let path = entry.path();
            if path.is_dir() {
                child_dirs.push(path);
            }
        }

        // If there's exactly one child directory, move its contents up
        if child_dirs.len() == 1 {
            let child_dir = &child_dirs[0];
            let temp_dir = destination.join("temp_move");
            
            // Move child directory to temp location
            tokio::fs::rename(child_dir, &temp_dir)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to move child directory: {}", e)))?;

            // Move contents from temp to destination
            let mut temp_entries = tokio::fs::read_dir(&temp_dir)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read temp directory: {}", e)))?;

            while let Some(entry) = temp_entries.next_entry().await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read temp entry: {}", e)))? {
                let entry_path = entry.path();
                let dest_path = destination.join(entry.file_name());
                
                tokio::fs::rename(&entry_path, &dest_path)
                    .await
                    .map_err(|e| SDKError::ManagerNotFound(format!("Failed to move entry: {}", e)))?;
            }

            // Remove temp directory
            tokio::fs::remove_dir(&temp_dir)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to remove temp directory: {}", e)))?;
        }

        Ok(())
    }

    /// Get the binary path for a specific SDK type and version
    pub fn get_binary_path(&self, sdk_type: &str, _version: &str, install_dir: &PathBuf) -> PathBuf {
        match sdk_type {
            "node" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("node.exe")
                } else {
                    install_dir.join("bin").join("node")
                }
            },
            "python" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("python.exe")
                } else {
                    install_dir.join("bin").join("python3")
                }
            },
            "java" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("bin").join("java.exe")
                } else {
                    install_dir.join("bin").join("java")
                }
            },
            "rust" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("bin").join("rustc.exe")
                } else {
                    install_dir.join("bin").join("rustc")
                }
            },
            "go" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("bin").join("go.exe")
                } else {
                    install_dir.join("bin").join("go")
                }
            },
            "php" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("php.exe")
                } else {
                    install_dir.join("bin").join("php")
                }
            },
            "ruby" => {
                if cfg!(target_os = "windows") {
                    install_dir.join("bin").join("ruby.exe")
                } else {
                    install_dir.join("bin").join("ruby")
                }
            },
            _ => install_dir.join("bin").join(sdk_type),
        }
    }

    /// Create symlinks for SDK activation
    pub async fn create_activation_links(
        &self,
        sdk_type: &str,
        version: &str,
        install_dir: &PathBuf,
        env_dir: &PathBuf,
    ) -> Result<(), SDKError> {
        // Create environment directory
        tokio::fs::create_dir_all(env_dir)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to create env directory: {}", e)))?;

        let binary_path = self.get_binary_path(sdk_type, version, install_dir);
        let link_path = env_dir.join(sdk_type);

        // Remove existing link if it exists
        if link_path.exists() {
            tokio::fs::remove_file(&link_path)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to remove existing link: {}", e)))?;
        }

        // Create symlink (or copy on Windows)
        if cfg!(target_os = "windows") {
            // On Windows, copy the binary instead of symlinking
            tokio::fs::copy(&binary_path, &link_path)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to copy binary: {}", e)))?;
        } else {
            // On Unix systems, create a symlink
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&binary_path, &link_path)
                    .map_err(|e| SDKError::ManagerNotFound(format!("Failed to create symlink: {}", e)))?;
            }
        }

        Ok(())
    }
}
