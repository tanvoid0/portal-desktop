//! Pairing and access tokens for the LAN surface (phase 4 of
//! `docs/development/REMOTE_ARCHITECTURE.md`), as free functions over a
//! `&DatabaseConnection`.
//!
//! The daemon's `/pair/*` routes and bearer auth call these, and so do the
//! thin Tauri command adapters, so there is one implementation and neither
//! transport owns it. Nothing here touches Tauri — it lives in `portal-core`.
//!
//! # Security model
//!
//! - **The token is 32 random bytes, base64url.** The database stores only its
//!   SHA-256 and auth looks a device up by that hash, so a leak of the database
//!   never yields a usable token. The plaintext exists only in transit and in a
//!   short-lived in-memory pickup buffer; it is never written to disk.
//! - **Each device has a random `pairing_secret`** (its SHA-256 stored at verify
//!   time). `/pair/status` refuses to hand over a token unless the caller
//!   presents the matching secret, so knowing a device_id — which the host UI
//!   displays — is not enough to claim an approval meant for another device.
//! - Comparisons of stored hashes are constant-time.

use std::collections::HashMap;
use std::sync::OnceLock;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domains::network::services::network_service::get_passcode_store;
use crate::entities::device_approval;

/// The host mints a single shared passcode under this key; any device that
/// presents it registers itself as pending. Not tied to a device_id.
pub const SHARED_PASSCODE_KEY: &str = "__shared__";

/// How long a freshly minted plaintext token stays available for the device to
/// collect over `/pair/status`. A device normally picks it up on its next poll
/// within seconds; after this it must re-pair.
const TOKEN_PICKUP_TTL_MINUTES: i64 = 5;

/// Access-token lifetime for a one-off ("temporary") approval.
const TEMPORARY_HOURS: i64 = 24;
/// Access-token lifetime for a "long_term" approval.
const LONG_TERM_DAYS: i64 = 30;

/// Plaintext tokens staged for pickup, keyed by device_id. The database holds
/// only the hash, so this in-memory buffer is the single place the plaintext
/// lives between approval and the device collecting it.
struct Staged {
    token: String,
    minted_at: DateTime<Utc>,
}

static STAGED_TOKENS: OnceLock<Mutex<HashMap<String, Staged>>> = OnceLock::new();

fn staged_tokens() -> &'static Mutex<HashMap<String, Staged>> {
    STAGED_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Hex SHA-256 of a value. Used for the token hash, the pairing-secret hash and
/// the stored passcode.
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

/// Constant-time compare of two equal-length hashes.
fn hashes_match(a: &str, b: &str) -> bool {
    ring::constant_time::verify_slices_are_equal(a.as_bytes(), b.as_bytes()).is_ok()
}

fn approval_expiry(approval_type: &str) -> DateTime<chrono::FixedOffset> {
    let span = if approval_type == "temporary" {
        Duration::hours(TEMPORARY_HOURS)
    } else {
        Duration::days(LONG_TERM_DAYS)
    };
    (Utc::now() + span).into()
}

/// Result of `/pair/verify`.
#[derive(Debug, Serialize)]
pub struct VerifyOutcome {
    pub verified: bool,
    pub message: String,
}

/// Result of `/pair/status`.
#[derive(Debug, Serialize)]
pub struct StatusOutcome {
    pub approved: bool,
    /// The access token, present only while the device is collecting it just
    /// after approval (within the pickup window). Absent while pending.
    pub token: Option<String>,
    pub expires_at: Option<String>,
    pub capabilities: Vec<String>,
    pub message: String,
}

/// Result of an approval (host side).
#[derive(Debug, Serialize)]
pub struct ApproveOutcome {
    pub success: bool,
    pub expires_at: Option<String>,
    pub message: String,
}

