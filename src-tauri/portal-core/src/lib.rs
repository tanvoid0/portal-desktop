//! Portal Desktop's core, with no idea that Tauri exists.
//!
//! Phase 1 of `docs/development/REMOTE_ARCHITECTURE.md`. Everything the app can
//! *do* belongs here; `src-tauri` keeps only the webview shell, the window-bound
//! plugins and the command adapters. The absence of a `tauri` line in this
//! crate's `Cargo.toml` is what enforces that — see the note there.
//!
//! Populated one slice at a time so the tree keeps building. `events` came first
//! because every other slice emits through it; then the schema (`entities`,
//! `migrations`); then these leaves, which the domains sit on top of. `domains`
//! itself is last, because its `commands.rs` files are the Tauri adapter and
//! have to be lifted out as they go.

pub mod app_paths;
pub mod command_executor;
pub mod database;
pub mod domains;
pub mod entities;
pub mod error;
pub mod events;
pub mod migrations;
pub mod process_ext;
pub mod utils;

pub use error::{AppError, AppResult};
