use crate::domains::ai::message::ChatMessage;
use crate::domains::ai::providers::{
    AIError, AIProvider, AgentPlatformProvider, ConfigurationStatus, GenerationOptions,
    GenerationResult, ProviderConfig, ProviderType,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Centralized AI service — all inference goes through agent-platform.
pub struct AIService {
    provider: Arc<RwLock<Option<Arc<dyn AIProvider>>>>,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            provider: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn register_provider(&self, provider: Arc<dyn AIProvider>) {
        *self.provider.write().await = Some(provider);
    }

    pub async fn register_provider_from_config(&self, config: ProviderConfig) {
        if config.provider_type != ProviderType::AgentPlatform {
            return;
        }
        self.register_provider(Arc::new(AgentPlatformProvider::new(config)))
            .await;
    }

    pub async fn set_default_provider(&self, _provider_type: ProviderType) -> Result<(), AIError> {
        self.get_provider().await?;
        Ok(())
    }

    pub async fn get_default_provider_type(&self) -> Option<ProviderType> {
        Some(ProviderType::AgentPlatform)
    }

    pub async fn get_provider(&self) -> Result<Arc<dyn AIProvider>, AIError> {
        let guard = self.provider.read().await;
        guard
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                AIError::ProviderNotAvailable(
                    "Agent Platform is not registered. Configure it under Settings → AI."
                        .to_string(),
                )
            })
    }

    /// Legacy optional type — always resolves to the registered agent-platform provider.
    pub async fn get_provider_typed(
        &self,
        _provider_type: Option<ProviderType>,
    ) -> Result<Arc<dyn AIProvider>, AIError> {
        self.get_provider().await
    }

    pub async fn generate(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
    ) -> Result<GenerationResult, AIError> {
        let config_status = self
            .check_provider_configuration(provider_type.clone())
            .await?;
        if !config_status.is_configured {
            return Err(AIError::ConfigurationIncomplete(config_status));
        }

        let options = options.unwrap_or_default();
        let provider = self.get_provider_typed(provider_type).await?;

        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match provider.generate(prompt, &options).await {
                Ok(result) => return Ok(result),
                Err(e) => {
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

        Err(last_error.unwrap_or_else(|| {
            AIError::GenericError("Generation failed after retries".to_string())
        }))
    }

    pub async fn generate_with_system(
        &self,
        system_message: &str,
        user_message: &str,
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
    ) -> Result<GenerationResult, AIError> {
        let options = options.unwrap_or_default();
        let provider = self.get_provider_typed(provider_type).await?;
        provider
            .generate_with_system(system_message, user_message, &options)
            .await
    }

    pub async fn get_registered_providers(&self) -> Vec<ProviderType> {
        if self.provider.read().await.is_some() {
            vec![ProviderType::AgentPlatform]
        } else {
            vec![]
        }
    }

    pub async fn test_provider(&self, _provider_type: ProviderType) -> Result<(), AIError> {
        let provider = self.get_provider().await?;
        provider.test_connection().await
    }

    pub async fn get_available_models(
        &self,
        provider_type: Option<ProviderType>,
    ) -> Result<Vec<String>, AIError> {
        let provider = self.get_provider_typed(provider_type).await?;
        provider.get_available_models().await
    }

    pub async fn check_provider_configuration(
        &self,
        _provider_type: Option<ProviderType>,
    ) -> Result<ConfigurationStatus, AIError> {
        let provider = self.get_provider().await?;
        Ok(provider.check_configuration())
    }

    pub async fn update_provider_config(&self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::AgentPlatform {
            return Err(AIError::GenericError(
                "Only AgentPlatform is supported".to_string(),
            ));
        }
        self.register_provider(Arc::new(AgentPlatformProvider::new(config)))
            .await;
        Ok(())
    }

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

    pub async fn generate_chat(
        &self,
        messages: &[ChatMessage],
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
    ) -> Result<GenerationResult, AIError> {
        let config_status = self
            .check_provider_configuration(provider_type.clone())
            .await?;
        if !config_status.is_configured {
            return Err(AIError::ConfigurationIncomplete(config_status));
        }
        let options = options.unwrap_or_default();
        let provider = self.get_provider_typed(provider_type).await?;
        provider.generate_chat(messages, &options).await
    }

    pub async fn generate_chat_stream(
        &self,
        messages: &[ChatMessage],
        options: Option<GenerationOptions>,
        provider_type: Option<ProviderType>,
        on_chunk: Box<dyn FnMut(String) -> Result<(), AIError> + Send>,
    ) -> Result<GenerationResult, AIError> {
        let config_status = self
            .check_provider_configuration(provider_type.clone())
            .await?;
        if !config_status.is_configured {
            return Err(AIError::ConfigurationIncomplete(config_status));
        }
        let options = options.unwrap_or_default();
        let provider = self.get_provider_typed(provider_type).await?;
        provider
            .generate_chat_stream(messages, &options, on_chunk)
            .await
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}
