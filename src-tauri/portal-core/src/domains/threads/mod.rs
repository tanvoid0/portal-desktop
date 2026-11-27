//! Server-owned, replayable Coder threads. One actor per thread (`hub`) folds
//! commands into `ThreadEvent`s (`decide`), persists them with the state
//! projection in one transaction (`store`), then fans them out as
//! `coder://thread-event` / `coder://thread-delta` frames.
//! See `docs/development/CODER_THREAD_LOG_DESIGN.md`.
//!
//! S2a: the hub also takes client commands (`ThreadHub::command`: dedup,
//! first-wins approvals, queue) and runs `Effect`s through registered
//! `ThreadEffects`. Until S3 registers them only the agent-CLI path feeds the hub.

pub mod apply;
pub mod decide;
pub mod entities;
pub mod hub;
pub mod legacy_import;
pub mod store;
pub mod types;

#[cfg(test)]
mod tests;

pub use hub::ThreadHub;
pub use types::*;
