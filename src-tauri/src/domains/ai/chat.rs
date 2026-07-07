pub use crate::domains::ai::message::ChatMessage;
use crate::domains::ai::message::ChatMessage as ChatTurn;
use crate::domains::ai::providers::ProviderType;
use crate::domains::ai::services::AIService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
    pub history: Vec<ChatTurn>,
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

    let mut messages = request.history;
    messages.push(ChatTurn {
        role: "user".to_string(),
        content: request.message,
    });

    let result = ai_service
        .generate_chat(&messages, Some(options), provider)
        .await
        .map_err(|e| format!("AI generation error: {}", e))?;

    Ok(result.content)
}
