//! Tauri adapter for the `elevation` domain. See `domains::environment` for why
//! this glob re-export + separate `commands` module exists.

pub use portal_core::domains::elevation::*;

pub mod commands;
