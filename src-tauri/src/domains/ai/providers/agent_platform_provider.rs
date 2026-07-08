use crate::domains::ai::catalog::{CatalogQuery, PlatformCatalog};
use crate::domains::ai::message::ChatMessage;
use crate::domains::ai::platform_config::DEFAULT_PLATFORM_BASE;
use crate::domains::ai::providers::{
    AIError, AIProvider, ConfigurationStatus, GenerationOptions, GenerationResult, ProviderConfig,
    ProviderType,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct AgentPlatformProvider {
    config: ProviderConfig,
    client: Client,
}

impl AgentPlatformProvider {
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
            .unwrap_or_else(|| DEFAULT_PLATFORM_BASE.to_string())
            .trim_end_matches('/')
            .to_string()
    }

    fn api_token(&self) -> Option<String> {
        self.config
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }

    fn model_for(&self, options: &GenerationOptions) -> Result<String, AIError> {
        options
            .model
            .clone()
            .or_else(|| {
                if self.config.model.is_empty() {
                    None
                } else {
                    Some(self.config.model.clone())
                }
            })
            .ok_or_else(|| {
                AIError::ConfigurationIncomplete(ConfigurationStatus {
                    is_configured: false,
                    missing_fields: vec!["model".to_string()],
                    warnings: vec![],
                })
            })
    }

    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req.header(
            "X-Agent-Platform-Client",
            super::super::platform_config::DESKTOP_CLIENT_ID,
        );
        match self.api_token() {
            Some(token) => req.bearer_auth(token),
            None => req,
        }
    }

    pub async fn fetch_catalog(&self, query: CatalogQuery) -> Result<PlatformCatalog, AIError> {
        let url = format!("{}/v1/catalog", self.base_url());
        let mut req = self.authed(self.client.get(&url));

        if let Some(providers) = &query.providers {
            for provider in providers {
                req = req.query(&[("providers", provider.as_str())]);
            }
        }
        if let Some(live) = query.live {
            req = req.query(&[("live", if live { "true" } else { "false" })]);
        }

        let response = req
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(format!("Failed to fetch catalog: {}", e)))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AIError::AuthenticationError(
                "Invalid or missing agent-platform API token".to_string(),
            ));
        }
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::InvalidResponse(format!(
                "agent-platform catalog returned status {}: {}",
                status, error_text
            )));
        }

        response
            .json::<PlatformCatalog>()
            .await
            .map_err(|e| AIError::InvalidResponse(format!("Invalid catalog response: {}", e)))
    }

    async fn chat_completion(
        &self,
        messages: Vec<Value>,
        options: &GenerationOptions,
        stream: bool,
    ) -> Result<reqwest::Response, AIError> {
        let model = self.model_for(options)?;
        let url = format!("{}/v1/chat/completions", self.base_url());
        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": stream,
        });
        if let Some(temp) = options.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(provider) = options
            .llm_provider
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            body["provider"] = json!(provider);
        }

        let timeout = std::time::Duration::from_millis(options.timeout_ms.unwrap_or(120_000));
        let response = self
            .authed(self.client.post(&url).json(&body))
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::TimeoutError(format!("Request timed out after {:?}", timeout))
                } else if e.is_connect() {
                    AIError::NetworkError(format!(
                        "Cannot reach agent-platform at {}. Is it running?",
                        self.base_url()
                    ))
                } else {
                    AIError::NetworkError(format!("Network error: {}", e))
                }
            })?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AIError::AuthenticationError(
                "Invalid or missing agent-platform API token".to_string(),
            ));
        }
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::InvalidResponse(format!(
                "agent-platform returned status {}: {}",
                status, error_text
            )));
        }
        Ok(response)
    }

    fn parse_completion(response: Value) -> Result<(String, String, Option<u32>), AIError> {
        let content = response
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let model = response
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let tokens = response
            .get("usage")
            .and_then(|u| u.get("total_tokens"))
            .and_then(Value::as_u64)
            .map(|n| n as u32);
        Ok((content, model, tokens))
    }

    async fn consume_sse_stream(
        &self,
        response: reqwest::Response,
        mut on_chunk: Option<Box<dyn FnMut(String) -> Result<(), AIError> + Send>>,
    ) -> Result<(String, String, Option<u32>), AIError> {
        let mut content = String::new();
        let mut model = String::new();
        let mut buf: Vec<u8> = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| AIError::NetworkError(format!("Stream error: {}", e)))?;
            buf.extend_from_slice(&bytes);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                let payload = match line.strip_prefix("data:") {
                    Some(p) => p.trim(),
                    None => continue,
                };
                if payload == "[DONE]" {
                    continue;
                }
                let Ok(value) = serde_json::from_str::<Value>(payload) else {
                    continue;
                };
                if let Some(m) = value.get("model").and_then(Value::as_str) {
                    model = m.to_string();
                }
                if let Some(delta) = value
                    .get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("delta"))
                    .and_then(|d| d.get("content"))
                    .and_then(Value::as_str)
                {
                    if !delta.is_empty() {
                        content.push_str(delta);
                        if let Some(cb) = on_chunk.as_mut() {
                            cb(delta.to_string())?;
                        }
                    }
                }
            }
        }

        Ok((content, model, None))
    }
}

