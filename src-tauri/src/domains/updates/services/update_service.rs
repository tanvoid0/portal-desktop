/// Update service - provides version information
/// Note: Actual update checking and installation is handled via the frontend
/// using Tauri's JavaScript updater API (@tauri-apps/api/updater)
pub struct UpdateService;

impl UpdateService {
    /// Get the current application version
    pub fn get_current_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

