use crate::domains::ai::providers::{AIError, ProviderType};
use crate::domains::ai::services::AIService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
    pub history: Vec<ChatMessage>,
    pub provider: Option<ProviderType>,
    pub conversation_id: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub model: Option<String>,
}

/// Send a message to the AI provider
pub async fn send_message(
    request: SendMessageRequest,
    ai_service: State<'_, Arc<AIService>>,
) -> Result<String, String> {
    let provider = request.provider;
    let options = crate::domains::ai::providers::GenerationOptions {
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        timeout_ms: None,
        model: request.model,
        extra_options: None,
    };

    // Convert history to prompt format
    let mut prompt = String::new();
    for msg in &request.history {
        prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }
    prompt.push_str(&format!("user: {}\nassistant:", request.message));

    let result = ai_service
        .generate(&prompt, Some(options), provider)
        .await
        .map_err(|e| format!("AI generation error: {}", e))?;

    Ok(result.content)
}

