//! Online DB-snapshot sync: push a trimmed, encrypted copy of the DB to portal_server,
//! pull one back and stage it for the next boot. See `snapshot` (file/crypto work),
//! `paths` (per-device project paths) and `service` (state, HTTP, orchestration).

pub mod paths;
pub mod service;
pub mod snapshot;
