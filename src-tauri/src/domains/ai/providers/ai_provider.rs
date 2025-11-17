use serde::{Deserialize, Serialize};
use std::error::Error;

/// AI provider types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    Ollama,
    Gemini,
}

/// Configuration options for AI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Temperature for generation (0.0 to 2.0)
    pub temperature: Option<f64>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Model to use (provider-specific)
    pub model: Option<String>,
    /// Additional provider-specific options
    pub extra_options: Option<serde_json::Value>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(2048),
            timeout_ms: Some(60000),
            model: None,
            extra_options: None,
        }
    }
}

/// Result of AI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    /// Generated text content
    pub content: String,
    /// Model used for generation
    pub model: String,
    /// Tokens used (if available)
    pub tokens_used: Option<u32>,
    /// Generation time in milliseconds
    pub generation_time_ms: Option<u64>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub default_options: GenerationOptions,
    pub enabled: bool,
}

impl ProviderConfig {
    /// Create a default configuration for a provider type
    pub fn default_for_type(provider_type: ProviderType) -> Self {
        let (base_url, model) = match provider_type {
            ProviderType::Ollama => (
                Some("http://localhost:11434".to_string()),
                "llama3.2:3b".to_string(),
            ),
            ProviderType::Gemini => (
                Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
                "gemini-pro".to_string(),
            ),
        };

        Self {
            provider_type,
            base_url,
            api_key: None,
            model,
            default_options: GenerationOptions::default(),
            enabled: false,
        }
    }
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationStatus {
    /// Whether the provider is fully configured
    pub is_configured: bool,
    /// Missing required configuration fields
    pub missing_fields: Vec<String>,
    /// Optional configuration warnings
    pub warnings: Vec<String>,
}

/// Custom error types for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIError {
    /// Provider not available or not configured
    ProviderNotAvailable(String),
    /// Configuration incomplete - includes what's missing
    ConfigurationIncomplete(ConfigurationStatus),
    /// Network error
    NetworkError(String),
    /// Timeout error
    TimeoutError(String),
    /// Invalid response from provider
    InvalidResponse(String),
    /// Authentication error
    AuthenticationError(String),
    /// Rate limit error
    RateLimitError(String),
    /// Generic error
    GenericError(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::ProviderNotAvailable(msg) => write!(f, "Provider not available: {}", msg),
            AIError::ConfigurationIncomplete(status) => {
                write!(f, "Configuration incomplete. Missing: {}", status.missing_fields.join(", "))
            },
            AIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AIError::TimeoutError(msg) => write!(f, "Timeout: {}", msg),
            AIError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            AIError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            AIError::RateLimitError(msg) => write!(f, "Rate limit: {}", msg),
            AIError::GenericError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Error for AIError {}

/// Trait that all AI providers must implement
#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available and configured
    async fn is_available(&self) -> Result<bool, AIError>;

    /// Test the connection to the provider
    async fn test_connection(&self) -> Result<(), AIError>;

    /// Generate text from a prompt
    async fn generate(
        &self,
        prompt: &str,
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError>;

    /// Generate text with system message (for chat-like interactions)
    async fn generate_with_system(
        &self,
        system_message: &str,
        user_message: &str,
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError> {
        // Default implementation combines system and user messages
        let combined_prompt = format!("System: {}\n\nUser: {}", system_message, user_message);
        self.generate(&combined_prompt, options).await
    }

    /// Get available models for this provider
    async fn get_available_models(&self) -> Result<Vec<String>, AIError>;

    /// Get the default model for this provider
    fn default_model(&self) -> &str;

    /// Check configuration status - returns what's missing
    fn check_configuration(&self) -> ConfigurationStatus;

    /// Update configuration dynamically
    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError>;
}

