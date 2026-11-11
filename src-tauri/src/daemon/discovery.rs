//! UDP discovery responder — phase 1's answer to "is a Portal Desktop on this
//! LAN": a probe in, a JSON reply out. No auth, nothing sensitive in the
//! reply — it only announces that something is here.

use std::net::SocketAddr;

use socket2::{Domain, Socket, Type};
use tokio::net::UdpSocket;

/// The literal probe Android broadcasts. Not JSON — just these bytes.
const PROBE: &[u8] = b"PORTALREMOTE?";

/// The well-known discovery port every Android client broadcasts `PROBE` to —
/// fixed on their side, so unlike the HTTP port this can't be offset per dev
/// instance without going deaf to real probes.
const DISCOVERY_PORT: u16 = 8765;

/// Bind `0.0.0.0:8765` with `SO_REUSEADDR` and answer every `PROBE` with
/// `{port, name, version, id: null}`, sent back to the sender's own address.
/// `SO_REUSEADDR` (rather than a plain `UdpSocket::bind`) matters once more
/// than one Portal Desktop process is running on this machine — each dev
/// instance binds the same port and a broadcast probe is delivered to all of
/// them, so every running instance answers with its own HTTP port instead of
/// only whichever instance happened to start first (see the daemon HTTP
/// port's own `PORTAL_INSTANCE` offset in `daemon::mod::configured_port` for
/// the same race, fixed differently because that port isn't fixed on the
/// client's side). Spawned alongside the LAN HTTP listener, under the same
/// enable condition — see `spawn` in `daemon::mod`.
pub fn spawn_responder(port: u16) {
    let host_name = super::host_name();
    tauri::async_runtime::spawn(async move {
        let addr: SocketAddr = ([0, 0, 0, 0], DISCOVERY_PORT).into();
        let socket = match bind_reusable(addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[daemon] could not bind UDP discovery on :{DISCOVERY_PORT}: {e}");
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

/// A non-blocking UDP socket bound to `addr` with `SO_REUSEADDR` set first,
/// handed to tokio. `std::net`/`tokio::net::UdpSocket::bind` offer no way to
/// set socket options before binding, hence going through `socket2`.
fn bind_reusable(addr: SocketAddr) -> std::io::Result<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.set_nonblocking(true)?;
    UdpSocket::from_std(socket.into())
}
