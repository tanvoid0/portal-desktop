//! Pairing and access tokens for the LAN/remote-control surface — the Android
//! contract's `/pair/request` long-poll and `/control` bearer auth call
//! these, and so do the Tauri command adapters, so there is one
//! implementation and neither transport owns it. Nothing here touches Tauri
//! — it lives in `portal-core`.
//!
//! # Approval types
//!
//! - `"temporary"`: 24 hours.
//! - `"long_term"`: 30 days.
//! - `"until_revoked"`: no expiry — `expires_at`/`token_expires_at` of `None`
//!   mean "live forever" everywhere liveness is checked. Ends only via
//!   [`revoke`].
//!
//! # QR pairing
//!
//! Scanning a QR code hands a phone a token with no server round trip at all,
//! so the token must already be live the moment the QR is drawn.
//! [`mint_qr_pairing`] mints an already-approved `until_revoked` grant, but —
//! since nobody clicked "allow" on the phone for it — gives it a 5-minute
//! pickup `expires_at`: a leaked or unscanned QR image grants nothing. The
//! first device to actually authenticate with the token (in [`verify_token`])
//! has that expiry cleared in the same write, converting it to a permanent
//! grant.
//!
//! # Security model
//!
//! - **The token is 32 random bytes, base64url.** The database stores only its
//!   SHA-256 and auth looks a device up by that hash, so a leak of the database
//!   never yields a usable token.
//! - Comparisons of stored hashes are constant-time.

use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rand::RngCore;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entities::device_approval;

/// Access-token lifetime for a one-off ("temporary") approval.
const TEMPORARY_HOURS: i64 = 24;
/// Access-token lifetime for a "long_term" approval.
const LONG_TERM_DAYS: i64 = 30;
/// Pickup window for a QR-minted grant before any device has used it — see
/// `mint_qr_pairing`/`verify_token`.
const QR_PICKUP_TTL_MINUTES: i64 = 5;

/// Hex SHA-256 of a value. Used for the token hash.
pub fn hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// `n` random bytes, base64url (no padding) — the access-token shape.
fn random_b64url(n: usize) -> String {
    let mut buf = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// Grant lifetime for `approval_type`. `None` means "forever" — only
/// [`revoke`] ends an `until_revoked` grant.
fn approval_expiry(approval_type: &str) -> Option<DateTime<FixedOffset>> {
    match approval_type {
        "until_revoked" => None,
        "temporary" => Some((Utc::now() + Duration::hours(TEMPORARY_HOURS)).into()),
        _ => Some((Utc::now() + Duration::days(LONG_TERM_DAYS)).into()),
    }
}

/// Result of an approval (host side).
#[derive(Debug, Serialize)]
pub struct ApproveOutcome {
    pub success: bool,
    pub expires_at: Option<String>,
    /// The plaintext access token, returned once to the caller — same trust
    /// level as it being typed into the approval dialog today.
    pub token: String,
    pub message: String,
}

/// One row of `list_devices`, deduplicated to the most recent per device_id.
#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../../../src/lib/domains/devices/generated/")]
pub struct DeviceSummary {
    pub device_id: String,
    pub device_name: String,
    pub approval_type: String,
    pub approved: bool,
    pub capabilities: Vec<String>,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

/// The most-recently created row for `device_id`, if any.
async fn most_recent_row(
    conn: &DatabaseConnection,
    device_id: &str,
) -> Result<Option<device_approval::Model>, String> {
    device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .order_by_desc(device_approval::Column::CreatedAt)
        .one(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))
}

/// A device's `/pair/request` landing as a pending row for the host to
/// approve or deny. `passcode`/`passcode_expires_at`/`pairing_secret` are
/// vestigial for this flow (the Android client sends neither a passcode nor a
/// secret) — dummy-filled since the first two columns are still NOT NULL.
pub async fn request_pairing(
    conn: &DatabaseConnection,
    device_id: &str,
    device_name: &str,
) -> Result<(), String> {
    let now = Utc::now();
    let approval = device_approval::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        device_id: Set(device_id.to_string()),
        device_name: Set(device_name.to_string()),
        device_info: Set("{}".to_string()),
        passcode: Set(String::new()),
        passcode_expires_at: Set(now.into()),
        approved: Set(false),
        approval_type: Set("pending".to_string()),
        approved_at: Set(None),
        expires_at: Set(None),
        access_token: Set(None),
        token_expires_at: Set(None),
        capabilities: Set("[]".to_string()),
        pairing_secret: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        last_used_at: Set(None),
    };
    device_approval::Entity::insert(approval)
        .exec(conn)
        .await
        .map_err(|e| format!("Failed to store device approval: {e}"))?;
    Ok(())
}

