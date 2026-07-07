//! Shared agent-platform connection settings for the desktop app.
//!
//! API tokens come only from the Agent Platform row in AI settings. Env vars may
//! still supply base URL / default model for dev/CI when settings are absent.

use super::providers::{ProviderConfig, ProviderType};
use super::services::AISettingsService;

pub const DEFAULT_PLATFORM_BASE: &str = "http://127.0.0.1:18410";
pub const DESKTOP_CLIENT_ID: &str = "portal-desktop";

/// Resolved platform endpoint + credentials.
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub base_url: String,
    pub api_token: Option<String>,
    pub default_model: Option<String>,
}

impl PlatformConfig {
    /// Read from the AgentPlatform provider row, then env vars for URL/model only.
    pub fn resolve(settings: &AISettingsService) -> Self {
        if let Ok(config) = settings.get_provider_config(ProviderType::AgentPlatform) {
            if config.enabled {
                return Self::from_provider_config(&config);
            }
        }
        Self::from_env()
    }

    pub fn from_provider_config(config: &ProviderConfig) -> Self {
        let base_url = config
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_PLATFORM_BASE)
            .trim_end_matches('/')
            .to_string();
        let api_token = config
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let default_model = if config.model.is_empty() {
            std::env::var("CODER_MODEL").ok().filter(|m| !m.is_empty())
        } else {
            Some(config.model.clone())
        };
        Self {
            base_url,
            api_token,
            default_model,
        }
    }

    pub fn from_env() -> Self {
        let base_url = std::env::var("CODER_PLATFORM_BASE_URL")
            .or_else(|_| std::env::var("AGENT_PLATFORM_BASE_URL"))
            .unwrap_or_else(|_| DEFAULT_PLATFORM_BASE.to_string())
            .trim_end_matches('/')
            .to_string();
        let default_model = std::env::var("CODER_MODEL").ok().filter(|m| !m.is_empty());
        Self {
            base_url,
            api_token: None,
            default_model,
        }
    }
}
