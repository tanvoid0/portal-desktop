/**
 * Project-Level Version Isolation Module
 * 
 * Handles .portal-version files and automatic version switching
 */

pub mod version_file;
pub mod shell_integration;
pub mod environment_manager;


use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub sdk_type: String,
    pub version: String,
    pub project_path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEnvironment {
    pub project_path: PathBuf,
    pub versions: HashMap<String, String>, // SDK type -> version
    pub environment_variables: HashMap<String, String>,
    pub shell_hooks: Vec<String>,
}
