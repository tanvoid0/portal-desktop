/**
 * Download Infrastructure Module
 * 
 * Provides version fetching and binary download capabilities for SDKs
 * using official sources for long-term sustainability.
 */

pub mod version_fetcher;
pub mod binary_downloader;
pub mod archive_handler;
pub mod sources;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub lts: bool,
    pub release_date: Option<String>,
    pub download_urls: HashMap<String, String>, // platform -> URL
    pub checksum: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub percentage: f64,
    pub speed: u64, // bytes per second
    pub eta: Option<u64>, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub progress: DownloadProgress,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallStage {
    FetchingVersions,
    Downloading,
    Extracting,
    Installing,
    Configuring,
    Complete,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub data_dir: Option<String>,
    pub config_file: Option<String>,
    pub environment: HashMap<String, String>,
}

