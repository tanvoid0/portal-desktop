//! Model-ops proxy: fine-tune projects, build jobs, local Ollama models and the
//! adapter registry.
//!
//! Portal owns none of this logic — it renders what `agent-platformd`'s
//! `/api/v1/model-ops/*` API returns (ADR 0020). Build jobs are polled rather
//! than streamed: the job endpoint already returns `log_tail`, so one poll gives
//! both status and logs. Opaque fields (`manifest`, `result`, `progress`,
//! `metadata`) are kept as raw JSON so the server can add to them without a
//! change here.

use super::providers::{AIError, AgentPlatformProvider};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Pipeline stages a build job runs, in execution order.
pub const BUILD_STAGES: [&str; 5] = ["prepare", "train", "merge", "quantize", "register"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistryEntry {
    pub id: i64,
    pub project_id: i64,
    #[serde(default)]
    pub project_name: Option<String>,
    pub version: String,
    pub ollama_tag: String,
    #[serde(default)]
    pub base_model: Option<String>,
    #[serde(default)]
    pub eval_score: Option<f64>,
    #[serde(default)]
    pub metadata: Value,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProject {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub manifest: Value,
    #[serde(default)]
    pub registry_entries: Vec<ModelRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBuildJob {
    pub id: i64,
    #[serde(default)]
    pub job_type: Option<String>,
    #[serde(default)]
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub stages: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub current_stage: Option<String>,
    #[serde(default)]
    pub register_alias: Option<String>,
    #[serde(default)]
    pub result: Value,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub progress: Value,
    #[serde(default)]
    pub log_tail: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

/// `POST /operations/build` returns only a job handle; we fetch the full job.
#[derive(Debug, Clone, Deserialize)]
struct ModelBuildOperationOut {
    job_id: i64,
}

impl AgentPlatformProvider {
    pub async fn model_projects(&self) -> Result<Vec<ModelProject>, AIError> {
        #[derive(Deserialize)]
        struct Out {
            projects: Vec<ModelProject>,
        }
        let out: Out = self
            .request_json(reqwest::Method::GET, "/api/v1/model-ops/projects", None, 20)
            .await?;
        Ok(out.projects)
    }

    pub async fn create_model_project(
        &self,
        name: String,
        description: Option<String>,
        base_model: Option<String>,
    ) -> Result<ModelProject, AIError> {
        let body = json!({ "name": name, "description": description, "base_model": base_model });
        self.request_json(
            reqwest::Method::POST,
            "/api/v1/model-ops/projects",
            Some(body),
            20,
        )
        .await
    }

    /// Local models Ollama has on this machine. Shapes vary by Ollama version, so
    /// each row stays raw JSON (the UI reads `name` and `size`).
    pub async fn ollama_models(&self) -> Result<Vec<Value>, AIError> {
        #[derive(Deserialize)]
        struct Out {
            models: Vec<Value>,
        }
        let out: Out = self
            .request_json(
                reqwest::Method::GET,
                "/api/v1/model-ops/ollama/models",
                None,
                20,
            )
            .await?;
        Ok(out.models)
    }

    /// Pull a model asynchronously; returns a build job the UI can poll.
    pub async fn pull_ollama_model(&self, name: String) -> Result<ModelBuildJob, AIError> {
        let body = json!({ "name": name, "async": true });
        self.request_json(
            reqwest::Method::POST,
            "/api/v1/model-ops/ollama/models/pull",
            Some(body),
            30,
        )
        .await
    }

    pub async fn model_registry(&self) -> Result<Vec<ModelRegistryEntry>, AIError> {
        #[derive(Deserialize)]
        struct Out {
            entries: Vec<ModelRegistryEntry>,
        }
        let out: Out = self
            .request_json(reqwest::Method::GET, "/api/v1/model-ops/registry", None, 20)
            .await?;
        Ok(out.entries)
    }

    pub async fn activate_registry_entry(
        &self,
        entry_id: i64,
    ) -> Result<ModelRegistryEntry, AIError> {
        let path = format!("/api/v1/model-ops/registry/{entry_id}/activate");
        self.request_json(reqwest::Method::POST, &path, None, 20)
            .await
    }

    pub async fn start_model_build(
        &self,
        project: String,
        stages: Vec<String>,
        register_alias: Option<String>,
    ) -> Result<ModelBuildJob, AIError> {
        let body = json!({
            "operation": "build",
            "input": {
                "project": project,
                "stages": stages,
                "register_alias": register_alias,
            }
        });
        let op: ModelBuildOperationOut = self
            .request_json(
                reqwest::Method::POST,
                "/api/v1/model-ops/operations/build",
                Some(body),
                30,
            )
            .await?;
        // operations/build returns only an id; fetch the full job so the UI has a
        // card to show immediately.
        self.model_build_job(op.job_id).await
    }

    pub async fn model_build_job(&self, job_id: i64) -> Result<ModelBuildJob, AIError> {
        let path = format!("/api/v1/model-ops/jobs/{job_id}");
        self.request_json(reqwest::Method::GET, &path, None, 20).await
    }

    pub async fn cancel_model_build_job(&self, job_id: i64) -> Result<ModelBuildJob, AIError> {
        let path = format!("/api/v1/model-ops/jobs/{job_id}/cancel");
        self.request_json(reqwest::Method::POST, &path, None, 20)
            .await
    }
}

/// Stages in canonical pipeline order, whatever order they were given in. Keeps
/// an operator's checkbox order from reordering the build.
pub fn ordered_stages(stages: &[String]) -> Vec<String> {
    BUILD_STAGES
        .iter()
        .filter(|s| stages.iter().any(|sel| sel == *s))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_stages_are_canonical_not_input_order() {
        let input = vec!["register".to_string(), "train".to_string(), "prepare".to_string()];
        assert_eq!(ordered_stages(&input), vec!["prepare", "train", "register"]);
    }

    #[test]
    fn ordered_stages_drops_unknown_stages() {
        let input = vec!["train".to_string(), "bogus".to_string()];
        assert_eq!(ordered_stages(&input), vec!["train"]);
    }
}
