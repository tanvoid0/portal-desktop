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
use sea_orm::DatabaseConnection;
use serde_json::json;
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

use crate::domains::network::services::pairing_service;
use crate::events::{EventSinkExt, Events};

use super::{AppState, Trust};

/// How long `/pair/request` holds the connection open before giving up on a
/// human answering. Android's own read timeout is 3 minutes; answering a
/// little before that turns a silent socket death (which Android would show
/// as "could not reach") into an explicit 403 ("turned down").
const PAIR_REQUEST_TIMEOUT: Duration = Duration::from_secs(170);

/// What a pending `/pair/request` was resolved with: `approve_device`/
/// `reject_device` on the Tauri side, or this module's own timeout branch.
/// `TimedOut` is kept distinct from `Denied` so the HTTP response (and, in
/// turn, the Android client) doesn't read a silence as a deliberate human
/// rejection — see `PAIR_REQUEST_TIMEOUT` above.
#[derive(Debug)]
pub enum PairResolution {
    Approved { token: String, host_name: String },
    Denied,
    TimedOut,
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
            PairResolution::TimedOut
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
        // Distinct status from an explicit Deny (403): REQUEST_TIMEOUT so a
        // future Android client can tell "nobody answered" apart from "a
        // human said no" instead of both reading as "turned down". The
        // current Android client only special-cases 403, so this falls
        // through to its generic "answered HTTP {code}" message — already
        // more honest than "turned down" for a timeout nobody decided.
        PairResolution::TimedOut => (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({ "error": "no response from host before timeout" })),
        )
            .into_response(),
    }
}

/// Per-socket backlog of [`drop_device`] signals before a socket counts as
/// lagged (see [`until_cut_off`]). Well past any realistic burst of revokes, so
/// a lag is rare — and costs one database read when it happens.
const DROP_CHANNEL_CAPACITY: usize = 64;

/// Every open `/events` and `/control` socket of a remote device listens here.
/// Process-wide rather than on `AppState`: the commands that end a grant (the
/// Tauri adapters) never see the daemon's state — same reason `PENDING` is.
static DROPS: OnceLock<broadcast::Sender<String>> = OnceLock::new();

fn drops() -> &'static broadcast::Sender<String> {
    DROPS.get_or_init(|| broadcast::channel(DROP_CHANNEL_CAPACITY).0)
}

/// Close every open `/events` and `/control` socket `device_id` holds. A
/// socket's capabilities are fixed when it opens, so whatever ends or changes a
/// grant must call this *after* its database write: the token check then
/// already refuses a reconnect (or hands back the new caps). Other devices'
/// sockets, and local peers, never react. A no-op when none is open.
pub fn drop_device(device_id: &str) {
    // `send` fails only when no socket is listening — nothing to drop.
    let _ = drops().send(device_id.to_string());
}

/// A remote socket's tie to its device, consumed by [`until_cut_off`]. Built
/// in the handler, ahead of anything slow, so it subscribes as early as it can.
pub(super) struct Cutoff {
    rx: broadcast::Receiver<String>,
    device_id: String,
    /// The caps this socket opened with — the grant [`Cutoff::granted`] must
    /// still find.
    caps: Vec<String>,
    /// Absent only where no app sits behind the listener (tests).
    db: Option<DatabaseConnection>,
}

impl Cutoff {
    /// `None` for a local peer, which no [`drop_device`] ever closes. (`Anon`
    /// is refused before either socket route, so it has no device to cut off.)
    pub(super) fn new(trust: &Trust, state: &AppState) -> Option<Self> {
        let Trust::Remote { device_id, caps } = trust else {
            return None;
        };
        Some(Self {
            rx: drops().subscribe(),
            device_id: device_id.clone(),
            caps: caps.clone(),
            db: state.db().map(|d| d.get_connection_clone()),
        })
    }

    /// Whether the database still holds exactly the grant this socket opened
    /// with. Fail closed: a read error, or no database to ask, is "no".
    async fn granted(&self) -> bool {
        let Some(db) = &self.db else { return false };
        matches!(
            pairing_service::live_capabilities(db, &self.device_id).await,
            Ok(Some(caps)) if caps == self.caps
        )
    }
}

