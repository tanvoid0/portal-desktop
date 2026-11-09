//! Per-project `.env` viewing with shared-credential linking. The view always
//! re-reads `.env*` files live off disk — see [`service::ProjectEnvService`].

pub mod entity;
pub mod service;

pub use service::{DriftState, EnvFileView, EnvLinkView, EnvVarView, ProjectEnv, ProjectEnvService};
