use sea_orm_migration::prelude::*;

pub mod m20240101_000001_create_projects_table;
pub mod m20240101_000002_create_tasks_table;
pub mod m20240101_000004_create_task_comments_table;
pub mod m20240101_000005_create_task_attachments_table;
pub mod m20240101_000006_create_task_templates_table;
pub mod m20240101_000007_create_saved_views_table;
pub mod m20240101_000008_create_frameworks_table;
pub mod m20240101_000009_create_ides_table;
pub mod m20240101_000010_create_framework_ide_mappings_table;

// Re-export all migrations for easy access
pub use m20240101_000001_create_projects_table::Migration as createProjectsTable;
pub use m20240101_000002_create_tasks_table::Migration as createTasksTable;
pub use m20240101_000004_create_task_comments_table::Migration as createTaskCommentsTable;
pub use m20240101_000005_create_task_attachments_table::Migration as createTaskAttachmentsTable;
pub use m20240101_000006_create_task_templates_table::Migration as createTaskTemplatesTable;
pub use m20240101_000007_create_saved_views_table::Migration as createSavedViewsTable;
pub use m20240101_000008_create_frameworks_table::Migration as createFrameworksTable;
pub use m20240101_000009_create_ides_table::Migration as createIdesTable;
pub use m20240101_000010_create_framework_ide_mappings_table::Migration as createFrameworkIdeMappingsTable;

// Create a function to get all migrations
pub fn get_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(createProjectsTable),
        Box::new(createTasksTable),
        Box::new(createTaskCommentsTable),
        Box::new(createTaskAttachmentsTable),
        Box::new(createTaskTemplatesTable),
        Box::new(createSavedViewsTable),
        Box::new(createFrameworksTable),
        Box::new(createIdesTable),
        Box::new(createFrameworkIdeMappingsTable),
    ]
}
