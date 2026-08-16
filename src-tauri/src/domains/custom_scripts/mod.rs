//! Tauri adapter for the `custom_scripts` domain.
//!
//! The domain itself lives in `portal_core::domains::custom_scripts`; this module glob
//! re-exports it so every `crate::domains::custom_scripts::…` path in the tree still
//! resolves, and adds the `commands` module, which cannot move because `#[rpc]`
//! expands to `#[tauri::command]`. Phase 1 of
//! `docs/development/REMOTE_ARCHITECTURE.md`.

pub use portal_core::domains::custom_scripts::*;

pub mod commands;
