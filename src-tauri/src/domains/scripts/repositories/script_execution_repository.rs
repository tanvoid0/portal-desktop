use crate::database::DatabaseManager;
use crate::entities::script_execution::{ActiveModel, Column, Entity, Model};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ScriptExecutionRepository {
    db_manager: Arc<DatabaseManager>,
}

impl ScriptExecutionRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn create(
        &self,
        id: String,
        block_id: Option<String>,
        command: String,
        parameters_json: String,
        working_directory: Option<String>,
        triggered_by: String,
    ) -> Result<Model, String> {
        let db = self.db_manager.get_connection();
        let id_clone = id.clone();

        let execution = ActiveModel {
            id: Set(id),
            block_id: Set(block_id),
            command: Set(command),
            parameters_json: Set(parameters_json),
            working_directory: Set(working_directory),
            status: Set("pending".to_string()),
            exit_code: Set(None),
            pid: Set(None),
            output: Set(String::new()),
            error: Set(None),
            started_at: Set(chrono::Utc::now().into()),
            finished_at: Set(None),
            triggered_by: Set(triggered_by),
        };

        // Note: For SQLite with string primary keys, insert() tries to return the inserted record
        // which can fail with "RecordNotFound" even though the insert succeeded.
        // We handle this by checking if the error is RecordNotFound and verifying the insert succeeded.
        match execution.insert(db).await {
            Ok(model) => Ok(model),
            Err(e) => {
                let error_str = e.to_string();
                // If error is RecordNotFound, the insert likely succeeded but SeaORM
                // couldn't retrieve it (SQLite limitation). Verify it exists.
                if error_str.contains("RecordNotFound")
                    || error_str.contains("Failed to find inserted item")
                {
                    // Verify the insert actually succeeded
                    Entity::find_by_id(&id_clone)
                        .one(db)
                        .await
                        .map_err(|e| format!("Failed to verify script execution insert: {}", e))?
                        .ok_or_else(|| {
                            format!(
                                "Failed to find inserted script execution with id: {}",
                                id_clone
                            )
                        })
                } else {
                    // Real error, return it
                    Err(format!("Failed to create script execution: {}", e))
                }
            }
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Model>, String> {
        let db = self.db_manager.get_connection();

        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to get script execution: {}", e))
    }

    pub async fn get_by_block(
        &self,
        block_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Model>, String> {
        let db = self.db_manager.get_connection();

        let mut query = Entity::find()
            .filter(Column::BlockId.eq(block_id))
            .order_by_desc(Column::StartedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        query
            .all(db)
            .await
            .map_err(|e| format!("Failed to get script executions: {}", e))
    }

    pub async fn get_running(&self) -> Result<Vec<Model>, String> {
        let db = self.db_manager.get_connection();

        Entity::find()
            .filter(Column::Status.eq("running"))
            .order_by_desc(Column::StartedAt)
            .all(db)
            .await
            .map_err(|e| format!("Failed to get running executions: {}", e))
    }

    pub async fn get_recent(&self, limit: u64) -> Result<Vec<Model>, String> {
        let db = self.db_manager.get_connection();

        Entity::find()
            .order_by_desc(Column::StartedAt)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| format!("Failed to get recent executions: {}", e))
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: String,
        exit_code: Option<i32>,
        error: Option<String>,
    ) -> Result<Model, String> {
        let db = self.db_manager.get_connection();

        let execution = Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to find execution: {}", e))?
            .ok_or_else(|| format!("Execution not found: {}", id))?;

        let mut active_model: ActiveModel = execution.into();
        active_model.status = Set(status.clone());

        if let Some(code) = exit_code {
            active_model.exit_code = Set(Some(code));
        }

        if let Some(err) = error {
            active_model.error = Set(Some(err));
        }

        // Set finished_at if status is terminal
        if status == "success" || status == "failed" || status == "cancelled" {
            active_model.finished_at = Set(Some(chrono::Utc::now().into()));
        }

        active_model
            .update(db)
            .await
            .map_err(|e| format!("Failed to update execution status: {}", e))
    }

    pub async fn update_pid(&self, id: &str, pid: i32) -> Result<Model, String> {
        let db = self.db_manager.get_connection();

        let execution = Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to find execution: {}", e))?
            .ok_or_else(|| format!("Execution not found: {}", id))?;

        let mut active_model: ActiveModel = execution.into();
        active_model.pid = Set(Some(pid));
        active_model.status = Set("running".to_string());

        active_model
            .update(db)
            .await
            .map_err(|e| format!("Failed to update execution pid: {}", e))
    }

    pub async fn append_output(&self, id: &str, new_output: &str) -> Result<Model, String> {
        let db = self.db_manager.get_connection();

        let execution = Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to find execution: {}", e))?
            .ok_or_else(|| format!("Execution not found: {}", id))?;

        let mut active_model: ActiveModel = execution.clone().into();
        let updated_output = format!("{}{}", execution.output, new_output);
        active_model.output = Set(updated_output);

        active_model
            .update(db)
            .await
            .map_err(|e| format!("Failed to append output: {}", e))
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let db = self.db_manager.get_connection();

        Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| format!("Failed to delete execution: {}", e))?;

        Ok(())
    }
}
