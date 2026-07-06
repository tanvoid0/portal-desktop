//! Tauri invoke handler registry.
//!
//! Commands are grouped by domain in [`crate::lib`] setup. Future refactors should
//! move each domain's command list into `domains/{name}/commands.rs` and aggregate
//! here via a `collect_commands!` macro to keep `lib.rs` setup focused on initialization.

pub const COMMAND_COUNT_ESTIMATE: usize = 200;
