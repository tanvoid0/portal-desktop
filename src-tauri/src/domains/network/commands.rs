//! Thin Tauri adapters for the `network` domain.
//!
//! The pairing logic lives in `portal_core::domains::network::services::
//! pairing_service` as free functions over a `&DatabaseConnection`. These
//! adapters are what the webview invokes over IPC; the daemon's `/pair/*` routes
//! call the same free functions directly (phase 4 of
//! `docs/development/REMOTE_ARCHITECTURE.md`).
//!
//! `verify_device_passcode`, `get_device_status` and `verify_access_token` used
//! to live here as commands; they are now daemon routes (`/pair/verify`,
//! `/pair/status`, and bearer auth) and are gone from the IPC surface.
//!
//! None of these commands carry a `cap`: pairing is loopback/IPC only. A phone
//! pairs through the daemon routes, and granting a phone the power to approve
//! *other* devices or read the pending list would defeat the point.

use crate::database::DatabaseManager;
use crate::domains::network::services::network_service::get_passcode_store;
use crate::domains::network::services::pairing_service::{self, SHARED_PASSCODE_KEY};
use crate::entities::device_approval;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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
pub struct GeneratePasscodeRequest {
    pub device_id: String,
    pub device_name: String,
    pub device_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratePasscodeResponse {
    pub passcode: String,
    pub expires_in_seconds: i64,
}

/// Mint the single shared pairing passcode the QR code carries. Any device that
/// presents it registers itself as pending via the daemon's `/pair/verify`.
#[portal_rpc_macro::rpc]
pub async fn generate_device_passcode(
    _request: GeneratePasscodeRequest,
    _db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<GeneratePasscodeResponse, String> {
    let store = get_passcode_store();
    let passcode = store.generate_passcode();
    store
        .store_passcode(SHARED_PASSCODE_KEY.to_string(), passcode.clone())
        .await;

    log_info!("device_auth", "Generated new shared device passcode");

    Ok(GeneratePasscodeResponse {
        passcode,
        expires_in_seconds: 300,
    })
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

/// Approve a pending device from the host UI, granting the chosen capabilities.
/// The token is minted and staged for the device to collect over `/pair/status`
/// — it is never returned here.
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

    Ok(ApproveDeviceResponse {
        success: outcome.success,
        expires_at: outcome.expires_at,
        message: outcome.message,
    })
}

/// Pending (unapproved, unexpired) device requests, for the host approval UI.
/// Deduplicated to the most recent row per device_id.
#[portal_rpc_macro::rpc]
pub async fn get_pending_device_approvals(
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.get_connection();

    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let devices = device_approval::Entity::find()
        .filter(device_approval::Column::Approved.eq(false))
        .filter(device_approval::Column::PasscodeExpiresAt.gt(now))
        .order_by_desc(device_approval::Column::CreatedAt)
        .all(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    use std::collections::HashMap;
    let mut by_device: HashMap<String, &device_approval::Model> = HashMap::new();
    for device in &devices {
        by_device.entry(device.device_id.clone()).or_insert(device);
    }

    let result = by_device
        .values()
        .map(|d| {
            serde_json::json!({
                "device_id": d.device_id,
                "device_name": d.device_name,
                "device_info": serde_json::from_str::<serde_json::Value>(&d.device_info)
                    .unwrap_or_else(|_| serde_json::json!({})),
                "created_at": d.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(result)
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
