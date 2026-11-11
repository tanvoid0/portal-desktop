//! Process lifecycle helpers shared by backend adapters.

use tokio::process::Child;

#[cfg(windows)]
use crate::process_ext::NoWindowExt;

/// Signal `child` to die. Does not wait for the OS to actually finish tearing
/// the process (and, on Windows, its tree — see below) down — a caller that
/// already knows the run is ending (e.g. a user-initiated cancel) should emit
/// its own terminal event right away rather than block on that. The
/// stdout-pump task still reaps it (its own `wait()`, once it sees EOF) to
/// settle the real exit code; `Child::wait` is safe to call again there even
/// if nothing here has resolved yet.
///
/// On Windows both this domain and coder's `claude-cli` path spawn through
/// `cmd /C claude` (the npm shim is a `.cmd`, which `CreateProcess` can't run
/// directly), so `child` here is the `cmd.exe` wrapper, not the real
/// `claude.exe` — `Child::start_kill` alone only terminates that wrapper,
/// and Windows does not tear down a process's descendants when the process
/// itself dies, so `claude.exe` is orphaned and keeps running. `taskkill /T`
/// kills the whole tree rooted at the wrapper's pid instead, which needs
/// `.output()` (blocked on here — just taskkill locating and signalling the
/// tree, not the tree actually exiting) rather than a fire-and-forget spawn:
/// taskkill walks the live process list for descendants of this pid, so it
/// has to run — and need not finish, only locate them — before
/// `start_kill()` below removes the wrapper from that list and leaves
/// nothing for it to walk from. `kill_on_drop(true)` on the spawned
/// `Command` (see `claude_code.rs`) only ever reaches the wrapper too, so
/// it's the same backstop, not a fix for this.
pub fn start_kill(child: &mut Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        let _ = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .no_window()
            .output();
    }
    let _ = child.start_kill();
}
