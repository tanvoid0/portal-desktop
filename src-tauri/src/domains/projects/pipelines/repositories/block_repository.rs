use crate::database::DatabaseManager;
use crate::entities::block::{
    ActiveModel as BlockActiveModel, Entity as BlockEntity, Model as BlockModel,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::sync::Arc;

pub struct BlockRepository {
    db_manager: Arc<DatabaseManager>,
}

impl BlockRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_all(&self) -> Result<Vec<BlockModel>, String> {
        let connection = self.db_manager.get_connection();
        let blocks = BlockEntity::find()
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch blocks: {}", e))?;
        Ok(blocks)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<BlockModel>, String> {
        let connection = self.db_manager.get_connection();
        let block = BlockEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch block: {}", e))?;
        Ok(block)
    }

    pub async fn create(
        &self,
        id: String,
        name: String,
        description: String,
        category: String,
        version: String,
        parameters_json: String,
        command: String,
        execution_type: String,
        default_config_json: String,
        tags_json: String,
        icon: Option<String>,
        author: Option<String>,
    ) -> Result<BlockModel, String> {
        let connection = self.db_manager.get_connection();

        let block = BlockActiveModel {
            id: Set(id),
            name: Set(name),
            description: Set(description),
            category: Set(category),
            version: Set(version),
            parameters_json: Set(parameters_json),
            command: Set(command),
            execution_type: Set(execution_type),
            default_config_json: Set(default_config_json),
            tags_json: Set(tags_json),
            icon: Set(icon),
            author: Set(author),
            ..Default::default()
        };

        let result = block
            .insert(connection)
            .await
            .map_err(|e| format!("Failed to create block: {}", e))?;

        Ok(result)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        category: Option<String>,
        parameters_json: Option<String>,
        command: Option<String>,
        execution_type: Option<String>,
        default_config_json: Option<String>,
        tags_json: Option<String>,
        icon: Option<String>,
        author: Option<String>,
    ) -> Result<BlockModel, String> {
        let connection = self.db_manager.get_connection();

        let existing = BlockEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch block: {}", e))?
            .ok_or_else(|| format!("Block not found: {}", id))?;

        let mut active: BlockActiveModel = existing.into();

        if let Some(v) = name {
            active.name = Set(v);
        }
        if let Some(v) = description {
            active.description = Set(v);
        }
        if let Some(v) = category {
            active.category = Set(v);
        }
        if let Some(v) = parameters_json {
            active.parameters_json = Set(v);
        }
        if let Some(v) = command {
            active.command = Set(v);
        }
        if let Some(v) = execution_type {
            active.execution_type = Set(v);
        }
        if let Some(v) = default_config_json {
            active.default_config_json = Set(v);
        }
        if let Some(v) = tags_json {
            active.tags_json = Set(v);
        }
        if let Some(v) = icon {
            active.icon = Set(Some(v));
        }
        if let Some(v) = author {
            active.author = Set(Some(v));
        }

        let result = active
            .update(connection)
            .await
            .map_err(|e| format!("Failed to update block: {}", e))?;

        Ok(result)
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let connection = self.db_manager.get_connection();

        BlockEntity::delete_by_id(id)
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete block: {}", e))?;

        Ok(())
    }
}
