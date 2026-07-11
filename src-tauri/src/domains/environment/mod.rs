//! System environment variable management with elevation support.
//!
//! Reads and writes persistent user/system environment variables without
//! requiring an app restart. System-scoped writes trigger OS elevation
//! (UAC on Windows, pkexec/sudo on Linux, osascript on macOS).

pub mod commands;
mod platform;
mod types;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
mod unix;
