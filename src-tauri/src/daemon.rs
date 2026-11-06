//! The local HTTP/WS surface, and the capability gate in front of it — phases 2
//! and 4 of `docs/development/REMOTE_ARCHITECTURE.md`.
//!
//! Runs as a task inside the Tauri process. Today it binds loopback only; the
//! LAN listener (same router, `0.0.0.0`) is spawned behind an explicit toggle
//! elsewhere, so the one security gate here protects both.
//!
//! # Trust gate
//!
//! Every request passes through [`classify`], which decides what the peer is:
//!
//! - **Any `Origin` header → 403, on every route and both listeners.** A browser
//!   cannot suppress it and WebSockets ignore CORS, so this is the only thing
//!   keeping a visited web page off a port that exposes terminal execution and a
//!   credential vault. Unconditional by design.
//! - **Loopback peer → `Local`**: the machine itself, unrestricted.
//! - **Remote peer with a live bearer token → `Remote { caps }`**: a paired
//!   device, allowed only the commands and events its granted capabilities cover.
//! - **Remote peer without a token → `Anon`**: may reach `/health` and `/pair/*`
//!   to pair, nothing else.
//!
//! `/rpc/{command}` refuses a remote peer whose caps do not cover the command's
//! `cap`; `/events` drops frames whose [`event_cap`] the peer does not hold.
//! Credentials and the other un-annotated commands carry no cap, so no remote
//! peer can ever reach them.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Extension, Path as AxumPath, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::database::DatabaseManager;
use crate::domains::network::services::pairing_service;
use crate::events::{EventSink, EventSinkExt, Events};

/// Default port. Deliberately clear of agent-platform's 18410.
pub const DEFAULT_PORT: u16 = 18420;

/// How many frames a slow client may fall behind before it starts losing the
/// oldest. Events are a live view, not a log.
const CHANNEL_CAPACITY: usize = 256;

/// Per-IP rate limit on `/pair/verify`: a brute-forcer gets this many tries per
/// window before being turned away, so the 6-digit passcode can't be sprayed.
const VERIFY_WINDOW: Duration = Duration::from_secs(60);
const VERIFY_MAX: u32 = 10;

/// One broadcast item: the serialized frame plus its event name pulled out, so a
/// per-client capability filter costs a string compare rather than a JSON parse.
#[derive(Debug)]
struct Wire {
    event: String,
    text: Arc<str>,
}

type Shared = Arc<Wire>;

/// What a request's peer is trusted as. Carried in request extensions by the
/// [`guard`] middleware for the handlers to read.
#[derive(Debug, Clone)]
enum Trust {
    /// Loopback: the machine itself. Unrestricted.
    Local,
    /// Remote, unauthenticated. May only pair.
    Anon,
    /// Remote, holding a live token. Scoped to `caps`.
    Remote {
        #[allow(dead_code)]
        device_id: String,
        caps: Vec<String>,
    },
}

/// [`EventSink`] that publishes to every attached WebSocket.
#[derive(Debug, Clone)]
pub struct WsSink(broadcast::Sender<Shared>);

impl WsSink {
    fn new(tx: broadcast::Sender<Shared>) -> Self {
        Self(tx)
    }
}

impl EventSink for WsSink {
    fn emit_json(&self, event: &str, payload: Value) -> bool {
        // Serialised once here rather than once per client.
        let Ok(text) = serde_json::to_string(&json!({ "event": event, "payload": payload })) else {
            return false;
        };
        let wire = Arc::new(Wire {
            event: event.to_string(),
            text: Arc::from(text),
        });
        // `send` fails only when there are no receivers — exactly the "nobody is
        // listening" answer the trait asks for. Deliberately does not log:
        // `utils::logger` mirrors through a sink while holding its own mutex.
        self.0.send(wire).is_ok()
    }
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Shared>,
    /// Absent only in tests that exercise routing without a running app.
    app: Option<tauri::AppHandle>,
    /// Per-IP `/pair/verify` attempt counters. A LAN has a handful of IPs, so the
    /// map stays tiny; entries are reset lazily per window rather than pruned.
    // ponytail: fixed-window counter; swap for a token bucket only if abuse shows.
    verify_hits: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
}

