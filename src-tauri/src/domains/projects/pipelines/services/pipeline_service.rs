use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{BlockRepository, PipelineRepository};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

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
    pub preset_key: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrontendPipelineRequest {
    name: String,
    description: Option<String>,
    project_id: Option<String>,
    steps: Value,
    variables: Option<Value>,
    secrets: Option<Value>,
    execution_context: Option<Value>,
    enabled: Option<bool>,
    preset_key: Option<String>,
    category: Option<String>,
}

impl FrontendPipelineRequest {
    fn into_pipeline_request(self) -> Result<PipelineRequest, String> {
        let project_id_str = self
            .project_id
            .ok_or_else(|| "projectId is required".to_string())?;
        let project_id = project_id_str
            .parse::<i32>()
            .map_err(|_| format!("Invalid project ID: {}", project_id_str))?;

        let execution_context = self.execution_context.unwrap_or(json!({
            "type": "sdk",
            "sdkType": "node",
            "workingDirectory": "${PROJECT_PATH}"
        }));

        Ok(PipelineRequest {
            name: self.name,
            description: self.description,
            project_id,
            steps_json: serde_json::to_string(&self.steps)
                .map_err(|e| format!("Failed to serialize steps: {}", e))?,
            variables_json: serde_json::to_string(
                &self.variables.unwrap_or(json!([])),
            )
            .map_err(|e| format!("Failed to serialize variables: {}", e))?,
            secrets_json: serde_json::to_string(&self.secrets.unwrap_or(json!([])))
                .map_err(|e| format!("Failed to serialize secrets: {}", e))?,
            execution_context_json: serde_json::to_string(&execution_context)
                .map_err(|e| format!("Failed to serialize execution context: {}", e))?,
            enabled: self.enabled.unwrap_or(true),
            preset_key: self.preset_key,
            category: self.category,
        })
    }
}

