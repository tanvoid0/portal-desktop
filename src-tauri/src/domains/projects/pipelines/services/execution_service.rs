use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::database::DatabaseManager;
use crate::domains::projects::pipelines::repositories::{ExecutionRepository, PipelineRepository};
// Note: Executors will be used when implementing full execution logic
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequestData {
    pub pipeline_id: String,
    pub variables: Option<HashMap<String, String>>,
    pub secrets: Option<HashMap<String, String>>,
}

pub struct ExecutionService {
    execution_repo: ExecutionRepository,
    pipeline_repo: PipelineRepository,
}

impl ExecutionService {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            execution_repo: ExecutionRepository::new(db_manager.clone()),
            pipeline_repo: PipelineRepository::new(db_manager),
        }
    }

    pub async fn execute_pipeline(
        &self,
        request: ExecutionRequestData,
    ) -> Result<String, String> {
        let execution_id = Uuid::new_v4().to_string();
        let pipeline_id = request.pipeline_id.parse::<i32>()
            .map_err(|_| "Invalid pipeline ID".to_string())?;
        
        // Get pipeline to determine project_id
        let pipeline = self.pipeline_repo.get_by_id(pipeline_id).await?
            .ok_or_else(|| "Pipeline not found".to_string())?;
        
        let variables_json = serde_json::to_string(&request.variables.unwrap_or_default())
            .map_err(|e| format!("Failed to serialize variables: {}", e))?;
        
        // Create execution record
        let _execution = self.execution_repo.create(
            execution_id.clone(),
            pipeline_id,
            pipeline.project_id,
            "pending".to_string(),
            "user".to_string(), // TODO: Get actual user
            serde_json::to_string(&Vec::<serde_json::Value>::new()).unwrap(),
            variables_json,
        ).await?;
        
        // Start execution in background
        let execution_repo = self.execution_repo.clone();
        let execution_id_clone = execution_id.clone();
        
        task::spawn(async move {
            // Update status to running
            let _ = execution_repo.update_status(&execution_id_clone, "running".to_string(), None).await;
            
            // TODO: Implement actual pipeline execution logic
            // This would:
            // 1. Load pipeline steps
            // 2. Resolve dependencies
            // 3. Execute steps using appropriate executor (SDK or Docker)
            // 4. Update step executions
            // 5. Update final status
            
            // For now, mark as success after a delay
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let _ = execution_repo.update_status(&execution_id_clone, "success".to_string(), None).await;
        });
        
        Ok(execution_id)
    }

    pub async fn get_execution(&self, execution_id: &str) -> Result<Option<serde_json::Value>, String> {
        let execution = self.execution_repo.get_by_id(execution_id).await?;
        Ok(execution.map(|e| serde_json::json!({
            "id": e.id,
            "pipelineId": e.pipeline_id.to_string(),
            "projectId": e.project_id.to_string(),
            "status": e.status,
            "startedAt": e.started_at.to_rfc3339(),
            "finishedAt": e.finished_at.map(|d| d.to_rfc3339()),
            "triggeredBy": e.triggered_by,
            "stepExecutions": serde_json::from_str::<serde_json::Value>(&e.step_executions_json).unwrap_or(serde_json::json!([])),
            "variables": serde_json::from_str::<serde_json::Value>(&e.variables_json).unwrap_or(serde_json::json!({})),
            "error": e.error,
        })))
    }

    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        self.execution_repo.update_status(execution_id, "cancelled".to_string(), None).await?;
        Ok(())
    }

    pub async fn get_executions_by_pipeline(&self, pipeline_id: i32) -> Result<Vec<serde_json::Value>, String> {
        let executions = self.execution_repo.get_by_pipeline(pipeline_id).await?;
        Ok(executions.into_iter().map(|e| serde_json::json!({
            "id": e.id,
            "pipelineId": e.pipeline_id.to_string(),
            "projectId": e.project_id.to_string(),
            "status": e.status,
            "startedAt": e.started_at.to_rfc3339(),
            "finishedAt": e.finished_at.map(|d| d.to_rfc3339()),
            "triggeredBy": e.triggered_by,
            "stepExecutions": serde_json::from_str::<serde_json::Value>(&e.step_executions_json).unwrap_or(serde_json::json!([])),
            "variables": serde_json::from_str::<serde_json::Value>(&e.variables_json).unwrap_or(serde_json::json!({})),
            "error": e.error,
        })).collect())
    }
}

