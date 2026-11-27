//! Portal Mobile: a thin client of the Portal Desktop daemon. Every `invoke`
//! the (unchanged) frontend makes is forwarded to `POST /rpc/{command}`; the
//! daemon's `/events` frames are re-emitted as Tauri events. Only the
//! `mobile_*` commands run locally; `mobile_control_send` is the page's way
//! onto `/control`, whose bearer token stays here.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::future::{select, Either};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::ipc::{InvokeBody, InvokeError};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const NOT_CONNECTED: &str = "not connected to a Portal Desktop";

/// Frames the `/control` socket may hold unsent. A trackpad produces them
/// faster than a stalled connection drains them; past this `mobile_control_send`
/// fails rather than buffering without bound.
const CONTROL_QUEUE: usize = 64;

/// Longest `{t:"error"}` message (in characters) the page is handed from `/control`.
const CONTROL_ERROR_MAX: usize = 200;

#[derive(Clone, Serialize, Deserialize)]
struct Conn {
    host: String,
    port: u16,
    token: String,
    name: String,
    /// This device's own name, as typed when it paired; `/control` sends it so
    /// the desktop can label a grant. Absent in files written before it existed.
    #[serde(default)]
    device: Option<String>,
    /// The desktop's stable id (`AppSettings::host_id`, carried in its discovery
    /// reply), when known — lets `discover()` recognize this same PC again after
    /// its LAN IP changes. Absent for a manual/QR pairing, which never saw one.
    #[serde(default)]
    id: Option<String>,
}

struct Inner {
    conn: Option<Conn>,
    caps: Vec<String>,
    /// The events socket is currently up.
    live: bool,
    bridge: Option<tauri::async_runtime::JoinHandle<()>>,
    /// Frame queue of the one lazily-opened `/control` socket (`Mobile::queue_control`).
    /// Dropping it ends that socket; a socket that ended on its own leaves a closed
    /// sender here, which the next send replaces. ponytail: no abort handle - a session
    /// still dialling when this drops ends at its first read; add a JoinHandle if a hung
    /// connect ever matters.
    control: Option<mpsc::Sender<String>>,
    /// The app is in the foreground (Android `onResume`/`onPause`; always `true` on desktop,
    /// which has no background state to speak of). Alerts only post a notification while this
    /// is `false` — the user is already looking at the screen the socket is updating otherwise.
    foreground: bool,
    /// `"{execution_id}:{status}"` of every terminal pipeline status already notified, so a
    /// pipeline that emits more than one terminal frame doesn't double-notify. ponytail: grows
    /// for the life of the process; fine for a run of pipeline executions, revisit with an LRU
    /// cap if a session ever runs long enough for it to matter.
    notified: HashSet<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            conn: None,
            caps: Vec::new(),
            live: false,
            bridge: None,
            control: None,
            foreground: true,
            notified: HashSet::new(),
        }
    }
}

#[derive(Clone, Default)]
struct Mobile(Arc<Mutex<Inner>>);

impl Mobile {
    fn status(&self) -> Value {
        let s = self.0.lock().unwrap();
        let c = s.conn.as_ref();
        json!({
            "connected": c.is_some() && s.live,
            "host": c.map(|c| &c.host),
            "port": c.map(|c| c.port),
            "name": c.map(|c| &c.name),
            "caps": s.caps,
        })
    }

    fn emit_status(&self, app: &AppHandle) {
        let _ = app.emit("mobile://connection", self.status());
    }

    fn conn(&self) -> Option<Conn> {
        self.0.lock().unwrap().conn.clone()
    }

    /// Stop the bridge and forget the connection (file included).
    fn clear(&self, app: &AppHandle) {
        self.reset(None);
        self.forgotten(app);
    }

    /// The state half of `clear`: drop the bridge and the connection. With a `token`, only
    /// while it is still the connection's, checked under the same lock as the wipe — a
    /// `/control` dial that was in flight when `set()` re-paired must not wipe the new
    /// connection when the old token's 401 finally lands. `false`: nothing touched.
    fn reset(&self, token: Option<&str>) -> bool {
        let mut s = self.0.lock().unwrap();
        if token.is_some_and(|t| s.conn.as_ref().map(|c| c.token.as_str()) != Some(t)) {
            return false;
        }
        if let Some(b) = s.bridge.take() {
            b.abort();
        }
        *s = Inner::default();
        true
    }

    /// The other half of `clear`: the stored file, and the UI.
    fn forgotten(&self, app: &AppHandle) {
        if let Some(p) = conn_path(app) {
            let _ = std::fs::remove_file(p);
        }
        self.emit_status(app);
    }

    /// Store `conn`, persist it, (re)start the events bridge.
    fn set(&self, app: &AppHandle, conn: Conn) {
        if let Some(p) = conn_path(app) {
            let _ = std::fs::create_dir_all(p.parent().unwrap());
            // ponytail: plain app-private file; move to Android Keystore if the threat model grows.
            let _ = std::fs::write(p, serde_json::to_vec(&conn).unwrap_or_default());
        }
        {
            let mut s = self.0.lock().unwrap();
            if let Some(b) = s.bridge.take() {
                b.abort();
            }
            s.conn = Some(conn);
            s.live = false;
            s.control = None; // its socket carries the old token
        }
        let (me, app2) = (self.clone(), app.clone());
        let handle = tauri::async_runtime::spawn(async move { me.bridge(app2).await });
        self.0.lock().unwrap().bridge = Some(handle);
        self.emit_status(app);
    }

    /// Only called from the `#[cfg(mobile)]` window-event handler in `run()`; the host build
    /// (desktop `cargo check`, no background concept) never flips it off `true`.
    #[cfg_attr(not(mobile), allow(dead_code))]
    fn set_foreground(&self, fg: bool) {
        self.0.lock().unwrap().foreground = fg;
    }

