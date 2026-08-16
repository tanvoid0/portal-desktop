use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::*;

use crate::migrations::Migrator;

/// Run all pending migrations using SeaORM's migration tracker.
pub async fn run_migrations(conn: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(conn, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use tempfile::tempdir;

    #[tokio::test]
    async fn migrations_apply_on_fresh_database() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.display());

        let conn = Database::connect(&url).await.expect("connect");
        run_migrations(&conn).await.expect("first migration run");

        // Idempotent: second run should succeed without error
        run_migrations(&conn).await.expect("second migration run");
    }
}
