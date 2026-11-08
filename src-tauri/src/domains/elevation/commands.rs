use portal_core::domains::elevation::{self, ElevatedRunResult, ElevationStatus};

#[portal_rpc_macro::rpc]
pub async fn elevation_status() -> Result<ElevationStatus, String> {
    Ok(elevation::elevation_status())
}

/// Run one external program elevated while Portal itself stays unprivileged.
/// For a single step that needs admin rights (installers, service control, …).
#[portal_rpc_macro::rpc]
pub async fn run_elevated(exe: String, args: Vec<String>) -> Result<ElevatedRunResult, String> {
    elevation::run_elevated(&exe, &args)
}

/// Relaunch Portal itself elevated and close this (unprivileged) instance.
/// For when the rest of the session needs admin rights, not just one step.
#[portal_rpc_macro::rpc]
pub async fn restart_elevated(app_handle: tauri::AppHandle) -> Result<(), String> {
    elevation::restart_elevated()?;
    app_handle.exit(0);
    Ok(())
}
