use std::env;

#[cfg(feature = "db")]
use sqlx::{Pool, Sqlite, SqlitePool};

pub mod events;
pub mod galaxies;
pub mod structures;
pub mod systems;

// Error types for database operations
#[cfg(feature = "db")]
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Galaxy not found: {name}")]
    GalaxyNotFound { name: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Migration error: {0}")]
    Migration(String),
}

/// Database connection manager
#[cfg(feature = "db")]
#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

#[cfg(feature = "db")]
impl Database {
    /// Create a new database connection with migrations
    pub async fn new() -> Result<Self, PersistenceError> {
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:galactic_war.db".to_string());

        let pool = SqlitePool::connect(&database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| PersistenceError::Migration(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Create a new in-memory database for testing
    pub async fn new_test() -> Result<Self, PersistenceError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| PersistenceError::Migration(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        // Test that we can get a pool reference
        let _pool = db.pool();

        // Clean up
        db.close().await;
    }

    #[tokio::test]
    async fn test_database_migrations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        // Test that tables were created by attempting to query them
        let galaxy_count = sqlx::query("SELECT COUNT(*) as count FROM galaxies")
            .fetch_one(db.pool())
            .await
            .expect("Failed to query galaxies table");

        assert_eq!(galaxy_count.get::<i32, _>("count"), 0);

        let system_count = sqlx::query("SELECT COUNT(*) as count FROM systems")
            .fetch_one(db.pool())
            .await
            .expect("Failed to query systems table");

        assert_eq!(system_count.get::<i32, _>("count"), 0);

        let structure_count = sqlx::query("SELECT COUNT(*) as count FROM structures")
            .fetch_one(db.pool())
            .await
            .expect("Failed to query structures table");

        assert_eq!(structure_count.get::<i32, _>("count"), 0);

        let event_count = sqlx::query("SELECT COUNT(*) as count FROM events")
            .fetch_one(db.pool())
            .await
            .expect("Failed to query events table");

        assert_eq!(event_count.get::<i32, _>("count"), 0);

        db.close().await;
    }
}