    /// Posts a native notification for `alert_for`'s mapping of one `/events` frame, but only
    /// while the app is backgrounded, and (for a pipeline's terminal status) at most once per
    /// execution — some pipelines emit more than one terminal-status frame.
    fn notify(&self, app: &AppHandle, event: &str, payload: &Value) {
        if self.0.lock().unwrap().foreground {
            return;
        }
        let Some(alert) = alert_for(event, payload) else {
            return;
        };
        if event == "pipeline-execution-update" {
            let key = format!(
                "{}:{}",
                payload["id"].as_str().unwrap_or(""),
                payload["status"].as_str().unwrap_or("")
            );
            if !self.0.lock().unwrap().notified.insert(key) {
                return;
            }
        }
        // The route is carried as `extra` data so a tap can navigate there (#641); ignored by
        // the OS on desktop, where there's no in-app route to open from a notification anyway.
        let _ = app
            .notification()
            .builder()
            .title(alert.title)
            .body(alert.body)
            .extra("route", alert.route)
            .show();
    }

    fn set_live(&self, app: &AppHandle, live: bool) {
        let changed = {
            let mut s = self.0.lock().unwrap();
            std::mem::replace(&mut s.live, live) != live
        };
        if changed {
            self.emit_status(app);
        }
    }

    /// Queue one frame for `/control`. `Some(new)` means no socket was live, so this call
    /// made a fresh queue and the caller must open the socket on it
    /// ([`Mobile::open_control`]); `None` means the live socket will send the frame.
    /// Never waits on the network, so a slow or dead desktop cannot stall the caller.
    fn queue_control(&self, frame: String) -> Result<Option<NewControl>, String> {
        let mut s = self.0.lock().unwrap();
        let conn = s.conn.as_ref().ok_or(NOT_CONNECTED)?;
        let frame = match &s.control {
            Some(tx) => match tx.try_send(frame) {
                Ok(()) => return Ok(None),
                Err(TrySendError::Full(_)) => return Err("the control socket is backed up".into()),
                Err(TrySendError::Closed(frame)) => frame, // it ended; a fresh one opens below
            },
            None => frame,
        };
        let req = control_request(conn).ok_or("bad connection details")?;
        let token = conn.token.clone();
        let (tx, rx) = mpsc::channel(CONTROL_QUEUE);
        let _ = tx.try_send(frame); // a fresh queue has room
        s.control = Some(tx);
        Ok(Some(NewControl { req, rx, token }))
    }

    /// Run the `/control` socket for a queue made by [`Mobile::queue_control`]: the
    /// daemon's `{t:"error"}` refusals reach the page as `mobile://control-error`.
    fn open_control(&self, app: &AppHandle, new: NewControl) {
        let (me, app) = (self.clone(), app.clone());
        tauri::async_runtime::spawn(async move {
            let emit = |err: Value| {
                let _ = app.emit("mobile://control-error", err);
            };
            me.control_task(new, emit, || me.forgotten(&app)).await;
        });
    }

    /// [`control_session`], then — on a 401, the token revoked — disconnect like every
    /// other call, but only if the token this socket dialled with is still the
    /// connection's: after a re-pair its 401 is stale news. `forgotten` runs only then.
    async fn control_task(
        &self,
        new: NewControl,
        on_error: impl Fn(Value),
        forgotten: impl FnOnce(),
    ) {
        if let Err(Stop::Revoked) = control_session(new.req, new.rx, on_error).await {
            if self.reset(Some(&new.token)) {
                forgotten();
            }
        }
    }

    /// Whoami + events socket, forever, with capped backoff. A 401 anywhere
    /// means the token was revoked: disconnect.
    async fn bridge(&self, app: AppHandle) {
        let mut delay = 1u64;
        loop {
            let Some(conn) = self.conn() else { return };
            match self.run_socket(&app, &conn).await {
                Err(Stop::Revoked) => return self.clear(&app),
                Ok(()) => delay = 1,
                Err(Stop::Other) => delay = (delay * 2).min(30),
            }
            self.set_live(&app, false);
            tokio::time::sleep(Duration::from_secs(delay)).await;
        }
    }

    async fn run_socket(&self, app: &AppHandle, conn: &Conn) -> Result<(), Stop> {
        let who = whoami(&conn.host, conn.port, &conn.token).await.map_err(|e| {
            if e.revoked {
                Stop::Revoked
            } else {
                Stop::Other
            }
        })?;
        self.0.lock().unwrap().caps = caps_of(&who);

        let req = ws_request(conn, "/events").ok_or(Stop::Other)?;
        let mut ws = dial(req).await?;
        self.set_live(app, true);
        while let Some(Ok(msg)) = ws.next().await {
            let Ok(text) = msg.into_text() else { continue };
            let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            if let Some(event) = frame["event"].as_str() {
                let payload = frame["payload"].clone();
                let _ = app.emit(event, payload.clone());
                self.notify(app, event, &payload);
            }
        }
        Ok(())
    }
}

enum Stop {
    Revoked,
    Other,
}

/// A `/control` socket still to be opened: its handshake, the queue it drains, and the
/// token it dials with (kept so a 401 can tell whether that token is still current).
struct NewControl {
    req: Request,
    rx: mpsc::Receiver<String>,
    token: String,
}

/// A WebSocket handshake to `path` on the connected desktop. The token rides in the
/// `Authorization` header, never the URL, so it cannot end up in a log or a proxy.
fn ws_request(conn: &Conn, path: &str) -> Option<Request> {
    let mut req = format!("ws://{}{path}", hostport(&conn.host, conn.port))
        .into_client_request()
        .ok()?;
    req.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", conn.token).parse().ok()?,
    );
    Some(req)
}

