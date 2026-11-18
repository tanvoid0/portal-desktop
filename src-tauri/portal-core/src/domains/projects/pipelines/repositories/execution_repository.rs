use crate::database::DatabaseManager;
use crate::entities::pipeline_execution::{
    ActiveModel as ExecutionActiveModel, Column, Entity as ExecutionEntity, Model as ExecutionModel,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;

/// The columns only an action row (`kind = 'action'`) carries: one run of a
/// command — an Action, a terminal run, a workflow step — rather than a pipeline.
#[derive(Clone, Debug, Default)]
pub struct ActionMeta {
    pub command: String,
    pub working_directory: Option<String>,
    pub block_id: Option<String>,
    pub workflow_run_id: Option<String>,
    pub workflow_id: Option<String>,
    pub step_id: Option<String>,
    pub workflow_plan_json: Option<String>,
}

/// A row for [`ExecutionRepository::insert`].
#[derive(Default)]
pub struct NewRun {
    pub id: String,
    pub pipeline_id: Option<i32>,
    pub project_id: Option<i32>,
    pub status: String,
    pub triggered_by: String,
    pub step_executions_json: String,
    pub variables_json: String,
    /// `None` = now.
    pub started_at: Option<DateTime<Utc>>,
    /// For a run that is already over (see `ExecutionService::record_finished_action`).
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    /// `Some` makes it a `kind = 'action'` row; `None` is a pipeline row.
    pub action: Option<ActionMeta>,
}

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
        // Pipeline runs only: an action may carry a project id too.
        let mut query = ExecutionEntity::find()
            .filter(crate::entities::pipeline_execution::Column::Kind.eq("pipeline"))
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

    /// Pipeline and release runs only — what the pipeline commands return.
    /// [`Self::get_recent`] with `None` has every kind.
    pub async fn get_all(&self, limit: Option<u64>) -> Result<Vec<ExecutionModel>, String> {
        self.get_recent(Some("pipeline"), limit).await
    }

    /// Newest first: rows of one `kind` ("pipeline" or "action"), or of every
    /// kind when `kind` is `None`.
    pub async fn get_recent(
        &self,
        kind: Option<&str>,
        limit: Option<u64>,
    ) -> Result<Vec<ExecutionModel>, String> {
        let connection = self.db_manager.get_connection();
        let mut query = ExecutionEntity::find().order_by_desc(Column::StartedAt);

        if let Some(kind) = kind {
            query = query.filter(Column::Kind.eq(kind));
        }
        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        let executions = query
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch executions: {}", e))?;
        Ok(executions)
    }

    /// Action runs of one block, newest first.
    pub async fn get_by_block(
        &self,
        block_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<ExecutionModel>, String> {
        let mut query = ExecutionEntity::find()
            .filter(Column::Kind.eq("action"))
            .filter(Column::BlockId.eq(block_id))
            .order_by_desc(Column::StartedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        query
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to fetch executions: {}", e))
    }

    /// Every step row of one workflow run, oldest first.
    pub async fn get_by_workflow_run(&self, run_id: &str) -> Result<Vec<ExecutionModel>, String> {
        ExecutionEntity::find()
            .filter(Column::WorkflowRunId.eq(run_id))
            .order_by_asc(Column::StartedAt)
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to fetch workflow run executions: {}", e))
    }

    /// Action runs that have not finished (`pending`, `queued`, `running`), newest first.
    pub async fn get_active_actions(&self) -> Result<Vec<ExecutionModel>, String> {
        ExecutionEntity::find()
            .filter(Column::Kind.eq("action"))
            .filter(Column::Status.is_in(["pending", "queued", "running"]))
            .order_by_desc(Column::StartedAt)
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to fetch active actions: {}", e))
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        ExecutionEntity::delete_by_id(id)
            .exec(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to delete execution: {}", e))?;
        Ok(())
    }

    /// Executions of every kind that cannot outlive the process that was driving
    /// them, newest first.
    pub async fn get_non_terminal(&self) -> Result<Vec<ExecutionModel>, String> {
        ExecutionEntity::find()
            .filter(Column::Status.is_in(["pending", "queued", "running"]))
            .order_by_desc(Column::StartedAt)
            .all(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to fetch unfinished executions: {}", e))
    }

    pub async fn create(
        &self,
        id: String,
        pipeline_id: Option<i32>,
        project_id: Option<i32>,
        status: String,
        triggered_by: String,
        step_executions_json: String,
        variables_json: String,
    ) -> Result<ExecutionModel, String> {
        self.insert(NewRun {
            id,
            pipeline_id,
            project_id,
            status,
            triggered_by,
            step_executions_json,
            variables_json,
            ..Default::default()
        })
        .await
    }

    pub async fn insert(&self, run: NewRun) -> Result<ExecutionModel, String> {
        let connection = self.db_manager.get_connection();

        let id_clone = run.id.clone();
        let mut execution = ExecutionActiveModel {
            id: Set(run.id),
            pipeline_id: Set(run.pipeline_id),
            project_id: Set(run.project_id),
            kind: Set("pipeline".to_string()),
            status: Set(run.status),
            started_at: Set(run.started_at.unwrap_or_else(Utc::now).into()),
            finished_at: Set(run.finished_at.map(Into::into)),
            triggered_by: Set(run.triggered_by),
            step_executions_json: Set(run.step_executions_json),
            variables_json: Set(run.variables_json),
            error: Set(run.error),
            ..Default::default()
        };
        if let Some(a) = run.action {
            execution.kind = Set("action".to_string());
            execution.command = Set(Some(a.command));
            execution.working_directory = Set(a.working_directory);
            execution.block_id = Set(a.block_id);
            execution.workflow_run_id = Set(a.workflow_run_id);
            execution.workflow_id = Set(a.workflow_id);
            execution.step_id = Set(a.step_id);
            execution.workflow_plan_json = Set(a.workflow_plan_json);
        }

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

    /// A Stop is final: a row that is already `cancelled` stays so unless this
    /// write is `cancelled` too. The run loop checks for Stop and then awaits
    /// before its own `success` / `failed` write; a Stop landing in that gap
    /// must not be undone by it. One conditional UPDATE, so no read can race it.
    pub async fn update_status(
        &self,
        id: &str,
        status: String,
        error: Option<String>,
    ) -> Result<ExecutionModel, String> {
        let mut update = ExecutionEntity::update_many()
            .col_expr(Column::Status, sea_orm::sea_query::Expr::value(status.clone()))
            .filter(Column::Id.eq(id));
        if let Some(error) = error {
            update = update.col_expr(Column::Error, sea_orm::sea_query::Expr::value(error));
        }
        if matches!(status.as_str(), "success" | "failed" | "cancelled") {
            update = update.col_expr(
                Column::FinishedAt,
                sea_orm::sea_query::Expr::value(Utc::now().fixed_offset()),
            );
        }
        if status != "cancelled" {
            update = update.filter(Column::Status.ne("cancelled"));
        }
        update
            .exec(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to update execution: {}", e))?;

        self.get_by_id(id)
            .await?
            .ok_or_else(|| "Execution not found".to_string())
    }

    /// The pid of an action run's process (pipeline rows never get one).
    pub async fn update_pid(&self, id: &str, pid: i32) -> Result<(), String> {
        ExecutionEntity::update_many()
            .col_expr(Column::Pid, sea_orm::sea_query::Expr::value(pid))
            .filter(Column::Id.eq(id))
            .exec(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to update pid: {}", e))?;
        Ok(())
    }

    /// One UPDATE, no reads: it runs per log line, under the executor's blob lock.
    pub async fn update_step_executions(
        &self,
        id: &str,
        step_executions_json: String,
    ) -> Result<(), String> {
        ExecutionEntity::update_many()
            .col_expr(
                Column::StepExecutionsJson,
                sea_orm::sea_query::Expr::value(step_executions_json),
            )
            .filter(Column::Id.eq(id))
            .exec(self.db_manager.get_connection())
            .await
            .map_err(|e| format!("Failed to update step executions: {}", e))?;
        Ok(())
    }
}
