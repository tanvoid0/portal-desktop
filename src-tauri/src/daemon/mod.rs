//! The local HTTP/WS surface, and the capability gate in front of it — the
//! Android remote-control pairing flow (phase 1 of the Android rewrite; see
//! `docs/development/REMOTE_ARCHITECTURE.md` for the phases before it).
//!
//! Split into submodules: this file keeps the router, the trust gate, and
//! `/health`/`/events`/`/rpc`; [`pairing`] is `/pair/request`; `control` is
//! `/control`; `discovery` is the UDP LAN responder.
//!
//! Runs as a task inside the Tauri process. Always binds loopback; the LAN
//! listener (same router, `0.0.0.0`) and the UDP discovery responder
//! additionally bind when the LAN toggle is on (the persisted setting, or
//! `PORTAL_DAEMON_LAN=1` as a dev override) — the one security gate here
//! protects all three.
//!
//! # Trust gate
//!
//! Every HTTP/WS request passes through [`classify`], which decides what the
//! peer is:
//!
//! - **Loopback peer, no `Origin` and no `Sec-Fetch-Site` header → `Local`**:
//!   the machine itself, unrestricted.
//! - **Either header is present → the free loopback pass above does not
//!   apply.** A browser cannot suppress `Origin` and WebSockets ignore CORS, so
//!   without this a visited web page could `fetch('http://127.0.0.1:18420/...')`
//!   and inherit full local trust with no token at all. `Sec-Fetch-Site` closes
//!   the same hole for a same-origin, same-method request (e.g. a page served
//!   by this daemon's own thin-client static route calling `fetch('/whoami')`
//!   on itself) — browsers omit `Origin` there but still send `Sec-Fetch-Site`,
//!   and no native Tauri/reqwest client ever sends it. A page-originated
//!   request (loopback or remote) instead falls through to the same bearer-token
//!   check below — a page cannot forge a token it was never handed, so this is
//!   what actually keeps it off a port that exposes terminal execution and a
//!   credential vault, not either header's value.
//! - **A live bearer token (header, or `?token=` on `/events` — a browser
//!   `WebSocket` cannot set a custom header) → `Remote { caps }`**: a paired
//!   device, allowed only the commands/events its granted capabilities cover.
//!   This applies to a page-originated loopback request too, now scoped like
//!   any other remote rather than exempted.
//! - **No token → `Anon`**: may reach `/health` and `/pair/request` to pair,
//!   nothing else — `/control` is deliberately NOT public, so an unauthenticated
//!   peer gets the same 401 `/rpc` gives it.
//!
//! `/rpc/{command}` refuses a remote peer whose caps do not cover the command's
//! `cap`; `/events` and `/control` drop frames whose [`event_cap`] the peer does
//! not hold.
//! Credentials and the other un-annotated commands carry no cap, so no remote
//! peer can ever reach them.

mod control;
mod discovery;
pub mod firewall;
mod input;
pub mod pairing;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Extension, Path as AxumPath, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

use crate::database::DatabaseManager;
use crate::domains::network::services::pairing_service;
use crate::domains::settings::services::settings_service::SettingsService;
use crate::events::EventSink;

/// Default port. Deliberately clear of agent-platform's 18410.
pub const DEFAULT_PORT: u16 = 18420;

/// Debug builds' default. 18420 is the installed release's — the shared task
/// board — and a dev build answering there would take the board into its own
/// database.
pub const DEV_PORT: u16 = 18421;

/// How many frames a slow client may fall behind before it starts losing the
/// oldest. Events are a live view, not a log.
const CHANNEL_CAPACITY: usize = 256;

/// Per-IP rate limit on `/pair/request`: a spammer gets this many tries per
/// window before being turned away.
const PAIR_REQUEST_WINDOW: Duration = Duration::from_secs(60);
const PAIR_REQUEST_MAX: u32 = 10;

