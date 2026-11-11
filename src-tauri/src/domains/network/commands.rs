//! Thin Tauri adapters for the `network` domain.
//!
//! The pairing logic lives in `portal_core::domains::network::services::
//! pairing_service` as free functions over a `&DatabaseConnection`. These
//! adapters are what the webview invokes over IPC; the daemon's `/pair/request`
//! and `/control` routes call the same free functions directly (phase 1 of
//! the Android remote-control rewrite; see `docs/development/
//! REMOTE_ARCHITECTURE.md` for the phases before it).
//!
//! None of these commands carry a `cap`: device management is loopback/IPC
//! only. A phone pairs through the daemon routes, and granting a phone the
//! power to approve/revoke *other* devices or read the device list would
//! defeat the point.

use crate::daemon::pairing::{resolve_pending, PairResolution};
use crate::database::DatabaseManager;
use crate::domains::network::services::pairing_service;
use crate::domains::settings::services::settings_service::SettingsService;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

#[allow(unused_imports)]
use crate::{log_debug, log_error, log_info, log_warn};

/// Get the local network IP: the address this host would use to reach the
/// internet, found by asking the OS for the source address of a UDP socket
/// "connected" to a public IP (no packet is sent). This picks the real LAN
/// adapter rather than a WSL/Docker/VM virtual one that interface enumeration
/// might list first. Falls back to scanning interfaces, then to loopback.
#[portal_rpc_macro::rpc]
pub fn get_local_network_ip() -> Result<String, String> {
    if let Some(ip) = primary_route_ip() {
        return Ok(ip.to_string());
    }

    let interfaces = get_network_interfaces();

    let private_ranges = [
        (
            Ipv4Addr::new(192, 168, 0, 0),
            Ipv4Addr::new(192, 168, 255, 255),
        ),
        (Ipv4Addr::new(10, 0, 0, 0), Ipv4Addr::new(10, 255, 255, 255)),
        (
            Ipv4Addr::new(172, 16, 0, 0),
            Ipv4Addr::new(172, 31, 255, 255),
        ),
    ];

    for (start, end) in &private_ranges {
        for ip in &interfaces {
            if let IpAddr::V4(ipv4) = ip {
                if is_in_range(*ipv4, *start, *end) && !ipv4.is_loopback() {
                    return Ok(ipv4.to_string());
                }
            }
        }
    }

    for ip in &interfaces {
        if let IpAddr::V4(ipv4) = ip {
            if !ipv4.is_loopback() && !ipv4.is_link_local() {
                return Ok(ipv4.to_string());
            }
        }
    }

    Ok("127.0.0.1".to_string())
}

/// The source address the OS would use to reach the public internet. Connecting
/// a UDP socket does not send anything; it just resolves the route.
fn primary_route_ip() -> Option<Ipv4Addr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() && !ip.is_link_local() => {
            Some(ip)
        }
        _ => None,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveDeviceRequest {
    pub device_id: String,
    pub approval_type: String, // "temporary" or "long_term"
    /// Capabilities to grant this device. The approval dialog fills this in; an
    /// empty list grants nothing.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveDeviceResponse {
    pub success: bool,
    pub expires_at: Option<String>,
    pub message: String,
}

