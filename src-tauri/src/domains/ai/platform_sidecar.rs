//! Host side of owning `agent-platformd`: spawn the child with portal-core's
//! environment, drain its output into portal's log, and kill it on exit. The
//! decision of *whether* to spawn is `platform_admin::start_platform`.

use crate::{log_info, log_warn};
use portal_core::domains::ai::platform_daemon as daemon;
use portal_core::process_ext::NoWindowExt;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

/// One daemon per portal process; the single-instance plugin guards the rest.
static CHILD: Mutex<Option<Child>> = Mutex::new(None);

/// Tell portal-core where the bundled sidecar sits. Tauri's `externalBin`
/// installs it beside the main executable, minus the target triple.
pub fn init() {
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
    {
        daemon::set_sidecar_dir(dir);
    }
}

/// Not draining a piped child blocks it once the OS buffer fills.
fn drain<R: Read + Send + 'static>(stream: R) {
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            log_info!("agent-platformd", "{}", line);
        }
    });
}

pub fn spawn() -> Result<u32, String> {
    let exe = daemon::daemon_exe().ok_or_else(|| {
        "agent-platformd was not found (bundled sidecar, AGENT_PLATFORM_DAEMON_EXE, or a local agent-platform build)"
            .to_string()
    })?;
    let dir = daemon::data_dir();
    // Key errors never carry the key; only the io kind is surfaced.
    let key = daemon::load_or_create_key(&dir)
        .map_err(|e| format!("Could not read or create the master key: {}", e.kind()))?;
    // ponytail: a Postgres DSN is not configurable from portal yet, so SQLite.
    let env = daemon::daemon_env(&dir, &key, daemon::PORT, "");

    let mut child = Command::new(&exe)
        .envs(env)
        .no_window()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not start {}: {}", exe.display(), e.kind()))?;
    let pid = child.id();
    if let Some(out) = child.stdout.take() {
        drain(out);
    }
    if let Some(err) = child.stderr.take() {
        drain(err);
    }
    // ponytail: no Windows job object, so a hard crash of portal orphans the
    // daemon (the next launch attaches to it). Add one if that bites.
    if let Some(mut old) = CHILD.lock().unwrap().replace(child) {
        let _ = old.kill();
        let _ = old.wait();
    }
    log_info!(
        "agent-platformd",
        "started pid {} from {}",
        pid,
        exe.display()
    );
    Ok(pid)
}

/// Kill the child we spawned. An attached (not spawned) daemon is left alone.
pub fn stop() {
    if let Some(mut child) = CHILD.lock().unwrap().take() {
        if child.kill().is_err() {
            log_warn!("agent-platformd", "could not kill pid {}", child.id());
        }
        let _ = child.wait();
    }
}