pub fn parse_pipeline_request(value: Value) -> Result<PipelineRequest, String> {
    if let Ok(request) = serde_json::from_value::<PipelineRequest>(value.clone()) {
        return Ok(request);
    }

    serde_json::from_value::<FrontendPipelineRequest>(value)
        .map_err(|e| format!("Failed to parse pipeline request: {}", e))?
        .into_pipeline_request()
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
        let pipeline = self
            .pipeline_repo
            .create(
                request.name,
                request.description,
                request.project_id,
                request.steps_json,
                request.variables_json,
                request.secrets_json,
                request.execution_context_json,
                request.enabled,
                request.preset_key,
                request.category,
            )
            .await?;
        Ok(pipeline.id)
    }

    fn pipeline_to_json(p: crate::entities::pipeline::Model) -> Value {
        json!({
            "id": p.id.to_string(),
            "name": p.name,
            "description": p.description,
            "projectId": p.project_id.to_string(),
            "steps": serde_json::from_str::<Value>(&p.steps_json).unwrap_or(json!([])),
            "variables": serde_json::from_str::<Value>(&p.variables_json).unwrap_or(json!([])),
            "secrets": serde_json::from_str::<Value>(&p.secrets_json).unwrap_or(json!([])),
            "executionContext": serde_json::from_str::<Value>(&p.execution_context_json).unwrap_or(json!({})),
            "enabled": p.enabled,
            "presetKey": p.preset_key,
            "category": p.category,
            "createdAt": p.created_at.map(|d| d.to_rfc3339()),
            "updatedAt": p.updated_at.map(|d| d.to_rfc3339()),
        })
    }

    pub async fn get_pipeline(
        &self,
        pipeline_id: i32,
    ) -> Result<Option<Value>, String> {
        let pipeline = self.pipeline_repo.get_by_id(pipeline_id).await?;
        Ok(pipeline.map(Self::pipeline_to_json))
    }

    pub async fn update_pipeline(
        &self,
        pipeline_id: i32,
        request: PipelineRequest,
    ) -> Result<i32, String> {
        let _pipeline = self
            .pipeline_repo
            .update(
                pipeline_id,
                Some(request.name),
                request.description,
                Some(request.steps_json),
                Some(request.variables_json),
                Some(request.secrets_json),
                Some(request.execution_context_json),
                Some(request.enabled),
                None,
                None,
            )
            .await?;
        Ok(pipeline_id)
    }

    pub async fn delete_pipeline(&self, pipeline_id: i32) -> Result<(), String> {
        self.pipeline_repo.delete(pipeline_id).await?;
        Ok(())
    }

    pub async fn get_pipelines(&self, project_id: i32) -> Result<Vec<Value>, String> {
        let pipelines = self.pipeline_repo.get_all_by_project(project_id).await?;
        Ok(pipelines.into_iter().map(Self::pipeline_to_json).collect())
    }

    fn block_to_json(block: crate::entities::block::Model) -> Value {
        json!({
            "id": block.id,
            "name": block.name,
            "description": block.description,
            "category": block.category,
            "version": block.version,
            "parameters": serde_json::from_str::<Value>(&block.parameters_json).unwrap_or(json!([])),
            "command": block.command,
            "executionType": block.execution_type,
            "defaultConfig": serde_json::from_str::<Value>(&block.default_config_json).unwrap_or(json!({})),
            "tags": serde_json::from_str::<Value>(&block.tags_json).unwrap_or(json!([])),
            "icon": block.icon,
            "author": block.author,
            "createdAt": block.created_at.map(|d| d.to_rfc3339()),
            "updatedAt": block.updated_at.map(|d| d.to_rfc3339()),
        })
    }

    pub async fn get_blocks(&self) -> Result<Vec<Value>, String> {
        let blocks = self.block_repo.get_all().await?;
        Ok(blocks.into_iter().map(Self::block_to_json).collect())
    }

    pub async fn create_block(&self, request: Value) -> Result<Value, String> {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Block name is required".to_string())?;
        let description = request
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let category = request
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("utility");
        let command = request
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Block command is required".to_string())?;
        let execution_type = request
            .get("executionType")
            .or_else(|| request.get("execution_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("command");

        let parameters_json = serde_json::to_string(
            &request.get("parameters").unwrap_or(&json!([])),
        )
        .map_err(|e| format!("Failed to serialize parameters: {}", e))?;
        let default_config_json = serde_json::to_string(
            &request
                .get("defaultConfig")
                .or_else(|| request.get("default_config"))
                .unwrap_or(&json!({})),
        )
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
        let tags_json = serde_json::to_string(&request.get("tags").unwrap_or(&json!([])))
            .map_err(|e| format!("Failed to serialize tags: {}", e))?;

        let id = Uuid::new_v4().to_string();
        let block = self
            .block_repo
            .create(
                id,
                name.to_string(),
                description.to_string(),
                category.to_string(),
                "1.0.0".to_string(),
                parameters_json,
                command.to_string(),
                execution_type.to_string(),
                default_config_json,
                tags_json,
                request.get("icon").and_then(|v| v.as_str()).map(str::to_string),
                request.get("author").and_then(|v| v.as_str()).map(str::to_string),
            )
            .await?;

        Ok(Self::block_to_json(block))
    }

    pub async fn update_block(&self, block_id: &str, request: Value) -> Result<Value, String> {
        let parameters_json = request
            .get("parameters")
            .map(|v| serde_json::to_string(v).map_err(|e| format!("Failed to serialize parameters: {}", e)))
            .transpose()?;
        let default_config_json = request
            .get("defaultConfig")
            .or_else(|| request.get("default_config"))
            .map(|v| serde_json::to_string(v).map_err(|e| format!("Failed to serialize default config: {}", e)))
            .transpose()?;
        let tags_json = request
            .get("tags")
            .map(|v| serde_json::to_string(v).map_err(|e| format!("Failed to serialize tags: {}", e)))
            .transpose()?;

        let block = self
            .block_repo
            .update(
                block_id,
                request.get("name").and_then(|v| v.as_str()).map(str::to_string),
                request.get("description").and_then(|v| v.as_str()).map(str::to_string),
                request.get("category").and_then(|v| v.as_str()).map(str::to_string),
                parameters_json,
                request.get("command").and_then(|v| v.as_str()).map(str::to_string),
                request
                    .get("executionType")
                    .or_else(|| request.get("execution_type"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                default_config_json,
                tags_json,
                request.get("icon").and_then(|v| v.as_str()).map(str::to_string),
                request.get("author").and_then(|v| v.as_str()).map(str::to_string),
            )
            .await?;

        Ok(Self::block_to_json(block))
    }

    pub async fn delete_block(&self, block_id: &str) -> Result<(), String> {
        self.block_repo.delete(block_id).await
    }
}
