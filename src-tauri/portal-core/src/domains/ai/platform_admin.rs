//! Health, start and pairing for the local agent-platform server.
//!
//! Portal owns `agent-platformd` directly: it spawns the bundled sidecar with
//! the user's existing agent-platform data (see [`super::platform_daemon`]) and
//! kills it on exit. The old arrangement launched the Agent Platform *app* and
//! refused to spawn the daemon, because a bare spawn serves the wrong database
//! with no key; [`super::platform_daemon::daemon_env`] is what makes a direct
//! spawn safe. The actual `Child` lives in the host (`src-tauri`), which passes a
//! spawn callback to [`start_platform`].
//!
//! Secrets: the master key and the minted token only ever travel in an
//! `Authorization` header or the settings file. No `detail` string or error here
//! interpolates a reqwest error, response body, header or either secret.

use super::platform_config::{DEFAULT_PLATFORM_BASE, DESKTOP_CLIENT_ID};
use super::platform_daemon::{self as daemon, Action, PortProbe};
use super::providers::{ProviderConfig, ProviderType};
use super::services::AISettingsService;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct PlatformHealth {
    pub base_url: String,
    pub reachable: bool,
    pub auth_required: bool,
    pub authenticated: bool,
    pub detail: Option<String>,
    /// Whether the `agent-platformd` binary is present, independent of `reachable`.
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartResult {
    pub started: bool,
    pub already_running: bool,
    pub pid: Option<u32>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairResult {
    pub ok: bool,
    pub detail: String,
}

fn config(settings: &AISettingsService) -> ProviderConfig {
    settings
        .get_provider_config(ProviderType::AgentPlatform)
        .unwrap_or_else(|_| ProviderConfig::default_for_type(ProviderType::AgentPlatform))
}

fn base_of(c: &ProviderConfig) -> String {
    c.base_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_PLATFORM_BASE)
        .trim_end_matches('/')
        .to_string()
}

fn get(client: &Client, url: &str, secs: u64) -> reqwest::RequestBuilder {
    client
        .get(url)
        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
        .timeout(Duration::from_secs(secs))
}

fn net_detail(e: &reqwest::Error) -> &'static str {
    if e.is_timeout() {
        "Timed out"
    } else if e.is_connect() {
        "Connection refused"
    } else {
        "Network error"
    }
}

async fn probe(base: &str, token: Option<&str>) -> PlatformHealth {
    let client = Client::new();
    let mut h = PlatformHealth {
        base_url: base.to_string(),
        reachable: false,
        auth_required: false,
        authenticated: false,
        detail: None,
        installed: daemon::daemon_exe().is_some(),
    };
    match get(&client, &format!("{base}/health"), 3).send().await {
        Ok(r) => {
            h.reachable = true;
            let body: Value = r.json().await.unwrap_or(Value::Null);
            h.auth_required = body["auth"]["required"].as_bool().unwrap_or(false);
        }
        Err(e) => {
            h.detail = Some(net_detail(&e).to_string());
            return h;
        }
    }
    let Some(token) = token else {
        h.detail = h.auth_required.then(|| "No token saved".to_string());
        h.authenticated = !h.auth_required;
        return h;
    };
    match get(&client, &format!("{base}/v1/catalog?live=false"), 8)
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(r) if r.status() == StatusCode::UNAUTHORIZED => {
            h.detail = Some("Token rejected by this install".to_string())
        }
        Ok(r) if r.status().is_success() => h.authenticated = true,
        Ok(r) => h.detail = Some(format!("Catalog returned status {}", r.status().as_u16())),
        Err(e) => h.detail = Some(net_detail(&e).to_string()),
    }
    h
}

pub async fn platform_health(settings: &AISettingsService) -> PlatformHealth {
    let c = config(settings);
    let token = c
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    probe(&base_of(&c), token).await
}

/// Only the daemon on our own default loopback address may be sent the install
/// key: a custom or remote base URL is never self-paired.
fn can_self_pair(h: &PlatformHealth) -> bool {
    h.base_url == DEFAULT_PLATFORM_BASE && h.reachable && h.auth_required && !h.authenticated
}

/// Repair a missing or dead token by pairing with the `master.key` already on
/// this machine, so a local install never asks the user to paste one. True when
/// a new token was saved.
///
/// One attempt until a probe next authenticates: a 401 that a new token cannot
/// fix must not mint a token on every health check.
pub async fn self_pair(settings: &AISettingsService, health: &PlatformHealth) -> bool {
    static TRIED: AtomicBool = AtomicBool::new(false);
    if health.authenticated {
        TRIED.store(false, Ordering::Relaxed);
    }
    if !can_self_pair(health) || TRIED.swap(true, Ordering::Relaxed) {
        return false;
    }
    // Read only: creating a key here would not match a daemon someone else started.
    match std::fs::read_to_string(daemon::data_dir().join("master.key")) {
        Ok(key) => pair_platform(settings, &key).await.ok,
        Err(_) => false,
    }
}

/// `/health` reachable, then does the daemon accept *our* master key. A status
/// request with no bearer answering 200 is an open (pre-ADR-0019) daemon, still
/// treated as ours, matching agent-platform's own `port_owner`.
async fn probe_port(base: &str, key: &str) -> PortProbe {
    let client = Client::new();
    if !matches!(get(&client, &format!("{base}/health"), 3).send().await, Ok(r) if r.status().is_success())
    {
        return PortProbe::Unreachable;
    }
    let status = format!("{base}/api/v1/system/status");
    if matches!(get(&client, &status, 5).send().await, Ok(r) if r.status().is_success()) {
        return PortProbe::AcceptsKey;
    }
    match get(&client, &status, 5).bearer_auth(key).send().await {
        Ok(r) if r.status().is_success() => PortProbe::AcceptsKey,
        _ => PortProbe::RejectsKey,
    }
}

