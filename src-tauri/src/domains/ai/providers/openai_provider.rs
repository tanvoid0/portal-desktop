use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: Client,
}

impl OpenAIProvider {
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
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }

    fn api_key(&self) -> Result<String, AIError> {
        self.config
            .api_key
            .clone()
            .ok_or_else(|| AIError::AuthenticationError("OpenAI API key not configured".to_string()))
    }
}

#[async_trait::async_trait]
impl AIProvider for OpenAIProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAI
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn is_available(&self) -> Result<bool, AIError> {
        Ok(self.api_key().is_ok())
    }

    async fn test_connection(&self) -> Result<(), AIError> {
        let api_key = self.api_key()?;
        let url = format!("{}/models", self.base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) if resp.status() == 401 => {
                Err(AIError::AuthenticationError("Invalid API key".to_string()))
            }
            Ok(resp) => Err(AIError::InvalidResponse(format!(
                "OpenAI API returned status {}",
                resp.status()
            ))),
            Err(e) => Err(AIError::NetworkError(format!("Network error: {}", e))),
        }
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

        let api_key = self.api_key()?;

        let model = options
            .model
            .as_ref()
            .or_else(|| Some(&self.config.model))
            .ok_or_else(|| AIError::ProviderNotAvailable("No model specified".to_string()))?;

        let url = format!("{}/chat/completions", self.base_url());

        #[derive(Serialize)]
        struct GenerateRequest {
            model: String,
            messages: Vec<Message>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        let request = GenerateRequest {
            model: model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: options.temperature,
            max_tokens: options.max_tokens,
        };

        let timeout = std::time::Duration::from_millis(options.timeout_ms.unwrap_or(60000));
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
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
            
            if status == 401 {
                return Err(AIError::AuthenticationError("Invalid API key".to_string()));
            } else if status == 429 {
                return Err(AIError::RateLimitError("Rate limit exceeded".to_string()));
            }

            return Err(AIError::InvalidResponse(format!(
                "OpenAI API returned status {}: {}",
                status, error_text
            )));
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            choices: Vec<Choice>,
            model: String,
            usage: Option<Usage>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: ResponseMessage,
        }

        #[derive(Deserialize)]
        struct ResponseMessage {
            content: String,
        }

        #[derive(Deserialize)]
        struct Usage {
            #[serde(rename = "total_tokens")]
            total_tokens: Option<u32>,
        }

        let result: GenerateResponse = response.json().await.map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse OpenAI response: {}", e))
        })?;

        let content = result
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AIError::InvalidResponse("No content in response".to_string()))?;

        let tokens_used = result.usage.and_then(|u| u.total_tokens);

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            content,
            model: result.model,
            tokens_used,
            generation_time_ms: Some(generation_time),
        })
    }

    async fn get_available_models(&self) -> Result<Vec<String>, AIError> {
        // OpenAI models are fixed, return common ones
        Ok(vec![
            "gpt-4".to_string(),
            "gpt-4-turbo-preview".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-4o".to_string(),
        ])
    }

    fn default_model(&self) -> &str {
        &self.config.model
    }

    fn check_configuration(&self) -> ConfigurationStatus {
        let mut missing_fields = Vec::new();
        let mut warnings = Vec::new();

        if self.config.api_key.is_none() {
            missing_fields.push("api_key".to_string());
        }

        if self.config.model.is_empty() {
            missing_fields.push("model".to_string());
        }

        if self.config.base_url.is_none() {
            warnings.push("Base URL not set, using default OpenAI API endpoint".to_string());
        }

        ConfigurationStatus {
            is_configured: missing_fields.is_empty(),
            missing_fields,
            warnings,
        }
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::OpenAI {
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