/// Validate the shared passcode and register (or refresh) the caller as a
/// pending device, recording the hash of its pairing secret. A bad or expired
/// passcode returns `verified: false` without touching the database.
pub async fn verify_and_register(
    conn: &DatabaseConnection,
    passcode: &str,
    device_id: &str,
    device_name: &str,
    device_info: &str,
    pairing_secret: &str,
) -> Result<VerifyOutcome, String> {
    if !get_passcode_store()
        .verify_passcode(SHARED_PASSCODE_KEY, passcode)
        .await
    {
        return Ok(VerifyOutcome {
            verified: false,
            message: "Invalid or expired passcode".to_string(),
        });
    }

    let now = Utc::now();
    let passcode_expires_at = (now + Duration::minutes(5)).into();
    let secret_hash = hash(pairing_secret);

    // Reuse an existing pending row for this device so repeated scans don't pile
    // up duplicate rows.
    let existing = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .filter(device_approval::Column::Approved.eq(false))
        .order_by_desc(device_approval::Column::CreatedAt)
        .one(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    if let Some(row) = existing {
        let mut m: device_approval::ActiveModel = row.into();
        m.device_name = Set(device_name.to_string());
        m.device_info = Set(device_info.to_string());
        m.passcode = Set(hash(passcode));
        m.passcode_expires_at = Set(passcode_expires_at);
        m.pairing_secret = Set(Some(secret_hash));
        m.updated_at = Set(now.into());
        device_approval::Entity::update(m)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to update device approval: {e}"))?;
    } else {
        let approval = device_approval::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            device_id: Set(device_id.to_string()),
            device_name: Set(device_name.to_string()),
            device_info: Set(device_info.to_string()),
            passcode: Set(hash(passcode)),
            passcode_expires_at: Set(passcode_expires_at),
            approved: Set(false),
            approval_type: Set("pending".to_string()),
            approved_at: Set(None),
            expires_at: Set(None),
            access_token: Set(None),
            token_expires_at: Set(None),
            capabilities: Set("[]".to_string()),
            pairing_secret: Set(Some(secret_hash)),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            last_used_at: Set(None),
        };
        device_approval::Entity::insert(approval)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to store device approval: {e}"))?;
    }

    Ok(VerifyOutcome {
        verified: true,
        message: "Passcode verified. Waiting for host approval.".to_string(),
    })
}

/// Approve every pending row for `device_id`, granting `capabilities`. Mints a
/// token, stores only its hash with the caps and expiry, and stages the
/// plaintext in memory for the device to collect over `/pair/status`.
pub async fn approve(
    conn: &DatabaseConnection,
    device_id: &str,
    approval_type: &str,
    capabilities: &[String],
) -> Result<ApproveOutcome, String> {
    let rows = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .filter(device_approval::Column::Approved.eq(false))
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
        m.expires_at = Set(Some(expires_at));
        m.access_token = Set(Some(token_hash.clone()));
        m.token_expires_at = Set(Some(expires_at));
        m.capabilities = Set(caps_json.clone());
        m.updated_at = Set(Utc::now().into());
        device_approval::Entity::update(m)
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to update device approval: {e}"))?;
    }

    staged_tokens().lock().await.insert(
        device_id.to_string(),
        Staged {
            token,
            minted_at: Utc::now(),
        },
    );

    Ok(ApproveOutcome {
        success: true,
        expires_at: Some(expires_at.to_rfc3339()),
        message: "Device approved".to_string(),
    })
}

/// A device polling for its approval. Gated by the pairing secret: the caller
/// must present the same secret recorded at verify time, or this fails. Hands
/// over the staged plaintext token once the device is approved and the grant is
/// still live.
pub async fn device_status(
    conn: &DatabaseConnection,
    device_id: &str,
    pairing_secret: &str,
) -> Result<StatusOutcome, String> {
    let row = device_approval::Entity::find()
        .filter(device_approval::Column::DeviceId.eq(device_id))
        .order_by_desc(device_approval::Column::CreatedAt)
        .one(conn)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

    let Some(row) = row else {
        return Ok(StatusOutcome {
            approved: false,
            token: None,
            expires_at: None,
            capabilities: Vec::new(),
            message: "Device not found".to_string(),
        });
    };

    // The secret gate. A row that never recorded a secret can never pass.
    let presented = hash(pairing_secret);
    let ok = row
        .pairing_secret
        .as_deref()
        .map(|stored| hashes_match(stored, &presented))
        .unwrap_or(false);
    if !ok {
        return Err("pairing secret does not match".to_string());
    }

    if !row.approved {
        return Ok(StatusOutcome {
            approved: false,
            token: None,
            expires_at: None,
            capabilities: Vec::new(),
            message: "Device is pending approval".to_string(),
        });
    }

    let now: DateTime<chrono::FixedOffset> = Utc::now().into();
    if !row.expires_at.map(|e| e > now).unwrap_or(false) {
        return Ok(StatusOutcome {
            approved: false,
            token: None,
            expires_at: None,
            capabilities: Vec::new(),
            message: "Device approval has expired".to_string(),
        });
    }

    let capabilities: Vec<String> = serde_json::from_str(&row.capabilities).unwrap_or_default();
    let token = staged_token(device_id).await;

    Ok(StatusOutcome {
        approved: true,
        token,
        expires_at: row.expires_at.map(|e| e.to_rfc3339()),
        capabilities,
        message: "Device is approved".to_string(),
    })
}

