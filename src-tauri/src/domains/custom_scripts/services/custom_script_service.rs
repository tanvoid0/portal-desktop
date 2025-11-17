use crate::domains::custom_scripts::repositories::CustomScriptRepository;
use crate::entities::custom_script::Model as CustomScriptModel;
use std::sync::Arc;
use crate::database::DatabaseManager;

pub struct CustomScriptService {
    repository: CustomScriptRepository,
}

impl CustomScriptService {
    pub fn new(db_manager: &Arc<DatabaseManager>) -> Self {
        Self {
            repository: CustomScriptRepository::new(db_manager.clone()),
        }
    }

    pub async fn get_all_scripts(&self) -> Result<Vec<CustomScriptModel>, String> {
        self.repository.get_all().await
    }

    pub async fn get_script(&self, id: i32) -> Result<Option<CustomScriptModel>, String> {
        self.repository.get_by_id(id).await
    }

    pub async fn create_script(
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
        self.repository.create(
            name,
            description,
            command,
            parameters_json,
            category,
            icon,
            requires_sudo,
            is_interactive,
        ).await
    }

    pub async fn update_script(
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
        self.repository.update(
            id,
            name,
            description,
            command,
            parameters_json,
            category,
            icon,
            requires_sudo,
            is_interactive,
        ).await
    }

    pub async fn delete_script(&self, id: i32) -> Result<(), String> {
        self.repository.delete(id).await
    }

    pub async fn record_script_run(&self, id: i32) -> Result<CustomScriptModel, String> {
        self.repository.increment_run_count(id).await
    }
}

