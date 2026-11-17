//! Agent-CLI integration domain — spawns a real coding-agent CLI (Claude Code
//! today) as a subprocess backend behind one trait, so a second backend
//! (Codex, Cursor) is an adapter module, not a rearchitecture. See
//! `backend::AgentBackend` / `backend::RunControl`.
//!
//! This domain knows nothing about `coder`.

pub mod backend;
pub mod claude_code;
pub mod codex;
pub mod entities;
pub mod events;
pub mod process;
pub mod service;

pub use events::AgentEvent;
pub use service::AgentRunService;
