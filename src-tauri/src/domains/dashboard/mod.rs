//! Tauri adapter for the `dashboard` domain.
//!
//! Unlike the other domains this one has no core half — it was only ever a
//! command file, so there is nothing in `portal_core::domains` to re-export.
//! Phase 1 of `docs/development/REMOTE_ARCHITECTURE.md`.

pub mod commands;
