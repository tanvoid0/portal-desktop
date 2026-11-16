pub mod docker_executor;
pub mod executor_trait;

pub use executor_trait::*;
// Unsupported in v1 Actions model — local shell runner only.
// The Docker executor type remains for possible future runners.
// pub use docker_executor::*;
