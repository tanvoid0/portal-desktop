//! The local HTTP/WS surface — phase 2 of `docs/development/REMOTE_ARCHITECTURE.md`.
//!
//! Everything the app can do lives behind services, not behind the webview. This
//! module is how anything that is not the webview reaches them: today a live
//! event stream, later `POST /rpc/{command}`.
//!
//! It runs as a task inside the Tauri process rather than a separate binary, so
//! there is no second process to supervise, orphan or fight over a port. The
//! crate split that would let it run headless is phase 1.
//!
//! # Trust boundary
//!
//! Binds loopback only. There is no token: reaching this port already means
//! running code on the machine. The one gap loopback does not close is a web
//! page — a browser will happily open `ws://127.0.0.1` from any origin, and
//! WebSockets are not subject to CORS — so upgrades carrying a browser `Origin`
//! are rejected outright. Remote binding, pairing and per-route capabilities are
//! phase 4, and this file must not bind `0.0.0.0` before that lands.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::events::EventSink;

/// Default port. Deliberately clear of agent-platform's 18410.
pub const DEFAULT_PORT: u16 = 18420;

/// How many frames a slow client may fall behind before it starts losing the
/// oldest. Events are a live view, not a log — a client that cannot keep up
/// wants the present, not a backlog it will never catch up on.
const CHANNEL_CAPACITY: usize = 256;

/// One frame on the wire. Named to match the Tauri event it mirrors so a client
/// can subscribe by the same string either way.
#[derive(Debug, Clone, serde::Serialize)]
struct Frame {
    event: String,
    payload: Value,
}

/// [`EventSink`] that publishes to every attached WebSocket.
#[derive(Debug, Clone)]
pub struct WsSink(broadcast::Sender<Arc<str>>);

impl WsSink {
    fn new(tx: broadcast::Sender<Arc<str>>) -> Self {
        Self(tx)
    }
}

impl EventSink for WsSink {
    fn emit_json(&self, event: &str, payload: Value) -> bool {
        let frame = Frame {
            event: event.to_string(),
            payload,
        };
        // Serialised once here rather than once per client: a coder run emits
        // the same frame to everyone watching.
        let Ok(text) = serde_json::to_string(&frame) else {
            return false;
        };
        // `send` fails only when there are no receivers, which is exactly the
        // "nobody is listening" answer the trait asks for. Deliberately does not
        // log: `utils::logger` mirrors through a sink while holding its own
        // mutex, so logging here can re-enter it — see REMOTE_ARCHITECTURE.md.
        self.0.send(Arc::from(text)).is_ok()
    }
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Arc<str>>,
    /// Absent only in tests that exercise the routing without a running app.
    app: Option<tauri::AppHandle>,
}

