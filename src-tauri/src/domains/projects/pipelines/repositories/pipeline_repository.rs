use crate::database::DatabaseManager;
use crate::entities::pipeline::{
    ActiveModel as PipelineActiveModel, Column as PipelineColumn, Entity as PipelineEntity,
    Model as PipelineModel,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[derive(Clone)]
pub struct PipelineRepository {
    db_manager: Arc<DatabaseManager>,
}

impl PipelineRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_all_by_project(&self, project_id: i32) -> Result<Vec<PipelineModel>, String> {
        let connection = self.db_manager.get_connection();
        let pipelines = PipelineEntity::find()
            .filter(PipelineColumn::ProjectId.eq(project_id))
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch pipelines: {}", e))?;
        Ok(pipelines)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<PipelineModel>, String> {
        let connection = self.db_manager.get_connection();
        let pipeline = PipelineEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch pipeline: {}", e))?;
        Ok(pipeline)
    }

    pub async fn get_by_preset_key(
        &self,
        project_id: i32,
        preset_key: &str,
    ) -> Result<Option<PipelineModel>, String> {
        let connection = self.db_manager.get_connection();
        let pipeline = PipelineEntity::find()
            .filter(PipelineColumn::ProjectId.eq(project_id))
            .filter(PipelineColumn::PresetKey.eq(preset_key))
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch pipeline by preset key: {}", e))?;
        Ok(pipeline)
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        project_id: i32,
        steps_json: String,
        variables_json: String,
        secrets_json: String,
        execution_context_json: String,
        enabled: bool,
        preset_key: Option<String>,
        category: Option<String>,
    ) -> Result<PipelineModel, String> {
        let connection = self.db_manager.get_connection();

        let pipeline = PipelineActiveModel {
            name: Set(name),
            description: Set(description),
            project_id: Set(project_id),
            steps_json: Set(steps_json),
            variables_json: Set(variables_json),
            secrets_json: Set(secrets_json),
            execution_context_json: Set(execution_context_json),
            enabled: Set(enabled),
            preset_key: Set(preset_key),
            category: Set(category),
            ..Default::default()
        };

        let result = pipeline
            .insert(connection)
            .await
            .map_err(|e| format!("Failed to create pipeline: {}", e))?;

        Ok(result)
    }

    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        steps_json: Option<String>,
        variables_json: Option<String>,
        secrets_json: Option<String>,
        execution_context_json: Option<String>,
        enabled: Option<bool>,
        preset_key: Option<Option<String>>,
        category: Option<Option<String>>,
    ) -> Result<PipelineModel, String> {
        let connection = self.db_manager.get_connection();

        let mut pipeline: PipelineActiveModel = PipelineEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find pipeline: {}", e))?
            .ok_or_else(|| "Pipeline not found".to_string())?
            .into();

        if let Some(name) = name {
            pipeline.name = Set(name);
        }
        if let Some(description) = description {
            pipeline.description = Set(Some(description));
        }
        if let Some(steps_json) = steps_json {
            pipeline.steps_json = Set(steps_json);
        }
        if let Some(variables_json) = variables_json {
            pipeline.variables_json = Set(variables_json);
        }
        if let Some(secrets_json) = secrets_json {
            pipeline.secrets_json = Set(secrets_json);
        }
        if let Some(execution_context_json) = execution_context_json {
            pipeline.execution_context_json = Set(execution_context_json);
        }
        if let Some(enabled) = enabled {
            pipeline.enabled = Set(enabled);
        }
        if let Some(preset_key) = preset_key {
            pipeline.preset_key = Set(preset_key);
        }
        if let Some(category) = category {
            pipeline.category = Set(category);
        }

        let result = pipeline
            .update(connection)
            .await
            .map_err(|e| format!("Failed to update pipeline: {}", e))?;

        Ok(result)
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        let connection = self.db_manager.get_connection();

        PipelineEntity::delete_by_id(id)
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete pipeline: {}", e))?;

        Ok(())
    }
}
