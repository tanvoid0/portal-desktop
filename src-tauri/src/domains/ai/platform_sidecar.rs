//! Host side of owning `agent-platformd`: spawn the child with portal-core's
//! environment, send its output to a log file, and kill it on exit. The
//! decision of *whether* to spawn is `platform_admin::start_platform`.

use crate::{log_info, log_warn};
use portal_core::domains::ai::platform_daemon as daemon;
use portal_core::process_ext::NoWindowExt;
use std::fs::File;
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

/// A file, never a pipe: a daemon orphaned by a hard crash of portal would
/// panic on every log line once the pipe's reader is gone, and then answers
/// nothing.
// ponytail: truncated at each spawn, no rotation within one run.
fn output(path: &std::path::Path) -> (Stdio, Stdio) {
    // Emptied, then appended to: an orphan on an old port may still hold the file.
    let both = File::create(path)
        .and_then(|_| File::options().append(true).open(path))
        .and_then(|f| Ok((f.try_clone()?, f)));
    match both {
        Ok((out, err)) => (out.into(), err.into()),
        Err(e) => {
            log_warn!("agent-platformd", "no log file: {}", e.kind());
            (Stdio::null(), Stdio::null())
        }
    }
}

pub fn spawn(port: u16) -> Result<u32, String> {
    let exe = daemon::daemon_exe().ok_or_else(|| {
        "agent-platformd was not found (bundled sidecar, AGENT_PLATFORM_DAEMON_EXE, or a local agent-platform build)"
            .to_string()
    })?;
    let dir = daemon::data_dir();
    // Key errors never carry the key; only the io kind is surfaced.
    let key = daemon::load_or_create_key(&dir)
        .map_err(|e| format!("Could not read or create the master key: {}", e.kind()))?;
    // ponytail: a Postgres DSN is not configurable from portal yet, so SQLite.
    let env = daemon::daemon_env(&dir, &key, port, "");
    let log = dir.join("agent-platformd.log");
    let (out, err) = output(&log);

    let child = Command::new(&exe)
        .envs(env)
        .no_window()
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .spawn()
        .map_err(|e| format!("Could not start {}: {}", exe.display(), e.kind()))?;
    let pid = child.id();
    // ponytail: no Windows job object, so a hard crash of portal orphans the
    // daemon (the next launch attaches to it). Add one if that bites.
    if let Some(mut old) = CHILD.lock().unwrap().replace(child) {
        let _ = old.kill();
        let _ = old.wait();
    }
    log_info!(
        "agent-platformd",
        "started pid {} from {} on port {}, output in {}",
        pid,
        exe.display(),
        port,
        log.display()
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