impl AppState {
    fn new(tx: broadcast::Sender<Shared>, app: Option<tauri::AppHandle>) -> Self {
        Self {
            tx,
            app,
            verify_hits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The managed database, when there is a running app behind this listener.
    fn db(&self) -> Option<Arc<DatabaseManager>> {
        self.app.as_ref().map(|a| {
            tauri::Manager::state::<Arc<DatabaseManager>>(a)
                .inner()
                .clone()
        })
    }

    /// Record one `/pair/verify` attempt from `ip`; false once over the limit.
    fn allow_verify(&self, ip: IpAddr) -> bool {
        let mut hits = self.verify_hits.lock().unwrap();
        let now = Instant::now();
        let entry = hits.entry(ip).or_insert((now, 0));
        if now.duration_since(entry.0) > VERIFY_WINDOW {
            *entry = (now, 0);
        }
        entry.1 += 1;
        entry.1 <= VERIFY_MAX
    }
}

/// Build the sink and start serving on loopback. Returns the sink immediately so
/// startup can wire it into the fanout; a bind failure leaves the app running
/// with the webview transport alone.
pub fn spawn(port: u16, app: tauri::AppHandle) -> Arc<dyn EventSink> {
    let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
    let sink = WsSink::new(tx.clone());

    let state = AppState::new(tx, Some(app));
    tauri::async_runtime::spawn(async move {
        let router = router(state);
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("[daemon] listening on http://{addr}");
                let service = router.into_make_service_with_connect_info::<SocketAddr>();
                if let Err(e) = axum::serve(listener, service).await {
                    eprintln!("[daemon] stopped: {e}");
                }
            }
            Err(e) => eprintln!("[daemon] could not bind {addr}: {e}"),
        }
    });

    Arc::new(sink)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/events", get(events))
        .route("/rpc/{command}", post(rpc))
        .route("/pair/verify", post(pair_verify))
        .route("/pair/status", post(pair_status))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

fn forbidden(msg: &str) -> Response {
    (StatusCode::FORBIDDEN, msg.to_string()).into_response()
}

/// The `Bearer` token on a request, if any.
fn bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|t| t.trim().to_string())
}

/// Paths an unauthenticated remote peer may reach: health and the pairing
/// handshake. Everything else requires loopback or a token.
fn is_public_path(path: &str) -> bool {
    matches!(path, "/health" | "/pair/verify" | "/pair/status")
}

/// Decide what a request's peer is trusted as. Pure and synchronous so it can be
/// unit-tested with a synthetic remote peer and a fake token resolver — the
/// loopback-only integration harness cannot otherwise produce a remote peer.
fn classify(
    peer: IpAddr,
    headers: &HeaderMap,
    path: &str,
    resolve: impl Fn(&str) -> Option<(String, Vec<String>)>,
) -> Result<Trust, Response> {
    // No browser may speak to this port, on any route or either listener.
    if headers.contains_key(header::ORIGIN) {
        return Err(forbidden("browser origins are not permitted on this port"));
    }

    // Loopback is the machine itself.
    if peer.is_loopback() {
        return Ok(Trust::Local);
    }

    // A remote peer with a live token is a known, scoped device.
    if let Some(token) = bearer(headers) {
        if let Some((device_id, caps)) = resolve(&token) {
            return Ok(Trust::Remote { device_id, caps });
        }
    }

    // Remote and unauthenticated: pairing and health only.
    if is_public_path(path) {
        Ok(Trust::Anon)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "pair this device before calling the API",
        )
            .into_response())
    }
}

/// The capability a remote peer must hold to receive an event. Fail-closed: an
/// event mapped to nothing here is withheld from every remote peer (local peers
/// are never filtered).
fn event_cap(event: &str) -> Option<&'static str> {
    if event.starts_with("coder") {
        Some("coder")
    } else if event.starts_with("terminal") || event.starts_with("shell-integration") {
        Some("terminal.exec")
    } else {
        None
    }
}

