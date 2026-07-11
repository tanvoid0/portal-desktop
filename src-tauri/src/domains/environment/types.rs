use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
    /// `user` = per-account persistent; `system` = machine-wide persistent.
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvPermissions {
    pub can_edit_user: bool,
    pub can_edit_system: bool,
    pub is_elevated: bool,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvChange {
    pub action: String,
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvApplyResult {
    pub success: bool,
    pub message: String,
    pub elevated: bool,
}
