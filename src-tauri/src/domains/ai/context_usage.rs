//! Context window and LLM token usage shapes from agent-platform chat APIs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Estimated input context breakdown (matches agent-platform `ContextUsageOut`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextUsage {
    pub context_window: u64,
    pub total_estimated: u64,
    pub percent_used: f64,
    pub prompt_budget: u64,
    pub reserved_output: u64,
    #[serde(default)]
    pub categories: HashMap<String, u64>,
}

/// Aggregated LLM token usage for one or more completion steps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmUsage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cost_usd: f64,
}

fn coerce_u64(v: &Value) -> u64 {
    v.as_u64()
        .or_else(|| v.as_i64().map(|n| n.max(0) as u64))
        .unwrap_or(0)
}

fn coerce_f64(v: &Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_i64().map(|n| n as f64))
        .or_else(|| v.as_u64().map(|n| n as f64))
        .unwrap_or(0.0)
}

pub fn parse_context_usage(value: &Value) -> Option<ContextUsage> {
    if value.is_null() {
        return None;
    }
    serde_json::from_value(value.clone()).ok().or_else(|| {
        Some(ContextUsage {
            context_window: coerce_u64(&value["context_window"]),
            total_estimated: coerce_u64(&value["total_estimated"]),
            percent_used: coerce_f64(&value["percent_used"]),
            prompt_budget: coerce_u64(&value["prompt_budget"]),
            reserved_output: coerce_u64(&value["reserved_output"]),
            categories: value
                .get("categories")
                .and_then(Value::as_object)
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), coerce_u64(v)))
                        .collect()
                })
                .unwrap_or_default(),
        })
    })
}

pub fn parse_llm_usage(value: &Value) -> Option<LlmUsage> {
    if value.is_null() {
        return None;
    }
    serde_json::from_value(value.clone()).ok().or_else(|| {
        Some(LlmUsage {
            prompt_tokens: coerce_u64(&value["prompt_tokens"]),
            completion_tokens: coerce_u64(&value["completion_tokens"]),
            total_tokens: coerce_u64(&value["total_tokens"]),
            cost_usd: coerce_f64(&value["cost_usd"]),
        })
    })
}
