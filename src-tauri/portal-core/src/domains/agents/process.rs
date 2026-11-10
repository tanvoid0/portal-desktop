//! Process lifecycle helpers shared by backend adapters.

use tokio::process::Child;

/// Kill `child` and wait for it to actually exit.
///
/// Coder's `claude-cli` path spawns through `cmd /C claude` on Windows (the
/// npm shim is a `.cmd`), so killing it can orphan the real process and needs
/// a tree-kill. The agents domain spawns the real `claude.exe` directly (see
/// `claude_code.rs`), so a plain `start_kill` already reaches the right
/// process — no tree-kill needed here.
pub async fn kill_and_wait(child: &mut Child) {
    let _ = child.start_kill();
    let _ = child.wait().await;
}