/// The daemon's port: `PORTAL_DAEMON_PORT` if set and parseable, else
/// [`DEV_PORT`] for a debug build and [`DEFAULT_PORT`] for a release. The one
/// place this is decided, so `lib.rs`'s `setup` and the QR code (which must
/// encode the port actually bound) never drift apart.
pub fn configured_port() -> u16 {
    std::env::var("PORTAL_DAEMON_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(if crate::app_paths::DEV {
            DEV_PORT
        } else {
            DEFAULT_PORT
        })
}

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
    /// This listener's port, echoed back in `/pair/request`'s approved
    /// response so Android has it even though it already dialed this same
    /// port to ask.
    port: u16,
    /// Per-IP `/pair/request` attempt counters. A LAN has a handful of IPs, so
    /// the map stays tiny; entries are reset lazily per window rather than
    /// pruned.
    // ponytail: fixed-window counter; swap for a token bucket only if abuse shows.
    pair_hits: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
    /// Where the browser/other-desktop web client's static build lives, if it
    /// was found at startup. See [`client_web_dir`].
    static_dir: Option<PathBuf>,
}

impl AppState {
    fn new(tx: broadcast::Sender<Shared>, app: Option<tauri::AppHandle>, port: u16) -> Self {
        let static_dir = app.as_ref().and_then(client_web_dir);
        Self {
            tx,
            app,
            port,
            pair_hits: Arc::new(Mutex::new(HashMap::new())),
            static_dir,
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

    /// Record one `/pair/request` attempt from `ip`; false once over the limit.
    fn allow_pair_request(&self, ip: IpAddr) -> bool {
        let mut hits = self.pair_hits.lock().unwrap();
        let now = Instant::now();
        let entry = hits.entry(ip).or_insert((now, 0));
        if now.duration_since(entry.0) > PAIR_REQUEST_WINDOW {
            *entry = (now, 0);
        }
        entry.1 += 1;
        entry.1 <= PAIR_REQUEST_MAX
    }
}

/// This machine's display name, best-effort. Shown to a pairing phone as the
/// desktop it is connecting to (`/pair/request`'s approved response, built by
/// the `approve_device` Tauri command outside this module; the `/control`
/// `hello` frame, built by `control::control` inside it). No crate pulled in
/// for this — just the env vars the OS already sets. `pub`: reachable
/// crate-wide the same way `daemon::pairing::resolve_pending` is, even though
/// `daemon` itself stays a plain `mod` (see the module doc on `lib.rs:1`).
pub fn host_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Portal Desktop".to_string())
}

/// Where the thin-client build (browser, other-desktop — `pnpm build:client`)
/// is found, if anywhere.
///
/// Dev: next to the repo, resolved at compile time from this crate's own
/// manifest dir (`src-tauri/../build-client`) — mirrors how `tauri.conf.json`'s
/// `frontendDist` is a relative path in dev. Release: Tauri's resource dir,
/// where `bundle.resources` in `tauri.conf.json` ships it next to the installed
/// exe (built by the `beforeBundleCommand` hook there, so packaging fails loud
/// if someone forgets to build it rather than shipping an app with no web client).
/// `None` either way the directory was never built — the fallback route is then
/// simply not added, so a missing build degrades to "no web client", not a panic.
fn client_web_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    let dir = if crate::app_paths::DEV {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("build-client")
    } else {
        app.path().resource_dir().ok()?.join("build-client")
    };
    dir.join("index.html").is_file().then_some(dir)
}

/// Whether the LAN listener and UDP discovery responder should bind: the
/// persisted setting, or `PORTAL_DAEMON_LAN=1` as a dev-time override that
/// skips the UI toggle.
fn lan_enabled() -> bool {
    std::env::var("PORTAL_DAEMON_LAN").ok().as_deref() == Some("1")
        || SettingsService::new()
            .get_remote_lan_enabled()
            .unwrap_or(false)
}