/// Mint an already-approved `until_revoked` grant for a QR code: a fresh
/// device_id and token, with only the token's hash stored. Returns the
/// plaintext once — the QR image carries it from here on, same trust level as
/// today's approval dialog handing a token to the local webview caller.
pub async fn mint_qr_pairing(
    conn: &DatabaseConnection,
    capabilities: &[String],
) -> Result<(String, String), String> {
    let device_id = Uuid::new_v4().to_string();
    let token = random_b64url(32);
    let token_hash = hash(&token);
    let now = Utc::now();
    let expires_at: DateTime<FixedOffset> = (now + Duration::minutes(QR_PICKUP_TTL_MINUTES)).into();
    let caps_json = serde_json::to_string(capabilities).unwrap_or_else(|_| "[]".to_string());

    let approval = device_approval::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        device_id: Set(device_id.clone()),
        device_name: Set("Unnamed device".to_string()),
        device_info: Set("{}".to_string()),
        passcode: Set(String::new()),
        passcode_expires_at: Set(now.into()),
        approved: Set(true),
        approval_type: Set("until_revoked".to_string()),
        approved_at: Set(Some(now.into())),
        expires_at: Set(Some(expires_at)),
        access_token: Set(Some(token_hash)),
        // Stays unset: only `expires_at` carries the pickup TTL, so
        // `verify_token`'s first-use clear (which only touches `expires_at`)
        // is enough on its own to make this permanent.
        token_expires_at: Set(None),
        capabilities: Set(caps_json),
        pairing_secret: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        last_used_at: Set(None),
    };
    device_approval::Entity::insert(approval)
        .exec(conn)
        .await
        .map_err(|e| format!("Failed to store device approval: {e}"))?;

    Ok((device_id, token))
}

