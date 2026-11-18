//! No `cap`: these are loopback/IPC-only — never served to a paired phone.
use crate::domains::uninstaller::{self as un, InstalledProgram, Leftover, UninstallOutcome};

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[portal_rpc_macro::rpc]
pub async fn uninstaller_list_programs() -> Result<Vec<InstalledProgram>, String> {
    blocking(un::list_programs).await
}

#[portal_rpc_macro::rpc]
pub async fn uninstaller_scan_leftovers(id: String) -> Result<Vec<Leftover>, String> {
    blocking(move || un::scan_leftovers(&id)).await
}

/// `cleanup_paths`: leftovers (from `uninstaller_scan_leftovers`) to remove once
/// the uninstall succeeds. Anything not in a pre-uninstall scan is refused.
#[portal_rpc_macro::rpc]
pub async fn uninstaller_run(
    id: String,
    silent: bool,
    cleanup_paths: Option<Vec<String>>,
) -> Result<UninstallOutcome, String> {
    blocking(move || un::uninstall(&id, silent, &cleanup_paths.unwrap_or_default())).await
}
