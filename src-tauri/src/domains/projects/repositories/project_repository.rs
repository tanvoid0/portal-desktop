use crate::database::{DatabaseManager, ProjectModel};
use crate::domains::projects::entities::ProjectResponse;
use crate::entities::project::{ActiveModel as ProjectActiveModel, Entity as ProjectEntity};
use crate::entities::project_framework::{
    ActiveModel as ProjectFrameworkActiveModel, Column as ProjectFrameworkColumn,
    Entity as ProjectFrameworkEntity,
};
use crate::entities::project_language::{
    ActiveModel as ProjectLanguageActiveModel, Column as ProjectLanguageColumn,
    Entity as ProjectLanguageEntity,
};
use crate::entities::project_package_manager::{
    ActiveModel as ProjectPackageManagerActiveModel, Column as ProjectPackageManagerColumn,
    Entity as ProjectPackageManagerEntity,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

fn junction_created_at() -> sea_orm::prelude::DateTimeWithTimeZone {
    chrono::Utc::now().into()
}

#[derive(Clone)]
pub struct ProjectRepository {
    db_manager: Arc<DatabaseManager>,
}

impl ProjectRepository {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn get_all(&self) -> Result<Vec<ProjectResponse>, String> {
        let connection = self.db_manager.get_connection();
        let projects = ProjectEntity::find()
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch projects: {}", e))?;

        let mut responses = Vec::with_capacity(projects.len());
        for project in projects {
            responses.push(self.to_response(project).await?);
        }

        Ok(responses)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<ProjectResponse>, String> {
        let connection = self.db_manager.get_connection();
        let project = ProjectEntity::find_by_id(id)
            .one(connection)
            .await
            .map_err(|e| format!("Failed to fetch project: {}", e))?;

        match project {
            Some(project) => Ok(Some(self.to_response(project).await?)),
            None => Ok(None),
        }
    }

    async fn to_response(&self, project: ProjectModel) -> Result<ProjectResponse, String> {
        let framework_ids = self.get_framework_ids(project.id).await?;
        let package_manager_ids = self.get_package_manager_ids(project.id).await?;
        let language_ids = self.get_language_ids(project.id).await?;

        Ok(ProjectResponse::from_model(
            project,
            framework_ids,
            package_manager_ids,
            language_ids,
        ))
    }

    async fn get_framework_ids(&self, project_id: i32) -> Result<Vec<i32>, String> {
        let connection = self.db_manager.get_connection();
        let rows = ProjectFrameworkEntity::find()
            .filter(ProjectFrameworkColumn::ProjectId.eq(project_id))
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch project frameworks: {}", e))?;

        Ok(rows.into_iter().map(|row| row.framework_id).collect())
    }

    async fn get_package_manager_ids(&self, project_id: i32) -> Result<Vec<i32>, String> {
        let connection = self.db_manager.get_connection();
        let rows = ProjectPackageManagerEntity::find()
            .filter(ProjectPackageManagerColumn::ProjectId.eq(project_id))
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch project package managers: {}", e))?;

        Ok(rows.into_iter().map(|row| row.package_manager_id).collect())
    }

    async fn get_language_ids(&self, project_id: i32) -> Result<Vec<i32>, String> {
        let connection = self.db_manager.get_connection();
        let rows = ProjectLanguageEntity::find()
            .filter(ProjectLanguageColumn::ProjectId.eq(project_id))
            .all(connection)
            .await
            .map_err(|e| format!("Failed to fetch project languages: {}", e))?;

        Ok(rows.into_iter().map(|row| row.language_id).collect())
    }

    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
        path: String,
        framework_ids: Vec<i32>,
        package_manager_ids: Vec<i32>,
        language_ids: Vec<i32>,
        build_command: Option<String>,
        start_command: Option<String>,
        test_command: Option<String>,
        output_directory: Option<String>,
        dev_port: Option<i32>,
        prod_port: Option<i32>,
    ) -> Result<ProjectResponse, String> {
        let connection = self.db_manager.get_connection();

        let project = ProjectActiveModel {
            name: Set(name),
            description: Set(description),
            path: Set(path),
            status: Set("active".to_string()),
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

        let result = project
            .insert(connection)
            .await
            .map_err(|e| format!("Failed to create project: {}", e))?;

        // Create junction table records
        self.set_project_frameworks(result.id, &framework_ids)
            .await?;
        self.set_project_languages(result.id, &language_ids).await?;
        self.set_project_package_managers(result.id, &package_manager_ids)
            .await?;

        self.to_response(result).await
    }

    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        path: Option<String>,
        status: Option<String>,
        framework_ids: Option<Vec<i32>>,
        package_manager_ids: Option<Vec<i32>>,
        language_ids: Option<Vec<i32>>,
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
    ) -> Result<Option<ProjectResponse>, String> {
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

        let result = project
            .update(connection)
            .await
            .map_err(|e| format!("Failed to update project: {}", e))?;

        // Update junction table records if provided
        if let Some(framework_ids) = framework_ids {
            self.set_project_frameworks(id, &framework_ids).await?;
        }
        if let Some(language_ids) = language_ids {
            self.set_project_languages(id, &language_ids).await?;
        }
        if let Some(package_manager_ids) = package_manager_ids {
            self.set_project_package_managers(id, &package_manager_ids)
                .await?;
        }

        Ok(Some(self.to_response(result).await?))
    }

    // Helper methods for managing many-to-many relationships
    async fn set_project_frameworks(
        &self,
        project_id: i32,
        framework_ids: &[i32],
    ) -> Result<(), String> {
        let connection = self.db_manager.get_connection();

        // Delete existing relationships
        ProjectFrameworkEntity::delete_many()
            .filter(ProjectFrameworkColumn::ProjectId.eq(project_id))
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete project frameworks: {}", e))?;

        // Insert new relationships
        for framework_id in framework_ids {
            let pf = ProjectFrameworkActiveModel {
                project_id: Set(project_id),
                framework_id: Set(*framework_id),
                created_at: Set(Some(junction_created_at())),
                ..Default::default()
            };
            pf.insert(connection)
                .await
                .map_err(|e| format!("Failed to create project framework: {}", e))?;
        }

        Ok(())
    }

    async fn set_project_languages(
        &self,
        project_id: i32,
        language_ids: &[i32],
    ) -> Result<(), String> {
        let connection = self.db_manager.get_connection();

        // Delete existing relationships
        ProjectLanguageEntity::delete_many()
            .filter(ProjectLanguageColumn::ProjectId.eq(project_id))
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete project languages: {}", e))?;

        // Insert new relationships
        for language_id in language_ids {
            let pl = ProjectLanguageActiveModel {
                project_id: Set(project_id),
                language_id: Set(*language_id),
                created_at: Set(Some(junction_created_at())),
                ..Default::default()
            };
            pl.insert(connection)
                .await
                .map_err(|e| format!("Failed to create project language: {}", e))?;
        }

        Ok(())
    }

    async fn set_project_package_managers(
        &self,
        project_id: i32,
        package_manager_ids: &[i32],
    ) -> Result<(), String> {
        let connection = self.db_manager.get_connection();

        // Delete existing relationships
        ProjectPackageManagerEntity::delete_many()
            .filter(ProjectPackageManagerColumn::ProjectId.eq(project_id))
            .exec(connection)
            .await
            .map_err(|e| format!("Failed to delete project package managers: {}", e))?;

        // Insert new relationships
        for package_manager_id in package_manager_ids {
            let ppm = ProjectPackageManagerActiveModel {
                project_id: Set(project_id),
                package_manager_id: Set(*package_manager_id),
                created_at: Set(Some(junction_created_at())),
                ..Default::default()
            };
            ppm.insert(connection)
                .await
                .map_err(|e| format!("Failed to create project package manager: {}", e))?;
        }

        Ok(())
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
