//! `GET /control` — Android's live socket: a `hello` frame on connect, then
//! ping/pong, mouse/keyboard input (gated on the `remote.input` capability —
//! see [`input`]), and this app's existing event fanout, filtered by
//! capability exactly like `/events`. Mirror/audio/casting are later phases;
//! any other client frame gets an explicit "not implemented yet" rather than
//! silently hanging.
//!
//! Known gap: a token revoked while a `/control` socket is already open does
//! not drop that socket — it only stops a *new* connection. Live
//! mid-connection revocation is not needed this phase.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::domains::network::services::pairing_service;

use super::input;
use super::{forward_one, AppState, Shared, Trust};

/// The device name a client may send once over `X-Portal-Device`, e.g. to
/// replace a QR grant's "Unnamed device" placeholder with its real name on
/// first connect.
fn device_header(headers: &HeaderMap) -> Option<String> {
    let name = headers.get("X-Portal-Device")?.to_str().ok()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

pub async fn control(
    upgrade: WebSocketUpgrade,
    Extension(trust): Extension<Trust>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // The guard has already refused browsers and anonymous remotes (`/control`
    // is not in `is_public_path`), so trust here is always Local or Remote.
    let caps = match &trust {
        Trust::Remote { caps, .. } => Some(caps.clone()),
        _ => None,
    };

    if let (Some(name), Trust::Remote { device_id, .. }) = (device_header(&headers), &trust) {
        if let Some(db) = state.db() {
            let _ = pairing_service::rename_device(db.get_connection(), device_id, &name).await;
        }
    }

    let host_name = super::host_name();
    upgrade.on_upgrade(move |socket| run(socket, state.tx.subscribe(), caps, host_name))
}

/// `hello`'s fixed shape. `screen`/`id`/`mac` are placeholders — mirroring and
/// device identity are later phases; Android only needs these keys present.
fn hello(host_name: &str) -> Message {
    Message::Text(
        json!({
            "t": "hello",
            "name": host_name,
            "version": env!("CARGO_PKG_VERSION"),
            "screen": { "width": 0, "height": 0 },
            "id": null,
            "mac": null,
        })
        .to_string()
        .into(),
    )
}

/// Whether `caps` (`None` for a local, unrestricted peer) covers remote
/// input. Fail-closed like `frame_allowed`: a remote peer that never
/// mentions `remote.input` gets nothing.
fn input_allowed(caps: &Option<Vec<String>>) -> bool {
    match caps {
        None => true,
        Some(caps) => caps.iter().any(|c| c == crate::rpc::REMOTE_INPUT_CAP),
    }
}

/// One client-sent frame, as the reply text to send back (if any). `ping` is
/// answered unconditionally. A mouse/keyboard frame ([`input::FRAME_TYPES`])
/// is applied through `input` when the peer holds `remote.input`
/// ([`input_allowed`]) and injection is available on this PC; otherwise it
/// gets back an explicit refusal rather than silent failure, so a phone
/// granted only e.g. `coder` learns why its trackpad does nothing. Every
/// other frame gets an explicit "not implemented yet" so nothing on the
/// client waits forever for a reply that will never come. `None` for a frame
/// with no `t` at all, or that isn't JSON — dropped rather than answered.
fn handle_client_frame(
    text: &str,
    input: &mut Option<input::State>,
    caps: &Option<Vec<String>>,
) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    let t = value.get("t")?.as_str()?;

    let reply = if t == "ping" {
        json!({ "t": "pong", "seq": value.get("seq") })
    } else if input::FRAME_TYPES.contains(&t) {
        if !input_allowed(caps) {
            json!({ "t": "error", "message": "this device is not permitted to send input" })
        } else if let Some(state) = input {
            match state.dispatch(t, &value) {
                Ok(()) => return None,
                Err(message) => json!({ "t": "error", "message": format!("input failed: {message}") }),
            }
        } else {
            json!({ "t": "error", "message": "input injection is unavailable on this PC" })
        }
    } else {
        json!({ "t": "error", "message": format!("not implemented yet: {t}") })
    };
    Some(reply.to_string())
}

/// `hello` first, unprompted, then both directions at once: the broadcast
/// fanout forwarded like `pump` (via the shared `forward_one`), and the
/// client's own frames answered in place.
async fn run(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<Shared>,
    caps: Option<Vec<String>>,
    host_name: String,
) {
    if socket.send(hello(&host_name)).await.is_err() {
        return;
    }

    let mut input_state = input::State::new();
    pump_control(&mut socket, &mut rx, &caps, &mut input_state).await;
    // However the loop below exited — client gone, send failure, server
    // shutting down — a modifier this phone pressed and never released must
    // not stay down on the real keyboard.
    if let Some(state) = &mut input_state {
        state.release_held();
    }
}

