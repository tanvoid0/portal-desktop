/**
 * Service Management Module
 * 
 * Handles service lifecycle management for databases and web servers
 */

pub mod service_manager;
pub mod process_tracker;
pub mod port_manager;
pub mod sdk_service;
pub mod terminal_integration;
pub mod navigation_service;
pub mod custom_directory_manager;

pub use process_tracker::ProcessTracker;
pub use port_manager::PortManager;
pub use sdk_service::SDKService;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub data_dir: Option<String>,
    pub config_file: Option<String>,
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub config: ServiceConfig,
    pub start_time: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLog {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}