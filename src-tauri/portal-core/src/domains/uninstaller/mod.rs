//! Software uninstaller: list installed programs, run their own uninstaller,
//! then (optionally) find and remove what it left behind.
//!
//! Windows only — it reads the `Uninstall` registry keys. Everything the
//! caller hands back (an id, a leftover path) is re-derived from the registry
//! and a fresh scan here, never trusted: the daemon exposes this to MCP, so a
//! client must not be able to name an arbitrary command or path.

use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(not(target_os = "windows"))]
pub use unsupported::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct InstalledProgram {
    /// Full registry key path; the handle for every other call.
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_location: String,
    pub install_date: String,
    pub size_kb: u64,
    pub uninstall_string: String,
    pub quiet_uninstall_string: String,
    /// MSI package (always has a silent mode).
    pub is_msi: bool,
    /// Flagged `SystemComponent` — hidden from Add/Remove Programs.
    pub system_component: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Leftover {
    /// `folder` or `registry`.
    pub kind: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallOutcome {
    pub exit_code: Option<i32>,
    /// The program's registry key is gone afterwards.
    pub removed: bool,
    pub silent: bool,
    pub message: String,
    /// Present when leftovers were requested and the uninstall succeeded.
    pub cleanup: Option<CleanupResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    /// Where registry keys were exported before deletion (empty if none).
    pub backup_dir: String,
}

/// Pull the `{GUID}` out of an `msiexec ... /X{GUID}` / `/I{GUID}` string.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn msi_guid(cmd: &str) -> Option<String> {
    let start = cmd.find('{')?;
    let end = start + cmd[start..].find('}')?;
    let g = &cmd[start..=end];
    (g.len() == 38).then(|| g.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_extraction() {
        assert_eq!(
            msi_guid("MsiExec.exe /I{0A1B2C3D-0000-1111-2222-333344445555}").as_deref(),
            Some("{0A1B2C3D-0000-1111-2222-333344445555}")
        );
        assert_eq!(msi_guid("C:\\x\\unins000.exe /SILENT"), None);
    }
}
