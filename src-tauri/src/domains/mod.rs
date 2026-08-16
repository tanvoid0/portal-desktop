//! Tauri command adapters, one module per domain that has commands.
//!
//! Each re-exports its `portal_core::domains::*` counterpart, so the module
//! paths `lib.rs`'s `generate_handler!` list uses are unchanged by the split.

pub use portal_core::domains::*;

pub mod ai;
pub mod automation;
pub mod autonomy;
pub mod coder;
pub mod credentials;
pub mod custom_scripts;
pub mod dashboard;
pub mod deployments;
pub mod disk;
pub mod documents;
pub mod environment;
pub mod github;
pub mod ide;
pub mod kubernetes;
pub mod languages;
pub mod learning;
pub mod network;
pub mod package_managers;
pub mod projects;
pub mod scripts;
pub mod sdk;
pub mod settings;
pub mod tasks;
pub mod terminal;
pub mod updates;