/// Build the sink and start serving on loopback.
///
/// Returns the sink immediately so startup can wire it into the fanout without
/// waiting for the socket; a bind failure is reported and leaves the app running
/// with the webview transport alone, since a busy port is not a reason to refuse
/// to start.
pub fn spawn(port: u16, app: tauri::AppHandle) -> Arc<dyn EventSink> {
    let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
    let sink = WsSink::new(tx.clone());

    let state = AppState { tx, app: Some(app) };
    tauri::async_runtime::spawn(async move {
        let app = router(state);

        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                println!("[daemon] listening on http://{addr}");
                if let Err(e) = axum::serve(listener, app).await {
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
        .with_state(state)
}

/// Refuse anything a browser sent.
///
/// Loopback stops other machines, not a page you happen to be visiting: a
/// browser opens `ws://127.0.0.1` from any origin and WebSockets ignore CORS.
/// Native clients send no `Origin`; browsers always do and cannot suppress it.
///
/// Deny-by-default on purpose. Phase 3 points the Tauri webview at these same
/// routes, and *that* origin gets allowlisted explicitly when it does — the list
/// does not start open.
fn browser_origin(headers: &HeaderMap) -> Option<Response> {
    headers.contains_key(axum::http::header::ORIGIN).then(|| {
        (
            StatusCode::FORBIDDEN,
            "browser origins are not permitted on this port",
        )
            .into_response()
    })
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
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    if let Some(refused) = browser_origin(&headers) {
        return refused;
    }

    upgrade.on_upgrade(move |socket| pump(socket, state.tx.subscribe()))
}

/// Run one command by name.
///
/// The name space is the command's own, identical to what the webview invokes
/// over IPC — the registry is populated by `#[rpc]` at each definition, so there
/// is no second list that could offer a different subset.
async fn rpc(
    AxumPath(command): AxumPath<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Option<axum::Json<Value>>,
) -> Response {
    if let Some(refused) = browser_origin(&headers) {
        return refused;
    }

    // Answered before anything else needs to exist: a name that was never
    // registered is a client bug, and stays a 404 whatever the app is doing.
    if !crate::rpc::is_registered(&command) {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "error": format!("no command `{command}`") })),
        )
            .into_response();
    }

    let Some(app) = state.app.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({ "error": "no app handle on this listener" })),
        )
            .into_response();
    };

    // No body means no arguments, which is most read commands.
    let args = body.map(|axum::Json(v)| v).unwrap_or(Value::Null);
    let args = if args.is_null() { json!({}) } else { args };

    match crate::rpc::dispatch(app, &command, args).await {
        Ok(value) => axum::Json(value).into_response(),
        // Distinguished from a failure so a client can tell "typo" from "it ran
        // and went wrong" — the two need different fixes.
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

/// Forward frames to one client until it goes away.
async fn pump(mut socket: WebSocket, mut rx: broadcast::Receiver<Arc<str>>) {
    loop {
        match rx.recv().await {
            Ok(frame) => {
                if socket
                    .send(Message::Text(frame.to_string().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            // Fell behind by more than the channel holds. Keep the connection and
            // resume from the present rather than dropping a client that was only
            // briefly slow.
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

    #[tokio::test]
    async fn frames_reach_a_subscriber_and_report_delivery() {
        let (tx, mut rx) = broadcast::channel(8);
        let sink = WsSink::new(tx);

        assert!(sink.emit("coder://change", json!({ "id": "abc" })));

        let frame: Value = serde_json::from_str(&rx.recv().await.unwrap()).unwrap();
        assert_eq!(frame["event"], "coder://change");
        assert_eq!(frame["payload"], json!({ "id": "abc" }));
    }

    /// The pty reader stops on this, so "no clients" must not read as delivered.
    #[tokio::test]
    async fn reports_undelivered_when_nobody_is_attached() {
        let (tx, rx) = broadcast::channel(8);
        let sink = WsSink::new(tx);
        drop(rx);

        assert!(!sink.emit("coder://change", json!({})));
    }

    /// Loopback keeps other machines out; it does not keep a web page out, since
    /// a browser may open `ws://127.0.0.1` from any origin and WebSockets are not
    /// subject to CORS. The guard is the only thing standing between a visited
    /// page and this machine's event stream, so it gets a test.
    #[tokio::test]
    async fn browser_origins_are_refused_the_event_stream() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, router(AppState { tx, app: None })).await;
        });

        let client = reqwest::Client::new();
        let upgrade = |req: reqwest::RequestBuilder| {
            req.header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Version", "13")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        };

        let health = client
            .get(format!("http://{addr}/health"))
            .send()
            .await
            .unwrap();
        assert!(health.status().is_success());

        // A native client sends no Origin and gets its upgrade.
        let allowed = upgrade(client.get(format!("http://{addr}/events")))
            .send()
            .await
            .unwrap();
        assert_eq!(allowed.status().as_u16(), 101);

        // Same request from a page is refused before the upgrade — proving the
        // guard rejects it, not a malformed handshake.
        let refused = upgrade(client.get(format!("http://{addr}/events")))
            .header("Origin", "https://evil.example")
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status().as_u16(), 403);
    }

    /// `/rpc` reaches services, so the same guard has to hold there — and a
    /// client needs to tell a name it got wrong from a call that went wrong.
    #[tokio::test]
    async fn rpc_refuses_browsers_and_separates_unknown_names_from_failures() {
        let (tx, _rx) = broadcast::channel(8);
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, router(AppState { tx, app: None })).await;
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

        // Registered, so routing got past the name check and reached dispatch —
        // which needs the app handle this bare listener does not have.
        let registered = client
            .post(format!("http://{addr}/rpc/coder_list_threads"))
            .send()
            .await
            .unwrap();
        assert_eq!(registered.status().as_u16(), 503);
    }
}
