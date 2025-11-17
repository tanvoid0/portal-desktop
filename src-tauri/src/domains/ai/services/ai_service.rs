use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
    GeminiProvider, OllamaProvider,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Centralized AI service that manages multiple providers
pub struct AIService {
    providers: Arc<RwLock<HashMap<ProviderType, Arc<dyn AIProvider>>>>,
    default_provider: Arc<RwLock<Option<ProviderType>>>,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            default_provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a provider with the service
    pub async fn register_provider(&self, provider: Arc<dyn AIProvider>) {
        let provider_type = provider.provider_type();
        let mut providers = self.providers.write().await;
        providers.insert(provider_type, provider);
    }

    /// Register a provider from configuration
    pub async fn register_provider_from_config(&self, config: ProviderConfig) {
        let provider: Arc<dyn AIProvider> = match config.provider_type {
            ProviderType::Ollama => Arc::new(OllamaProvider::new(config)),
            ProviderType::Gemini => Arc::new(GeminiProvider::new(config)),
        };
        self.register_provider(provider).await;
    }

    /// Set the default provider
    pub async fn set_default_provider(&self, provider_type: ProviderType) -> Result<(), AIError> {
        let providers = self.providers.read().await;
        if !providers.contains_key(&provider_type) {
            return Err(AIError::ProviderNotAvailable(format!(
                "Provider {:?} is not registered",
                provider_type
            )));
        }
        drop(providers);

        let mut default = self.default_provider.write().await;
        *default = Some(provider_type);
        Ok(())
    }

    /// Get the default provider
    pub async fn get_default_provider_type(&self) -> Option<ProviderType> {
        self.default_provider.read().await.clone()
    }

    /// Get a provider by type
    pub async fn get_provider(&self, provider_type: Option<ProviderType>) -> Result<Arc<dyn AIProvider>, AIError> {
        let provider_type = match provider_type {
            Some(pt) => pt,
            None => {
                self.default_provider.read().await.clone()
                    .ok_or_else(|| AIError::ProviderNotAvailable("No default provider set".to_string()))?
            }
        };

        let providers = self.providers.read().await;
        let provider = providers
            .get(&provider_type)
            .ok_or_else(|| AIError::ProviderNotAvailable(format!("Provider {:?} not found", provider_type)))?;

        Ok(Arc::clone(provider))
    }

    /// Generate text using the specified provider (or default)
    pub async fn generate(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
    ) -> Result<GenerationResult, AIError> {
        // Validate configuration first
        let config_status = self.check_provider_configuration(provider_type.clone()).await?;
        if !config_status.is_configured {
            return Err(AIError::ConfigurationIncomplete(config_status));
        }

        let options = options.unwrap_or_default();
        let provider = self.get_provider(provider_type).await?;
        
        // Retry logic with exponential backoff
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match provider.generate(prompt, &options).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Don't retry configuration errors
                    if matches!(&e, AIError::ConfigurationIncomplete(_)) {
                        return Err(e);
                    }
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        let delay = std::time::Duration::from_millis(100 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AIError::GenericError("Generation failed after retries".to_string())))
    }

    /// Generate text with system message
    pub async fn generate_with_system(
        &self,
        system_message: &str,
        user_message: &str,
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
    ) -> Result<GenerationResult, AIError> {
        let options = options.unwrap_or_default();
        let provider = self.get_provider(provider_type).await?;
        
        provider.generate_with_system(system_message, user_message, &options).await
    }

    /// Get all registered provider types
    pub async fn get_registered_providers(&self) -> Vec<ProviderType> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Test connection to a provider
    pub async fn test_provider(&self, provider_type: ProviderType) -> Result<(), AIError> {
        let provider = self.get_provider(Some(provider_type)).await?;
        provider.test_connection().await
    }

    /// Get available models for a provider
    pub async fn get_available_models(&self, provider_type: Option<ProviderType>) -> Result<Vec<String>, AIError> {
        let provider = self.get_provider(provider_type).await?;
        provider.get_available_models().await
    }

    /// Check configuration status for a provider
    pub async fn check_provider_configuration(&self, provider_type: Option<ProviderType>) -> Result<ConfigurationStatus, AIError> {
        let provider = self.get_provider(provider_type).await?;
        Ok(provider.check_configuration())
    }

    /// Update provider configuration dynamically
    pub async fn update_provider_config(&self, config: ProviderConfig) -> Result<(), AIError> {
        let provider_type = config.provider_type.clone();
        
        // Create new provider with updated config
        let new_provider: Arc<dyn AIProvider> = match provider_type {
            ProviderType::Ollama => Arc::new(OllamaProvider::new(config)),
            ProviderType::Gemini => Arc::new(GeminiProvider::new(config)),
        };
        
        // Replace existing provider with new one
        let mut providers = self.providers.write().await;
        providers.insert(provider_type, new_provider);
        Ok(())
    }

    /// Validate configuration before attempting generation
    pub async fn validate_before_generate(
        &self,
        provider_type: Option<ProviderType>,
    ) -> Result<(), AIError> {
        let status = self.check_provider_configuration(provider_type).await?;
        
        if !status.is_configured {
            return Err(AIError::ConfigurationIncomplete(status));
        }
        
        Ok(())
    }

    /// Get available Ollama models (downloadable models from library)
    /// This is different from get_available_models which returns installed models
    pub async fn get_available_ollama_models(&self) -> Result<std::collections::HashMap<String, Vec<std::collections::HashMap<String, String>>>, AIError> {
        use crate::domains::sdk::ollama_manager::OllamaManager;
        
        let models = OllamaManager::get_available_models()
            .await
            .map_err(|e| AIError::GenericError(format!("Failed to get available Ollama models: {}", e)))?;
        
        // Convert from HashMap<String, Vec<serde_json::Value>> to HashMap<String, Vec<HashMap<String, String>>>
        let converted: std::collections::HashMap<String, Vec<std::collections::HashMap<String, String>>> = models
            .into_iter()
            .map(|(family, models)| {
                let converted_models: Vec<std::collections::HashMap<String, String>> = models
                    .into_iter()
                    .filter_map(|v| {
                        if let serde_json::Value::Object(obj) = v {
                            let mut map = std::collections::HashMap::new();
                            for (k, v) in obj {
                                if let Some(s) = v.as_str() {
                                    map.insert(k, s.to_string());
                                } else {
                                    map.insert(k, v.to_string());
                                }
                            }
                            Some(map)
                        } else {
                            None
                        }
                    })
                    .collect();
                (family, converted_models)
            })
            .collect();
        
        Ok(converted)
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}

