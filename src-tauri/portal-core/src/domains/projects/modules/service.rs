//! Caches [`detect::scan`] results in `project_modules`.

use super::detect::{self, DetectedModule};
use super::stack;
use crate::database::DatabaseManager;
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use crate::domains::projects::services::project_service::ProjectService;
use crate::entities::project_module::{ActiveModel, Column, Entity, Model};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::path::PathBuf;
use std::sync::Arc;

pub struct ModuleService {
    db: Arc<DatabaseManager>,
    projects: ProjectRepository,
}

impl ModuleService {
    pub fn new(db: &Arc<DatabaseManager>) -> Self {
        Self {
            db: db.clone(),
            projects: ProjectRepository::new(db.clone()),
        }
    }

    pub async fn list(&self, project_id: i32) -> Result<Vec<Model>, String> {
        Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .all(self.db.get_connection())
            .await
            .map_err(|e| format!("Failed to load modules: {e}"))
    }

    /// Every project's modules at once, for the list page.
    pub async fn list_all(&self) -> Result<Vec<Model>, String> {
        Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| format!("Failed to load modules: {e}"))
    }

    /// Rescan one project from disk and replace its rows. A missing folder
    /// clears the rows rather than failing, so a moved project just shows none.
    /// A project with no stack recorded (every discovered one, until now) gets
    /// it filled from the same scan; one that has a stack is left alone.
    pub async fn refresh(&self, project_id: i32) -> Result<Vec<Model>, String> {
        let project = self
            .projects
            .get_by_id(project_id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        let root = PathBuf::from(&project.path);
        let needs_stack = !project.has_stack();
        let (found, detected) = tokio::task::spawn_blocking(move || {
            let found: Vec<DetectedModule> = detect::scan(&root);
            let detected = needs_stack.then(|| stack::detect_with(&root, &found));
            (found, detected)
        })
        .await
        .map_err(|e| e.to_string())?;
        self.replace(project_id, found).await?;
        if let Some(detected) = detected {
            ProjectService::new(&self.db)
                .store_stack(&project, &detected, false)
                .await?;
        }
        self.list(project_id).await
    }

    /// Rescan every project. Sequential: it runs in the background on the list
    /// page and must not compete with the UI for the disk.
    pub async fn refresh_all(&self) -> Result<Vec<Model>, String> {
        for project in self.projects.get_all().await? {
            let _ = self.refresh(project.id).await;
        }
        self.list_all().await
    }

    async fn replace(&self, project_id: i32, found: Vec<DetectedModule>) -> Result<(), String> {
        let conn = self.db.get_connection();
        Entity::delete_many()
            .filter(Column::ProjectId.eq(project_id))
            .exec(conn)
            .await
            .map_err(|e| format!("Failed to clear modules: {e}"))?;
        for m in found {
            ActiveModel {
                project_id: Set(project_id),
                rel_path: Set(m.rel_path),
                name: Set(m.name),
                stack: Set(m.stack),
                role: Set(m.role),
                ..Default::default()
            }
            .insert(conn)
            .await
            .map_err(|e| format!("Failed to save module: {e}"))?;
        }
        Ok(())
    }
}
