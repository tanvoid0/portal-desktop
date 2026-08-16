/// Unified error type for the application
///
/// This provides a consistent error handling approach across all domains
/// and replaces the various string-based error returns.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    #[error("Migration error: {0}")]
    Migration(String),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Path error: {0}")]
    Path(String),

    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    // Network/HTTP errors
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Network error: {0}")]
    Network(String),

    // Kubernetes errors
    #[error("Kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),

    #[error("Kubernetes config error: {0}")]
    KubernetesConfig(String),

    // Docker/Container errors
    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    // Process errors
    #[error("Process error: {0}")]
    Process(String),

    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    // SDK/Language errors
    #[error("SDK error: {0}")]
    Sdk(String),

    #[error("Language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    // Project errors
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Project error: {0}")]
    Project(String),

    // Deployment errors
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),

    #[error("Deployment error: {0}")]
    Deployment(String),

    // Pipeline errors
    #[error("Pipeline not found: {0}")]
    PipelineNotFound(String),

    #[error("Pipeline execution error: {0}")]
    PipelineExecution(String),

    #[error("Block execution error: {0}")]
    BlockExecution(String),

    // AI errors
    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[error("AI provider not found: {0}")]
    AiProviderNotFound(String),

    #[error("AI request error: {0}")]
    AiRequest(String),

    // Authentication/Authorization errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Settings error: {0}")]
    Settings(String),

    // Generic errors
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("{0}")]
    Other(String),
}

// Implement From<AppError> for String to maintain compatibility with existing code
// that returns Result<T, String>
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

// Implement From<String> for AppError to ease migration
impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Other(error)
    }
}

impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::Other(error.to_string())
    }
}

// Type alias for Results using AppError
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::ProjectNotFound("test-project".to_string());
        assert_eq!(err.to_string(), "Project not found: test-project");
    }

    #[test]
    fn test_error_conversion() {
        let err = AppError::Internal("test error".to_string());
        let err_string: String = err.into();
        assert_eq!(err_string, "Internal error: test error");
    }

    #[test]
    fn test_string_to_error() {
        let err: AppError = "custom error".into();
        assert_eq!(err.to_string(), "custom error");
    }
}