/// Attach to a healthy daemon that accepts our key, report (never adopt) a
/// foreign one, otherwise spawn through `spawn` (which returns the child's pid).
pub async fn start_platform(
    settings: &AISettingsService,
    spawn: impl FnOnce() -> Result<u32, String>,
) -> StartResult {
    let fail = |detail: &str, pid: Option<u32>| StartResult {
        started: false,
        already_running: false,
        pid,
        detail: detail.to_string(),
    };
    let base = base_of(&config(settings));
    // A user-configured remote/other-port platform is not ours to start.
    if base != DEFAULT_PLATFORM_BASE {
        return fail(
            "agent-platform is configured at a custom address; start it yourself",
            None,
        );
    }
    let key = match daemon::load_or_create_key(&daemon::data_dir()) {
        Ok(k) => k,
        Err(_) => {
            return fail(
                "Could not read or create the agent-platform master key",
                None,
            )
        }
    };
    match daemon::decide(probe_port(&base, &key).await) {
        Action::Attach => {
            return StartResult {
                started: false,
                already_running: true,
                pid: None,
                detail: "agent-platform is already running".to_string(),
            }
        }
        Action::Foreign => {
            return fail(
                "Port 18410 is held by an agent-platform server that rejects this install's key; not starting a second one",
                None,
            )
        }
        Action::Spawn => {}
    }
    let pid = match spawn() {
        Ok(pid) => pid,
        Err(detail) => return fail(&detail, None),
    };
    // First launch runs migrations, so allow a slow boot.
    for _ in 0..120 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if probe(&base, None).await.reachable {
            return StartResult {
                started: true,
                already_running: false,
                pid: Some(pid),
                detail: "Started agent-platformd".to_string(),
            };
        }
    }
    fail(
        "Started agent-platformd but /health did not answer within 60s",
        Some(pid),
    )
}

/// First workspace id from `{"workspaces":[{"id":..}]}` (or a bare array).
pub fn first_workspace_id(body: &Value) -> Option<i64> {
    body.get("workspaces")
        .unwrap_or(body)
        .as_array()?
        .first()?
        .get("id")?
        .as_i64()
}

/// The one-time `token` field of a mint response.
pub fn minted_token(body: &Value) -> Option<String> {
    body.get("token")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub async fn pair_platform(settings: &AISettingsService, master_key: &str) -> PairResult {
    let err = |d: &str| PairResult {
        ok: false,
        detail: d.to_string(),
    };
    let master_key = master_key.trim();
    if master_key.is_empty() {
        return err("Master key is empty");
    }
    let mut cfg = config(settings);
    let base = base_of(&cfg);
    let client = Client::new();

    let list = match get(&client, &format!("{base}/api/v1/workspaces/"), 10)
        .bearer_auth(master_key)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return err(match net_detail(&e) {
                "Network error" => "Network error talking to agent-platform",
                _ => "agent-platform is not reachable. Start it first",
            })
        }
    };
    if matches!(
        list.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return err("Master key rejected");
    }
    if !list.status().is_success() {
        return err("agent-platform could not list workspaces");
    }
    let body: Value = list.json().await.unwrap_or(Value::Null);
    let Some(ws) = first_workspace_id(&body) else {
        return err("No workspaces exist on this install. Create one in agent-platform first");
    };

    let mint = match client
        .post(format!("{base}/api/v1/workspaces/{ws}/api-tokens/"))
        .header("X-Agent-Platform-Client", DESKTOP_CLIENT_ID)
        .bearer_auth(master_key)
        .json(&json!({ "name": "portal", "scopes": ["*"] }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return err("Network error while minting the token"),
    };
    if matches!(
        mint.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return err("Master key rejected");
    }
    if !mint.status().is_success() {
        return err("agent-platform refused to mint a token");
    }
    let body: Value = mint.json().await.unwrap_or(Value::Null);
    let Some(token) = minted_token(&body) else {
        return err("agent-platform returned no token");
    };

    cfg.api_key = Some(token);
    cfg.enabled = true;
    match settings.save_provider_config(cfg) {
        // The command re-registers the live provider with the saved config.
        Ok(()) => PairResult {
            ok: true,
            detail: "Paired: a new token was saved".to_string(),
        },
        Err(_) => err("Token minted but saving settings failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_pairs_only_the_default_local_daemon() {
        let mut h = PlatformHealth {
            base_url: DEFAULT_PLATFORM_BASE.to_string(),
            reachable: true,
            auth_required: true,
            authenticated: false,
            detail: None,
            installed: true,
        };
        assert!(can_self_pair(&h));
        h.authenticated = true;
        assert!(!can_self_pair(&h));
        h.authenticated = false;
        h.base_url = "http://192.168.1.5:18410".to_string();
        assert!(!can_self_pair(&h));
    }

    #[test]
    fn parses_workspace_and_token_bodies() {
        assert_eq!(
            first_workspace_id(&json!({"workspaces":[{"id":7},{"id":9}]})),
            Some(7)
        );
        assert_eq!(first_workspace_id(&json!([{"id":3}])), Some(3));
        assert_eq!(first_workspace_id(&json!({"workspaces":[]})), None);
        assert_eq!(
            minted_token(&json!({"token":" agp_live_x "})).as_deref(),
            Some("agp_live_x")
        );
        assert_eq!(minted_token(&json!({"token":""})), None);
    }
}
