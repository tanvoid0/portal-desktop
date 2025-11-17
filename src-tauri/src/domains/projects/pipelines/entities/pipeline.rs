use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineModel {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub project_id: i32,
    pub steps_json: String, // JSON array of PipelineStep
    pub variables_json: String, // JSON array of PipelineVariable
    pub secrets_json: String, // JSON array of secret IDs
    pub execution_context_json: String, // JSON ExecutionContext
    pub enabled: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContextModel {
    pub execution_type: String, // "sdk" or "docker"
    pub sdk_type: Option<String>,
    pub sdk_version: Option<String>,
    pub docker_image: Option<String>,
    pub dockerfile: Option<String>,
    pub docker_context: Option<String>,
    pub working_directory: String,
    pub environment_json: Option<String>, // JSON object
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineVariableModel {
    pub name: String,
    pub value: String,
    pub var_type: String, // "string", "number", "boolean"
    pub description: Option<String>,
    pub scope: String, // "project" or "pipeline"
}

