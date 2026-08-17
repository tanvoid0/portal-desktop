//! Project discovery — bulk registry population for the `projects` domain.
//!
//! The disk domain's `ProjectDevCleaner` already walks drives, but it exists to
//! find *regenerable temp directories*: its output is built from cleanable items,
//! so a project with no `node_modules` produces no items and never appears. This
//! module answers the other question — "what projects exist, are they alive, and
//! is any of their work only on this disk?" — and feeds the answer into the
//! projects registry so the two views share one source of truth.
//!
//! Deliberately shells out to `git` rather than linking libgit2: the projects
//! domain already probes git that way (`ProjectService::get_git_info`) and this
//! keeps the build free of a native dependency.

pub mod analysis;
pub mod git;
pub mod scan;

pub use analysis::*;
pub use git::*;
pub use scan::*;
