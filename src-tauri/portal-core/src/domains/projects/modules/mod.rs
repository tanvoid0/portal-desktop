//! Apps, services and packages inside a project: found on disk by [`detect`],
//! cached in `project_modules` by [`service`].

pub mod detect;
pub mod service;

pub use detect::DetectedModule;
pub use service::ModuleService;
