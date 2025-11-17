use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutionModel {
    pub id: String,
    pub pipeline_id: i32,
    pub project_id: i32,
    pub status: String, // "pending", "running", "success", "failed", "cancelled", "skipped"
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub triggered_by: String,
    pub step_executions_json: String, // JSON array of StepExecution
    pub variables_json: String, // JSON object
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionModel {
    pub id: String,
    pub step_id: String,
    pub step_name: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub output: String,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub retry_count: i32,
    pub logs_json: String, // JSON array of strings
}