/// Build the sink and start serving on loopback. Returns the sink immediately so
/// startup can wire it into the fanout; a bind failure leaves the app running
/// with the webview transport alone.
///
/// Also binds `0.0.0.0` (same port) plus the UDP discovery responder when the
/// LAN toggle is on (see [`lan_enabled`]) — off by default, since without it a
/// phone on the LAN times out reaching this port, which is what pairing needs
/// and what this guards.
pub fn spawn(port: u16, app: tauri::AppHandle) -> Arc<dyn EventSink> {
    let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
    let sink = WsSink::new(tx.clone());
    let state = AppState::new(tx, Some(app), port);

    serve_on(state.clone(), Ipv4Addr::LOCALHOST, port, "loopback");
    if lan_enabled() {
        serve_on(state, Ipv4Addr::UNSPECIFIED, port, "LAN");
        discovery::spawn_responder(port);
        firewall::spawn_check(port);
    }

    Arc::new(sink)
}

fn serve_on(state: AppState, host: Ipv4Addr, port: u16, label: &'static str) {
    tauri::async_runtime::spawn(async move {
        let router = router(state);
        let addr = SocketAddr::from((host, port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("[daemon] listening on http://{addr} ({label})");
                let service = router.into_make_service_with_connect_info::<SocketAddr>();
                if let Err(e) = axum::serve(listener, service).await {
                    eprintln!("[daemon] stopped ({label}): {e}");
                }
            }
            Err(e) => eprintln!("[daemon] could not bind {addr} ({label}): {e}"),
        }
    });
}

fn router(state: AppState) -> Router {
    let static_dir = state.static_dir.clone();
    let router = Router::new()
        .route("/health", get(health))
        .route("/whoami", get(whoami))
        .route("/events", get(events))
        .route("/rpc/{command}", post(rpc))
        .route("/pair/request", post(pairing::pair_request))
        .route("/control", get(control::control))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state);

    // The web client's own files: unauthenticated on purpose, same as any
    // other static site — it is just the app shell, no data. Added after
    // `.with_state` so it falls outside the `guard` layer above (middleware
    // only covers routes present before `.layer` is called); the pairing and
    // `/rpc` calls that shell makes still go through `guard` like any other
    // page-originated request.
    match static_dir {
        Some(dir) => {
            let index = dir.join("index.html");
            router.fallback_service(ServeDir::new(dir).not_found_service(ServeFile::new(index)))
        }
        None => router,
    }
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

/// A `key=value` pair out of a request's raw query string, undecoded — fine
/// for a pairing token, which is base64url (no characters that need escaping).
fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    query?.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| v.to_string())
    })
}

/// Paths an unauthenticated remote peer may reach: health and the pairing
/// handshake. Everything else — including `/control` — requires loopback or a
/// token.
fn is_public_path(path: &str) -> bool {
    matches!(path, "/health" | "/pair/request")
}

