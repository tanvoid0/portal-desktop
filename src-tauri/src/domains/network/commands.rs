use crate::database::DatabaseManager;
use crate::domains::network::services::network_service::get_passcode_store;
use crate::entities::device_approval;
use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tauri::command;
use uuid::Uuid;

// Import logger macros
#[allow(unused_imports)]
use crate::{log_debug, log_error, log_info, log_warn};

/// Get the local network IP address (non-loopback, private network preferred)
/// Returns the first available IPv4 address that is not loopback
#[command]
pub fn get_local_network_ip() -> Result<String, String> {
    // Try to get network interfaces
    let interfaces = get_network_interfaces();

    // Prefer private network ranges
    let private_ranges = [
        // 192.168.x.x
        (
            Ipv4Addr::new(192, 168, 0, 0),
            Ipv4Addr::new(192, 168, 255, 255),
        ),
        // 10.x.x.x
        (Ipv4Addr::new(10, 0, 0, 0), Ipv4Addr::new(10, 255, 255, 255)),
        // 172.16.x.x - 172.31.x.x
        (
            Ipv4Addr::new(172, 16, 0, 0),
            Ipv4Addr::new(172, 31, 255, 255),
        ),
    ];

    // First, try to find an IP in private ranges
    for (start, end) in &private_ranges {
        for ip in &interfaces {
            if let IpAddr::V4(ipv4) = ip {
                if is_in_range(*ipv4, *start, *end) && !ipv4.is_loopback() {
                    return Ok(ipv4.to_string());
                }
            }
        }
    }

    // If no private IP found, return first non-loopback IPv4
    for ip in &interfaces {
        if let IpAddr::V4(ipv4) = ip {
            if !ipv4.is_loopback() && !ipv4.is_link_local() {
                return Ok(ipv4.to_string());
            }
        }
    }

    // Fallback: return localhost
    Ok("127.0.0.1".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratePasscodeRequest {
    pub device_id: String,
    pub device_name: String,
    pub device_info: String, // JSON string with device details
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratePasscodeResponse {
    pub passcode: String,
    pub expires_in_seconds: i64,
}

/// Generate a 6-digit passcode for device authentication
#[command]
pub async fn generate_device_passcode(
    request: GeneratePasscodeRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<GeneratePasscodeResponse, String> {
    log_info!(
        "device_auth",
        "Generating passcode for device_id: {}",
        request.device_id
    );

    let store = get_passcode_store();
    let passcode = store.generate_passcode();
    store
        .store_passcode(request.device_id.clone(), passcode.clone())
        .await;

    // Store device approval request in database
    let conn = db.get_connection_clone();

    // Check if there's already a pending approval for this device_id
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let existing = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(&request.device_id))
        .filter(device_approval::Column::Approved.eq(false))
        .filter(device_approval::Column::PasscodeExpiresAt.gt(now))
        .order_by_desc(device_approval::Column::CreatedAt)
        .one(&conn)
        .await
        .map_err(|e| {
            let err_msg = format!("Database error: {}", e);
            log_error!("device_auth", "{}", err_msg);
            err_msg
        })?;

    if let Some(existing_device) = existing {
        log_info!("device_auth", "Found existing pending approval for device_id: {}, updating instead of creating duplicate", request.device_id);

        // Update existing pending approval instead of creating a duplicate
        let mut device: device_approval::ActiveModel = existing_device.into();
        device.device_name = Set(request.device_name);
        device.device_info = Set(request.device_info);
        device.passcode = Set(hash_passcode(&passcode));
        device.passcode_expires_at = Set((Utc::now() + Duration::minutes(5)).into());
        device.updated_at = Set(Utc::now().into());

        device_approval::Entity::update(device)
            .exec(&conn)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to update device approval: {}", e);
                log_error!("device_auth", "{}", err_msg);
                err_msg
            })?;

        log_info!(
            "device_auth",
            "Updated existing approval for device_id: {}",
            request.device_id
        );
    } else {
        log_info!(
            "device_auth",
            "No existing pending approval found, creating new one for device_id: {}",
            request.device_id
        );

        // Create new approval request
        let approval_id = Uuid::new_v4().to_string();

        let approval = device_approval::ActiveModel {
            id: Set(approval_id.clone()),
            device_id: Set(request.device_id.clone()),
            device_name: Set(request.device_name),
            device_info: Set(request.device_info),
            passcode: Set(hash_passcode(&passcode)),
            passcode_expires_at: Set((Utc::now() + Duration::minutes(5)).into()),
            approved: Set(false),
            approval_type: Set("pending".to_string()),
            approved_at: Set(None),
            expires_at: Set(None),
            access_token: Set(None),
            token_expires_at: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            last_used_at: Set(None),
        };

        device_approval::Entity::insert(approval)
            .exec(&conn)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to store device approval: {}", e);
                log_error!("device_auth", "{}", err_msg);
                err_msg
            })?;

        log_info!(
            "device_auth",
            "Created new approval request id: {} for device_id: {}",
            approval_id,
            request.device_id
        );
    }

    log_info!(
        "device_auth",
        "Passcode generated successfully for device_id: {}",
        request.device_id
    );

    Ok(GeneratePasscodeResponse {
        passcode,
        expires_in_seconds: 300, // 5 minutes
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyPasscodeRequest {
    pub device_id: String,
    pub passcode: String,
    pub approval_type: String, // "temporary" or "long_term"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyPasscodeResponse {
    pub approved: bool,
    pub access_token: Option<String>,
    pub expires_at: Option<String>,
    pub message: String,
}

/// Verify passcode and approve device (requires user confirmation in UI)
#[command]
pub async fn verify_device_passcode(
    request: VerifyPasscodeRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<VerifyPasscodeResponse, String> {
    // Verify passcode
    let store = get_passcode_store();
    if !store
        .verify_passcode(&request.device_id, &request.passcode)
        .await
    {
        return Ok(VerifyPasscodeResponse {
            approved: false,
            access_token: None,
            expires_at: None,
            message: "Invalid or expired passcode".to_string(),
        });
    }

    // Remove passcode after verification
    store.remove_passcode(&request.device_id).await;

    // Check if device already exists in database
    let conn = db.get_connection_clone();
    let device = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(&request.device_id))
        .one(&conn)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if device.is_none() {
        return Ok(VerifyPasscodeResponse {
            approved: false,
            access_token: None,
            expires_at: None,
            message: "Device approval request not found".to_string(),
        });
    }

    // Return pending status - actual approval happens via approve_device command
    Ok(VerifyPasscodeResponse {
        approved: false,
        access_token: None,
        expires_at: None,
        message: "Passcode verified. Waiting for host approval.".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveDeviceRequest {
    pub device_id: String,
    pub approval_type: String, // "temporary" or "long_term"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveDeviceResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub expires_at: Option<String>,
    pub message: String,
}

/// Approve a device (called after user confirms in UI)
#[command]
pub async fn approve_device(
    request: ApproveDeviceRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<ApproveDeviceResponse, String> {
    log_info!(
        "device_auth",
        "Approving device_id: {} with type: {}",
        request.device_id,
        request.approval_type
    );

    let conn = db.get_connection_clone();

    // Find all pending device approvals for this device_id (handle duplicates)
    let devices = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(&request.device_id))
        .filter(device_approval::Column::Approved.eq(false))
        .order_by_desc(device_approval::Column::CreatedAt)
        .all(&conn)
        .await
        .map_err(|e| {
            let err_msg = format!("Database error: {}", e);
            log_error!("device_auth", "{}", err_msg);
            err_msg
        })?;

    if devices.is_empty() {
        let err_msg = format!(
            "Device approval not found for device_id: {}",
            request.device_id
        );
        log_warn!("device_auth", "{}", err_msg);
        return Err(err_msg);
    }

    log_info!(
        "device_auth",
        "Found {} pending approval(s) for device_id: {}",
        devices.len(),
        request.device_id
    );

    // Calculate expiration (1 month max, or shorter for temporary)
    let expires_at = if request.approval_type == "temporary" {
        Some((Utc::now() + Duration::hours(24)).into()) // 24 hours for temporary
    } else {
        Some((Utc::now() + Duration::days(30)).into()) // 30 days for long-term
    };

    // Generate access token
    let access_token = generate_access_token(&request.device_id);
    let token_expires_at = expires_at;

    // Update all pending approvals for this device_id to prevent duplicates from showing up
    for device in &devices {
        let mut device_model: device_approval::ActiveModel = device.clone().into();
        device_model.approved = Set(true);
        device_model.approval_type = Set(request.approval_type.clone());
        device_model.approved_at = Set(Some(Utc::now().into()));
        device_model.expires_at = Set(expires_at);
        device_model.access_token = Set(Some(access_token.clone()));
        device_model.token_expires_at = Set(token_expires_at);
        device_model.updated_at = Set(Utc::now().into());

        device_approval::Entity::update(device_model)
            .exec(&conn)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to update device approval: {}", e);
                log_error!("device_auth", "{}", err_msg);
                err_msg
            })?;
    }

    log_info!(
        "device_auth",
        "Successfully approved {} device approval(s) for device_id: {}",
        devices.len(),
        request.device_id
    );

    Ok(ApproveDeviceResponse {
        success: true,
        access_token: Some(access_token),
        expires_at: expires_at.map(|dt| dt.to_rfc3339()),
        message: "Device approved successfully".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenResponse {
    pub valid: bool,
    pub device_id: Option<String>,
    pub message: String,
}

/// Verify access token for API requests
#[command]
pub async fn verify_access_token(
    request: VerifyTokenRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<VerifyTokenResponse, String> {
    let conn = db.get_connection_clone();

    // Find device by access token
    let device = device_approval::Entity::find()
        .filter(device_approval::Column::AccessToken.eq(&request.access_token))
        .one(&conn)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(device) = device {
        // Check if approved and not expired
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let device_id = device.device_id.clone();
        if device.approved
            && device.expires_at.map(|e| e > now).unwrap_or(false)
            && device.token_expires_at.map(|e| e > now).unwrap_or(false)
        {
            // Update last_used_at
            let mut device_active: device_approval::ActiveModel = device.into();
            device_active.last_used_at = Set(Some(Utc::now().into()));
            device_approval::Entity::update(device_active)
                .exec(&conn)
                .await
                .ok(); // Don't fail if update fails

            return Ok(VerifyTokenResponse {
                valid: true,
                device_id: Some(device_id),
                message: "Token is valid".to_string(),
            });
        }
    }

    Ok(VerifyTokenResponse {
        valid: false,
        device_id: None,
        message: "Invalid or expired token".to_string(),
    })
}

/// Get device approval status by device_id (for polling)
#[derive(Debug, Serialize, Deserialize)]
pub struct GetDeviceStatusRequest {
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDeviceStatusResponse {
    pub approved: bool,
    pub access_token: Option<String>,
    pub expires_at: Option<String>,
    pub message: String,
}

/// Get device approval status (allows device to poll for approval)
#[command]
pub async fn get_device_status(
    request: GetDeviceStatusRequest,
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<GetDeviceStatusResponse, String> {
    let conn = db.get_connection_clone();

    let device = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(&request.device_id))
        .one(&conn)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(device) = device {
        if device.approved && device.access_token.is_some() {
            // Check if not expired
            let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
            if device.expires_at.map(|e| e > now).unwrap_or(false) {
                return Ok(GetDeviceStatusResponse {
                    approved: true,
                    access_token: device.access_token,
                    expires_at: device.expires_at.map(|dt| dt.to_rfc3339()),
                    message: "Device is approved".to_string(),
                });
            } else {
                return Ok(GetDeviceStatusResponse {
                    approved: false,
                    access_token: None,
                    expires_at: None,
                    message: "Device approval has expired".to_string(),
                });
            }
        } else {
            return Ok(GetDeviceStatusResponse {
                approved: false,
                access_token: None,
                expires_at: None,
                message: "Device is pending approval".to_string(),
            });
        }
    }

    Ok(GetDeviceStatusResponse {
        approved: false,
        access_token: None,
        expires_at: None,
        message: "Device not found".to_string(),
    })
}

/// Get pending device approvals (for UI display)
#[command]
pub async fn get_pending_device_approvals(
    db: tauri::State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    log_info!("device_auth", "Fetching pending device approvals");

    let conn = db.get_connection_clone();

    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    let devices = device_approval::Entity::find()
        .filter(device_approval::Column::Approved.eq(false))
        .filter(device_approval::Column::PasscodeExpiresAt.gt(now))
        .order_by_desc(device_approval::Column::CreatedAt)
        .all(&conn)
        .await
        .map_err(|e| {
            let err_msg = format!("Database error: {}", e);
            log_warn!("device_auth", "{}", err_msg);
            err_msg
        })?;

    log_info!(
        "device_auth",
        "Found {} total pending device approval(s)",
        devices.len()
    );

    // Deduplicate by device_id - keep only the most recent one per device_id
    use std::collections::HashMap;
    let mut device_map: HashMap<String, &device_approval::Model> = HashMap::new();

    for device in &devices {
        let device_id = &device.device_id;
        // Only keep the first (most recent) one for each device_id
        device_map.entry(device_id.clone()).or_insert(device);
    }

    log_info!(
        "device_auth",
        "After deduplication: {} unique device approval(s)",
        device_map.len()
    );

    if device_map.len() < devices.len() {
        log_warn!(
            "device_auth",
            "Found {} duplicate device approval(s) - they will be cleaned up on next approval",
            devices.len() - device_map.len()
        );
    }

    let result: Vec<serde_json::Value> = device_map
        .values()
        .map(|d| serde_json::json!({
            "device_id": d.device_id,
            "device_name": d.device_name,
            "device_info": serde_json::from_str::<serde_json::Value>(&d.device_info).unwrap_or(serde_json::json!({})),
            "created_at": d.created_at.to_rfc3339(),
        }))
        .collect();

    Ok(result)
}

fn hash_passcode(passcode: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(passcode.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}

fn generate_access_token(device_id: &str) -> String {
    let payload = format!("{}:{}", device_id, Utc::now().timestamp());
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}

fn is_in_range(ip: Ipv4Addr, start: Ipv4Addr, end: Ipv4Addr) -> bool {
    let ip_u32 = u32::from(ip);
    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);
    ip_u32 >= start_u32 && ip_u32 <= end_u32
}

fn get_network_interfaces() -> Vec<IpAddr> {
    let mut addresses = Vec::new();

    // Use platform-specific methods to get network interfaces
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::io::{BufRead, BufReader};

        // Read /proc/net/route to find active interfaces
        if let Ok(file) = fs::File::open("/proc/net/route") {
            let reader = BufReader::new(file);
            let mut interfaces = Vec::new();

            for line in reader.lines().skip(1) {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 0 {
                        interfaces.push(parts[0].to_string());
                    }
                }
            }

            // Get IP addresses for each interface
            for interface in interfaces {
                if let Ok(_addr_file) =
                    fs::File::open(format!("/sys/class/net/{}/address", interface))
                {
                    // Interface exists, try to get its IP
                    if let Ok(ifconfig_output) = std::process::Command::new("ip")
                        .args(&["addr", "show", &interface])
                        .output()
                    {
                        let output = String::from_utf8_lossy(&ifconfig_output.stdout);
                        for line in output.lines() {
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
    }

    #[cfg(target_os = "macos")]
    {
        // Use ifconfig on macOS
        if let Ok(output) = std::process::Command::new("ifconfig").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("inet ") && !line.contains("127.0.0.1") && !line.contains("::1") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for (i, part) in parts.iter().enumerate() {
                        if part == &"inet" && i + 1 < parts.len() {
                            let ip_str = parts[i + 1];
                            if let Ok(ip) = ip_str.parse::<IpAddr>() {
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
        // Use ipconfig on Windows
        if let Ok(output) = std::process::Command::new("ipconfig").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut current_section = String::new();

            for line in output_str.lines() {
                if line.contains("adapter") {
                    current_section = line.to_string();
                } else if line.contains("IPv4") || line.contains("IPv4 Address") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let ip_str = parts[1].trim();
                        if let Ok(ip) = ip_str.parse::<IpAddr>() {
                            if !ip.is_loopback() {
                                addresses.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: try to use a simple method that works cross-platform
    if addresses.is_empty() {
        // Try to connect to a remote address to determine local IP
        // This is a fallback method
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            // Try to connect to a public DNS server (doesn't actually connect)
            if let Err(_) = socket.connect("8.8.8.8:80") {
                // This is expected, but we can get the local address
            }
            if let Ok(addr) = socket.local_addr() {
                if let IpAddr::V4(ipv4) = addr.ip() {
                    if !ipv4.is_loopback() {
                        addresses.push(IpAddr::V4(ipv4));
                    }
                }
            }
        }
    }

    addresses
}
