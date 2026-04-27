//! Database Connection Management
//! Version: 1.2.0

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr};
use std::path::Path;
use tracing::{error, info};

/// Database pool type
pub type DbPool = DatabaseConnection;

/// Create database connection pool
///
/// # Arguments
/// * `db_path` - Path to the SQLite database file
///
/// # Returns
/// * `Result<DbPool, DbErr>` - Database connection pool or error
pub async fn create_db_pool(db_path: &Path) -> Result<DbPool, DbErr> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DbErr::Custom(format!("Failed to create database directory: {}", e)))?;
    }

    // Build connection string
    let connection_string = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

    info!("Connecting to database: {}", connection_string);

    // Create connection pool
    let db = Database::connect(&connection_string).await?;

    // Enable foreign keys
    db.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .map_err(|e| {
            error!("Failed to enable foreign keys: {}", e);
            DbErr::Custom(format!("Failed to enable foreign keys: {}", e))
        })?;

    info!("Database connection established successfully");

    Ok(db)
}

/// Get database connection (alias for create_db_pool)
///
/// This is the main entry point for getting a database connection
pub async fn get_db_connection(db_path: &Path) -> Result<DbPool, DbErr> {
    create_db_pool(db_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_db_pool() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let result = create_db_pool(&db_path).await;

        assert!(result.is_ok());

        // Verify file was created
        assert!(db_path.exists());
    }
}