/// Decide what a request's peer is trusted as. Pure and synchronous so it can be
/// unit-tested with a synthetic remote peer and a fake token resolver — the
/// loopback-only integration harness cannot otherwise produce a remote peer.
fn classify(
    peer: IpAddr,
    headers: &HeaderMap,
    path: &str,
    query_token: Option<&str>,
    resolve: impl Fn(&str) -> Option<(String, Vec<String>)>,
) -> Result<Trust, Response> {
    // Loopback with no Origin AND no Sec-Fetch-Site is the machine itself
    // calling its own daemon (every native command does this). A page's own
    // `fetch`/WebSocket carries `Sec-Fetch-Site` even on a same-origin, same-
    // method GET where browsers omit `Origin` entirely — so once the daemon
    // started serving its own page (the thin-client static route), a script
    // on that page could otherwise `fetch('/whoami')` from loopback and get
    // unscoped `Local` trust with no token at all. No native Tauri/reqwest
    // client ever sends this header, so its mere presence is page-originated,
    // same as `Origin` — see the module doc for why that doesn't get the pass.
    if peer.is_loopback()
        && !headers.contains_key(header::ORIGIN)
        && !headers.contains_key("sec-fetch-site")
    {
        return Ok(Trust::Local);
    }

    // A live token is a known, scoped device - page-originated or not.
    let token = bearer(headers).or_else(|| query_token.map(str::to_string));
    if let Some(token) = token {
        if let Some((device_id, caps)) = resolve(&token) {
            return Ok(Trust::Remote { device_id, caps });
        }
    }

    // Unauthenticated: pairing and health only.
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
    } else if event.starts_with("agents") {
        Some("agents")
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

    // `/events` is the one path a browser `WebSocket` needs this for: the
    // handshake request can't carry a custom header.
    let query_token = (path == "/events")
        .then(|| query_param(req.uri().query(), "token"))
        .flatten();
    let token = bearer(&headers).or_else(|| query_token.clone());

    let identity = match (&token, state.db()) {
        (Some(t), Some(db)) => pairing_service::verify_token(db.get_connection(), t)
            .await
            .ok()
            .flatten(),
        _ => None,
    };

    let trust = match classify(
        peer_ip,
        &headers,
        &path,
        query_token.as_deref(),
        |t| match (&token, &identity) {
            (Some(tok), Some(id)) if tok == t => Some(id.clone()),
            _ => None,
        },
    ) {
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

/// Who the caller is, for a paired phone verifying its token (a bad token is
/// already a 401 from the guard). Local peers hold every capability.
async fn whoami(Extension(trust): Extension<Trust>) -> Response {
    let (local, device_id, caps) = match trust {
        Trust::Remote { device_id, caps } => (false, Some(device_id), caps),
        _ => (
            true,
            None,
            crate::rpc::capabilities()
                .into_iter()
                .map(String::from)
                .collect(),
        ),
    };
    axum::Json(json!({
        "name": host_name(),
        "version": env!("CARGO_PKG_VERSION"),
        "local": local,
        "device_id": device_id,
        "caps": caps,
    }))
    .into_response()
}

async fn events(
    upgrade: WebSocketUpgrade,
    Extension(trust): Extension<Trust>,
    State(state): State<AppState>,
) -> Response {
    // The guard has already refused anyone without a token or a loopback-no-
    // Origin pass, so trust here is Local (full stream) or Remote (filtered by
    // caps) - a page-originated connection only ever arrives as the latter.
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

    let caller = caller_for(&trust);
    match crate::rpc::dispatch(app, &caller, &command, args).await {
        Ok(value) => axum::Json(value).into_response(),
        Err(crate::rpc::DispatchError::Unknown) => (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "error": format!("no command `{command}`") })),
        )
            .into_response(),
        // A command-level `caller.require(...)` denial looks like any other
        // `Failed` to `dispatch` — recovered here via `denied()` so it gets
        // the same 403 shape the static per-command cap check above does,
        // rather than a generic 500.
        Err(crate::rpc::DispatchError::Failed(message)) => match caller.denied() {
            Some(cap) => (
                StatusCode::FORBIDDEN,
                axum::Json(json!({ "error": message, "capability": cap })),
            )
                .into_response(),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({ "error": message })),
            )
                .into_response(),
        },
    }
}

