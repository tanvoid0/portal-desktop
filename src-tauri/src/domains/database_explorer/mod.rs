//! Tauri adapter for the `database_explorer` domain.
//!
//! The domain lives in `portal_core::domains::database_explorer`; this glob
//! re-exports it and adds `commands`, which must live here because `#[rpc]`
//! expands to `#[tauri::command]`. Mirrors `domains/sdk/mod.rs`.

pub use portal_core::domains::database_explorer::*;

pub mod commands;
