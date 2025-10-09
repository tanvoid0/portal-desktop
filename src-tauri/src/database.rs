use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tauri::{AppHandle, Manager};

use crate::entities::project as project_entity;
use crate::entities::task as task_entity;

// Re-export the ProjectModel and TaskModel from the entity for compatibility
pub use project_entity::Model as ProjectModel;
pub use task_entity::Model as TaskModel;

#[derive(Clone)]
pub struct DatabaseManager {
    conn: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, sea_orm::DbErr> {
        println!("[DatabaseManager] Starting database initialization...");
        
        // Get the app data directory
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| {
                println!("[DatabaseManager] Failed to get app data dir: {}", e);
                sea_orm::DbErr::Custom(format!("Failed to get app data dir: {}", e))
            })?;
        
        println!("[DatabaseManager] App data directory: {}", app_data_dir.display());
        
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| {
                println!("[DatabaseManager] Failed to create app data dir: {}", e);
                sea_orm::DbErr::Custom(format!("Failed to create app data dir: {}", e))
            })?;
        
        // Set up the database path
        let db_path = app_data_dir.join("portal_desktop.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
        
        println!("[DatabaseManager] Database path: {}", db_path.display());
        println!("[DatabaseManager] Database URL: {}", database_url);
        
        // Connect to the database
        println!("[DatabaseManager] Connecting to database...");
        let conn = Database::connect(&database_url).await
            .map_err(|e| {
                println!("[DatabaseManager] Failed to connect to database: {}", e);
                e
            })?;
        
        println!("[DatabaseManager] Database connected successfully");
        
        // Run migrations to ensure database schema is up to date
        println!("[DatabaseManager] Running migrations...");
        Self::run_migrations(&conn).await
            .map_err(|e| {
                println!("[DatabaseManager] Failed to run migrations: {}", e);
                e
            })?;
        
        println!("[DatabaseManager] Migrations completed successfully");
        println!("[DatabaseManager] Database initialization completed");
        
        Ok(DatabaseManager { conn })
    }
    
    /// Get a reference to the database connection
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }
    
    /// Get a clone of the database connection for repositories
    pub fn get_connection_clone(&self) -> DatabaseConnection {
        self.conn.clone()
    }
    

    async fn run_migrations(conn: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        // Run all migrations using the generated registry
        let migrations = crate::migrations::get_migrations();
        
        // Create a schema manager for running migrations
        let schema_manager = sea_orm_migration::SchemaManager::new(conn);
        
        for migration in migrations {
            migration.up(&schema_manager).await?;
        }
        Ok(())
    }
}