/// `Trust` as the dispatch layer sees it. `Anon` maps to an empty-caps
/// `Caller` (fails closed) rather than local/unrestricted, even though the
/// guard above never lets an `Anon` peer reach `/rpc` today — so a future
/// routing change can't silently hand it one.
fn caller_for(trust: &Trust) -> crate::rpc::Caller {
    match trust {
        Trust::Local => crate::rpc::Caller::local(),
        Trust::Remote { caps, .. } => crate::rpc::Caller::remote(caps.clone()),
        Trust::Anon => crate::rpc::Caller::remote(vec![]),
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

/// One broadcast frame, forwarded to `socket` if `caps` allows it (`None` is
/// unrestricted, i.e. a local peer). `false` once the socket should be
/// dropped — a send failure, or the channel closing for good.
async fn forward_one(
    socket: &mut WebSocket,
    frame: Result<Shared, broadcast::error::RecvError>,
    caps: &Option<Vec<String>>,
) -> bool {
    match frame {
        Ok(wire) => {
            if let Some(caps) = caps {
                match event_cap(&wire.event) {
                    Some(cap) if caps.iter().any(|c| c == cap) => {}
                    _ => return true,
                }
            }
            socket
                .send(Message::Text(wire.text.to_string().into()))
                .await
                .is_ok()
        }
        Err(broadcast::error::RecvError::Lagged(n)) => {
            let notice = json!({ "event": "events://lagged", "payload": { "skipped": n } });
            socket
                .send(Message::Text(notice.to_string().into()))
                .await
                .is_ok()
        }
        Err(broadcast::error::RecvError::Closed) => false,
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
        if !forward_one(&mut socket, rx.recv().await, &caps).await {
            return;
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
    fn loopback_without_origin_is_local_and_unrestricted() {
        let t = classify(
            "127.0.0.1".parse().unwrap(),
            &HeaderMap::new(),
            "/rpc/decrypt_credential",
            None,
            |_| None,
        )
        .unwrap();
        assert!(matches!(t, Trust::Local));
    }

    /// An `Origin` header means a page made the request — the free loopback
    /// pass does not apply to it even though the peer IP is 127.0.0.1, same as
    /// any other unauthenticated peer: health/pair are public, nothing else is.
    #[test]
    fn loopback_with_origin_does_not_get_the_free_local_pass() {
        let h = with_header(header::ORIGIN, "https://evil.example");
        let local = "127.0.0.1".parse().unwrap();
        assert!(matches!(
            classify(local, &h, "/health", None, |_| None).unwrap(),
            Trust::Anon
        ));
        assert!(classify(local, &h, "/rpc/decrypt_credential", None, |_| None).is_err());
        assert!(classify(local, &h, "/events", None, |_| None).is_err());
    }

    /// `Sec-Fetch-Site` closes the same hole for a same-origin request a
    /// browser sends with no `Origin` header at all (e.g. the thin-client
    /// static page's own `fetch('/whoami')` back at loopback) — this is the
    /// case `loopback_with_origin_does_not_get_the_free_local_pass` above
    /// doesn't cover, since there `Origin` is present.
    #[test]
    fn loopback_with_sec_fetch_site_does_not_get_the_free_local_pass() {
        let h = with_header(
            axum::http::HeaderName::from_static("sec-fetch-site"),
            "same-origin",
        );
        let local = "127.0.0.1".parse().unwrap();
        assert!(matches!(
            classify(local, &h, "/health", None, |_| None).unwrap(),
            Trust::Anon
        ));
        assert!(classify(local, &h, "/rpc/decrypt_credential", None, |_| None).is_err());
        assert!(classify(local, &h, "/whoami", None, |_| None).is_err());
    }

    /// The actual fix: a page-originated request, loopback or remote, with a
    /// live token is scoped exactly like any other paired device.
    #[test]
    fn origin_with_a_live_token_is_scoped_to_its_caps_not_exempted() {
        let h = {
            let mut h = with_header(header::ORIGIN, "http://localhost:18420");
            h.insert(header::AUTHORIZATION, "Bearer good".parse().unwrap());
            h
        };
        let resolve = |tok: &str| (tok == "good").then(|| ("dev-1".to_string(), vec!["coder".to_string()]));
        for peer in ["127.0.0.1".parse().unwrap(), remote()] {
            let t = classify(peer, &h, "/rpc/coder_list_threads", None, resolve).unwrap();
            match t {
                Trust::Remote { device_id, caps } => {
                    assert_eq!(device_id, "dev-1");
                    assert_eq!(caps, vec!["coder".to_string()]);
                }
                other => panic!("expected Remote, got {other:?}"),
            }
        }
    }

    /// `/events` is the one path a query-string token is honoured on — a
    /// browser `WebSocket` handshake cannot carry a custom header.
    #[test]
    fn events_accepts_a_query_token_a_header_token_is_refused_elsewhere() {
        let resolve = |tok: &str| (tok == "good").then(|| ("dev-1".to_string(), vec![]));
        assert!(matches!(
            classify(remote(), &HeaderMap::new(), "/events", Some("good"), resolve).unwrap(),
            Trust::Remote { .. }
        ));
        // Same query string on a different path is just ignored by `guard`
        // (it only reads it for `/events`) - `classify` itself would honour
        // whatever it's handed, so this documents the call site's job, not
        // `classify`'s.
        assert!(classify(remote(), &HeaderMap::new(), "/rpc/coder_list_threads", None, resolve).is_err());
    }

    #[test]
    fn remote_with_a_live_token_is_scoped_to_its_caps() {
        let h = with_header(header::AUTHORIZATION, "Bearer good");
        let t = classify(remote(), &h, "/rpc/coder_list_threads", None, |tok| {
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
            classify(remote(), &HeaderMap::new(), "/pair/request", None, |_| None).unwrap(),
            Trust::Anon
        ));
        assert!(classify(
            remote(),
            &HeaderMap::new(),
            "/rpc/coder_list_threads",
            None,
            |_| None
        )
        .is_err());
        assert!(classify(remote(), &HeaderMap::new(), "/events", None, |_| None).is_err());
    }

    #[test]
    fn remote_with_a_bad_token_is_refused_on_protected_paths() {
        let h = with_header(header::AUTHORIZATION, "Bearer nope");
        assert!(classify(remote(), &h, "/rpc/coder_list_threads", None, |_| None).is_err());
    }

    /// `/control` carries a bearer, not a public path — same 401 an
    /// unauthenticated remote gets on `/rpc` today, and since `classify`
    /// returns that `Err` before `guard` ever calls `next.run`, the request
    /// never reaches the handler, so the WebSocket upgrade never happens
    /// either.
    #[test]
    fn control_is_not_public_and_refuses_an_unauthenticated_remote() {
        assert!(classify(remote(), &HeaderMap::new(), "/control", None, |_| None).is_err());
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

    /// The token check is the only thing between a visited page and this port
    /// (the loopback free pass does not apply once `Origin` is set), so it
    /// keeps an end-to-end test over a real listener.
    #[tokio::test]
    async fn unauthenticated_browser_origins_are_refused_end_to_end() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let service = router(AppState::new(tx, None, 0))
                .into_make_service_with_connect_info::<SocketAddr>();
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

        // The same request from a page, with no token, is refused before the
        // upgrade — same 401 any other unauthenticated caller gets on a
        // protected path, not a blanket Origin-based 403.
        let refused = upgrade(client.get(format!("http://{addr}/events")))
            .header("Origin", "https://evil.example")
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status().as_u16(), 401);
    }

    #[tokio::test]
    async fn rpc_refuses_unauthenticated_browsers_and_separates_unknown_names_from_failures() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let service = router(AppState::new(tx, None, 0))
                .into_make_service_with_connect_info::<SocketAddr>();
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
        assert_eq!(from_a_page.status().as_u16(), 401);

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

    #[tokio::test]
    async fn whoami_reports_local_with_every_cap_and_refuses_remote_anon() {
        let reply = whoami(Extension(Trust::Local)).await;
        let body = axum::body::to_bytes(reply.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["local"], true);
        assert!(v["device_id"].is_null());
        assert_eq!(v["caps"].as_array().unwrap().len(), crate::rpc::capabilities().len());

        let paired = whoami(Extension(Trust::Remote {
            device_id: "dev-1".into(),
            caps: vec!["coder".into()],
        }))
        .await;
        let body = axum::body::to_bytes(paired.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["local"], false);
        assert_eq!(v["device_id"], "dev-1");
        assert_eq!(v["caps"], json!(["coder"]));

        // `/whoami` is not public: an unauthenticated remote gets the 401.
        assert!(classify(remote(), &HeaderMap::new(), "/whoami", None, |_| None).is_err());
    }

    #[test]
    fn caller_for_local_may_exec() {
        assert!(caller_for(&Trust::Local).require("agents.exec").is_ok());
    }

    #[test]
    fn caller_for_coder_only_remote_may_not_exec_but_may_coder() {
        let caller = caller_for(&Trust::Remote {
            device_id: "dev-1".into(),
            caps: vec!["coder".into()],
        });
        assert!(caller.require("agents.exec").is_err());
        assert!(caller.require("coder").is_ok());
    }

    #[test]
    fn caller_for_anon_fails_closed() {
        assert!(caller_for(&Trust::Anon).require("coder").is_err());
    }
}
