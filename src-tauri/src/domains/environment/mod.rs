//! Tauri adapter for the `environment` domain.
//!
//! The domain itself lives in `portal_core::domains::environment`; this module glob
//! re-exports it so every `crate::domains::environment::…` path in the tree still
//! resolves, and adds the `commands` module, which cannot move because `#[rpc]`
//! expands to `#[tauri::command]`. Phase 1 of
//! `docs/development/REMOTE_ARCHITECTURE.md`.

pub use portal_core::domains::environment::*;

pub mod commands;
