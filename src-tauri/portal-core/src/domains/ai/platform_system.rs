//! `GET /api/v1/system/status` and `/api/v1/system/logs` on the local
//! agent-platform daemon — the data behind portal's Status/Logs screen.
//!
//! Both daemon routes are free-form aggregator payloads (see agent-platform's
//! `server/src/system.rs`): `/status` fans in version, uptime, readiness,
//! process counts and paths; `/logs` returns `{ lines, next, dropped }` where
//! `next` is the cursor to poll back with. Portal keeps them as `Value` and
//! normalizes the handful of fields it renders on the frontend, so a daemon
//! that grows a field does not need a change here.
//!
//! Secrets: the saved token only ever travels in the `Authorization` header.
//! No error string below interpolates a reqwest error, response body or token.

use super::platform_config::{DEFAULT_PLATFORM_BASE, DESKTOP_CLIENT_ID};
use super::providers::{ProviderConfig, ProviderType};
use super::services::AISettingsService;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::time::Duration;

/// Base URL + saved token from the AgentPlatform provider config.
fn resolve(settings: &AISettingsService) -> (String, Option<String>) {
    let cfg = settings
        .get_provider_config(ProviderType::AgentPlatform)
        .unwrap_or_else(|_| ProviderConfig::default_for_type(ProviderType::AgentPlatform));
    let base = cfg
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_PLATFORM_BASE)
        .trim_end_matches('/')
        .to_string();
    let token = cfg
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    (base, token)
}

async fn get_json(settings: &AISettingsService, path: &str) -> Result<Value, String> {
    let (base, token) = resolve(settings);
    let mut req = Client::new()
        .get(format!("{base}{path}"))
        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
        .timeout(Duration::from_secs(10));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.map_err(|e| {
        if e.is_timeout() {
            "agent-platform timed out".to_string()
        } else if e.is_connect() {
            "Cannot reach agent-platform. Is it running?".to_string()
        } else {
            "Network error talking to agent-platform".to_string()
        }
    })?;
    if resp.status() == StatusCode::UNAUTHORIZED {
        return Err(
            "agent-platform rejected the request: no valid token. Pair it in Settings → AI → Providers."
                .to_string(),
        );
    }
    if !resp.status().is_success() {
        return Err(format!(
            "agent-platform returned status {}",
            resp.status().as_u16()
        ));
    }
    resp.json::<Value>()
        .await
        .map_err(|_| "agent-platform returned an unreadable body".to_string())
}

/// Daemon status aggregate: version, uptime, readiness, process counts, paths.
pub async fn system_status(settings: &AISettingsService) -> Result<Value, String> {
    get_json(settings, "/api/v1/system/status").await
}

/// Recent daemon log lines. Pass the `next` cursor from the previous response
/// (0 for the first poll) to get only what has been written since.
pub async fn system_logs(settings: &AISettingsService, after: u64) -> Result<Value, String> {
    get_json(settings, &format!("/api/v1/system/logs?after={after}")).await
}