/// Approve every pending row for `device_id`, granting `capabilities`. Mints
/// a token and stores only its hash with the caps and expiry.
pub async fn approve(
    conn: &DatabaseConnection,
    device_id: &str,
    approval_type: &str,
    capabilities: &[String],
) -> Result<ApproveOutcome, String> {
    let rows = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .filter(device_approval::Column::Approved.eq(false))
        // Only a row still in its initial "pending" state is actually
        // awaiting a decision. Without this, a row `reject()` already moved
        // to "rejected" (e.g. the 170s `/pair/request` timeout in
        // `daemon::pairing`) still matches `Approved.eq(false)` and a later
        // click on the same stale pending card resurrects it, minting a
        // token for a phone that already gave up — see QA-123.
        .filter(device_approval::Column::ApprovalType.eq("pending"))
        .order_by_desc(device_approval::Column::CreatedAt)
        .all(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    if rows.is_empty() {
        return Err(format!(
            "Device approval not found for device_id: {device_id}"
        ));
    }

    let expires_at = approval_expiry(approval_type);
    let token = random_b64url(32);
    let token_hash = hash(&token);
    let caps_json = serde_json::to_string(capabilities).unwrap_or_else(|_| "[]".to_string());

    for row in &rows {
        let mut m: device_approval::ActiveModel = row.clone().into();
        m.approved = Set(true);
        m.approval_type = Set(approval_type.to_string());
        m.approved_at = Set(Some(Utc::now().into()));
        m.expires_at = Set(expires_at);
        m.access_token = Set(Some(token_hash.clone()));
        m.token_expires_at = Set(expires_at);
        m.capabilities = Set(caps_json.clone());
        m.updated_at = Set(Utc::now().into());
        device_approval::Entity::update(m)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to update device approval: {e}"))?;
    }

    Ok(ApproveOutcome {
        success: true,
        expires_at: expires_at.map(|e| e.to_rfc3339()),
        token,
        message: "Device approved".to_string(),
    })
}

/// Set `approved=false, approval_type="revoked"` on every row for
/// `device_id`. The only way to end an `until_revoked` grant.
pub async fn revoke(conn: &DatabaseConnection, device_id: &str) -> Result<(), String> {
    let rows = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .all(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    for row in rows {
        let mut m: device_approval::ActiveModel = row.into();
        m.approved = Set(false);
        m.approval_type = Set("revoked".to_string());
        m.updated_at = Set(Utc::now().into());
        device_approval::Entity::update(m)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to update device approval: {e}"))?;
    }
    Ok(())
}

/// Decline the still-pending rows for `device_id` — a host "deny", or a
/// `/pair/request` that timed out waiting for an answer.
pub async fn reject(conn: &DatabaseConnection, device_id: &str) -> Result<(), String> {
    let rows = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .filter(device_approval::Column::Approved.eq(false))
        .all(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    for row in rows {
        let mut m: device_approval::ActiveModel = row.into();
        m.approved = Set(false);
        m.approval_type = Set("rejected".to_string());
        m.updated_at = Set(Utc::now().into());
        device_approval::Entity::update(m)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to update device approval: {e}"))?;
    }
    Ok(())
}

/// Replace the granted capabilities on the most-recent row for `device_id`.
/// No token re-mint needed — capabilities are read fresh from the database on
/// every `/control`/`/rpc` call already.
pub async fn update_capabilities(
    conn: &DatabaseConnection,
    device_id: &str,
    capabilities: &[String],
) -> Result<(), String> {
    let Some(row) = most_recent_row(conn, device_id).await? else {
        return Err(format!(
            "Device approval not found for device_id: {device_id}"
        ));
    };
    let mut m: device_approval::ActiveModel = row.into();
    m.capabilities = Set(serde_json::to_string(capabilities).unwrap_or_else(|_| "[]".to_string()));
    m.updated_at = Set(Utc::now().into());
    device_approval::Entity::update(m)
        .exec(conn)
        .await
        .map_err(|e| format!("Failed to update device approval: {e}"))?;
    Ok(())
}

/// Replace the device_name on the most-recent row for `device_id` — `/control`
/// calls this when a client sends `X-Portal-Device`, e.g. to turn a QR grant's
/// `"Unnamed device"` placeholder into the phone's real name on first connect.
pub async fn rename_device(
    conn: &DatabaseConnection,
    device_id: &str,
    name: &str,
) -> Result<(), String> {
    let Some(row) = most_recent_row(conn, device_id).await? else {
        return Err(format!(
            "Device approval not found for device_id: {device_id}"
        ));
    };
    let mut m: device_approval::ActiveModel = row.into();
    m.device_name = Set(name.to_string());
    m.updated_at = Set(Utc::now().into());
    device_approval::Entity::update(m)
        .exec(conn)
        .await
        .map_err(|e| format!("Failed to update device approval: {e}"))?;
    Ok(())
}

/// Every device, most-recent row per device_id, every approval_type (pending,
/// approved, revoked, rejected) — the host's Devices page buckets them.
pub async fn list_devices(conn: &DatabaseConnection) -> Result<Vec<DeviceSummary>, String> {
    let rows = device_approval::Entity::find()
        .order_by_desc(device_approval::Column::CreatedAt)
        .all(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    // Rows arrive newest-first, so the first one seen for a device_id is kept.
    let mut by_device: HashMap<String, device_approval::Model> = HashMap::new();
    for row in rows {
        by_device.entry(row.device_id.clone()).or_insert(row);
    }

    // Expired, never-scanned QR grants: a scan clears `expires_at` on first
    // use (see `verify_token`), so a `until_revoked` row that still carries
    // one and is past it was never picked up and never will be — drop it
    // instead of letting it pile up in every list forever (QA-113).
    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut devices: Vec<_> = by_device
        .into_values()
        .filter(|row| {
            !(row.approval_type == "until_revoked" && row.expires_at.is_some_and(|e| e <= now))
        })
        .collect();
    devices.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(devices
        .into_iter()
        .map(|row| DeviceSummary {
            device_id: row.device_id,
            device_name: row.device_name,
            approval_type: row.approval_type,
            approved: row.approved,
            capabilities: serde_json::from_str(&row.capabilities).unwrap_or_default(),
            expires_at: row.expires_at.map(|e| e.to_rfc3339()),
            last_used_at: row.last_used_at.map(|e| e.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        })
        .collect())
}

/// Verify a bearer token. Hashes it and looks the device up by that hash, so
/// the plaintext is never compared against anything stored. Returns the
/// device_id and its granted capabilities when the token is approved and
/// unexpired.
pub async fn verify_token(
    conn: &DatabaseConnection,
    token: &str,
) -> Result<Option<(String, Vec<String>)>, String> {
    let token_hash = hash(token);
    let row = device_approval::Entity::find()
        .filter(device_approval::Column::AccessToken.eq(&token_hash))
        .one(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let now: DateTime<FixedOffset> = Utc::now().into();
    // `None` means "until_revoked" (or a QR grant past its pickup window — see
    // below) — live forever until `revoke` sets approved=false.
    let live = row.approved
        && row.expires_at.map(|e| e > now).unwrap_or(true)
        && row.token_expires_at.map(|e| e > now).unwrap_or(true);
    if !live {
        return Ok(None);
    }

    let device_id = row.device_id.clone();
    let capabilities: Vec<String> = serde_json::from_str(&row.capabilities).unwrap_or_default();
    let is_first_use = row.last_used_at.is_none();
    let is_until_revoked = row.approval_type == "until_revoked";

    let mut m: device_approval::ActiveModel = row.into();
    m.last_used_at = Set(Some(Utc::now().into()));
    // A QR-minted grant's 5-minute pickup `expires_at` becomes permanent the
    // first time a device actually authenticates with it — an unscanned/leaked
    // QR image should not grant anything to whoever eventually sees it, but
    // the device that used the token gets the permanent grant it was minted
    // for. One DB round trip, not two: folded into the last-used stamp below.
    if is_until_revoked && is_first_use {
        m.expires_at = Set(None);
    }
    device_approval::Entity::update(m)
        .exec(conn)
        .await
        .map_err(|e| format!("Failed to update device approval: {e}"))?;

    Ok(Some((device_id, capabilities)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::runner::run_migrations;
    use sea_orm::Database;
    use tempfile::tempdir;

    async fn fresh_db() -> DatabaseConnection {
        let dir = tempdir().expect("temp dir");
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("t.db").display());
        let conn = Database::connect(&url).await.expect("connect");
        run_migrations(&conn).await.expect("migrate");
        // Keep the temp dir alive for the life of the connection.
        std::mem::forget(dir);
        conn
    }

    #[test]
    fn token_is_random_and_url_safe() {
        let a = random_b64url(32);
        let b = random_b64url(32);
        assert_ne!(a, b, "two tokens must differ");
        assert_eq!(URL_SAFE_NO_PAD.decode(&a).unwrap().len(), 32);
        assert!(!a.contains('+') && !a.contains('/') && !a.contains('='));
    }

    #[test]
    fn hashing_is_stable_and_distinct() {
        assert_eq!(hash("abc"), hash("abc"));
        assert_ne!(hash("abc"), hash("abd"));
    }

    #[test]
    fn until_revoked_has_no_expiry_others_do() {
        assert!(approval_expiry("until_revoked").is_none());
        assert!(approval_expiry("temporary").is_some());
        assert!(approval_expiry("long_term").is_some());
        assert!(approval_expiry("anything_else").is_some());
    }

    #[tokio::test]
    async fn approve_after_reject_does_not_ghost_pair() {
        let conn = fresh_db().await;

        request_pairing(&conn, "dev-timeout", "Phone").await.unwrap();
        reject(&conn, "dev-timeout").await.unwrap();

        let err = approve(&conn, "dev-timeout", "temporary", &["coder".to_string()])
            .await
            .unwrap_err();
        assert!(err.contains("not found"));

        let devices = list_devices(&conn).await.unwrap();
        assert_eq!(devices[0].approval_type, "rejected");
        assert!(!devices[0].approved);
    }

    #[tokio::test]
    async fn request_pairing_then_approve_grants_a_live_token() {
        let conn = fresh_db().await;

        request_pairing(&conn, "dev-1", "Phone").await.unwrap();

        let devices = list_devices(&conn).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].approval_type, "pending");
        assert!(!devices[0].approved);

        let outcome = approve(&conn, "dev-1", "long_term", &["coder".to_string()])
            .await
            .unwrap();
        assert!(outcome.success);
        assert!(outcome.expires_at.is_some());

        let (did, caps) = verify_token(&conn, &outcome.token)
            .await
            .unwrap()
            .expect("live token");
        assert_eq!(did, "dev-1");
        assert_eq!(caps, vec!["coder".to_string()]);

        // Only the hash is stored — the plaintext never appears in the row.
        let row = device_approval::Entity::find()
            .filter(device_approval::Column::DeviceId.eq("dev-1"))
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            row.access_token.as_deref(),
            Some(hash(&outcome.token).as_str())
        );
        assert_ne!(row.access_token.as_deref(), Some(outcome.token.as_str()));

        revoke(&conn, "dev-1").await.unwrap();
        assert!(verify_token(&conn, &outcome.token).await.unwrap().is_none());

        // A garbage bearer is rejected.
        assert!(verify_token(&conn, "nonsense").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn until_revoked_approval_never_expires_on_its_own() {
        let conn = fresh_db().await;
        request_pairing(&conn, "dev-2", "Tablet").await.unwrap();
        let outcome = approve(&conn, "dev-2", "until_revoked", &[]).await.unwrap();
        assert!(outcome.expires_at.is_none(), "until_revoked has no expiry");

        assert!(verify_token(&conn, &outcome.token).await.unwrap().is_some());
        revoke(&conn, "dev-2").await.unwrap();
        assert!(verify_token(&conn, &outcome.token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn qr_pairing_is_permanent_only_after_first_use() {
        let conn = fresh_db().await;
        let (device_id, token) = mint_qr_pairing(&conn, &["coder".to_string()])
            .await
            .unwrap();

        // Live immediately — no server round trip needed for a QR scan.
        let (did, caps) = verify_token(&conn, &token)
            .await
            .unwrap()
            .expect("live token");
        assert_eq!(did, device_id);
        assert_eq!(caps, vec!["coder".to_string()]);

        // The pickup window is gone after first use.
        let row = device_approval::Entity::find()
            .filter(device_approval::Column::DeviceId.eq(&device_id))
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert!(
            row.expires_at.is_none(),
            "first use should clear the pickup expiry"
        );

        // Still live on a second use — now permanent, not just within the window.
        assert!(verify_token(&conn, &token).await.unwrap().is_some());

        revoke(&conn, &device_id).await.unwrap();
        assert!(verify_token(&conn, &token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn expired_unscanned_qr_grant_is_pruned_from_list_devices() {
        let conn = fresh_db().await;
        let (device_id, _token) = mint_qr_pairing(&conn, &[]).await.unwrap();

        // Still within its pickup window.
        assert_eq!(list_devices(&conn).await.unwrap().len(), 1);

        // Force it into the past, as if the 5-minute window lapsed unscanned.
        let row = most_recent_row(&conn, &device_id).await.unwrap().unwrap();
        let mut m: device_approval::ActiveModel = row.into();
        m.expires_at = Set(Some((Utc::now() - Duration::minutes(1)).into()));
        device_approval::Entity::update(m).exec(&conn).await.unwrap();

        assert!(list_devices(&conn).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_devices_keeps_only_the_most_recent_row_per_device() {
        let conn = fresh_db().await;
        request_pairing(&conn, "dev-3", "First").await.unwrap();
        approve(&conn, "dev-3", "temporary", &[]).await.unwrap();
        revoke(&conn, "dev-3").await.unwrap();
        request_pairing(&conn, "dev-3", "Reconnected").await.unwrap();

        let devices = list_devices(&conn).await.unwrap();
        assert_eq!(devices.len(), 1, "one device_id, however many rows it has");
        assert_eq!(devices[0].device_name, "Reconnected");
        assert_eq!(devices[0].approval_type, "pending");
    }
}
