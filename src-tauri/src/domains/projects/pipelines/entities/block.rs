use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String, // "build", "test", "deploy", "utility", "custom"
    pub version: String,
    pub parameters_json: String, // JSON array of BlockParameter
    pub command: String,
    pub execution_type: String, // "command", "script", "docker"
    pub default_config_json: String, // JSON object
    pub tags_json: String, // JSON array of strings
    pub icon: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParameterModel {
    pub name: String,
    pub param_type: String, // "string", "number", "boolean", "select", "file", "directory"
    pub description: String,
    pub required: bool,
    pub default_value_json: Option<String>, // JSON value
    pub options_json: Option<String>, // JSON array of strings
    pub validation_json: Option<String>, // JSON validation object
}

