use sea_orm_migration::prelude::*;

pub mod m20240101_000001_create_frameworks_table;
pub mod m20240101_000002_create_package_managers_table;
pub mod m20240101_000003_create_languages_table;
pub mod m20240101_000004_create_ides_table;
pub mod m20240101_000005_create_projects_table;
pub mod m20240101_000006_create_framework_ide_mappings_table;
pub mod m20240101_000007_create_tasks_table;
pub mod m20240101_000008_create_task_comments_table;
pub mod m20240101_000009_create_task_attachments_table;
pub mod m20240101_000010_create_task_templates_table;
pub mod m20240101_000011_create_pipelines_table;
pub mod m20240101_000012_create_pipeline_executions_table;
pub mod m20240101_000013_create_saved_views_table;
pub mod m20240101_000014_create_learning_tables;
pub mod m20240101_000015_create_blocks_table;
pub mod m20240101_000016_create_credentials_table;
pub mod m20240101_000017_create_documents_table;
pub mod m20240101_000018_create_ai_tables;
pub mod m20240101_000019_create_custom_scripts_table;
pub mod m20240101_000020_create_project_frameworks_table;
pub mod m20240101_000021_create_project_languages_table;
pub mod m20240101_000022_create_project_package_managers_table;
pub mod m20240101_000023_create_device_approvals_table;
pub mod m20240101_000024_create_sdk_tables;
pub mod m20240101_000025_create_script_executions_table;
pub mod m20260325_000026_create_terminal_command_history_table;
pub mod m20260325_000027_create_terminal_sessions_table;
pub mod m20260325_000028_create_terminal_notes_table;
pub mod m20260325_000029_add_preset_key_to_pipelines;
pub mod m20260330_000030_create_deployments_table;
pub mod m20260706_000031_create_coder_tables;
pub mod m20260706_000032_create_coder_file_changes_table;
pub mod runner;

// Re-export all migrations for easy access
pub use m20240101_000001_create_frameworks_table::Migration as createFrameworksTable;
pub use m20240101_000002_create_package_managers_table::Migration as createPackageManagersTable;
pub use m20240101_000003_create_languages_table::Migration as createLanguagesTable;
pub use m20240101_000004_create_ides_table::Migration as createIdesTable;
pub use m20240101_000005_create_projects_table::Migration as createProjectsTable;
pub use m20240101_000006_create_framework_ide_mappings_table::Migration as createFrameworkIdeMappingsTable;
pub use m20240101_000007_create_tasks_table::Migration as createTasksTable;
pub use m20240101_000008_create_task_comments_table::Migration as createTaskCommentsTable;
pub use m20240101_000009_create_task_attachments_table::Migration as createTaskAttachmentsTable;
pub use m20240101_000010_create_task_templates_table::Migration as createTaskTemplatesTable;
pub use m20240101_000011_create_pipelines_table::Migration as createPipelinesTable;
pub use m20240101_000012_create_pipeline_executions_table::Migration as createPipelineExecutionsTable;
pub use m20240101_000013_create_saved_views_table::Migration as createSavedViewsTable;
pub use m20240101_000014_create_learning_tables::Migration as createLearningTables;
pub use m20240101_000015_create_blocks_table::Migration as createBlocksTable;
pub use m20240101_000016_create_credentials_table::Migration as createCredentialsTable;
pub use m20240101_000017_create_documents_table::Migration as createDocumentsTable;
pub use m20240101_000018_create_ai_tables::Migration as createAiTables;
pub use m20240101_000019_create_custom_scripts_table::Migration as createCustomScriptsTable;
pub use m20240101_000020_create_project_frameworks_table::Migration as createProjectFrameworksTable;
pub use m20240101_000021_create_project_languages_table::Migration as createProjectLanguagesTable;
pub use m20240101_000022_create_project_package_managers_table::Migration as createProjectPackageManagersTable;
pub use m20240101_000023_create_device_approvals_table::Migration as createDeviceApprovalsTable;
pub use m20240101_000024_create_sdk_tables::Migration as createSdkTables;
pub use m20240101_000025_create_script_executions_table::Migration as createScriptExecutionsTable;
pub use m20260325_000026_create_terminal_command_history_table::Migration
    as createTerminalCommandHistoryTable;
pub use m20260325_000027_create_terminal_sessions_table::Migration
    as createTerminalSessionsTable;
pub use m20260325_000028_create_terminal_notes_table::Migration
    as createTerminalNotesTable;
pub use m20260325_000029_add_preset_key_to_pipelines::Migration
    as addPresetKeyToPipelines;
pub use m20260330_000030_create_deployments_table::Migration
    as createDeploymentsTable;
pub use m20260706_000031_create_coder_tables::Migration as createCoderTables;
pub use m20260706_000032_create_coder_file_changes_table::Migration
    as createCoderFileChangesTable;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        get_migrations()
    }
}

// Create a function to get all migrations
// Organized by logical dependency order:
// 1. Base entity tables (frameworks, package_managers, languages, ides)
// 2. Projects (with foreign key columns, but no constraints yet)
// 3. Framework mappings (depends on frameworks and ides)
// 4. Tasks (self-referential)
// 5. Task-related tables (depend on tasks)
// 6. Pipelines (depends on projects)
// 7. Pipeline executions (depends on projects and pipelines)
// 8. Independent tables
// 9. Junction tables for many-to-many relationships
pub fn get_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        // 1. Base entity tables (no dependencies)
        Box::new(createFrameworksTable),
        Box::new(createPackageManagersTable),
        Box::new(createLanguagesTable),
        Box::new(createIdesTable),
        // 2. Projects (with foreign key columns and constraints)
        Box::new(createProjectsTable),
        // 3. Framework mappings (depends on frameworks and ides)
        Box::new(createFrameworkIdeMappingsTable),
        // 4. Tasks (self-referential)
        Box::new(createTasksTable),
        // 5. Task-related tables (depend on tasks)
        Box::new(createTaskCommentsTable),
        Box::new(createTaskAttachmentsTable),
        Box::new(createTaskTemplatesTable),
        // 6. Pipelines (depends on projects)
        Box::new(createPipelinesTable),
        // 7. Pipeline executions (depends on projects and pipelines)
        Box::new(createPipelineExecutionsTable),
        // 8. Independent tables (no foreign key dependencies)
        Box::new(createSavedViewsTable),
        Box::new(createLearningTables), // Now includes is_important flags
        Box::new(createBlocksTable),
        Box::new(createCredentialsTable),
        Box::new(createDocumentsTable),
        Box::new(createAiTables),
        Box::new(createCustomScriptsTable),
        // 9. Junction tables for many-to-many relationships (depends on projects and entity tables)
        Box::new(createProjectFrameworksTable),
        Box::new(createProjectLanguagesTable),
        Box::new(createProjectPackageManagersTable),
        // 10. Device approvals table (for browser authentication)
        Box::new(createDeviceApprovalsTable),
        // 11. SDK management tables
        Box::new(createSdkTables),
        // 12. Script executions table (depends on blocks)
        Box::new(createScriptExecutionsTable),
        // Terminal command history (independent)
        Box::new(createTerminalCommandHistoryTable),
        // Terminal sessions + notes (independent)
        Box::new(createTerminalSessionsTable),
        Box::new(createTerminalNotesTable),
        Box::new(addPresetKeyToPipelines),
        Box::new(createDeploymentsTable),
        Box::new(createCoderTables),
        Box::new(createCoderFileChangesTable),
    ]
}
