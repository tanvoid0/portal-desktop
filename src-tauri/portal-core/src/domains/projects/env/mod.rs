//! Per-project `.env` viewing with shared-credential linking. The view always
//! re-reads `.env*` files live off disk — see [`service::ProjectEnvService`].
//! [`backup`] is the bulk side: whole files to and from the vault.

pub mod backup;
pub mod entity;
pub mod files_entity;
pub mod service;

pub use backup::{
    EnvBackupFile, EnvBackupReport, EnvBackupScan, EnvBackupService, EnvFileRef, EnvFileState,
    EnvItemResult, EnvOutcome, EnvRestoreItem,
};
pub use service::{
    DriftState, EnvFileView, EnvLinkView, EnvVarView, ProjectEnv, ProjectEnvService,
};
