//! Health, start and pairing for the local agent-platform server.
//!
//! Starting it means starting the **Agent Platform app**, which owns the
//! `agent-platformd` child — the same way Portal starts Docker Desktop and not
//! `dockerd`. See [`exe_path`] for why Portal must not spawn the daemon itself.
//!
//! Secrets: the master key and the minted token only ever travel in an
//! `Authorization` header or the settings file. No `detail` string or error here
//! interpolates a reqwest error, response body, header or either secret.

use super::platform_config::{DEFAULT_PLATFORM_BASE, DESKTOP_CLIENT_ID};
use super::providers::{ProviderConfig, ProviderType};
use super::services::AISettingsService;
use crate::process_ext::NoWindowExt;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

/// Where to get the app when [`exe_path`] finds nothing installed.
pub const RELEASES_URL: &str = "https://github.com/tanvoid0/agent-platform/releases/latest";

#[derive(Debug, Clone, Serialize)]
pub struct PlatformHealth {
    pub base_url: String,
    pub reachable: bool,
    pub auth_required: bool,
    pub authenticated: bool,
    pub detail: Option<String>,
    /// Whether the Agent Platform app is installed, independent of `reachable`.
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartResult {
    pub started: bool,
    pub already_running: bool,
    pub pid: Option<u32>,
    pub detail: String,
    /// Set instead of launching when the app is not installed.
    pub install_url: Option<String>,
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
        installed: is_installed(),
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
    let token = c.api_key.as_deref().map(str::trim).filter(|s| !s.is_empty());
    probe(&base_of(&c), token).await
}

/// The **app**, not the daemon: `AGENT_PLATFORM_APP_EXE` override, else
/// `%LOCALAPPDATA%\Programs\agent-platform\agent-platform.exe`.
///
/// Portal must never spawn `agent-platformd` itself. The daemon takes its
/// database path, master key, workspace root, media and config dirs entirely
/// from the environment; spawned bare it falls back to its own defaults —
/// `data/agent_platform.db` resolved against *our* working directory, and no
/// master key at all, which is an open server. It then binds the same fixed
/// port, so the app finds it, reads it as one of ours and attaches to it as a
/// pure client — now serving a database that is not the user's.
///
/// The app is the only thing that knows that environment. It also holds the
/// single-instance lock and adopts a daemon already on the port instead of
/// starting a second one, which is exactly what we want from here.
fn exe_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("AGENT_PLATFORM_APP_EXE") {
        return Some(PathBuf::from(p));
    }
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("Programs/agent-platform/agent-platform.exe"))
}

fn is_installed() -> bool {
    exe_path().filter(|p| p.is_file()).is_some()
}

pub async fn start_platform(settings: &AISettingsService) -> StartResult {
    let fail = |detail: String| StartResult {
        started: false,
        already_running: false,
        pid: None,
        detail,
        install_url: None,
    };
    let base = base_of(&config(settings));
    if probe(&base, None).await.reachable {
        return StartResult {
            started: false,
            already_running: true,
            pid: None,
            detail: "agent-platform is already running".to_string(),
            install_url: None,
        };
    }

    let Some(exe) = exe_path().filter(|p| p.is_file()) else {
        return StartResult {
            started: false,
            already_running: false,
            pid: None,
            detail: "The Agent Platform app is not installed".to_string(),
            install_url: Some(RELEASES_URL.to_string()),
        };
    };

    let mut cmd = std::process::Command::new(&exe);
    // Tray-first, like Docker Desktop: the app comes up minimized with its
    // server running rather than stealing focus from Portal.
    cmd.arg("--minimized")
        .no_window()
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return fail(format!("Could not launch {}: {}", exe.display(), e.kind())),
    };
    let pid = child.id();
    // Detached: dropping a std Child neither kills nor waits on it.
    drop(child);

    // The app has to boot, take its single-instance lock, then start the daemon,
    // which runs migrations on a first launch.
    for _ in 0..120 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if probe(&base, None).await.reachable {
            return StartResult {
                started: true,
                already_running: false,
                pid: Some(pid),
                detail: format!("Started {}", exe.display()),
                install_url: None,
            };
        }
    }
    StartResult {
        started: false,
        already_running: false,
        pid: Some(pid),
        detail: format!("Launched {} but /health did not answer within 60s", exe.display()),
        install_url: None,
    }
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
