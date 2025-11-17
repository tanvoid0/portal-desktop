use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{PipelineRepository, BlockRepository};
use crate::entities::pipeline::Model as PipelineModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRequest {
    pub name: String,
    pub description: Option<String>,
    pub project_id: i32,
    pub steps_json: String,
    pub variables_json: String,
    pub secrets_json: String,
    pub execution_context_json: String,
    pub enabled: bool,
}

pub struct PipelineService {
    pipeline_repo: PipelineRepository,
    block_repo: BlockRepository,
}

impl PipelineService {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            pipeline_repo: PipelineRepository::new(db_manager.clone()),
            block_repo: BlockRepository::new(db_manager),
        }
    }

    pub async fn create_pipeline(&self, request: PipelineRequest) -> Result<i32, String> {
        let pipeline = self.pipeline_repo.create(
            request.name,
            request.description,
            request.project_id,
            request.steps_json,
            request.variables_json,
            request.secrets_json,
            request.execution_context_json,
            request.enabled,
        ).await?;
        Ok(pipeline.id)
    }

    pub async fn get_pipeline(&self, pipeline_id: i32) -> Result<Option<serde_json::Value>, String> {
        let pipeline = self.pipeline_repo.get_by_id(pipeline_id).await?;
        Ok(pipeline.map(|p| serde_json::json!({
            "id": p.id.to_string(),
            "name": p.name,
            "description": p.description,
            "projectId": p.project_id.to_string(),
            "steps": serde_json::from_str::<serde_json::Value>(&p.steps_json).unwrap_or(serde_json::json!([])),
            "variables": serde_json::from_str::<serde_json::Value>(&p.variables_json).unwrap_or(serde_json::json!([])),
            "secrets": serde_json::from_str::<serde_json::Value>(&p.secrets_json).unwrap_or(serde_json::json!([])),
            "executionContext": serde_json::from_str::<serde_json::Value>(&p.execution_context_json).unwrap_or(serde_json::json!({})),
            "enabled": p.enabled,
            "createdAt": p.created_at.map(|d| d.to_rfc3339()),
            "updatedAt": p.updated_at.map(|d| d.to_rfc3339()),
        })))
    }

    pub async fn update_pipeline(
        &self,
        pipeline_id: i32,
        request: PipelineRequest,
    ) -> Result<i32, String> {
        let _pipeline = self.pipeline_repo.update(
            pipeline_id,
            Some(request.name),
            request.description,
            Some(request.steps_json),
            Some(request.variables_json),
            Some(request.secrets_json),
            Some(request.execution_context_json),
            Some(request.enabled),
        ).await?;
        Ok(pipeline_id)
    }

    pub async fn delete_pipeline(&self, pipeline_id: i32) -> Result<(), String> {
        self.pipeline_repo.delete(pipeline_id).await?;
        Ok(())
    }

    pub async fn get_pipelines(&self, project_id: i32) -> Result<Vec<serde_json::Value>, String> {
        let pipelines = self.pipeline_repo.get_all_by_project(project_id).await?;
        Ok(pipelines.into_iter().map(|p| serde_json::json!({
            "id": p.id.to_string(),
            "name": p.name,
            "description": p.description,
            "projectId": p.project_id.to_string(),
            "steps": serde_json::from_str::<serde_json::Value>(&p.steps_json).unwrap_or(serde_json::json!([])),
            "variables": serde_json::from_str::<serde_json::Value>(&p.variables_json).unwrap_or(serde_json::json!([])),
            "secrets": serde_json::from_str::<serde_json::Value>(&p.secrets_json).unwrap_or(serde_json::json!([])),
            "executionContext": serde_json::from_str::<serde_json::Value>(&p.execution_context_json).unwrap_or(serde_json::json!({})),
            "enabled": p.enabled,
            "createdAt": p.created_at.map(|d| d.to_rfc3339()),
            "updatedAt": p.updated_at.map(|d| d.to_rfc3339()),
        })).collect())
    }
}

