//! Coding agent domain.
//!
//! Architecture (Option B — "portal drives, platform is the brain"):
//! - The agent loop runs here in Rust. It owns tool execution, the permission
//!   system, and thread state.
//! - The agent-platform (`http://127.0.0.1:18410`) is used only as an
//!   OpenAI-compatible LLM via `/v1/chat/completions` (tool-calling format),
//!   plus optional delegated capability tools over `/api/v1`.
//! - Tools execute against the user's native desktop through existing domains
//!   (filesystem, terminal, ...), so the agent can do far more than the
//!   platform's sandboxed Python executor.

pub mod commands;
pub mod diff;
pub mod entities;
pub mod permissions;
pub mod service;
pub mod tools;
pub mod types;

pub use commands::*;
pub use service::CoderService;