/// Approve a pending device from the host UI, granting the chosen
/// capabilities. Also wakes up a long-polling `/pair/request`, if this device
/// came in through that path rather than a QR scan — `resolve_pending` is a
/// no-op when there is no pending request for this device_id.
#[portal_rpc_macro::rpc]
pub async fn approve_device(
    request: ApproveDeviceRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<ApproveDeviceResponse, String> {
    log_info!(
        "device_auth",
        "Approving device_id: {} ({}) with caps: {:?}",
        request.device_id,
        request.approval_type,
        request.capabilities
    );

    let outcome = pairing_service::approve(
        db.get_connection(),
        &request.device_id,
        &request.approval_type,
        &request.capabilities,
    )
    .await?;

    resolve_pending(
        &request.device_id,
        PairResolution::Approved {
            token: outcome.token.clone(),
            host_name: crate::daemon::host_name(),
        },
    );

    Ok(ApproveDeviceResponse {
        success: outcome.success,
        expires_at: outcome.expires_at,
        message: outcome.message,
    })
}

/// Decline a pending `/pair/request` from the host UI (or let it time out on
/// its own — the daemon calls `pairing_service::reject` for that case too).
#[portal_rpc_macro::rpc]
pub async fn reject_device(
    device_id: String,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    log_info!("device_auth", "Rejecting device_id: {}", device_id);
    pairing_service::reject(db.get_connection(), &device_id).await?;
    resolve_pending(&device_id, PairResolution::Denied);
    Ok(())
}

/// End a device's access — the only way to stop an `until_revoked` grant.
#[portal_rpc_macro::rpc]
pub async fn revoke_device(
    device_id: String,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    log_info!("device_auth", "Revoking device_id: {}", device_id);
    pairing_service::revoke(db.get_connection(), &device_id).await
}

/// Change a device's granted capabilities without re-minting its token.
#[portal_rpc_macro::rpc]
pub async fn update_device_capabilities(
    device_id: String,
    capabilities: Vec<String>,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<(), String> {
    pairing_service::update_capabilities(db.get_connection(), &device_id, &capabilities).await
}

/// Every device the host has ever seen, most-recent row per device_id, for
/// the Devices management page.
#[portal_rpc_macro::rpc]
pub async fn list_devices(
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<pairing_service::DeviceSummary>, String> {
    pairing_service::list_devices(db.get_connection()).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairingQrResponse {
    pub qr_url: String,
    pub device_id: String,
}

/// Mint an until-revoked grant and build the `portalremote://` URL its QR
/// code carries — scanning it hands the phone a live, already-approved
/// credential with no server round trip (unlike `/pair/request`, which this
/// skips entirely; see the module doc on `pairing_service`).
#[portal_rpc_macro::rpc]
pub async fn generate_pairing_qr(
    capabilities: Vec<String>,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<PairingQrResponse, String> {
    let (device_id, token) =
        pairing_service::mint_qr_pairing(db.get_connection(), &capabilities).await?;
    let host = get_local_network_ip()?;
    let port = crate::daemon::configured_port();
    Ok(PairingQrResponse {
        qr_url: format!("portalremote://{host}:{port}/{token}"),
        device_id,
    })
}

/// Every capability name a device can be granted, for the pairing/capability
/// checkbox UI. Backed by the same registry `/rpc/{command}` gates against,
/// so this list can never drift from what capabilities actually do something.
#[portal_rpc_macro::rpc]
pub fn remote_list_capabilities() -> Result<Vec<String>, String> {
    Ok(crate::rpc::capabilities()
        .into_iter()
        .map(String::from)
        .collect())
}

/// Whether the LAN listener and UDP discovery responder are enabled. See
/// `set_remote_lan_enabled` for why toggling this does not take effect until
/// the next restart.
#[portal_rpc_macro::rpc]
pub fn get_remote_lan_enabled() -> Result<bool, String> {
    SettingsService::new().get_remote_lan_enabled()
}

/// Persist the LAN toggle — checked by `daemon::spawn` at startup.
#[portal_rpc_macro::rpc]
pub fn set_remote_lan_enabled(enabled: bool) -> Result<(), String> {
    SettingsService::new().set_remote_lan_enabled(enabled)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteConnectivityStatus {
    pub lan_enabled: bool,
    pub firewall_rule_present: bool,
    pub port: u16,
    /// Name of a connected VPN-looking network adapter, if any — see
    /// `daemon::firewall::active_vpn_adapter`'s doc for why this is
    /// report-only (no "fix" an app can apply to someone else's VPN client).
    pub vpn_adapter: Option<String>,
}

/// Verifiable facts about whether a phone on the LAN could actually reach
/// this daemon — deliberately not a reachability probe. A same-host TCP
/// self-connect to this machine's own LAN IP would succeed via Windows'
/// loopback short-circuit regardless of whether an external device could
/// get in, which is exactly the misleading "all clear" this is meant to
/// avoid; see `daemon::firewall`'s module doc. Untyped on the wire: this
/// crate's structs don't derive `TS` (that only works in `portal-core`), so
/// the frontend hand-types this.
#[portal_rpc_macro::rpc]
pub fn get_remote_connectivity_status() -> Result<RemoteConnectivityStatus, String> {
    let firewall_rule_present = crate::daemon::firewall::current_exe_path()
        .map(|exe| crate::daemon::firewall::rule_present(&exe))
        .unwrap_or(false);
    Ok(RemoteConnectivityStatus {
        lan_enabled: SettingsService::new().get_remote_lan_enabled()?,
        firewall_rule_present,
        port: crate::daemon::configured_port(),
        vpn_adapter: crate::daemon::firewall::active_vpn_adapter(),
    })
}

fn is_in_range(ip: Ipv4Addr, start: Ipv4Addr, end: Ipv4Addr) -> bool {
    let ip = u32::from(ip);
    u32::from(start) <= ip && ip <= u32::from(end)
}

fn get_network_interfaces() -> Vec<IpAddr> {
    let mut addresses = Vec::new();

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::io::{BufRead, BufReader};

        if let Ok(file) = fs::File::open("/proc/net/route") {
            let reader = BufReader::new(file);
            let mut interfaces = Vec::new();
            for line in reader.lines().skip(1).map_while(Result::ok) {
                if let Some(iface) = line.split_whitespace().next() {
                    interfaces.push(iface.to_string());
                }
            }
            for interface in interfaces {
                if let Ok(output) = std::process::Command::new("ip")
                    .args(["addr", "show", &interface])
                    .output()
                {
                    let out = String::from_utf8_lossy(&output.stdout);
                    for line in out.lines() {
                        if line.contains("inet ") && !line.contains("127.0.0.1") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let ip_str = parts[1].split('/').next().unwrap_or("");
                                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                    addresses.push(ip);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ifconfig").output() {
            let out = String::from_utf8_lossy(&output.stdout);
            for line in out.lines() {
                if line.contains("inet ") && !line.contains("127.0.0.1") && !line.contains("::1") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for (i, part) in parts.iter().enumerate() {
                        if *part == "inet" && i + 1 < parts.len() {
                            if let Ok(ip) = parts[i + 1].parse::<IpAddr>() {
                                addresses.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = {
            use crate::process_ext::NoWindowExt;
            std::process::Command::new("ipconfig").no_window().output()
        } {
            let out = String::from_utf8_lossy(&output.stdout);
            for line in out.lines() {
                if line.contains("IPv4") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        if let Ok(ip) = parts[1].trim().parse::<IpAddr>() {
                            if !ip.is_loopback() {
                                addresses.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }

    addresses
}
