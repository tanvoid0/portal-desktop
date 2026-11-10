//! UDP discovery responder — phase 1's answer to "is a Portal Desktop on this
//! LAN": a probe in, a JSON reply out. No auth, nothing sensitive in the
//! reply — it only announces that something is here.

use tokio::net::UdpSocket;

/// The literal probe Android broadcasts. Not JSON — just these bytes.
const PROBE: &[u8] = b"PORTALREMOTE?";

/// Bind `0.0.0.0:8765` and answer every `PROBE` with `{port, name, version,
/// id: null}`, sent back to the sender's own address. Spawned alongside the
/// LAN HTTP listener, under the same enable condition — see `spawn` in
/// `daemon::mod`.
pub fn spawn_responder(port: u16) {
    let host_name = super::host_name();
    tauri::async_runtime::spawn(async move {
        let socket = match UdpSocket::bind(("0.0.0.0", 8765)).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[daemon] could not bind UDP discovery on :8765: {e}");
                return;
            }
        };
        let mut buf = [0u8; 64];
        loop {
            let Ok((len, from)) = socket.recv_from(&mut buf).await else {
                continue;
            };
            if &buf[..len] != PROBE {
                continue;
            }
            let reply = serde_json::json!({
                "port": port,
                "name": host_name,
                "version": env!("CARGO_PKG_VERSION"),
                "id": null,
            })
            .to_string();
            let _ = socket.send_to(reply.as_bytes(), from).await;
        }
    });
}
