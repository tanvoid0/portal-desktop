/**
 * SDK Manager Domain Module
 */

pub mod commands;
pub mod entities;
pub mod services;
pub mod traits;
pub mod managers;
pub mod factory;
pub mod download;
pub mod project;
pub mod version_fetcher;
pub mod version_installer;
pub mod manager_detector;
pub mod ollama_manager;


#[derive(Debug, thiserror::Error)]
pub enum SDKError {
    #[error("SDK manager not found: {0}")]
    ManagerNotFound(String),
    #[error("SDK version not found: {0}")]
    VersionNotFound(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

impl From<SDKError> for String {
    fn from(error: SDKError) -> Self {
        error.to_string()
    }
}

