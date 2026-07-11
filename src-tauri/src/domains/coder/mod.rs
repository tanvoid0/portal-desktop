//! Coding agent domain.
//!
//! Agent turns run through agent-platform `POST /api/v1/coder/chat/stream` when
//! configured (AI → Providers). Workspace tools execute on the desktop host and
//! results are posted back to the platform (`portal-desktop` client id).
//! Thread state is mirrored locally for the sidebar; `platform_thread_id` links
//! the SQLite row to `coder_chat_threads` on the platform.

pub mod agent_mode;
pub mod commands;
pub mod diff;
pub mod entities;
pub mod git_commit;
pub mod git_status;
pub mod multitask;
pub mod permissions;
pub mod platform_stream;
pub mod service;
pub mod tools;
pub mod types;
pub mod worktree;

pub use commands::*;
pub use service::CoderService;
