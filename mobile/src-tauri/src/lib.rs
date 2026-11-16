//! Portal Mobile: a thin client of the Portal Desktop daemon. Every `invoke`
//! the (unchanged) frontend makes is forwarded to `POST /rpc/{command}`; the
//! daemon's `/events` frames are re-emitted as Tauri events. Only the
//! `mobile_*` commands run locally.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::ipc::{InvokeBody, InvokeError};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::net::UdpSocket;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Error as WsError;

const NOT_CONNECTED: &str = "not connected to a Portal Desktop";

#[derive(Clone, Serialize, Deserialize)]
struct Conn {
    host: String,
    port: u16,
    token: String,
    name: String,
}

struct Inner {
    conn: Option<Conn>,
    caps: Vec<String>,
    /// The events socket is currently up.
    live: bool,
    bridge: Option<tauri::async_runtime::JoinHandle<()>>,
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
        {
            let mut s = self.0.lock().unwrap();
            if let Some(b) = s.bridge.take() {
                b.abort();
            }
            *s = Inner::default();
        }
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

        let mut req = format!("ws://{}/events", hostport(&conn.host, conn.port))
            .into_client_request()
            .map_err(|_| Stop::Other)?;
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", conn.token)
                .parse()
                .map_err(|_| Stop::Other)?,
        );
        let (mut ws, _) = match tokio_tungstenite::connect_async(req).await {
            Ok(ok) => ok,
            Err(WsError::Http(r)) if r.status().as_u16() == 401 => return Err(Stop::Revoked),
            Err(_) => return Err(Stop::Other),
        };
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
    }))
}

#[derive(Deserialize)]
struct Args {
    host: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    device: Option<String>,
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
        "mobile_discover" => Ok(discover().await),
        "mobile_connect" => {
            let host = need(a.host, "host")?;
            let port = a.port.ok_or("missing `port`")?;
            let token = need(a.token, "token")?;
            let who = whoami(&host, port, &token).await.map_err(|e| e.msg)?;
            let name = who["name"].as_str().unwrap_or("Portal Desktop").to_string();
            m.set(&app, Conn { host, port, token, name });
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
            m.set(&app, Conn { host, port, token: token.to_string(), name });
            Ok(m.status())
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
        m.clear(&app); // token revoked on the desktop
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
    fn alert_for_default_none() {
        assert!(alert_for("mobile://connection", &json!({})).is_none());
    }
}
