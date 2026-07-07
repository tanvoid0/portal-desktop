//! Consumer for agent-platform `POST /api/v1/coder/chat/stream` SSE.

use crate::domains::ai::context_usage::{parse_context_usage, parse_llm_usage, ContextUsage, LlmUsage};
use serde_json::Value;

use super::types::{ChatMessage, PendingApproval};
use super::tools;

#[derive(Debug, Clone)]
pub struct PlatformDone {
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub pending: Option<PendingApproval>,
    pub final_text: Option<String>,
    pub context_usage: Option<ContextUsage>,
    pub llm_usage: Option<LlmUsage>,
}

/// Parse `event:` / `data:` blocks from accumulated SSE bytes.
pub fn drain_sse_events(buf: &mut Vec<u8>) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    loop {
        let text = String::from_utf8_lossy(buf);
        let Some(idx) = text.find("\n\n") else {
            break;
        };
        let block: Vec<u8> = buf.drain(..idx + 2).collect();
        let block = String::from_utf8_lossy(&block);
        let mut event = String::new();
        let mut data = String::new();
        for line in block.lines() {
            let line = line.trim_end_matches('\r');
            if let Some(rest) = line.strip_prefix("event:") {
                event = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("data:") {
                data = rest.trim().to_string();
            }
        }
        if event.is_empty() {
            continue;
        }
        let value: Value = if data.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&data).unwrap_or(Value::Null)
        };
        out.push((event, value));
    }
    out
}

pub fn pending_from_platform(value: &Value) -> Option<PendingApproval> {
    let call_id = value.get("call_id")?.as_str()?.to_string();
    let tool = value
        .get("name")
        .or_else(|| value.get("tool"))
        .and_then(Value::as_str)?
        .to_string();
    let arguments = value
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    Some(PendingApproval {
        call_id,
        tool: tool.clone(),
        arguments: arguments.clone(),
        suggested_rule: tools::suggested_rule(&tool, &arguments),
        summary: tools::summarize(&tool, &arguments),
    })
}

pub fn messages_from_done(value: &Value) -> Vec<ChatMessage> {
    value
        .get("messages")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|m| serde_json::from_value(m.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn final_text_from_messages(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && m.tool_calls.is_none())
        .and_then(|m| m.content.clone())
        .filter(|c| !c.is_empty())
}

pub fn done_from_event(data: &Value) -> PlatformDone {
    let messages = messages_from_done(data);
    let title = data
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("New session")
        .to_string();
    let pending = data.get("pending_call").and_then(|p| {
        if p.is_null() {
            None
        } else {
            pending_from_platform(p)
        }
    });
    let final_text = final_text_from_messages(&messages);
    let context_usage = data.get("context_usage").and_then(parse_context_usage);
    let llm_usage = data.get("usage").and_then(parse_llm_usage);
    PlatformDone {
        title,
        messages,
        pending,
        final_text,
        context_usage,
        llm_usage,
    }
}
