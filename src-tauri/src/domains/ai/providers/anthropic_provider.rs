use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: Client,
}

impl AnthropicProvider {
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
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string())
    }

    fn api_key(&self) -> Result<String, AIError> {
        self.config
            .api_key
            .clone()
            .ok_or_else(|| AIError::AuthenticationError("Anthropic API key not configured".to_string()))
    }
}

#[async_trait::async_trait]
impl AIProvider for AnthropicProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Anthropic
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    async fn is_available(&self) -> Result<bool, AIError> {
        Ok(self.api_key().is_ok())
    }

    async fn test_connection(&self) -> Result<(), AIError> {
        let api_key = self.api_key()?;
        let model = self.default_model();
        let url = format!("{}/messages", self.base_url());

        #[derive(Serialize)]
        struct TestRequest {
            model: String,
            max_tokens: u32,
            messages: Vec<Message>,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        let request = TestRequest {
            model: model.to_string(),
            max_tokens: 10,
            messages: vec![Message {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) if resp.status() == 401 => {
                Err(AIError::AuthenticationError("Invalid API key".to_string()))
            }
            Ok(resp) => Err(AIError::InvalidResponse(format!(
                "Anthropic API returned status {}",
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

        let url = format!("{}/messages", self.base_url());

        #[derive(Serialize)]
        struct GenerateRequest {
            model: String,
            max_tokens: u32,
            messages: Vec<Message>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
        }

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        let max_tokens = options.max_tokens.unwrap_or(2048);

        let request = GenerateRequest {
            model: model.clone(),
            max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: options.temperature,
        };

        let timeout = std::time::Duration::from_millis(options.timeout_ms.unwrap_or(60000));
        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
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
                "Anthropic API returned status {}: {}",
                status, error_text
            )));
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            content: Vec<ContentBlock>,
            model: String,
            usage: Option<Usage>,
        }

        #[derive(Deserialize)]
        struct ContentBlock {
            #[serde(rename = "type")]
            block_type: String,
            text: Option<String>,
        }

        #[derive(Deserialize)]
        struct Usage {
            #[serde(rename = "input_tokens")]
            input_tokens: Option<u32>,
            #[serde(rename = "output_tokens")]
            output_tokens: Option<u32>,
        }

        let result: GenerateResponse = response.json().await.map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse Anthropic response: {}", e))
        })?;

        let content = result
            .content
            .iter()
            .filter_map(|block| {
                if block.block_type == "text" {
                    block.text.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if content.is_empty() {
            return Err(AIError::InvalidResponse("No text content in response".to_string()));
        }

        let tokens_used = result.usage.map(|u| {
            u.input_tokens.unwrap_or(0) + u.output_tokens.unwrap_or(0)
        });

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            content,
            model: result.model,
            tokens_used,
            generation_time_ms: Some(generation_time),
        })
    }

    async fn get_available_models(&self) -> Result<Vec<String>, AIError> {
        // Anthropic models are fixed, return common ones
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
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
            warnings.push("Base URL not set, using default Anthropic API endpoint".to_string());
        }

        ConfigurationStatus {
            is_configured: missing_fields.is_empty(),
            missing_fields,
            warnings,
        }
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::Anthropic {
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

