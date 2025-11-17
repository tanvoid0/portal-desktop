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
pub mod m20240101_000011_create_learning_tables;
pub mod m20240101_000012_add_important_flags;
pub mod m20240101_000013_create_pipelines_table;
pub mod m20240101_000014_create_blocks_table;
pub mod m20240101_000015_create_pipeline_executions_table;
pub mod m20240101_000016_create_credentials_table;
pub mod m20240101_000017_create_documents_table;
pub mod m20240101_000018_create_ai_tables;
pub mod m20240101_000019_create_custom_scripts_table;

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
pub use m20240101_000011_create_learning_tables::Migration as createLearningTables;
pub use m20240101_000012_add_important_flags::Migration as addImportantFlags;
pub use m20240101_000013_create_pipelines_table::Migration as createPipelinesTable;
pub use m20240101_000014_create_blocks_table::Migration as createBlocksTable;
pub use m20240101_000015_create_pipeline_executions_table::Migration as createPipelineExecutionsTable;
pub use m20240101_000016_create_credentials_table::Migration as createCredentialsTable;
pub use m20240101_000017_create_documents_table::Migration as createDocumentsTable;
pub use m20240101_000018_create_ai_tables::Migration as createAiTables;
pub use m20240101_000019_create_custom_scripts_table::Migration as createCustomScriptsTable;

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
        Box::new(createLearningTables),
        Box::new(addImportantFlags),
        Box::new(createPipelinesTable),
        Box::new(createBlocksTable),
        Box::new(createPipelineExecutionsTable),
        Box::new(createCredentialsTable),
        Box::new(createDocumentsTable),
        Box::new(createAiTables),
        Box::new(createCustomScriptsTable),
    ]
}
