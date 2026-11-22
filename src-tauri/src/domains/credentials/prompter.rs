//! Desktop side of the vault gate: `VaultSession::require_key` calls
//! `prompt()` when a command needs the key and the vault is locked. We emit
//! `vault://unlock-requested`; the frontend shows the (cancellable) unlock
//! dialog and answers through `vault_unlock_settled`. Concurrent callers share
//! one prompt.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::time::Duration;

use portal_core::domains::credentials::services::VaultPrompter;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

/// Nobody answered (window closed, frontend hung, headless daemon): fail as locked.
const PROMPT_TIMEOUT: Duration = Duration::from_secs(120);

pub struct TauriVaultPrompter {
    app: AppHandle,
    waiters: Mutex<Vec<oneshot::Sender<bool>>>,
}

impl TauriVaultPrompter {
    pub fn new(app: AppHandle) -> Self {
        Self { app, waiters: Mutex::new(Vec::new()) }
    }

    /// Frontend's answer: `true` = unlocked, `false` = dismissed.
    pub fn settle(&self, ok: bool) {
        let waiters = std::mem::take(&mut *self.waiters.lock().unwrap());
        for w in waiters {
            let _ = w.send(ok);
        }
    }
}

impl VaultPrompter for TauriVaultPrompter {
    fn prompt(&self) -> Pin<Box<dyn Future<Output = bool> + Send>> {
        let (tx, rx) = oneshot::channel();
        let first = {
            let mut waiters = self.waiters.lock().unwrap();
            waiters.retain(|w| !w.is_closed()); // drop callers that timed out
            waiters.push(tx);
            waiters.len() == 1
        };
        if first {
            let _ = self.app.emit("vault://unlock-requested", ());
        }
        Box::pin(async move {
            matches!(tokio::time::timeout(PROMPT_TIMEOUT, rx).await, Ok(Ok(true)))
        })
    }
}
