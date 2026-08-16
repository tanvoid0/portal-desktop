use crate::database::DatabaseManager;
use crate::entities::pipeline_execution::{
    ActiveModel as ExecutionActiveModel, Entity as ExecutionEntity, Model as ExecutionModel,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

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

    pub async fn get_by_pipeline(
        &self,
        pipeline_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let mut query = ExecutionEntity::find()
            .filter(crate::entities::pipeline_execution::Column::PipelineId.eq(pipeline_id))
            .order_by_desc(crate::entities::pipeline_execution::Column::StartedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let executions = query
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch executions: {}", e))?;
        Ok(executions)
    }

    pub async fn get_by_project(
        &self,
        project_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let mut query = ExecutionEntity::find()
            .filter(crate::entities::pipeline_execution::Column::ProjectId.eq(project_id))
            .order_by_desc(crate::entities::pipeline_execution::Column::StartedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let executions = query
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch executions: {}", e))?;
        Ok(executions)
    }

    pub async fn get_all(&self, limit: Option<u64>) -> Result<Vec<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let mut query = ExecutionEntity::find()
            .order_by_desc(crate::entities::pipeline_execution::Column::StartedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let executions = query
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
        let id_clone = id.clone();
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

        // SQLite with string primary keys: insert() may return RecordNotFound even when
        // the row was written. Verify by id when that happens (see script_execution_repository).
        match execution.insert(connection).await {
            Ok(model) => Ok(model),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("RecordNotFound")
                    || error_str.contains("Failed to find inserted item")
                {
                    ExecutionEntity::find_by_id(&id_clone)
                        .one(connection)
                        .await
                        .map_err(|e| format!("Failed to verify execution insert: {}", e))?
                        .ok_or_else(|| {
                            format!("Failed to find inserted execution with id: {}", id_clone)
                        })
                } else {
                    Err(format!("Failed to create execution: {}", e))
                }
            }
        }
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

        let result = execution
            .update(connection)
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;

        Ok(result)
    }

    pub async fn update_step_executions(
        &self,
        id: &str,
        step_executions_json: String,
    ) -> Result<ExecutionModel, String> {
        let connection = self.db_manager.get_connection();

        let mut execution: ExecutionActiveModel = ExecutionEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find execution: {}", e))?
            .ok_or_else(|| "Execution not found".to_string())?
            .into();

        execution.step_executions_json = Set(step_executions_json);

        let result = execution
            .update(connection)
            .await
            .map_err(|e| format!("Failed to update step executions: {}", e))?;

        Ok(result)
    }
}