/// The staged plaintext token for `device_id`, if one is within the pickup
/// window. Re-handed on each poll inside the window (harmless — same token, same
/// secret-authenticated device); expired entries are swept so the buffer can't
/// grow without bound.
async fn staged_token(device_id: &str) -> Option<String> {
    let mut map = staged_tokens().lock().await;
    let cutoff = Utc::now() - Duration::minutes(TOKEN_PICKUP_TTL_MINUTES);
    map.retain(|_, s| s.minted_at > cutoff);
    map.get(device_id).map(|s| s.token.clone())
}

/// Verify a bearer token. Hashes it and looks the device up by that hash, so the
/// plaintext is never compared against anything stored. Returns the device_id
/// and its granted capabilities when the token is approved and unexpired.
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

    let now: DateTime<chrono::FixedOffset> = Utc::now().into();
    let live = row.approved
        && row.expires_at.map(|e| e > now).unwrap_or(false)
        && row.token_expires_at.map(|e| e > now).unwrap_or(false);
    if !live {
        return Ok(None);
    }

    let device_id = row.device_id.clone();
    let capabilities: Vec<String> = serde_json::from_str(&row.capabilities).unwrap_or_default();

    // Best-effort last-used stamp; a failure here must never fail auth.
    let mut m: device_approval::ActiveModel = row.into();
    m.last_used_at = Set(Some(Utc::now().into()));
    let _ = device_approval::Entity::update(m).exec(conn).await;

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
        assert!(hashes_match(&hash("s"), &hash("s")));
        assert!(!hashes_match(&hash("s"), &hash("t")));
    }

    #[tokio::test]
    async fn full_pairing_round_trip() {
        let conn = fresh_db().await;
        get_passcode_store()
            .store_passcode(SHARED_PASSCODE_KEY.to_string(), "424242".to_string())
            .await;

        // Wrong passcode: not verified, nothing written.
        let bad = verify_and_register(&conn, "000000", "dev-1", "Phone", "{}", "secret-1")
            .await
            .unwrap();
        assert!(!bad.verified);

        // Right passcode: registered pending.
        let ok = verify_and_register(&conn, "424242", "dev-1", "Phone", "{}", "secret-1")
            .await
            .unwrap();
        assert!(ok.verified);

        // A known device_id with the wrong secret cannot read status.
        assert!(device_status(&conn, "dev-1", "not-the-secret")
            .await
            .is_err());

        // Pending before approval.
        let pending = device_status(&conn, "dev-1", "secret-1").await.unwrap();
        assert!(!pending.approved && pending.token.is_none());

        // Approve with a capability subset.
        approve(&conn, "dev-1", "long_term", &["coder".to_string()])
            .await
            .unwrap();

        // The device collects its token, scoped to the granted caps.
        let granted = device_status(&conn, "dev-1", "secret-1").await.unwrap();
        assert!(granted.approved);
        let token = granted.token.expect("token handed over once approved");
        assert_eq!(granted.capabilities, vec!["coder".to_string()]);

        // The plaintext token authenticates and carries its caps.
        let (did, caps) = verify_token(&conn, &token)
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
        assert_eq!(row.access_token.as_deref(), Some(hash(&token).as_str()));
        assert_ne!(row.access_token.as_deref(), Some(token.as_str()));

        // A garbage bearer is rejected.
        assert!(verify_token(&conn, "nonsense").await.unwrap().is_none());
    }
}
