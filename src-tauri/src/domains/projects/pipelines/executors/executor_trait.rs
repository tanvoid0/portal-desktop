use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub command: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub timeout: Option<u64>, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub trait Executor: Send + Sync {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult, String>;
    fn can_execute(&self, execution_type: &str) -> bool;
}

