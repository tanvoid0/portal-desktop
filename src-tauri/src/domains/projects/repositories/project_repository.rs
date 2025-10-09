use crate::database::{DatabaseManager, ProjectModel};
use crate::entities::project::{Entity as ProjectEntity, ActiveModel as ProjectActiveModel};
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use std::sync::Arc;

pub struct ProjectRepository {
    db_manager: Arc<DatabaseManager>,
}

impl ProjectRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_all(&self) -> Result<Vec<ProjectModel>, String> {
        let connection = self.db_manager.get_connection();
        let projects = ProjectEntity::find()
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch projects: {}", e))?;
        Ok(projects)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<ProjectModel>, String> {
        let connection = self.db_manager.get_connection();
        let project = ProjectEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch project: {}", e))?;
        Ok(project)
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        path: String,
        framework: Option<String>,
        package_manager: Option<String>,
        build_command: Option<String>,
        start_command: Option<String>,
        test_command: Option<String>,
        output_directory: Option<String>,
        dev_port: Option<i32>,
        prod_port: Option<i32>,
    ) -> Result<ProjectModel, String> {
        let connection = self.db_manager.get_connection();
        
        let project = ProjectActiveModel {
            name: Set(name),
            description: Set(description),
            path: Set(path),
            status: Set("active".to_string()),
            framework: Set(framework),
            package_manager: Set(package_manager),
            build_command: Set(build_command),
            start_command: Set(start_command),
            test_command: Set(test_command),
            output_directory: Set(output_directory),
            dev_port: Set(dev_port),
            prod_port: Set(prod_port),
            starred: Set(false),
            open_count: Set(0),
            last_opened: Set(None),
            size: Set(0),
            file_count: Set(0),
            git_repository: Set(None),
            git_branch: Set(None),
            git_commit: Set(None),
            has_uncommitted_changes: Set(false),
            last_commit: Set(None),
            created_at: Set(None), // Will be set by database
            updated_at: Set(None), // Will be set by database
            ..Default::default()
        };
        
        let result = project.insert(connection).await
            .map_err(|e| format!("Failed to create project: {}", e))?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        path: Option<String>,
        status: Option<String>,
        framework: Option<String>,
        package_manager: Option<String>,
        build_command: Option<String>,
        start_command: Option<String>,
        test_command: Option<String>,
        output_directory: Option<String>,
        dev_port: Option<i32>,
        prod_port: Option<i32>,
        starred: Option<bool>,
        open_count: Option<i32>,
        last_opened: Option<chrono::DateTime<chrono::Utc>>,
        size: Option<i64>,
        file_count: Option<i32>,
        git_repository: Option<String>,
        git_branch: Option<String>,
        git_commit: Option<String>,
        has_uncommitted_changes: Option<bool>,
        last_commit: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<ProjectModel>, String> {
        let connection = self.db_manager.get_connection();
        
        let mut project: ProjectActiveModel = ProjectEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to find project: {}", e))?
            .ok_or_else(|| "Project not found".to_string())?
            .into();

        // Update fields if provided
        if let Some(name) = name {
            project.name = Set(name);
        }
        if let Some(description) = description {
            project.description = Set(Some(description));
        }
        if let Some(path) = path {
            project.path = Set(path);
        }
        if let Some(status) = status {
            project.status = Set(status);
        }
        if let Some(framework) = framework {
            project.framework = Set(Some(framework));
        }
        if let Some(package_manager) = package_manager {
            project.package_manager = Set(Some(package_manager));
        }
        if let Some(build_command) = build_command {
            project.build_command = Set(Some(build_command));
        }
        if let Some(start_command) = start_command {
            project.start_command = Set(Some(start_command));
        }
        if let Some(test_command) = test_command {
            project.test_command = Set(Some(test_command));
        }
        if let Some(output_directory) = output_directory {
            project.output_directory = Set(Some(output_directory));
        }
        if let Some(dev_port) = dev_port {
            project.dev_port = Set(Some(dev_port));
        }
        if let Some(prod_port) = prod_port {
            project.prod_port = Set(Some(prod_port));
        }
        if let Some(starred) = starred {
            project.starred = Set(starred);
        }
        if let Some(open_count) = open_count {
            project.open_count = Set(open_count);
        }
        if let Some(last_opened) = last_opened {
            project.last_opened = Set(Some(last_opened.into()));
        }
        if let Some(size) = size {
            project.size = Set(size);
        }
        if let Some(file_count) = file_count {
            project.file_count = Set(file_count);
        }
        if let Some(git_repository) = git_repository {
            project.git_repository = Set(Some(git_repository));
        }
        if let Some(git_branch) = git_branch {
            project.git_branch = Set(Some(git_branch));
        }
        if let Some(git_commit) = git_commit {
            project.git_commit = Set(Some(git_commit));
        }
        if let Some(has_uncommitted_changes) = has_uncommitted_changes {
            project.has_uncommitted_changes = Set(has_uncommitted_changes);
        }
        if let Some(last_commit) = last_commit {
            project.last_commit = Set(Some(last_commit.into()));
        }

        let result = project.update(connection).await
            .map_err(|e| format!("Failed to update project: {}", e))?;
        Ok(Some(result))
    }

    pub async fn delete(&self, id: i32) -> Result<bool, String> {
        let connection = self.db_manager.get_connection();
        let result = ProjectEntity::delete_by_id(id)
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete project: {}", e))?;
        Ok(result.rows_affected > 0)
    }

}