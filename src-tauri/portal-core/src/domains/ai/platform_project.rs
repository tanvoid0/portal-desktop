//! `project_id` resolution for `POST /api/v1/processes`.
//!
//! agent-platform makes `project_id` mandatory once the caller authenticates
//! with a workspace-scoped token — `processes.rs::start_process` answers 400
//! `"project_id is required for a workspace-scoped token."` without one. A
//! master key, or an install with auth off, must *not* send one: the server
//! leaves the column NULL and the process stays unscoped, which is the
//! behaviour every existing portal install has today.
//!
//! Only the two pure halves live here. The HTTP is at the call sites, which
//! already own a client (async in `coder::service`, blocking in `disk::verify`)
//! and their own auth header style.

use serde_json::Value;

/// Name of the project this app provisions when the workspace has none.
pub const PROJECT_NAME: &str = "portal-desktop";

/// Workspace tokens are `agp_…`; a master key is anything else, and it is
/// unrestricted, so it neither needs nor wants a project scope.
pub fn needs_project_id(api_token: Option<&str>) -> bool {
    api_token
        .map(str::trim)
        .is_some_and(|t| t.starts_with("agp_"))
}

/// First id out of `GET /api/v1/projects/` → `{"projects": [{"id": …}]}`.
/// The list is already filtered to the token's workspace server-side.
pub fn first_project_id(list: &Value) -> Option<i64> {
    list.get("projects")?
        .as_array()?
        .iter()
        .find_map(|p| p.get("id").and_then(Value::as_i64))
}

/// Body for `POST /api/v1/projects/`. No `workspace_id`: the server takes it
/// from the token's principal and ignores anything a workspace token sends.
pub fn create_body() -> Value {
    serde_json::json!({
        "name": PROJECT_NAME,
        "description": "Auto-created by Portal Desktop for platform processes.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_workspace_tokens_need_a_project() {
        assert!(needs_project_id(Some("agp_abc123")));
        assert!(!needs_project_id(Some("sk-master-key")));
        assert!(!needs_project_id(Some("  ")));
        assert!(!needs_project_id(None));
    }

    #[test]
    fn first_project_id_reads_the_list_shape() {
        let list = json!({ "projects": [{ "id": 7, "name": "a" }, { "id": 9 }] });
        assert_eq!(first_project_id(&list), Some(7));
        assert_eq!(first_project_id(&json!({ "projects": [] })), None);
        assert_eq!(first_project_id(&json!({})), None);
    }
}
