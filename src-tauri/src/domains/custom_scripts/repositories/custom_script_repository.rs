use crate::database::DatabaseManager;
use crate::entities::custom_script::{Entity as CustomScriptEntity, ActiveModel as CustomScriptActiveModel, Model as CustomScriptModel};
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use std::sync::Arc;

pub struct CustomScriptRepository {
    db_manager: Arc<DatabaseManager>,
}

impl CustomScriptRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_all(&self) -> Result<Vec<CustomScriptModel>, String> {
        let connection = self.db_manager.get_connection();
        let scripts = CustomScriptEntity::find()
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch custom scripts: {}", e))?;
        Ok(scripts)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<CustomScriptModel>, String> {
        let connection = self.db_manager.get_connection();
        let script = CustomScriptEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch custom script: {}", e))?;
        Ok(script)
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        command: String,
        parameters_json: String,
        category: Option<String>,
        icon: Option<String>,
        requires_sudo: bool,
        is_interactive: bool,
    ) -> Result<CustomScriptModel, String> {
        let connection = self.db_manager.get_connection();
        
        let script = CustomScriptActiveModel {
            name: Set(name),
            description: Set(description),
            command: Set(command),
            parameters_json: Set(parameters_json),
            category: Set(category),
            icon: Set(icon),
            requires_sudo: Set(requires_sudo),
            is_interactive: Set(is_interactive),
            created_at: Set(None),
            updated_at: Set(None),
            last_run_at: Set(None),
            run_count: Set(0),
            ..Default::default()
        };
        
        let result = script.insert(connection).await
            .map_err(|e| format!("Failed to create custom script: {}", e))?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        command: Option<String>,
        parameters_json: Option<String>,
        category: Option<String>,
        icon: Option<String>,
        requires_sudo: Option<bool>,
        is_interactive: Option<bool>,
    ) -> Result<CustomScriptModel, String> {
        let connection = self.db_manager.get_connection();
        
        let mut script: CustomScriptActiveModel = CustomScriptEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find custom script: {}", e))?
            .ok_or_else(|| "Custom script not found".to_string())?
            .into();
        
        if let Some(name) = name {
            script.name = Set(name);
        }
        if let Some(description) = description {
            script.description = Set(Some(description));
        }
        if let Some(command) = command {
            script.command = Set(command);
        }
        if let Some(parameters_json) = parameters_json {
            script.parameters_json = Set(parameters_json);
        }
        if let Some(category) = category {
            script.category = Set(Some(category));
        }
        if let Some(icon) = icon {
            script.icon = Set(Some(icon));
        }
        if let Some(requires_sudo) = requires_sudo {
            script.requires_sudo = Set(requires_sudo);
        }
        if let Some(is_interactive) = is_interactive {
            script.is_interactive = Set(is_interactive);
        }
        script.updated_at = Set(Some(chrono::Utc::now().into()));
        
        let result = script.update(connection).await
            .map_err(|e| format!("Failed to update custom script: {}", e))?;
        Ok(result)
    }

    pub async fn delete(&self, id: i32) -> Result<(), String> {
        let connection = self.db_manager.get_connection();
        CustomScriptEntity::delete_by_id(id)
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete custom script: {}", e))?;
        Ok(())
    }

    pub async fn increment_run_count(&self, id: i32) -> Result<CustomScriptModel, String> {
        let connection = self.db_manager.get_connection();
        
        let mut script: CustomScriptActiveModel = CustomScriptEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find custom script: {}", e))?
            .ok_or_else(|| "Custom script not found".to_string())?
            .into();
        
        script.run_count = Set(script.run_count.as_ref() + 1);
        script.last_run_at = Set(Some(chrono::Utc::now().into()));
        
        let result = script.update(connection).await
            .map_err(|e| format!("Failed to update custom script: {}", e))?;
        Ok(result)
    }
}

