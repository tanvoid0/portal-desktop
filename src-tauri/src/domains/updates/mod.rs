//! Tauri adapter for the `updates` domain.
//!
//! There is no `portal_core` half: the only thing this domain answers is the
//! running binary's version, which has to be read in the binary crate to be
//! the version `tauri.conf.json` ships.

pub mod commands;
