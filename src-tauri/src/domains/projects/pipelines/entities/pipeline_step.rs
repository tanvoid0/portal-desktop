use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepModel {
    pub id: String,
    pub block_id: String,
    pub name: String,
    pub config_json: String, // JSON object
    pub depends_on_json: String, // JSON array of step IDs
    pub condition: Option<String>,
    pub retries: Option<i32>,
    pub retry_delay: Option<i32>,
    pub timeout: Option<i32>,
    pub parallel: Option<bool>,
    pub on_success_json: Option<String>, // JSON array of step IDs
    pub on_failure_json: Option<String>, // JSON array of step IDs
}