/// The `/control` handshake: `ws_request`'s plus this device's own name, when it has
/// one the daemon can read. It reads the header as visible ASCII, so any other name is
/// left off rather than failing the socket.
fn control_request(conn: &Conn) -> Option<Request> {
    let mut req = ws_request(conn, "/control")?;
    let name = conn.device.as_deref().filter(|d| d.is_ascii());
    if let Some(name) = name.and_then(|d| d.parse().ok()) {
        req.headers_mut().insert("X-Portal-Device", name);
    }
    Some(req)
}

/// Open `req`'s socket; a 401 means the token was revoked.
async fn dial(req: Request) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Stop> {
    match tokio_tungstenite::connect_async(req).await {
        Ok((ws, _)) => Ok(ws),
        Err(WsError::Http(r)) if r.status().as_u16() == 401 => Err(Stop::Revoked),
        Err(_) => Err(Stop::Other),
    }
}

/// The only `/control` frame the page needs: a `{t:"error"}` refusal (an
/// unimplemented frame type), re-emitted as just `{t, message}` with the message capped — the
/// daemon chooses the frame, so nothing else in it is handed on. `hello` and the fanout
/// the daemon forwards on this socket are already delivered by `/events`, so everything
/// else is dropped.
fn control_error(text: &str) -> Option<Value> {
    let frame: Value = serde_json::from_str(text).ok()?;
    if frame["t"] != "error" {
        return None;
    }
    let message = frame["message"].as_str().unwrap_or_default();
    let message: String = message.chars().take(CONTROL_ERROR_MAX).collect();
    Some(json!({ "t": "error", "message": message }))
}

/// One `/control` socket's life: dial, write each queued frame, hand the daemon's
/// refusals to `on_error`. `Ok` when the queue's sender was dropped (disconnect) or
/// the socket ended; the next send then opens a fresh one. ponytail: no reconnect or
/// backoff here - a dead desktop costs one dial per send, and `/events` already shows
/// it; add a backoff in `queue_control` if that dial rate ever matters.
async fn control_session(
    req: Request,
    mut rx: mpsc::Receiver<String>,
    on_error: impl Fn(Value),
) -> Result<(), Stop> {
    let (mut sink, mut stream) = dial(req).await?.split();
    loop {
        let queued = std::pin::pin!(rx.recv());
        match select(queued, stream.next()).await {
            Either::Left((Some(text), _)) => {
                sink.send(Message::Text(text))
                    .await
                    .map_err(|_| Stop::Other)?;
            }
            Either::Right((Some(Ok(msg)), _)) => {
                if let Some(err) = msg.into_text().ok().and_then(|t| control_error(&t)) {
                    on_error(err);
                }
            }
            _ => return Ok(()),
        }
    }
}

/// A notification to surface for an `/events` frame: title, body, and the
/// in-app route a tap should open.
#[derive(Debug, PartialEq, Eq)]
struct Alert {
    title: String,
    body: String,
    route: String,
}

