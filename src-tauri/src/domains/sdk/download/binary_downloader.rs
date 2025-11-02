/**
 * Binary Downloader
 * 
 * Handles downloading SDK binaries with progress tracking
 */

use super::{DownloadProgress, InstallProgress, InstallStage, VersionInfo};
use crate::domains::sdk::SDKError;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub struct BinaryDownloader {
    client: Client,
    cache_dir: PathBuf,
}

impl BinaryDownloader {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            cache_dir,
        }
    }

    /// Download a version with progress tracking
    pub async fn download_version(
        &self,
        version_info: &VersionInfo,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<PathBuf, SDKError> {
        let (platform, arch) = self.get_platform_info();
        let platform_arch = format!("{}-{}", platform, arch);
        
        let download_url = version_info
            .download_urls
            .get(&platform_arch)
            .ok_or_else(|| SDKError::ManagerNotFound(format!("No download URL for platform {}", platform_arch)))?;

        let filename = self.extract_filename_from_url(download_url);
        let cache_path = self.cache_dir.join(&filename);

        // Check if already downloaded
        if cache_path.exists() {
            progress_sender.send(InstallProgress {
                stage: InstallStage::Complete,
                progress: DownloadProgress {
                    total_bytes: 0,
                    downloaded_bytes: 0,
                    percentage: 100.0,
                    speed: 0,
                    eta: None,
                },
                message: "Already downloaded".to_string(),
            }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;
            
            return Ok(cache_path);
        }

        // Send initial progress
        progress_sender.send(InstallProgress {
            stage: InstallStage::Downloading,
            progress: DownloadProgress {
                total_bytes: 0,
                downloaded_bytes: 0,
                percentage: 0.0,
                speed: 0,
                eta: None,
            },
            message: format!("Starting download of {}", version_info.version),
        }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to create cache directory: {}", e)))?;

        // Download with progress tracking
        self.download_with_progress(download_url, &cache_path, progress_sender).await?;

        Ok(cache_path)
    }

    /// Download file with progress tracking
    async fn download_with_progress(
        &self,
        url: &str,
        output_path: &PathBuf,
        progress_sender: mpsc::UnboundedSender<InstallProgress>,
    ) -> Result<(), SDKError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to start download: {}", e)))?;

        let total_size = response
            .content_length()
            .unwrap_or(0);

        let mut file = File::create(output_path)
            .await
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to create file: {}", e)))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_update = std::time::Instant::now();
        let mut last_downloaded = 0u64;

        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| SDKError::ManagerNotFound(format!("Download error: {}", e)))?;
            
            file.write_all(&chunk)
                .await
                .map_err(|e| SDKError::ManagerNotFound(format!("Write error: {}", e)))?;

            downloaded += chunk.len() as u64;

            // Update progress every 100ms or every 1MB
            let now = std::time::Instant::now();
            if now.duration_since(last_update).as_millis() > 100 || downloaded - last_downloaded > 1024 * 1024 {
                let percentage = if total_size > 0 {
                    (downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                };

                let speed = if now.duration_since(last_update).as_secs() > 0 {
                    (downloaded - last_downloaded) / now.duration_since(last_update).as_secs()
                } else {
                    0
                };

                let eta = if speed > 0 && total_size > downloaded {
                    Some((total_size - downloaded) / speed)
                } else {
                    None
                };

                let progress = DownloadProgress {
                    total_bytes: total_size,
                    downloaded_bytes: downloaded,
                    percentage,
                    speed,
                    eta,
                };

                progress_sender.send(InstallProgress {
                    stage: InstallStage::Downloading,
                    progress,
                    message: format!("Downloading... {:.1}%", percentage),
                }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

                last_update = now;
                last_downloaded = downloaded;
            }
        }

        // Send completion
        progress_sender.send(InstallProgress {
            stage: InstallStage::Extracting,
            progress: DownloadProgress {
                total_bytes: total_size,
                downloaded_bytes: downloaded,
                percentage: 100.0,
                speed: 0,
                eta: None,
            },
            message: "Download complete, extracting...".to_string(),
        }).map_err(|_| SDKError::ManagerNotFound("Failed to send progress".to_string()))?;

        Ok(())
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> String {
        url.split('/')
            .last()
            .unwrap_or("download")
            .to_string()
    }

    /// Get current platform and architecture
    fn get_platform_info(&self) -> (String, String) {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        
        let platform = match os {
            "macos" => "darwin",
            "windows" => "win32",
            "linux" => "linux",
            _ => "unknown",
        };
        
        let architecture = match arch {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            _ => "unknown",
        };
        
        (platform.to_string(), architecture.to_string())
    }

    /// Verify download checksum if available
    pub async fn verify_checksum(&self, file_path: &PathBuf, expected_checksum: &str) -> Result<bool, SDKError> {
        use sha2::{Sha256, Digest};
        use std::io::Read;

        let mut file = std::fs::File::open(file_path)
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to open file for checksum: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to read file: {}", e)))?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        let hex_hash = format!("{:x}", hash);
        
        Ok(hex_hash == expected_checksum)
    }
}
