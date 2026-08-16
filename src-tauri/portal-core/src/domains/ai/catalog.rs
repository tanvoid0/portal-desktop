//! Types for agent-platform `GET /v1/catalog` discovery API.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CatalogModelCapabilities {
    #[serde(default)]
    pub chat: Option<bool>,
    #[serde(default)]
    pub tools: Option<bool>,
    #[serde(default)]
    pub vision_input: Option<bool>,
    #[serde(default)]
    pub embeddings: Option<bool>,
    #[serde(default)]
    pub image_generation: Option<bool>,
    #[serde(default)]
    pub streaming: Option<bool>,
    #[serde(default)]
    pub probe_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CatalogProviderCapabilities {
    #[serde(default)]
    pub streaming: Option<bool>,
    #[serde(default)]
    pub tools: Option<bool>,
    #[serde(default)]
    pub json_mode: Option<bool>,
    #[serde(default)]
    pub modalities: Option<Value>,
    #[serde(default)]
    pub model_discovery: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCatalog {
    pub object: String,
    pub resolved_defaults: ResolvedDefaults,
    pub providers: Vec<CatalogProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDefaults {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogProvider {
    pub id: String,
    pub label: String,
    pub configured: bool,
    #[serde(default)]
    pub reachable: Option<bool>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub capabilities: Option<CatalogProviderCapabilities>,
    pub models: Vec<CatalogModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogModel {
    pub id: String,
    /// Parent provider id when present; agent-platform v1 catalog omits this on model rows.
    #[serde(default)]
    pub provider: Option<String>,
    pub source: String,
    #[serde(default)]
    pub backend_id: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub capabilities: Option<CatalogModelCapabilities>,
}

#[cfg(test)]
mod tests {
    use super::PlatformCatalog;

    #[test]
    fn deserializes_v1_catalog_without_model_provider_field() {
        let json = r#"{
            "object": "catalog",
            "resolved_defaults": { "provider": "ollama", "model": "llama3" },
            "providers": [{
                "id": "ollama",
                "label": "Ollama",
                "configured": true,
                "reachable": false,
                "default_model": "llama3",
                "models": [{ "id": "llama3", "source": "alias" }]
            }]
        }"#;

        let catalog: PlatformCatalog = serde_json::from_str(json).expect("catalog json");
        assert_eq!(catalog.object, "catalog");
        assert_eq!(catalog.providers[0].models[0].id, "llama3");
        assert!(catalog.providers[0].models[0].provider.is_none());
    }
}

/// Query parameters for `GET /v1/catalog`.
#[derive(Debug, Clone, Default)]
pub struct CatalogQuery {
    /// `None` = effective default provider only; `Some(vec!["all"])` = all providers.
    pub providers: Option<Vec<String>>,
    /// `Some(false)` = YAML aliases only (no upstream fetches).
    pub live: Option<bool>,
    /// When true with live catalog, probe Ollama models for tools/vision flags.
    pub probe_capabilities: Option<bool>,
}

impl CatalogQuery {
    pub fn all_aliases() -> Self {
        Self {
            providers: Some(vec!["all".to_string()]),
            live: Some(false),
            probe_capabilities: None,
        }
    }

    pub fn all_live() -> Self {
        Self {
            providers: Some(vec!["all".to_string()]),
            live: None,
            probe_capabilities: Some(true),
        }
    }
}
