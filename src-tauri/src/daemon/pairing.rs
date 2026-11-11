//! `POST /pair/request` — the Android contract's pairing entry point: no
//! auth, no passcode, body `{"device": "<name>"}`. Holds the response open
//! until a human approves or denies on the desktop (long-poll), or ~170s pass
//! with no answer, since letting the client's socket just die reads to
//! Android as a confusing "could not reach" rather than "turned down".

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::domains::network::services::pairing_service;
use crate::events::{EventSinkExt, Events};

use super::AppState;

/// How long `/pair/request` holds the connection open before giving up on a
/// human answering. Android's own read timeout is 3 minutes; answering a
/// little before that turns a silent socket death (which Android would show
/// as "could not reach") into an explicit 403 ("turned down").
const PAIR_REQUEST_TIMEOUT: Duration = Duration::from_secs(170);

/// What a pending `/pair/request` was resolved with: `approve_device`/
/// `reject_device` on the Tauri side, or this module's own timeout branch.
#[derive(Debug)]
pub enum PairResolution {
    Approved { token: String, host_name: String },
    Denied,
}

type Pending = HashMap<String, oneshot::Sender<PairResolution>>;

static PENDING: OnceLock<Mutex<Pending>> = OnceLock::new();

fn pending() -> &'static Mutex<Pending> {
    PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Resolve a pending `/pair/request` for `device_id`, if one is still
/// waiting. Called by the `approve_device`/`reject_device` Tauri commands. A
/// no-op if the request already timed out and removed itself (or there never
/// was one — e.g. a QR-minted device, which has no `/pair/request` to wake).
pub fn resolve_pending(device_id: &str, resolution: PairResolution) {
    if let Some(tx) = pending().lock().unwrap().remove(device_id) {
        let _ = tx.send(resolution);
    }
}

#[derive(serde::Deserialize)]
pub struct PairRequestBody {
    device: String,
}

/// `POST /pair/request`. Mints a pending row, surfaces it to the host UI via
/// `device://pending` (the same event the old `/pair/verify` emitted, so
/// nothing on the frontend's event-listening side has to change), then waits.
///
/// `pub(super)`, not `pub`: only `daemon::mod`'s router names this directly;
/// wider than that would outgrow `AppState`'s own visibility and the compiler
/// says so (`private_interfaces`).
pub(super) async fn pair_request(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(body): Json<PairRequestBody>,
) -> Response {
    if !state.allow_pair_request(peer.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "too many requests; wait a minute" })),
        )
            .into_response();
    }

    let Some(db) = state.db() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "database unavailable" })),
        )
            .into_response();
    };

    let device_id = Uuid::new_v4().to_string();
    if let Err(e) =
        pairing_service::request_pairing(db.get_connection(), &device_id, &body.device).await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response();
    }

    if let Some(app) = state.app.as_ref() {
        tauri::Manager::state::<Events>(app).inner().emit(
            "device://pending",
            json!({ "device_id": device_id, "device_name": body.device }),
        );
    }

    let (tx, rx) = oneshot::channel();
    pending().lock().unwrap().insert(device_id.clone(), tx);

    let resolution = tokio::select! {
        resolved = rx => resolved.unwrap_or(PairResolution::Denied),
        _ = tokio::time::sleep(PAIR_REQUEST_TIMEOUT) => {
            pending().lock().unwrap().remove(&device_id);
            // Nobody answered in time — stop sitting "pending" forever.
            let _ = pairing_service::reject(db.get_connection(), &device_id).await;
            PairResolution::Denied
        }
    };

    match resolution {
        PairResolution::Approved { token, host_name } => Json(json!({
            "token": token,
            "port": state.port,
            "name": host_name,
        }))
        .into_response(),
        PairResolution::Denied => StatusCode::FORBIDDEN.into_response(),
    }
}
