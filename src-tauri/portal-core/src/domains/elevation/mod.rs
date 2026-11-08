//! Generic process elevation: check status, run one step elevated, or
//! relaunch the whole app elevated (UAC on Windows, pkexec/osascript on
//! Linux/macOS).
//!
//! The `environment` domain predates this and keeps its own bespoke
//! elevation code for system-scoped env var writes; any *new* elevation
//! need should go through here instead of growing another one-off.

pub mod types;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(target_os = "windows")]
use windows as platform;
#[cfg(not(target_os = "windows"))]
use unix as platform;

pub use types::{ElevatedRunResult, ElevationStatus};

pub fn elevation_status() -> ElevationStatus {
    ElevationStatus {
        is_elevated: platform::is_elevated(),
        platform: std::env::consts::OS.to_string(),
    }
}

/// Run `exe args` elevated and wait for it to exit. Portal itself stays
/// unprivileged — use this for one specific step that needs admin rights.
pub fn run_elevated(exe: &str, args: &[String]) -> Result<ElevatedRunResult, String> {
    platform::run_elevated(exe, args)
}

/// Relaunch the current executable elevated. On success the caller should
/// close this (unprivileged) instance — the elevated copy takes over.
pub fn restart_elevated() -> Result<(), String> {
    platform::restart_elevated()
}
