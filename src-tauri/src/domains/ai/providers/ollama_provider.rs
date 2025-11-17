use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    config: ProviderConfig,
    client: Client,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string())
    }

    async fn check_service_running(&self) -> Result<(), AIError> {
        let url = format!("{}/api/tags", self.base_url());
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(_) => Err(AIError::ProviderNotAvailable(
                "Ollama service returned error status".to_string(),
            )),
            Err(e) => Err(AIError::NetworkError(format!(
                "Failed to connect to Ollama: {}",
                e
            ))),
        }
    }
}

#[async_trait::async_trait]
impl AIProvider for OllamaProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    async fn is_available(&self) -> Result<bool, AIError> {
        match self.check_service_running().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn test_connection(&self) -> Result<(), AIError> {
        self.check_service_running().await
    }

    async fn generate(
        &self,
        prompt: &str,
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError> {
        let start_time = Instant::now();

        // Check configuration first
        let config_status = self.check_configuration();
        if !config_status.is_configured {
            return Err(AIError::ConfigurationIncomplete(config_status));
        }

        // Check if service is running
        self.check_service_running().await?;

        let model = options
            .model
            .as_ref()
            .or_else(|| Some(&self.config.model))
            .ok_or_else(|| AIError::ProviderNotAvailable("No model specified".to_string()))?;

        let url = format!("{}/api/generate", self.base_url());

        #[derive(Serialize)]
        struct GenerateRequest {
            model: String,
            prompt: String,
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            num_predict: Option<u32>,
        }

        let request = GenerateRequest {
            model: model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            temperature: options.temperature,
            num_predict: options.max_tokens,
        };

        let timeout = std::time::Duration::from_millis(options.timeout_ms.unwrap_or(60000));
        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::TimeoutError(format!("Request timed out after {:?}", timeout))
                } else {
                    AIError::NetworkError(format!("Network error: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::InvalidResponse(format!(
                "Ollama API returned status {}: {}",
                status, error_text
            )));
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
            model: String,
            #[serde(rename = "eval_count")]
            eval_count: Option<u32>,
        }

        let result: GenerateResponse = response.json().await.map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse Ollama response: {}", e))
        })?;

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            content: result.response,
            model: result.model,
            tokens_used: result.eval_count,
            generation_time_ms: Some(generation_time),
        })
    }

    async fn get_available_models(&self) -> Result<Vec<String>, AIError> {
        self.check_service_running().await?;

        let url = format!("{}/api/tags", self.base_url());
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(format!("Failed to fetch models: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::InvalidResponse(
                "Failed to fetch Ollama models".to_string(),
            ));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            model: String,
            #[serde(default)]
            size: Option<u64>,
            #[serde(default)]
            digest: Option<String>,
        }

        let result: ModelsResponse = response.json().await.map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse models response: {}", e))
        })?;

        // Extract model names from the response
        Ok(result.models.into_iter().map(|m| m.model).collect())
    }

    fn default_model(&self) -> &str {
        &self.config.model
    }

    fn check_configuration(&self) -> ConfigurationStatus {
        let mut missing_fields = Vec::new();
        let mut warnings = Vec::new();

        // Ollama doesn't require API key, but needs service running
        if self.config.base_url.is_none() {
            warnings.push("Base URL not set, using default: http://localhost:11434".to_string());
        }

        if self.config.model.is_empty() {
            missing_fields.push("model".to_string());
        }

        ConfigurationStatus {
            is_configured: missing_fields.is_empty(),
            missing_fields,
            warnings,
        }
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::Ollama {
            return Err(AIError::GenericError(
                "Provider type mismatch".to_string(),
            ));
        }
        self.config = config;
        // Recreate client to ensure clean state
        self.client = Client::new();
        Ok(())
    }
}

