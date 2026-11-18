use crate::database::DatabaseManager;
use crate::entities::script_execution::{ActiveModel, Column, Entity, Model};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

/// Which workflow run/step a row belongs to. Default = ad-hoc run.
#[derive(Clone, Default)]
pub struct WorkflowLink {
    pub run_id: Option<String>,
    pub workflow_id: Option<String>,
    pub step_id: Option<String>,
    pub plan_json: Option<String>,
}

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
        workflow: WorkflowLink,
    ) -> Result<Model, String> {
        let db = self.db_manager.get_connection();
        let id_clone = id.clone();

        // block_id has an FK to blocks, but built-in blocks live in code, not in that table.
        // Keep the link only when the row exists, otherwise record the run as ad-hoc.
        let block_id = match block_id {
            Some(b) => crate::entities::block::Entity::find_by_id(&b)
                .one(db)
                .await
                .map_err(|e| format!("Failed to look up block: {}", e))?
                .map(|_| b),
            None => None,
        };

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
            workflow_run_id: Set(workflow.run_id),
            workflow_id: Set(workflow.workflow_id),
            step_id: Set(workflow.step_id),
            workflow_plan_json: Set(workflow.plan_json),
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

    pub async fn get_by_workflow_run(&self, run_id: &str) -> Result<Vec<Model>, String> {
        Entity::find()
            .filter(Column::WorkflowRunId.eq(run_id))
            .order_by_asc(Column::StartedAt)
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to get workflow run executions: {}", e))
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

    /// Rows still "pending" at startup never got a process; nothing can finish them.
    pub async fn get_pending(&self) -> Result<Vec<Model>, String> {
        Entity::find()
            .filter(Column::Status.eq("pending"))
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to get pending executions: {}", e))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn workflow_run_rows_come_back_oldest_first() {
        let dir = tempfile::tempdir().unwrap();
        let db = DatabaseManager::new(dir.path().to_path_buf())
            .await
            .unwrap();
        let repo = ScriptExecutionRepository::new(Arc::new(db));
        let link = |run: &str, step: &str| WorkflowLink {
            run_id: Some(run.into()),
            step_id: Some(step.into()),
            ..Default::default()
        };
        for (id, run, step) in [("a", "r1", "build"), ("b", "r2", "x"), ("c", "r1", "test")] {
            let (cmd, params, user) = ("echo".to_string(), "{}".to_string(), "user".to_string());
            repo.create(id.into(), None, cmd, params, None, user, link(run, step))
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        let rows = repo.get_by_workflow_run("r1").await.unwrap();
        let steps: Vec<_> = rows.iter().map(|r| r.step_id.as_deref().unwrap()).collect();
        assert_eq!(steps, ["build", "test"]);
    }
}
