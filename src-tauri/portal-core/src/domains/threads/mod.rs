//! Server-owned, replayable Coder threads. One actor per thread (`hub`) folds
//! commands into `ThreadEvent`s (`decide`), persists them with the state
//! projection in one transaction (`store`), then fans them out as
//! `coder://thread-event` / `coder://thread-delta` frames.
//! See `docs/development/CODER_THREAD_LOG_DESIGN.md`.
//!
//! S1: only the agent-CLI path feeds the hub, and it only observes.

pub mod apply;
pub mod decide;
pub mod entities;
pub mod hub;
pub mod store;
pub mod types;

#[cfg(test)]
mod tests;

pub use hub::ThreadHub;
pub use types::*;