/// Resolves once the socket behind `cutoff` must close: [`drop_device`] named
/// its device, or the signal channel lagged — the lost signal may have been
/// this device's — and the database no longer backs the grant the socket
/// opened with. Never resolves for `None`, a local peer.
pub(super) async fn until_cut_off(cutoff: Option<Cutoff>) {
    let Some(mut cutoff) = cutoff else {
        return std::future::pending().await;
    };
    // A revoke that landed between the trust check and this subscription sent
    // its signal to nobody; the database has it. Skipped when there is no
    // database to ask.
    if cutoff.db.is_some() && !cutoff.granted().await {
        return;
    }
    loop {
        match cutoff.rx.recv().await {
            Ok(id) if id == cutoff.device_id => return,
            Ok(_) => {}
            Err(broadcast::error::RecvError::Lagged(_)) => {
                if !cutoff.granted().await {
                    return;
                }
            }
            Err(broadcast::error::RecvError::Closed) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portal_core::migrations::runner::run_migrations;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::time::timeout;

    /// Long enough that a `Cutoff` which is going to fire has fired.
    const SOON: Duration = Duration::from_millis(300);

    type Waiting = Pin<Box<dyn Future<Output = ()> + Send>>;

    /// A `Cutoff` for `device` waiting in [`until_cut_off`], on a private
    /// two-slot channel: a test can lag that without lagging every other
    /// test's sockets on the shared one.
    fn open(
        device: &str,
        caps: &[&str],
        db: Option<&DatabaseConnection>,
    ) -> (broadcast::Sender<String>, Waiting) {
        let (tx, rx) = broadcast::channel(2);
        let cutoff = Cutoff {
            rx,
            device_id: device.to_string(),
            caps: caps.iter().map(|c| c.to_string()).collect(),
            db: db.cloned(),
        };
        (tx, Box::pin(until_cut_off(Some(cutoff))))
    }

    /// Whether the socket behind `waiting` was cut off within [`SOON`].
    async fn closed(waiting: &mut Waiting) -> bool {
        timeout(SOON, waiting).await.is_ok()
    }

    /// Overflow the two-slot channel, so its receiver lags.
    fn lag(tx: &broadcast::Sender<String>) {
        for _ in 0..4 {
            tx.send("noise".to_string()).unwrap();
        }
    }

    #[tokio::test]
    async fn only_its_own_device_cuts_a_socket_off() {
        let (tx, mut waiting) = open("dev", &["coder"], None);
        tx.send("someone-else".to_string()).unwrap();
        assert!(
            !closed(&mut waiting).await,
            "another device's drop is not ours"
        );
        tx.send("dev".to_string()).unwrap();
        assert!(closed(&mut waiting).await);
    }

    #[tokio::test]
    async fn a_lag_with_no_way_to_check_the_grant_closes_the_socket() {
        let (tx, mut waiting) = open("dev", &["coder"], None);
        lag(&tx);
        assert!(
            closed(&mut waiting).await,
            "the lost signal may have been ours"
        );
    }

    #[tokio::test]
    async fn a_lag_is_settled_by_the_database() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("t.db").display());
        let conn = sea_orm::Database::connect(&url).await.unwrap();
        run_migrations(&conn).await.unwrap();
        pairing_service::request_pairing(&conn, "dev", "Phone")
            .await
            .unwrap();
        pairing_service::approve(&conn, "dev", "until_revoked", &["coder".to_string()])
            .await
            .unwrap();

        // The grant stands: a lag costs a read, not the socket.
        let (tx, mut waiting) = open("dev", &["coder"], Some(&conn));
        assert!(!closed(&mut waiting).await);
        lag(&tx);
        assert!(
            !closed(&mut waiting).await,
            "grant unchanged, so the socket stays"
        );

        // Caps narrowed behind its back: the next lag finds a different grant.
        pairing_service::update_capabilities(&conn, "dev", &["projects".to_string()])
            .await
            .unwrap();
        lag(&tx);
        assert!(closed(&mut waiting).await, "caps changed");

        // Revoked behind its back.
        let (tx, mut waiting) = open("dev", &["projects"], Some(&conn));
        assert!(!closed(&mut waiting).await);
        pairing_service::revoke(&conn, "dev").await.unwrap();
        lag(&tx);
        assert!(closed(&mut waiting).await, "revoked");

        // A revoke that beat the subscription sent its signal to nobody, and
        // nothing lags: the first check alone has to catch it.
        let (_tx, mut waiting) = open("dev", &["projects"], Some(&conn));
        assert!(
            closed(&mut waiting).await,
            "revoked before the socket subscribed"
        );
    }
}
