//! Tauri adapter for the `autonomy` domain.
//!
//! The domain itself lives in `portal_core::domains::autonomy`; this module glob
//! re-exports it so every `crate::domains::autonomy::…` path in the tree still
//! resolves, and adds the `commands` module, which cannot move because `#[rpc]`
//! expands to `#[tauri::command]`. Phase 1 of
//! `docs/development/REMOTE_ARCHITECTURE.md`.

pub use portal_core::domains::autonomy::*;

pub mod commands;
