use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig, ProviderType,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    config: ProviderConfig,
    client: Client,
}

impl GeminiProvider {
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
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string())
    }

    fn api_key(&self) -> Result<String, AIError> {
        self.config
            .api_key
            .clone()
            .ok_or_else(|| AIError::AuthenticationError("Gemini API key not configured".to_string()))
    }
}

#[async_trait::async_trait]
impl AIProvider for GeminiProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Gemini
    }

    fn name(&self) -> &str {
        "Gemini"
    }

    async fn is_available(&self) -> Result<bool, AIError> {
        Ok(self.api_key().is_ok())
    }

    async fn test_connection(&self) -> Result<(), AIError> {
        let api_key = self.api_key()?;
        let model = self.default_model();
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url(),
            model,
            api_key
        );

        #[derive(Serialize)]
        struct TestRequest {
            contents: Vec<Content>,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        let request = TestRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: "test".to_string(),
                }],
            }],
        };

        let response = self
            .client
            .post(&url)
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
                "Gemini API returned status {}",
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

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url(),
            model,
            api_key
        );

        #[derive(Serialize)]
        struct GenerateRequest {
            contents: Vec<Content>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_config: Option<GenerationConfig>,
        }

        #[derive(Serialize)]
        struct Content {
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Serialize)]
        struct GenerationConfig {
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_output_tokens: Option<u32>,
        }

        let request = GenerateRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: options.temperature,
                max_output_tokens: options.max_tokens,
            }),
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
            
            if status == 401 {
                return Err(AIError::AuthenticationError("Invalid API key".to_string()));
            } else if status == 429 {
                return Err(AIError::RateLimitError("Rate limit exceeded".to_string()));
            }

            return Err(AIError::InvalidResponse(format!(
                "Gemini API returned status {}: {}",
                status, error_text
            )));
        }

        #[derive(Deserialize)]
        struct GenerateResponse {
            candidates: Vec<Candidate>,
        }

        #[derive(Deserialize)]
        struct Candidate {
            content: ResponseContent,
            #[serde(rename = "tokenCount")]
            token_count: Option<u32>,
        }

        #[derive(Deserialize)]
        struct ResponseContent {
            parts: Vec<ResponsePart>,
        }

        #[derive(Deserialize)]
        struct ResponsePart {
            text: String,
        }

        let result: GenerateResponse = response.json().await.map_err(|e| {
            AIError::InvalidResponse(format!("Failed to parse Gemini response: {}", e))
        })?;

        let content = result
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| AIError::InvalidResponse("No content in response".to_string()))?;

        let tokens_used = result
            .candidates
            .first()
            .and_then(|c| c.token_count);

        let generation_time = start_time.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            content,
            model: model.clone(),
            tokens_used,
            generation_time_ms: Some(generation_time),
        })
    }

    async fn get_available_models(&self) -> Result<Vec<String>, AIError> {
        // Gemini models are fixed, return common ones
        Ok(vec![
            "gemini-pro".to_string(),
            "gemini-pro-vision".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
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
            warnings.push("Base URL not set, using default Gemini API endpoint".to_string());
        }

        ConfigurationStatus {
            is_configured: missing_fields.is_empty(),
            missing_fields,
            warnings,
        }
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::Gemini {
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

