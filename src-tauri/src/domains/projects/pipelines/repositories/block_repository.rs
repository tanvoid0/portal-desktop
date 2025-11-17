use crate::database::DatabaseManager;
use crate::entities::block::{Entity as BlockEntity, ActiveModel as BlockActiveModel, Model as BlockModel};
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
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
        
        let result = block.insert(connection)
            .await
            .map_err(|e| format!("Failed to create block: {}", e))?;
        
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