#[async_trait::async_trait]
impl AIProvider for AgentPlatformProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::AgentPlatform
    }

    fn name(&self) -> &str {
        "Agent Platform"
    }

    async fn is_available(&self) -> Result<bool, AIError> {
        let url = format!("{}/v1/health/readiness", self.base_url());
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;
        Ok(response.status().is_success())
    }

    async fn test_connection(&self) -> Result<(), AIError> {
        let status = self.check_configuration();
        if !status.is_configured {
            return Err(AIError::ConfigurationIncomplete(status));
        }

        // Prefer /v1/catalog when a token is set — validates auth + upstream.
        if self.api_token().is_some() {
            if self.fetch_catalog(CatalogQuery::default()).await.is_ok() {
                return Ok(());
            }

            let url = format!("{}/v1/models", self.base_url());
            let response = self
                .authed(self.client.get(&url))
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await
                .map_err(|e| AIError::NetworkError(e.to_string()))?;
            if response.status().is_success() {
                return Ok(());
            }
            if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                return Err(AIError::AuthenticationError(
                    "Invalid agent-platform API token".to_string(),
                ));
            }
        }

        self.is_available().await?.then_some(()).ok_or_else(|| {
            AIError::ProviderNotAvailable(format!(
                "agent-platform is not reachable at {}",
                self.base_url()
            ))
        })
    }

    async fn generate(
        &self,
        prompt: &str,
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError> {
        let start = Instant::now();
        let messages = vec![json!({"role": "user", "content": prompt})];
        let response = self.chat_completion(messages, options, false).await?;
        let value: Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;
        let (content, model, tokens_used) = Self::parse_completion(value)?;
        Ok(GenerationResult {
            content,
            model,
            tokens_used,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        options: &GenerationOptions,
        mut on_chunk: Box<dyn FnMut(String) -> Result<(), AIError> + Send>,
    ) -> Result<GenerationResult, AIError> {
        let start = Instant::now();
        let messages = vec![json!({"role": "user", "content": prompt})];
        let response = self.chat_completion(messages, options, true).await?;
        let (content, model, tokens_used) =
            self.consume_sse_stream(response, Some(on_chunk)).await?;
        Ok(GenerationResult {
            content,
            model,
            tokens_used,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn generate_with_system(
        &self,
        system_message: &str,
        user_message: &str,
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError> {
        let start = Instant::now();
        let messages = vec![
            json!({"role": "system", "content": system_message}),
            json!({"role": "user", "content": user_message}),
        ];
        let response = self.chat_completion(messages, options, false).await?;
        let value: Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;
        let (content, model, tokens_used) = Self::parse_completion(value)?;
        Ok(GenerationResult {
            content,
            model,
            tokens_used,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn generate_chat(
        &self,
        messages: &[ChatMessage],
        options: &GenerationOptions,
    ) -> Result<GenerationResult, AIError> {
        let start = Instant::now();
        let api_messages: Vec<Value> = messages
            .iter()
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect();
        let response = self.chat_completion(api_messages, options, false).await?;
        let value: Value = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))?;
        let (content, model, tokens_used) = Self::parse_completion(value)?;
        Ok(GenerationResult {
            content,
            model,
            tokens_used,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn generate_chat_stream(
        &self,
        messages: &[ChatMessage],
        options: &GenerationOptions,
        mut on_chunk: Box<dyn FnMut(String) -> Result<(), AIError> + Send>,
    ) -> Result<GenerationResult, AIError> {
        let start = Instant::now();
        let api_messages: Vec<Value> = messages
            .iter()
            .map(|m| json!({"role": m.role, "content": m.content}))
            .collect();
        let response = self.chat_completion(api_messages, options, true).await?;
        let (content, model, tokens_used) =
            self.consume_sse_stream(response, Some(on_chunk)).await?;
        Ok(GenerationResult {
            content,
            model,
            tokens_used,
            generation_time_ms: Some(start.elapsed().as_millis() as u64),
        })
    }

    async fn get_available_models(&self) -> Result<Vec<String>, AIError> {
        let catalog = match self.fetch_catalog(CatalogQuery::all_aliases()).await {
            Ok(catalog) => catalog,
            Err(_) => self.fetch_catalog(CatalogQuery::default()).await?,
        };

        let mut ids: Vec<String> = catalog
            .providers
            .iter()
            .flat_map(|p| p.models.iter().map(|m| m.id.clone()))
            .collect();
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    fn default_model(&self) -> &str {
        &self.config.model
    }

    fn check_configuration(&self) -> ConfigurationStatus {
        let mut missing_fields = Vec::new();
        let mut warnings = Vec::new();

        if self.config.base_url.is_none() {
            warnings.push(format!(
                "Base URL not set, using default: {}",
                DEFAULT_PLATFORM_BASE
            ));
        }

        if self.config.model.is_empty() {
            missing_fields.push("model".to_string());
        }

        if self.api_token().is_none() {
            warnings.push(
                "No API token set in Settings — required when agent-platform enforces authentication"
                    .to_string(),
            );
        }

        ConfigurationStatus {
            is_configured: missing_fields.is_empty(),
            missing_fields,
            warnings,
        }
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<(), AIError> {
        if config.provider_type != ProviderType::AgentPlatform {
            return Err(AIError::GenericError("Provider type mismatch".to_string()));
        }
        self.config = config;
        self.client = Client::new();
        Ok(())
    }
}
