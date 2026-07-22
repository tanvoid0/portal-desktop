use sea_orm::{Database, DatabaseConnection};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::entities::project as project_entity;
use crate::migrations::runner::run_migrations;
use crate::{log_error, log_info, log_warn};

pub use project_entity::Model as ProjectModel;

#[derive(Clone)]
pub struct DatabaseManager {
    conn: DatabaseConnection,
}

impl DatabaseManager {
    /// Initialize database in the given application data directory.
    pub async fn new(data_dir: PathBuf) -> Result<Self, sea_orm::DbErr> {
        log_info!("DatabaseManager", "Starting database initialization...");

        std::fs::create_dir_all(&data_dir).map_err(|e| {
            log_error!("DatabaseManager", "Failed to create data dir: {}", e);
            sea_orm::DbErr::Custom(format!("Failed to create data dir: {}", e))
        })?;

        Self::set_dir_permissions(&data_dir);

        // Migrate legacy database from CWD-relative path if present
        Self::migrate_legacy_database(&data_dir);

        let db_path = data_dir.join("portal_desktop.db");
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

        log_info!("DatabaseManager", "Database path: {}", db_path.display());

        log_info!("DatabaseManager", "Connecting to database...");
        let conn = Database::connect(&database_url).await.map_err(|e| {
            log_error!("DatabaseManager", "Failed to connect to database: {}", e);
            e
        })?;

        log_info!("DatabaseManager", "Running migrations...");
        run_migrations(&conn).await.map_err(|e| {
            log_error!("DatabaseManager", "Failed to run migrations: {}", e);
            e
        })?;

        Self::set_file_permissions(&db_path);

        log_info!(
            "DatabaseManager",
            "Database initialization completed successfully"
        );

        Ok(DatabaseManager { conn })
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub fn get_connection_clone(&self) -> DatabaseConnection {
        self.conn.clone()
    }

    fn migrate_legacy_database(data_dir: &Path) {
        let new_db = data_dir.join("portal_desktop.db");
        if new_db.exists() {
            return;
        }

        // Data dir of the pre-rename identifier. Copy the whole directory so the
        // disk-utility DB and any sibling state come across with the main DB.
        if let Some(legacy_dir) =
            dirs::data_dir().map(|d| d.join(crate::app_paths::LEGACY_APP_IDENTIFIER))
        {
            if legacy_dir.join("portal_desktop.db").exists() {
                crate::app_paths::copy_files_into(&legacy_dir, data_dir);
                return;
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            let legacy_db = cwd.join("data").join("portal_desktop.db");
            if legacy_db.exists() {
                if let Err(e) = std::fs::copy(&legacy_db, &new_db) {
                    log_warn!(
                        "DatabaseManager",
                        "Failed to migrate legacy database: {}",
                        e
                    );
                } else {
                    log_info!(
                        "DatabaseManager",
                        "Migrated legacy database from {}",
                        legacy_db.display()
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    fn set_dir_permissions(path: &Path) {
        if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o700)) {
            log_warn!(
                "DatabaseManager",
                "Failed to set directory permissions: {}",
                e
            );
        }
    }

    #[cfg(not(unix))]
    fn set_dir_permissions(_path: &Path) {}

    #[cfg(unix)]
    fn set_file_permissions(path: &Path) {
        if path.exists() {
            if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
                log_warn!(
                    "DatabaseManager",
                    "Failed to set database file permissions: {}",
                    e
                );
            }
        }
    }

    #[cfg(not(unix))]
    fn set_file_permissions(_path: &Path) {}
}
