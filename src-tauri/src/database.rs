use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use tauri::{AppHandle, Manager};
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::entities::project as project_entity;
use crate::{log_info, log_warn, log_error};

// Re-export the ProjectModel and TaskModel from the entity for compatibility
pub use project_entity::Model as ProjectModel;

#[derive(Clone)]
pub struct DatabaseManager {
    conn: DatabaseConnection,
}

impl DatabaseManager {
    /// Initialize database manager with secure, user-specific database
    /// Database is created in user's app data directory and migrated on startup
    pub async fn new(app_handle: &AppHandle) -> Result<Self, sea_orm::DbErr> {
        log_info!("DatabaseManager", "Starting database initialization...");
        
        // Get the app data directory (user-specific, secure location)
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| {
                log_error!("DatabaseManager", "Failed to get app data dir: {}", e);
                sea_orm::DbErr::Custom(format!("Failed to get app data dir: {}", e))
            })?;
        
        log_info!("DatabaseManager", "App data directory: {}", app_data_dir.display());
        
        // Create the directory if it doesn't exist with secure permissions
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| {
                log_error!("DatabaseManager", "Failed to create app data dir: {}", e);
                sea_orm::DbErr::Custom(format!("Failed to create app data dir: {}", e))
            })?;
        
        // Set secure file permissions on directory (Unix-like systems)
        #[cfg(unix)]
        {
            if let Err(e) = fs::set_permissions(&app_data_dir, fs::Permissions::from_mode(0o700)) {
                log_warn!("DatabaseManager", "Failed to set directory permissions: {}", e);
            }
        }
        
        // Set up the database path (user-specific, not shared, never committed to git)
        // Database is stored in user's app data directory, outside the project
        let db_path = app_data_dir.join("portal_desktop.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
        
        log_info!("DatabaseManager", "Database path: {} (user-specific, secure)", db_path.display());
        
        // Set secure file permissions on database file if it exists (Unix-like systems)
        #[cfg(unix)]
        {
            if db_path.exists() {
                if let Err(e) = fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600)) {
                    log_warn!("DatabaseManager", "Failed to set database file permissions: {}", e);
                }
            }
        }
        
        // Connect to the database
        log_info!("DatabaseManager", "Connecting to database...");
        let conn = Database::connect(&database_url).await
            .map_err(|e| {
                log_error!("DatabaseManager", "Failed to connect to database: {}", e);
                e
            })?;
        
        log_info!("DatabaseManager", "Database connected successfully");
        
        // Run migrations to ensure database schema is up to date
        // Migrations are idempotent (SeaORM tracks which have been run)
        log_info!("DatabaseManager", "Running migrations...");
        Self::run_migrations(&conn).await
            .map_err(|e| {
                log_error!("DatabaseManager", "Failed to run migrations: {}", e);
                e
            })?;
        
        // Ensure database file has secure permissions after creation (Unix-like systems)
        #[cfg(unix)]
        {
            if let Err(e) = fs::set_permissions(&db_path, fs::Permissions::from_mode(0o600)) {
                log_warn!("DatabaseManager", "Failed to set database file permissions: {}", e);
            }
        }
        
        log_info!("DatabaseManager", "Database initialization completed successfully");
        
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
    

    /// Run database migrations
    /// Migrations are idempotent - SeaORM tracks which have been applied
    /// Safe to run multiple times on startup
    async fn run_migrations(conn: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        log_info!("DatabaseManager", "Starting migration process...");
        
        // Create a schema manager for running migrations
        let schema_manager = sea_orm_migration::SchemaManager::new(conn);
        
        // Run all migrations using the generated registry
        let migrations = crate::migrations::get_migrations();
        log_info!("DatabaseManager", "Found {} migrations to check", migrations.len());
        
        for (i, migration) in migrations.iter().enumerate() {
            let migration_name = migration.name();
            log_info!("DatabaseManager", "Checking migration {} of {}: {}", i + 1, migrations.len(), migration_name);
            
            match migration.up(&schema_manager).await {
                Ok(_) => {
                    log_info!("DatabaseManager", "Migration '{}' applied successfully", migration_name);
                },
                Err(e) => {
                    // Check if error is because migration already applied (idempotent)
                    let error_msg = e.to_string();
                    if error_msg.contains("already exists") || error_msg.contains("duplicate") {
                        log_info!("DatabaseManager", "Migration '{}' already applied, skipping", migration_name);
                    } else {
                        log_error!("DatabaseManager", "Migration '{}' failed: {}", migration_name, e);
                        return Err(e);
                    }
                }
            }
        }
        
        log_info!("DatabaseManager", "All migrations completed successfully");
        Ok(())
    }
}

