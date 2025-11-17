use crate::database::DatabaseManager;
use crate::entities::pipeline_execution::{Entity as ExecutionEntity, ActiveModel as ExecutionActiveModel, Model as ExecutionModel};
use sea_orm::{EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter};
use std::sync::Arc;
use chrono::{Utc, DateTime};

#[derive(Clone)]
pub struct ExecutionRepository {
    db_manager: Arc<DatabaseManager>,
}

impl ExecutionRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let execution = ExecutionEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch execution: {}", e))?;
        Ok(execution)
    }

    pub async fn get_by_pipeline(&self, pipeline_id: i32) -> Result<Vec<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let executions = ExecutionEntity::find()
            .filter(crate::entities::pipeline_execution::Column::PipelineId.eq(pipeline_id))
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch executions: {}", e))?;
        Ok(executions)
    }

    pub async fn create(
        &self,
        id: String,
        pipeline_id: i32,
        project_id: i32,
        status: String,
        triggered_by: String,
        step_executions_json: String,
        variables_json: String,
    ) -> Result<ExecutionModel, String> {
        let connection = self.db_manager.get_connection();
        
        let now: DateTime<Utc> = Utc::now();
        let execution = ExecutionActiveModel {
            id: Set(id),
            pipeline_id: Set(pipeline_id),
            project_id: Set(project_id),
            status: Set(status),
            started_at: Set(now.into()),
            triggered_by: Set(triggered_by),
            step_executions_json: Set(step_executions_json),
            variables_json: Set(variables_json),
            ..Default::default()
        };
        
        let result = execution.insert(connection)
            .await
            .map_err(|e| format!("Failed to create execution: {}", e))?;
        
        Ok(result)
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: String,
        error: Option<String>,
    ) -> Result<ExecutionModel, String> {
        let connection = self.db_manager.get_connection();
        
        let mut execution: ExecutionActiveModel = ExecutionEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find execution: {}", e))?
            .ok_or_else(|| "Execution not found".to_string())?
            .into();
        
        let status_clone = status.clone();
        execution.status = Set(status);
        if let Some(error) = error {
            execution.error = Set(Some(error));
        }
        if status_clone == "success" || status_clone == "failed" || status_clone == "cancelled" {
            let now: DateTime<Utc> = Utc::now();
            execution.finished_at = Set(Some(now.into()));
        }
        
        let result = execution.update(connection)
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;
        
        Ok(result)
    }
}