/// The trust middleware. Resolves a bearer token against the database up front
/// (the one async step) so [`classify`] stays pure, then stashes the resulting
/// [`Trust`] in the request for the handlers.
async fn guard(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers().clone();
    let path = req.uri().path().to_string();
    let peer_ip = peer.ip();

    let token = bearer(&headers);
    let identity = match (&token, peer_ip.is_loopback(), state.db()) {
        (Some(t), false, Some(db)) => pairing_service::verify_token(db.get_connection(), t)
            .await
            .ok()
            .flatten(),
        _ => None,
    };

    let trust = match classify(peer_ip, &headers, &path, |t| match (&token, &identity) {
        (Some(tok), Some(id)) if tok == t => Some(id.clone()),
        _ => None,
    }) {
        Ok(trust) => trust,
        Err(resp) => return resp,
    };

    req.extensions_mut().insert(trust);
    next.run(req).await
}

async fn health() -> Response {
    axum::Json(json!({
        "ok": true,
        "name": "portal-desktop",
        "version": env!("CARGO_PKG_VERSION"),
    }))
    .into_response()
}

async fn events(
    upgrade: WebSocketUpgrade,
    Extension(trust): Extension<Trust>,
    State(state): State<AppState>,
) -> Response {
    // The guard has already refused browser origins and unauthenticated remotes,
    // so trust here is Local (full stream) or Remote (filtered by caps).
    let caps = match trust {
        Trust::Remote { caps, .. } => Some(caps),
        _ => None,
    };
    upgrade.on_upgrade(move |socket| pump(socket, state.tx.subscribe(), caps))
}

/// Run one command by name. The registry is the command's own namespace, shared
/// with IPC; a remote peer is additionally held to the command's capability.
async fn rpc(
    AxumPath(command): AxumPath<String>,
    Extension(trust): Extension<Trust>,
    State(state): State<AppState>,
    body: Option<axum::Json<Value>>,
) -> Response {
    if !crate::rpc::is_registered(&command) {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "error": format!("no command `{command}`") })),
        )
            .into_response();
    }

    // Local (the machine itself) is unrestricted; a remote peer must hold the
    // command's capability. A command with no cap is unreachable to any remote.
    if let Trust::Remote { caps, .. } = &trust {
        let needed = crate::rpc::capability(&command);
        let allowed = matches!(needed, Some(cap) if caps.iter().any(|c| c == cap));
        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(json!({
                    "error": "this device is not permitted to call this command",
                    "capability": needed,
                })),
            )
                .into_response();
        }
    }

    let Some(app) = state.app.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({ "error": "no app handle on this listener" })),
        )
            .into_response();
    };

    let args = body.map(|axum::Json(v)| v).unwrap_or(Value::Null);
    let args = if args.is_null() { json!({}) } else { args };

    match crate::rpc::dispatch(app, &command, args).await {
        Ok(value) => axum::Json(value).into_response(),
        Err(crate::rpc::DispatchError::Unknown) => (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "error": format!("no command `{command}`") })),
        )
            .into_response(),
        Err(crate::rpc::DispatchError::Failed(message)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "error": message })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct PairVerifyBody {
    passcode: String,
    device_id: String,
    #[serde(default)]
    device_name: String,
    #[serde(default)]
    device_info: String,
    pairing_secret: String,
}

/// `POST /pair/verify` — a device presents the shared passcode and its pairing
/// secret and registers itself as pending. Rate-limited per IP so the passcode
/// cannot be brute-forced; emits `device://pending` so the host UI lights up.
async fn pair_verify(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(body): Json<PairVerifyBody>,
) -> Response {
    if !state.allow_verify(peer.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "too many attempts; wait a minute" })),
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

    match pairing_service::verify_and_register(
        db.get_connection(),
        &body.passcode,
        &body.device_id,
        &body.device_name,
        &body.device_info,
        &body.pairing_secret,
    )
    .await
    {
        Ok(outcome) => {
            if outcome.verified {
                if let Some(app) = state.app.as_ref() {
                    tauri::Manager::state::<Events>(app).inner().emit(
                        "device://pending",
                        json!({ "device_id": body.device_id, "device_name": body.device_name }),
                    );
                }
            }
            Json(json!({ "verified": outcome.verified, "message": outcome.message }))
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct PairStatusBody {
    device_id: String,
    pairing_secret: String,
}

/// `POST /pair/status` — a device polls for approval, presenting its pairing
/// secret. Returns the token once, scoped to the granted capabilities.
async fn pair_status(State(state): State<AppState>, Json(body): Json<PairStatusBody>) -> Response {
    let Some(db) = state.db() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "database unavailable" })),
        )
            .into_response();
    };

    match pairing_service::device_status(db.get_connection(), &body.device_id, &body.pairing_secret)
        .await
    {
        Ok(s) => Json(json!({
            "approved": s.approved,
            "token": s.token,
            "expires_at": s.expires_at,
            "capabilities": s.capabilities,
            "message": s.message,
        }))
        .into_response(),
        // The only error is a pairing-secret mismatch: a forbidden, not a 500.
        Err(_) => (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "pairing secret does not match" })),
        )
            .into_response(),
    }
}

