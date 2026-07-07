//! Shared thread title helpers (fallback + optional smart title via platform).

use crate::domains::ai::platform_config::{PlatformConfig, DESKTOP_CLIENT_ID};
use reqwest::Client;
use serde_json::{json, Value};

pub const PLACEHOLDER_CHAT: &str = "New chat";
pub const PLACEHOLDER_SESSION: &str = "New session";

pub fn smart_titles_enabled() -> bool {
    match std::env::var("CHAT_SMART_TITLES").as_deref() {
        Ok("0") | Ok("false") | Ok("off") | Ok("FALSE") | Ok("OFF") => false,
        _ => true,
    }
}

pub fn is_placeholder_title(title: &str) -> bool {
    let t = title.trim();
    t.is_empty() || t == PLACEHOLDER_CHAT || t == PLACEHOLDER_SESSION
}

pub fn fallback_title_from_message(message: &str, default: &str) -> String {
    let text: String = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return default.to_string();
    }
    if text.len() <= 48 {
        text
    } else {
        format!("{}...", &text[..45])
    }
}

pub fn should_apply_generated_title(current: &str, fallback: &str) -> bool {
    is_placeholder_title(current) || current == fallback
}

fn trim_smart_title(raw: &str) -> String {
    let mut title = raw.trim().trim_matches('"').trim_matches('\'').to_string();
    if title.len() > 80 {
        title.truncate(80);
    }
    let words: Vec<&str> = title.split_whitespace().take(6).collect();
    words.join(" ")
}

/// Small background LLM call — mirrors agent-platform smart title behavior.
pub async fn generate_smart_title(
    client: &Client,
    cfg: &PlatformConfig,
    message: &str,
) -> Result<String, String> {
    let url = format!("{}/v1/chat/completions", cfg.base_url.trim_end_matches('/'));
    let model = cfg
        .default_model
        .as_deref()
        .filter(|m| !m.is_empty())
        .unwrap_or("gpt-4o-mini");
    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "Generate a short conversation title of at most 6 words. Reply with only the title text, no quotes."
            },
            { "role": "user", "content": message }
        ],
        "stream": false,
        "max_tokens": 24,
        "temperature": 0.3
    });

    let mut req = client
        .post(&url)
        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
        .json(&body);
    if let Some(token) = &cfg.api_token {
        req = req.bearer_auth(token);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("smart title request failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("smart title returned {status}: {text}"));
    }

    let value: Value = resp
        .json()
        .await
        .map_err(|e| format!("smart title invalid json: {e}"))?;
    let content = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let title = trim_smart_title(&content);
    if title.is_empty() {
        return Err("smart title empty".into());
    }
    Ok(title)
}