/// The read/forward loop itself, split out from [`run`] so every exit path
/// runs through one `return` and [`run`] can guarantee the held-keys cleanup
/// after it rather than repeating it at each `return` here.
async fn pump_control(
    socket: &mut WebSocket,
    rx: &mut broadcast::Receiver<Shared>,
    caps: &Option<Vec<String>>,
    input_state: &mut Option<input::State>,
) {
    loop {
        tokio::select! {
            frame = rx.recv() => {
                if !forward_one(socket, frame, caps).await {
                    return;
                }
            }
            msg = socket.recv() => {
                let Some(Ok(msg)) = msg else { return };
                let Message::Text(text) = msg else { continue };
                let Some(reply) = handle_client_frame(&text, input_state, caps) else { continue };
                if socket.send(Message::Text(reply.into())).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use axum::Router;
    use futures_util::{SinkExt, StreamExt};
    use std::net::{Ipv4Addr, SocketAddr};
    use tokio_tungstenite::tungstenite::Message as WsMessage;
    use tokio_tungstenite::MaybeTlsStream;

    /// A bare router that skips `guard` and `AppState::db` entirely, handing
    /// every request a fixed `Trust` via an `Extension` layer — the same
    /// workaround `daemon::mod`'s own tests lean on for routes that need more
    /// than a bare listener can give (no real database, no real bearer).
    async fn spawn_control(trust: Trust) -> SocketAddr {
        let (tx, _rx) = broadcast::channel(8);
        let state = AppState::new(tx, None, 0);
        let app = Router::new()
            .route("/control", get(control))
            .layer(axum::Extension(trust))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app.into_make_service()).await;
        });
        addr
    }

    async fn next_json(
        ws: &mut tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> Value {
        match ws.next().await.unwrap().unwrap() {
            WsMessage::Text(text) => serde_json::from_str(&text).unwrap(),
            other => panic!("expected a text frame, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn control_sends_hello_on_upgrade() {
        let addr = spawn_control(Trust::Local).await;
        let (mut ws, _resp) = tokio_tungstenite::connect_async(format!("ws://{addr}/control"))
            .await
            .unwrap();

        let frame = next_json(&mut ws).await;
        assert_eq!(frame["t"], "hello");
        assert!(frame.get("name").is_some());
        assert_eq!(frame["version"], env!("CARGO_PKG_VERSION"));
        assert!(frame.get("screen").is_some());
    }

    #[tokio::test]
    async fn control_answers_ping_with_pong() {
        let addr = spawn_control(Trust::Remote {
            device_id: "dev-1".to_string(),
            caps: vec!["coder".to_string()],
        })
        .await;
        let (mut ws, _resp) = tokio_tungstenite::connect_async(format!("ws://{addr}/control"))
            .await
            .unwrap();
        next_json(&mut ws).await; // the hello frame, not under test here

        ws.send(WsMessage::Text(
            json!({ "t": "ping", "seq": 42 }).to_string(),
        ))
        .await
        .unwrap();
        assert_eq!(next_json(&mut ws).await, json!({ "t": "pong", "seq": 42 }));
    }

    /// A remote peer holding some other capability (`coder`, same as
    /// `control_answers_ping_with_pong`) must not be able to move the mouse —
    /// this is the test that would catch `remote.input` being forgotten from
    /// the gate, since it never reaches `input::State::dispatch` (and so
    /// never touches the real mouse) either way.
    #[tokio::test]
    async fn control_refuses_input_frames_without_the_capability() {
        let addr = spawn_control(Trust::Remote {
            device_id: "dev-1".to_string(),
            caps: vec!["coder".to_string()],
        })
        .await;
        let (mut ws, _resp) = tokio_tungstenite::connect_async(format!("ws://{addr}/control"))
            .await
            .unwrap();
        next_json(&mut ws).await; // hello

        ws.send(WsMessage::Text(
            json!({ "t": "mouse_move", "dx": 10, "dy": 10 }).to_string(),
        ))
        .await
        .unwrap();
        let reply = next_json(&mut ws).await;
        assert_eq!(reply["t"], "error");
        assert!(reply["message"].as_str().unwrap().contains("not permitted"));
    }

    #[test]
    fn input_allowed_is_unrestricted_for_a_local_peer_and_fail_closed_for_remote() {
        assert!(input_allowed(&None));
        assert!(!input_allowed(&Some(vec!["coder".to_string()])));
        assert!(input_allowed(&Some(vec![crate::rpc::REMOTE_INPUT_CAP.to_string()])));
    }
}