/// True if a remote peer holding `caps` may receive this serialized frame. Fail
/// closed: an unparseable frame or an event mapped to no capability is withheld.
fn frame_allowed(frame: &str, caps: &[String]) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(frame) else {
        return false;
    };
    let Some(event) = value.get("event").and_then(|e| e.as_str()) else {
        return false;
    };
    match event_cap(event) {
        Some(cap) => caps.iter().any(|c| c == cap),
        None => false,
    }
}

/// Forward frames to one client until it goes away. `caps == None` is a local
/// peer and gets the whole stream; `Some` is a remote peer and is filtered.
async fn pump(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<Shared>,
    caps: Option<Vec<String>>,
) {
    loop {
        match rx.recv().await {
            Ok(wire) => {
                if let Some(caps) = &caps {
                    match event_cap(&wire.event) {
                        Some(cap) if caps.iter().any(|c| c == cap) => {}
                        _ => continue,
                    }
                }
                if socket
                    .send(Message::Text(wire.text.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                let notice = json!({ "event": "events://lagged", "payload": { "skipped": n } });
                if socket
                    .send(Message::Text(notice.to_string().into()))
                    .await
                    .is_err()
                {
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
    use crate::events::EventSinkExt;

    fn remote() -> IpAddr {
        "192.168.1.50".parse().unwrap()
    }

    fn with_header(name: header::HeaderName, value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(name, value.parse().unwrap());
        h
    }

    #[test]
    fn loopback_is_local_and_unrestricted() {
        let t = classify(
            "127.0.0.1".parse().unwrap(),
            &HeaderMap::new(),
            "/rpc/decrypt_credential",
            |_| None,
        )
        .unwrap();
        assert!(matches!(t, Trust::Local));
    }

    #[test]
    fn any_origin_is_refused_on_every_route() {
        let h = with_header(header::ORIGIN, "https://evil.example");
        // Even loopback, even a public path.
        assert!(classify("127.0.0.1".parse().unwrap(), &h, "/health", |_| None).is_err());
        assert!(classify(remote(), &h, "/pair/verify", |_| None).is_err());
        assert!(classify(remote(), &h, "/events", |_| None).is_err());
    }

    #[test]
    fn remote_with_a_live_token_is_scoped_to_its_caps() {
        let h = with_header(header::AUTHORIZATION, "Bearer good");
        let t = classify(remote(), &h, "/rpc/coder_list_threads", |tok| {
            (tok == "good").then(|| ("dev-1".to_string(), vec!["coder".to_string()]))
        })
        .unwrap();
        match t {
            Trust::Remote { device_id, caps } => {
                assert_eq!(device_id, "dev-1");
                assert_eq!(caps, vec!["coder".to_string()]);
            }
            other => panic!("expected Remote, got {other:?}"),
        }
    }

    #[test]
    fn remote_without_a_token_may_pair_but_not_call() {
        assert!(matches!(
            classify(remote(), &HeaderMap::new(), "/pair/verify", |_| None).unwrap(),
            Trust::Anon
        ));
        assert!(classify(
            remote(),
            &HeaderMap::new(),
            "/rpc/coder_list_threads",
            |_| None
        )
        .is_err());
        assert!(classify(remote(), &HeaderMap::new(), "/events", |_| None).is_err());
    }

    #[test]
    fn remote_with_a_bad_token_is_refused_on_protected_paths() {
        let h = with_header(header::AUTHORIZATION, "Bearer nope");
        assert!(classify(remote(), &h, "/rpc/coder_list_threads", |_| None).is_err());
    }

    #[test]
    fn events_are_filtered_by_capability_fail_closed() {
        assert_eq!(event_cap("coder://change"), Some("coder"));
        assert_eq!(event_cap("terminal-output"), Some("terminal.exec"));
        assert_eq!(event_cap("shell-integration-event"), Some("terminal.exec"));
        assert_eq!(event_cap("k8s://pods"), None);

        let coder = vec!["coder".to_string()];
        assert!(frame_allowed(
            r#"{"event":"coder://x","payload":{}}"#,
            &coder
        ));
        // Holds coder, not terminal.exec.
        assert!(!frame_allowed(
            r#"{"event":"terminal-output","payload":{}}"#,
            &coder
        ));
        // Unmapped event is withheld even from a capable peer.
        assert!(!frame_allowed(
            r#"{"event":"ai-stream-chunk-1","payload":{}}"#,
            &coder
        ));
        // Garbage frame is withheld.
        assert!(!frame_allowed("not json", &coder));
    }

    #[tokio::test]
    async fn frames_reach_a_subscriber_carrying_their_event_name() {
        let (tx, mut rx) = broadcast::channel(8);
        let sink = WsSink::new(tx);

        assert!(sink.emit("coder://change", json!({ "id": "abc" })));

        let wire = rx.recv().await.unwrap();
        assert_eq!(wire.event, "coder://change");
        let frame: Value = serde_json::from_str(&wire.text).unwrap();
        assert_eq!(frame["event"], "coder://change");
        assert_eq!(frame["payload"], json!({ "id": "abc" }));
    }

    #[tokio::test]
    async fn reports_undelivered_when_nobody_is_attached() {
        let (tx, rx) = broadcast::channel(8);
        let sink = WsSink::new(tx);
        drop(rx);
        assert!(!sink.emit("coder://change", json!({})));
    }

    /// The origin guard is the only thing between a visited page and this port,
    /// so it keeps an end-to-end test over a real listener.
    #[tokio::test]
    async fn browser_origins_are_refused_end_to_end() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let service =
                router(AppState::new(tx, None)).into_make_service_with_connect_info::<SocketAddr>();
            let _ = axum::serve(listener, service).await;
        });

        let client = reqwest::Client::new();
        let upgrade = |req: reqwest::RequestBuilder| {
            req.header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        };

        // Health is open to a loopback caller.
        let health = client
            .get(format!("http://{addr}/health"))
            .send()
            .await
            .unwrap();
        assert!(health.status().is_success());

        // A native (no-Origin) loopback client gets its upgrade.
        let allowed = upgrade(client.get(format!("http://{addr}/events")))
            .send()
            .await
            .unwrap();
        assert_eq!(allowed.status().as_u16(), 101);

        // The same request from a page is refused before the upgrade.
        let refused = upgrade(client.get(format!("http://{addr}/events")))
            .header("Origin", "https://evil.example")
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status().as_u16(), 403);
    }

    #[tokio::test]
    async fn rpc_refuses_browsers_and_separates_unknown_names_from_failures() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let service =
                router(AppState::new(tx, None)).into_make_service_with_connect_info::<SocketAddr>();
            let _ = axum::serve(listener, service).await;
        });

        let client = reqwest::Client::new();

        let from_a_page = client
            .post(format!("http://{addr}/rpc/coder_list_threads"))
            .header("Origin", "https://evil.example")
            .json(&json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(from_a_page.status().as_u16(), 403);

        let typo = client
            .post(format!("http://{addr}/rpc/no_such_command"))
            .json(&json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(typo.status().as_u16(), 404);

        // Registered, loopback (Local): got past the name check to dispatch,
        // which needs the app handle this bare listener does not have.
        let registered = client
            .post(format!("http://{addr}/rpc/coder_list_threads"))
            .send()
            .await
            .unwrap();
        assert_eq!(registered.status().as_u16(), 503);
    }
}