/// Maps one `/events` frame (`event` name + its `payload`, same shapes
/// `run_socket` re-emits as Tauri events) to an optional [`Alert`]. Pure and
/// synchronous — `Mobile::notify` wires it to the OS notification API.
fn alert_for(event: &str, payload: &Value) -> Option<Alert> {
    match event {
        "agents://event" => {
            let run_id = payload["run_id"].as_str()?;
            let route = format!("/ai/coder/claude-code?session={run_id}");
            match payload["event"]["type"].as_str()? {
                // Wording below matches app-shell.svelte's in-app toast for the same
                // event (#642) — the two surfaces shouldn't read differently.
                "permission_request" => {
                    let tool = payload["event"]["tool_name"].as_str().unwrap_or("a tool");
                    Some(Alert {
                        title: "Agent needs your approval".into(),
                        body: format!("Wants to run {tool}"),
                        route,
                    })
                }
                // `is_error` turns run finishes mirror the backend's own
                // convention (see `AgentEvent::TurnDone`'s doc comment): the
                // `Exited` failure alert below covers that case instead.
                "turn_done" if !payload["event"]["is_error"].as_bool().unwrap_or(false) => {
                    let body: String = payload["event"]["final_text"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.chars().take(120).collect::<String>())
                        .unwrap_or_else(|| "The run completed".into());
                    Some(Alert { title: "Agent finished".into(), body, route })
                }
                "exited" => match payload["event"]["code"].as_i64()? {
                    0 => None,
                    code => Some(Alert {
                        title: "Agent run failed".into(),
                        body: format!("Exited with code {code}"),
                        route,
                    }),
                },
                _ => None,
            }
        }
        // Every provider's thread frame, but only Portal threads alert from here: CLI
        // runs still alert from `agents://event` above (also when the desktop predates
        // the thread log) until S5 removes those emits; alerting both would double.
        "coder://thread-event" => {
            if payload["provider"].as_str()? != "portal" {
                return None;
            }
            let route = format!("/ai/coder?id={}", payload["thread_id"].as_str()?);
            let ev = &payload["event"];
            match ev["type"].as_str()? {
                // Tool name only, like the CLI arm: `summary` can hold a whole shell
                // command (`$ curl -H "Authorization: …"`), which a lock screen shows.
                "approval_requested" => {
                    let tool = ev["approval"]["tool"].as_str().unwrap_or("a tool");
                    Some(Alert {
                        title: "Agent needs your approval".into(),
                        body: format!("Wants to run {tool}"),
                        route,
                    })
                }
                // stopped/interrupted: the user ended it, nothing to say.
                "turn_ended" => match ev["outcome"].as_str()? {
                    "exhausted" => Some(Alert {
                        title: "Agent paused".into(),
                        body: "It hit its step limit and is waiting for you to continue".into(),
                        route,
                    }),
                    "completed" => {
                        let body: String = ev["final_text"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.chars().take(120).collect::<String>())
                            .unwrap_or_else(|| "The run completed".into());
                        Some(Alert { title: "Agent finished".into(), body, route })
                    }
                    "error" => {
                        let body: String = ev["error"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.chars().take(120).collect::<String>())
                            .unwrap_or_else(|| "The run failed".into());
                        Some(Alert { title: "Agent run failed".into(), body, route })
                    }
                    _ => None,
                },
                _ => None,
            }
        }
        "pipeline-execution-update" => {
            let execution_id = payload["id"].as_str()?;
            // Same route the in-app toast uses (app-shell.svelte): one page for this
            // event regardless of surface. The old `/projects/{id}/pipelines/run/...`
            // needed a `projects` cap a phone may not have (#642).
            let route = format!("/pipeline-runs?run={execution_id}");
            match payload["status"].as_str()? {
                "success" => Some(Alert {
                    title: "Pipeline run finished".into(),
                    body: "The pipeline run completed".into(),
                    route,
                }),
                "failed" => Some(Alert {
                    title: "Pipeline run failed".into(),
                    body: "The pipeline run failed".into(),
                    route,
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

struct HttpErr {
    msg: String,
    revoked: bool,
}

fn conn_path(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_data_dir().ok()?.join("connection.json"))
}

fn hostport(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn caps_of(who: &Value) -> Vec<String> {
    who["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_default()
}

async fn whoami(host: &str, port: u16, token: &str) -> Result<Value, HttpErr> {
    let res = client(Duration::from_secs(8))
        .get(format!("http://{}/whoami", hostport(host, port)))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| HttpErr {
            msg: format!("cannot reach {host}:{port}: {e}"),
            revoked: false,
        })?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(HttpErr {
            msg: rpc_error(status.as_u16(), &body),
            revoked: status.as_u16() == 401,
        });
    }
    serde_json::from_str(&body).map_err(|e| HttpErr {
        msg: format!("bad /whoami reply: {e}"),
        revoked: false,
    })
}

/// Error text for a non-2xx `/rpc` reply: the daemon's own body, verbatim.
fn rpc_error(status: u16, body: &str) -> String {
    if body.trim().is_empty() {
        format!("daemon answered HTTP {status}")
    } else {
        body.to_string()
    }
}

fn pair_error(status: u16) -> String {
    match status {
        403 => "pairing was denied on the desktop".into(),
        408 => "nobody answered the pairing request on the desktop in time".into(),
        429 => "too many pairing attempts; wait a minute and retry".into(),
        s => format!("pairing failed: desktop answered HTTP {s}"),
    }
}

/// One UDP discovery reply -> a result row (host comes from the sender).
fn parse_reply(buf: &[u8], from: std::net::IpAddr) -> Option<Value> {
    let v: Value = serde_json::from_slice(buf).ok()?;
    Some(json!({
        "host": from.to_string(),
        "port": v["port"].as_u64()?,
        "name": v["name"].as_str().unwrap_or("Portal Desktop"),
        "version": v["version"].as_str().unwrap_or(""),
        "id": v["id"].as_str(),
        "paired": false,
    }))
}

#[derive(Deserialize)]
struct Args {
    host: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    device: Option<String>,
    frame: Option<Value>,
    /// The desktop's stable id, when pairing from a `mobile_discover` result row.
    id: Option<String>,
}

async fn local(app: AppHandle, m: Mobile, cmd: &str, args: Value) -> Result<Value, String> {
    let a: Args = serde_json::from_value(if args.is_null() { json!({}) } else { args })
        .map_err(|e| format!("bad arguments: {e}"))?;
    let need = |v: Option<String>, n: &str| v.ok_or(format!("missing `{n}`"));
    match cmd {
        "mobile_status" => Ok(m.status()),
        "mobile_disconnect" => {
            m.clear(&app);
            Ok(m.status())
        }
        "mobile_discover" => Ok(discover_marking_paired(&app, &m).await),
        "mobile_connect" => {
            let host = need(a.host, "host")?;
            let port = a.port.ok_or("missing `port`")?;
            let token = need(a.token, "token")?;
            let who = whoami(&host, port, &token).await.map_err(|e| e.msg)?;
            let name = who["name"].as_str().unwrap_or("Portal Desktop").to_string();
            m.set(&app, Conn { host, port, token, name, device: a.device, id: a.id });
            Ok(m.status())
        }
        "mobile_pair" => {
            let host = need(a.host, "host")?;
            let port = a.port.ok_or("missing `port`")?;
            let device = need(a.device, "device")?;
            let res = client(Duration::from_secs(175))
                .post(format!("http://{}/pair/request", hostport(&host, port)))
                .json(&json!({ "device": device }))
                .send()
                .await
                .map_err(|e| format!("cannot reach {host}:{port}: {e}"))?;
            if !res.status().is_success() {
                return Err(pair_error(res.status().as_u16()));
            }
            let v: Value = res
                .json()
                .await
                .map_err(|e| format!("bad pairing reply: {e}"))?;
            let token = v["token"].as_str().ok_or("pairing reply had no token")?;
            let name = v["name"].as_str().unwrap_or("Portal Desktop").to_string();
            let port = v["port"].as_u64().map_or(port, |p| p as u16);
            m.set(
                &app,
                Conn { host, port, token: token.to_string(), name, device: Some(device), id: a.id },
            );
            Ok(m.status())
        }
        // Fire-and-forget: the frame is only queued; the socket, its token and its
        // refusals (`mobile://control-error`) stay on this side of the bridge.
        "mobile_control_send" => {
            let frame = a.frame.ok_or("missing `frame`")?;
            if let Some(new) = m.queue_control(frame.to_string())? {
                m.open_control(&app, new);
            }
            Ok(Value::Null)
        }
        _ => Err(format!("unknown command {cmd}")),
    }
}

async fn discover() -> Value {
    let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await else {
        return json!([]);
    };
    let _ = sock.set_broadcast(true);
    if sock
        .send_to(b"PORTALREMOTE?", "255.255.255.255:8765")
        .await
        .is_err()
    {
        return json!([]);
    }
    let mut found: Vec<Value> = vec![];
    let mut buf = [0u8; 1024];
    let _ = tokio::time::timeout(Duration::from_millis(1500), async {
        while let Ok((n, from)) = sock.recv_from(&mut buf).await {
            if let Some(row) = parse_reply(&buf[..n], from.ip()) {
                if !found
                    .iter()
                    .any(|f| f["host"] == row["host"] && f["port"] == row["port"])
                {
                    found.push(row);
                }
            }
        }
    })
    .await;
    json!(found)
}

/// Tags each row of a `discover()` result `"paired": bool` against `conn`'s `id`
/// (when it has one), and returns an updated [`Conn`] when the matching row's
/// host/port moved — the paired PC's id is the same even after a DHCP lease
/// changes its address, so this is what a caller re-persists to follow it.
/// Pure and synchronous so the matching logic is tested without a real socket.
fn mark_paired(rows: &mut Value, conn: &Conn) -> Option<Conn> {
    let id = conn.id.as_deref().filter(|id| !id.is_empty())?;
    let arr = rows.as_array_mut()?;
    for row in arr {
        if row["id"].as_str() != Some(id) {
            continue;
        }
        row["paired"] = json!(true);
        let (Some(host), Some(port)) = (row["host"].as_str(), row["port"].as_u64()) else {
            continue;
        };
        if host != conn.host || port as u16 != conn.port {
            return Some(Conn { host: host.to_string(), port: port as u16, ..conn.clone() });
        }
    }
    None
}

/// [`discover`], then [`mark_paired`] against the stored connection and, if it
/// moved, persisted via [`Mobile::set`] so the next reconnect dials the right address.
async fn discover_marking_paired(app: &AppHandle, m: &Mobile) -> Value {
    let mut rows = discover().await;
    if let Some(conn) = m.conn() {
        if let Some(moved) = mark_paired(&mut rows, &conn) {
            m.set(app, moved);
        }
    }
    rows
}

async fn forward(app: AppHandle, m: Mobile, cmd: String, args: Value) -> Result<Value, String> {
    let conn = m.conn().ok_or(NOT_CONNECTED)?;
    if !cmd.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("invalid command name {cmd}"));
    }
    let args = if args.is_null() { json!({}) } else { args };
    let res = client(Duration::from_secs(120))
        .post(format!("http://{}/rpc/{cmd}", hostport(&conn.host, conn.port)))
        .bearer_auth(&conn.token)
        .json(&args)
        .send()
        .await
        .map_err(|e| format!("cannot reach {}:{}: {e}", conn.host, conn.port))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if status.as_u16() == 401 {
        // Only clear when this request's token is still the stored one. A 401
        // for a token already replaced by a re-pair must not wipe the fresh
        // connection (the /control reset(Some(token)) and /events generation
        // guards protect their paths the same way; forward() is the untracked
        // one-shot).
        if m.conn().is_some_and(|cur| cur.token == conn.token) {
            m.clear(&app);
        }
    }
    if !status.is_success() {
        return Err(rpc_error(status.as_u16(), &body));
    }
    if body.is_empty() {
        Ok(Value::Null)
    } else {
        serde_json::from_str(&body).map_err(|e| format!("bad reply: {e}"))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mobile = Mobile::default();
    let handler_state = mobile.clone();
    #[cfg(mobile)]
    let window_event_state = handler_state.clone();
    let b = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            if let Some(conn) = conn_path(app.handle())
                .and_then(|p| std::fs::read(p).ok())
                .and_then(|b| serde_json::from_slice::<Conn>(&b).ok())
            {
                mobile.set(app.handle(), conn);
            }
            Ok(())
        })
        .invoke_handler(move |invoke| {
            let cmd = invoke.message.command().to_string();
            let app = invoke.message.webview().app_handle().clone();
            let args = match invoke.message.payload() {
                InvokeBody::Json(v) => v.clone(),
                InvokeBody::Raw(_) => Value::Null,
            };
            let m = handler_state.clone();
            invoke.resolver.respond_async(async move {
                let out = if cmd.starts_with("mobile_") {
                    local(app, m, &cmd, args).await
                } else {
                    forward(app, m, cmd, args).await
                };
                out.map_err(InvokeError::from)
            });
            true
        });
    #[cfg(mobile)]
    let b = b.on_window_event({
        let mobile = window_event_state;
        move |_window, event| match event {
            tauri::WindowEvent::Resumed => mobile.set_foreground(true),
            tauri::WindowEvent::Suspended => mobile.set_foreground(false),
            _ => {}
        }
    });
    #[cfg(mobile)]
    let b = b.plugin(tauri_plugin_barcode_scanner::init());
    b.run(tauri::generate_context!())
        .expect("error while running Portal Mobile");
}

#[cfg(test)]
// tungstenite's handshake callback fixes its own (large) `Err` type.
#[allow(clippy::result_large_err)]
mod tests {
    use super::*;

    #[test]
    fn pure_helpers() {
        assert_eq!(hostport("192.168.1.5", 18420), "192.168.1.5:18420");
        assert_eq!(hostport("fe80::1", 1), "[fe80::1]:1");
        assert_eq!(rpc_error(500, r#"{"error":"boom"}"#), r#"{"error":"boom"}"#);
        assert!(rpc_error(502, "").contains("502"));
        assert!(pair_error(403).contains("denied"));
        assert!(pair_error(408).contains("in time"));
        assert!(pair_error(429).contains("too many"));
        let ip = "10.0.0.2".parse().unwrap();
        let row = parse_reply(br#"{"port":18420,"name":"PC","version":"1","id":null}"#, ip).unwrap();
        assert_eq!(row["host"], "10.0.0.2");
        assert_eq!(row["port"], 18420);
        assert!(parse_reply(b"nope", ip).is_none());
    }

    #[test]
    fn alert_for_agent_events() {
        let permission = json!({
            "run_id": "r1", "seq": 1,
            "event": { "type": "permission_request", "request_id": "p1", "tool_name": "Bash", "tool_input": {}, "suggestions": [] }
        });
        let a = alert_for("agents://event", &permission).unwrap();
        assert_eq!(a.route, "/ai/coder/claude-code?session=r1");
        assert!(a.body.contains("Bash"));

        let finished = json!({
            "run_id": "r1", "seq": 2,
            "event": { "type": "turn_done", "final_text": "done", "is_error": false }
        });
        let a = alert_for("agents://event", &finished).unwrap();
        assert_eq!(a.body, "done");

        let finished_no_text = json!({
            "run_id": "r1", "seq": 2,
            "event": { "type": "turn_done", "is_error": false }
        });
        let a = alert_for("agents://event", &finished_no_text).unwrap();
        assert_eq!(a.body, "The run completed");

        let errored_turn = json!({
            "run_id": "r1", "seq": 3,
            "event": { "type": "turn_done", "is_error": true }
        });
        assert!(alert_for("agents://event", &errored_turn).is_none());

        let exited_bad = json!({ "run_id": "r1", "seq": 4, "event": { "type": "exited", "code": 1 } });
        let a = alert_for("agents://event", &exited_bad).unwrap();
        assert!(a.title.contains("failed"));

        let exited_ok = json!({ "run_id": "r1", "seq": 5, "event": { "type": "exited", "code": 0 } });
        assert!(alert_for("agents://event", &exited_ok).is_none());

        let exited_unknown = json!({ "run_id": "r1", "seq": 6, "event": { "type": "exited" } });
        assert!(alert_for("agents://event", &exited_unknown).is_none());
    }

    #[test]
    fn alert_for_thread_events() {
        let frame = |event: Value| json!({ "thread_id": "t1", "provider": "portal", "seq": 1, "event": event });
        let approval = frame(json!({ "type": "approval_requested",
            "approval": { "call_id": "c", "tool": "Bash", "summary": "rm -rf x" } }));
        let a = alert_for("coder://thread-event", &approval).unwrap();
        assert_eq!(a.route, "/ai/coder?id=t1");
        assert_eq!(a.body, "Wants to run Bash"); // never the command itself

        let exhausted = frame(json!({ "type": "turn_ended", "outcome": "exhausted" }));
        assert_eq!(alert_for("coder://thread-event", &exhausted).unwrap().title, "Agent paused");

        let done = frame(json!({ "type": "turn_ended", "outcome": "completed", "final_text": "ok" }));
        assert_eq!(alert_for("coder://thread-event", &done).unwrap().body, "ok");

        let err = frame(json!({ "type": "turn_ended", "outcome": "error", "error": "boom" }));
        assert_eq!(alert_for("coder://thread-event", &err).unwrap().body, "boom");

        let stopped = frame(json!({ "type": "turn_ended", "outcome": "stopped" }));
        assert!(alert_for("coder://thread-event", &stopped).is_none());

        // CLI threads alert from agents://event; a frame without a provider is skipped.
        let mut cli = approval.clone();
        cli["provider"] = json!("claude-code");
        assert!(alert_for("coder://thread-event", &cli).is_none());
        let mut bare = approval;
        bare.as_object_mut().unwrap().remove("provider");
        assert!(alert_for("coder://thread-event", &bare).is_none());
    }

    #[test]
    fn alert_for_pipeline_events() {
        let success = json!({ "id": "e1", "projectId": "2", "status": "success" });
        let a = alert_for("pipeline-execution-update", &success).unwrap();
        assert_eq!(a.route, "/pipeline-runs?run=e1");

        let failed = json!({ "id": "e1", "projectId": "2", "status": "failed" });
        assert!(alert_for("pipeline-execution-update", &failed).is_some());

        let running = json!({ "id": "e1", "projectId": "2", "status": "running" });
        assert!(alert_for("pipeline-execution-update", &running).is_none());
    }

    #[test]
    fn mark_paired_flags_the_matching_id_and_follows_it_to_a_new_address() {
        let paired = Conn { id: Some("abc".into()), ..conn() };

        // No match: nothing flagged, no address update.
        let mut rows = json!([{ "host": "10.0.0.9", "port": 1, "id": "other" }]);
        assert!(mark_paired(&mut rows, &paired).is_none());
        assert_eq!(rows[0]["paired"], Value::Null);

        // Same id, same address: flagged, nothing to follow.
        let mut rows = json!([{ "host": paired.host, "port": paired.port, "id": "abc" }]);
        assert!(mark_paired(&mut rows, &paired).is_none());
        assert_eq!(rows[0]["paired"], json!(true));

        // Same id, moved address: flagged, and the caller gets the new address to persist.
        let mut rows = json!([{ "host": "10.0.0.9", "port": 9999, "id": "abc" }]);
        let moved = mark_paired(&mut rows, &paired).unwrap();
        assert_eq!(rows[0]["paired"], json!(true));
        assert_eq!(moved.host, "10.0.0.9");
        assert_eq!(moved.port, 9999);
        assert_eq!(moved.token, paired.token); // everything else carries over unchanged

        // A connection with no id (manual/QR pairing) never matches.
        let mut rows = json!([{ "host": "10.0.0.9", "port": 1, "id": "abc" }]);
        assert!(mark_paired(&mut rows, &Conn { id: None, ..conn() }).is_none());
    }

    #[test]
    fn alert_for_default_none() {
        assert!(alert_for("mobile://connection", &json!({})).is_none());
    }

    fn conn() -> Conn {
        Conn {
            host: "127.0.0.1".into(),
            port: 18420,
            token: "s3cret".into(),
            name: "PC".into(),
            device: None,
            id: None,
        }
    }

    /// Bounded, so a regression hangs nothing: the sockets below are all loopback.
    fn block<F: std::future::Future>(f: F) -> F::Output {
        tauri::async_runtime::block_on(async {
            tokio::time::timeout(Duration::from_secs(10), f)
                .await
                .expect("timed out")
        })
    }

    #[test]
    fn a_connection_file_from_before_device_existed_still_loads() {
        let old = r#"{"host":"h","port":1,"token":"t","name":"n"}"#;
        assert_eq!(serde_json::from_str::<Conn>(old).unwrap().device, None);
    }

    #[test]
    fn control_error_passes_refusals_and_drops_everything_else() {
        let refusal = r#"{"t":"error","message":"this device is not permitted to send input"}"#;
        assert_eq!(
            control_error(refusal).unwrap(),
            json!({ "t": "error", "message": "this device is not permitted to send input" })
        );
        // The daemon chooses the frame: only `t` and `message` are handed on, and the
        // message is capped by characters, not bytes.
        let long = json!({
            "t": "error",
            "message": "é".repeat(CONTROL_ERROR_MAX + 50),
            "extra": { "x": 1 },
        });
        assert_eq!(
            control_error(&long.to_string()).unwrap(),
            json!({ "t": "error", "message": "é".repeat(CONTROL_ERROR_MAX) })
        );
        // No message, or one that is not a string, is an empty one.
        for bare in [r#"{"t":"error"}"#, r#"{"t":"error","message":{"a":1}}"#] {
            assert_eq!(
                control_error(bare).unwrap(),
                json!({ "t": "error", "message": "" })
            );
        }
        // `hello`, a pong, the fanout `/control` forwards (already on `/events`), its
        // lagged notice, and noise: none of it is the page's business.
        for frame in [
            r#"{"t":"hello","name":"PC"}"#,
            r#"{"t":"pong","seq":1}"#,
            r#"{"event":"coder://change","payload":{}}"#,
            r#"{"event":"events://lagged","payload":{"skipped":3}}"#,
            r#"{"t":42}"#,
            "{}",
            "not json",
        ] {
            assert!(control_error(frame).is_none(), "{frame}");
        }
    }

    #[test]
    fn ws_requests_carry_the_token_in_a_header_never_the_url() {
        let header = |req: &Request, k: &str| {
            req.headers()
                .get(k)
                .map(|v| v.to_str().unwrap().to_string())
        };
        let mut c = conn();
        let req = control_request(&c).unwrap();
        assert_eq!(req.uri().to_string(), "ws://127.0.0.1:18420/control");
        assert_eq!(
            header(&req, "authorization").as_deref(),
            Some("Bearer s3cret")
        );
        assert_eq!(header(&req, "x-portal-device"), None);

        c.device = Some("Pixel 9a".into());
        let req = control_request(&c).unwrap();
        assert_eq!(header(&req, "x-portal-device").as_deref(), Some("Pixel 9a"));

        // A name the daemon cannot read as a header is left off; the socket still opens.
        for name in ["Zoë's phone", "two\nlines"] {
            c.device = Some(name.into());
            let req = control_request(&c).unwrap();
            assert!(req.headers().get("x-portal-device").is_none(), "{name:?}");
            assert!(header(&req, "authorization").is_some());
        }

        // `/events` is the same handshake, minus the device name.
        let req = ws_request(&c, "/events").unwrap();
        assert_eq!(req.uri().path(), "/events");
        assert_eq!(req.uri().query(), None);
        assert_eq!(header(&req, "x-portal-device"), None);
    }

    #[test]
    fn control_queue_opens_once_reuses_the_socket_and_reopens_after_it_dies() {
        let m = Mobile::default();
        assert_eq!(
            m.queue_control("a".into()).err().as_deref(),
            Some(NOT_CONNECTED)
        );

        m.0.lock().unwrap().conn = Some(conn());
        // No socket yet: the first frame hands back a queue to open one on.
        let new = m
            .queue_control("a".into())
            .unwrap()
            .expect("opens a socket");
        let mut rx = new.rx;
        assert_eq!(new.req.uri().path(), "/control");
        assert_eq!(new.token, "s3cret");
        assert_eq!(rx.try_recv().unwrap(), "a");
        // Later frames ride the same socket.
        assert!(m.queue_control("b".into()).unwrap().is_none());
        assert_eq!(rx.try_recv().unwrap(), "b");
        // A stalled socket costs frames, not memory.
        for _ in 0..CONTROL_QUEUE {
            assert!(m.queue_control("x".into()).unwrap().is_none());
        }
        assert!(m.queue_control("x".into()).is_err());
        // Once the socket has ended, the next frame opens a fresh one and is not lost.
        drop(rx);
        let new = m.queue_control("c".into()).unwrap().expect("replaces it");
        let mut rx = new.rx;
        assert_eq!(rx.try_recv().unwrap(), "c");
        // Disconnect resets `Inner`, dropping the sender: the session sees its queue close.
        *m.0.lock().unwrap() = Inner::default();
        assert!(matches!(
            rx.try_recv(),
            Err(mpsc::error::TryRecvError::Disconnected)
        ));
    }

    /// A loopback stand-in for the daemon's `/control`, for one socket: reports the
    /// handshake it got, greets with a `hello` and a fanout frame like the real one,
    /// answers each text frame with an `{t:"error"}` carrying it back, and reports
    /// `closed` when the client goes away.
    async fn fake_control() -> (u16, mpsc::UnboundedReceiver<String>) {
        use tokio_tungstenite::tungstenite::handshake::server::{
            ErrorResponse, Request as In, Response as Out,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (seen, seen_rx) = mpsc::unbounded_channel();
        tauri::async_runtime::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let handshake = seen.clone();
            let record = move |req: &In, resp: Out| -> Result<Out, ErrorResponse> {
                let h = |k: &str| {
                    req.headers()
                        .get(k)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-")
                        .to_string()
                };
                let line = format!(
                    "{} auth={} device={}",
                    req.uri(),
                    h("authorization"),
                    h("x-portal-device")
                );
                let _ = handshake.send(line);
                Ok(resp)
            };
            let mut ws = tokio_tungstenite::accept_hdr_async(tcp, record)
                .await
                .unwrap();
            for unprompted in [
                r#"{"t":"hello"}"#,
                r#"{"event":"coder://change","payload":{}}"#,
            ] {
                ws.send(Message::Text(unprompted.into())).await.unwrap();
            }
            while let Some(Ok(Message::Text(text))) = ws.next().await {
                let _ = seen.send(text.clone());
                let refusal = json!({ "t": "error", "message": text }).to_string();
                ws.send(Message::Text(refusal)).await.unwrap();
            }
            let _ = seen.send("closed".into());
        });
        (port, seen_rx)
    }

    #[test]
    fn control_session_delivers_queued_frames_and_reemits_only_errors() {
        block(async {
            let (port, mut seen) = fake_control().await;
            let c = Conn {
                port,
                device: Some("Pixel".into()),
                ..conn()
            };
            let frame = r#"{"t":"mouse_move","dx":50,"dy":0}"#;
            let (tx, rx) = mpsc::channel(CONTROL_QUEUE);
            tx.try_send(frame.into()).unwrap(); // queued before the socket is even open
            let (errs, mut errs_rx) = mpsc::unbounded_channel();
            let session = tauri::async_runtime::spawn(control_session(
                control_request(&c).unwrap(),
                rx,
                move |e| {
                    let _ = errs.send(e);
                },
            ));

            // Bearer and device name in the handshake, no `?token=` in the URL.
            assert_eq!(
                seen.recv().await.unwrap(),
                "/control auth=Bearer s3cret device=Pixel"
            );
            assert_eq!(seen.recv().await.unwrap(), frame);
            // The daemon's `hello` and fanout frame came first and never reached the page:
            // the first thing re-emitted is the refusal of the frame we sent.
            let err = errs_rx.recv().await.unwrap();
            assert_eq!(err, json!({ "t": "error", "message": frame }));
            // Dropping the queue (disconnect) ends the session and closes the socket.
            drop(tx);
            assert!(matches!(session.await.unwrap(), Ok(())));
            assert_eq!(seen.recv().await.unwrap(), "closed");
        });
    }

    /// A loopback stand-in for a daemon that has revoked the token: every handshake is a 401.
    async fn fake_401() -> u16 {
        use tokio_tungstenite::tungstenite::handshake::server::{
            ErrorResponse, Request as In, Response as Out,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tauri::async_runtime::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                let refuse = |_: &In, _: Out| -> Result<Out, ErrorResponse> {
                    let unauthorized = tokio_tungstenite::tungstenite::http::Response::builder();
                    Err(unauthorized.status(401).body(None).unwrap())
                };
                let _ = tokio_tungstenite::accept_hdr_async(tcp, refuse).await;
            }
        });
        port
    }

    /// A `/control` dial still in flight when the device re-pairs gets the OLD token's 401
    /// after the new connection is stored: that stale news must not wipe the new one. A
    /// 401 for the token the connection still holds does disconnect.
    #[test]
    fn a_stale_401_leaves_a_newer_connection_alone() {
        block(async {
            let port = fake_401().await;
            let m = Mobile::default();
            m.0.lock().unwrap().conn = Some(Conn { port, ..conn() });
            let dial = m
                .queue_control("a".into())
                .unwrap()
                .expect("opens a socket");

            // The device re-pairs while that dial is in flight: `set()` stores the new token.
            let fresh = Conn {
                port,
                token: "fresh".into(),
                ..conn()
            };
            m.0.lock().unwrap().conn = Some(fresh);
            let forgotten = std::cell::Cell::new(false);
            m.control_task(dial, |_| {}, || forgotten.set(true)).await;
            assert!(!forgotten.get());
            assert_eq!(m.conn().unwrap().token, "fresh");

            // Dialled with the token the connection holds now: revoked, so disconnect.
            let dial = m
                .queue_control("b".into())
                .unwrap()
                .expect("opens a socket");
            m.control_task(dial, |_| {}, || forgotten.set(true)).await;
            assert!(forgotten.get());
            assert!(m.conn().is_none());
        });
    }

    #[test]
    fn control_session_tells_a_revoked_token_from_an_unreachable_desktop() {
        block(async {
            let revoked = Conn {
                port: fake_401().await,
                ..conn()
            };
            let (_tx, rx) = mpsc::channel(1);
            let res = control_session(control_request(&revoked).unwrap(), rx, |_| {}).await;
            assert!(matches!(res, Err(Stop::Revoked)));

            // Nothing listening: not a revocation, so the caller must not disconnect.
            let gone = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let unreachable = Conn {
                port: gone.local_addr().unwrap().port(),
                ..conn()
            };
            drop(gone);
            let (_tx, rx) = mpsc::channel(1);
            let res = control_session(control_request(&unreachable).unwrap(), rx, |_| {}).await;
            assert!(matches!(res, Err(Stop::Other)));
        });
    }
}
