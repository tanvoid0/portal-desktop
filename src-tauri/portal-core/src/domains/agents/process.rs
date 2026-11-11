//! Process lifecycle helpers shared by backend adapters.

use tokio::process::Child;

/// Signal `child` to die. Does not wait for exit — a caller that already
/// knows the run is ending (e.g. a user-initiated cancel) should emit its own
/// terminal event right away rather than block on the OS actually tearing the
/// process down. The stdout-pump task still reaps it (its own `wait()`, once
/// it sees EOF) to settle the real exit code; `Child::wait` is safe to call
/// again there even if nothing here has resolved yet.
///
/// On Windows both this domain and coder's `claude-cli` path spawn through
/// `cmd /C claude` (the npm shim is a `.cmd`, which `CreateProcess` can't run
/// directly) — same pre-existing `start_kill`-only behavior as coder here,
/// not a tree-kill. `kill_on_drop(true)` on the spawned `Command` (see
/// `claude_code.rs`) is the backstop if this is never called at all.
pub fn start_kill(child: &mut Child) {
    let _ = child.start_kill();
}
