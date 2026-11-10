//! Tauri adapter for the `agents` domain.
//!
//! The domain itself lives in `portal_core::domains::agents`; this module
//! glob re-exports it so every `crate::domains::agents::…` path resolves, and
//! adds the `commands` module, which cannot move because `#[rpc]` expands to
//! `#[tauri::command]`.

pub use portal_core::domains::agents::*;

pub mod commands;
pub use commands::*;
