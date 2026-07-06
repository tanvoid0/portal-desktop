//! Disk-cleanup domain: scans a tree, proposes regenerable/large-old items for
//! removal, moves confirmed items to the Recycle Bin (reversible), and keeps an
//! audit log. Ported from the standalone `portal_disk_utility` app.
//!
//! Read-only with respect to user files except `quarantine_paths`, which only
//! ever moves explicitly-confirmed items to the OS trash. See
//! `docs/development/DISK_UTILITY_MIGRATION.md`.

pub mod classify;
pub mod commands;
pub mod db;
pub mod disk;
pub mod locations;
pub mod projects;
pub mod quarantine;
pub mod scan;
// Agent-Platform verification path — retained as a fallback but no longer the
// default (the command uses `verify_ai`). Dead-code allowed until it is either
// wired back in as an option or removed. See DISK_UTILITY_MIGRATION.md.
#[allow(dead_code)]
pub mod verify;
pub mod verify_ai;
