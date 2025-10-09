use sea_orm_migration::prelude::*;

pub mod m20240101_000001_create_projects_table;
pub mod m20240101_000002_create_tasks_table;

// Re-export all migrations for easy access
pub use m20240101_000001_create_projects_table::Migration as createProjectsTable;
pub use m20240101_000002_create_tasks_table::Migration as createTasksTable;

// Create a function to get all migrations
pub fn get_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(createProjectsTable),
        Box::new(createTasksTable),
    ]
}
