//! Types for agent-platform `GET /v1/catalog` discovery API.

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub models: Vec<CatalogModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogModel {
    pub id: String,
    pub provider: String,
    pub source: String,
    #[serde(default)]
    pub backend_id: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

/// Query parameters for `GET /v1/catalog`.
#[derive(Debug, Clone, Default)]
pub struct CatalogQuery {
    /// `None` = effective default provider only; `Some(vec!["all"])` = all providers.
    pub providers: Option<Vec<String>>,
    /// `Some(false)` = YAML aliases only (no upstream fetches).
    pub live: Option<bool>,
}

impl CatalogQuery {
    pub fn all_aliases() -> Self {
        Self {
            providers: Some(vec!["all".to_string()]),
            live: Some(false),
        }
    }

    pub fn all_live() -> Self {
        Self {
            providers: Some(vec!["all".to_string()]),
            live: None,
        }
    }
}
