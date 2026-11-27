//! Tauri adapter for the `threads` domain: the domain lives in
//! `portal_core::domains::threads`; this adds the `commands` module (`#[rpc]`
//! expands to `#[tauri::command]`, so it cannot move into the core crate).

pub use portal_core::domains::threads::*;

pub mod commands;
pub use commands::*;
