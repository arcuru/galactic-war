#[cfg(feature = "db")]
use super::{Database, PersistenceError};
#[cfg(feature = "db")]
use crate::models::GalaxyRow;
#[cfg(feature = "db")]
use sqlx::Row;

#[cfg(feature = "db")]
impl Database {
    /// Check if a galaxy exists in the database
    pub async fn galaxy_exists(&self, galaxy_name: &str) -> Result<bool, PersistenceError> {
        let result = sqlx::query("SELECT 1 FROM galaxies WHERE name = ? LIMIT 1")
            .bind(galaxy_name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.is_some())
    }

    /// Create a new galaxy in the database
    pub async fn create_galaxy(
        &self,
        galaxy_name: &str,
        config_file: &str,
        tick: usize,
    ) -> Result<(), PersistenceError> {
        sqlx::query("INSERT INTO galaxies (name, config_file, tick) VALUES (?, ?, ?)")
            .bind(galaxy_name)
            .bind(config_file)
            .bind(tick as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get galaxy metadata from the database
    pub async fn get_galaxy(
        &self,
        galaxy_name: &str,
    ) -> Result<Option<GalaxyRow>, PersistenceError> {
        let result = sqlx::query(
            "SELECT name, config_file, tick, created_at, updated_at FROM galaxies WHERE name = ?",
        )
        .bind(galaxy_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            Ok(Some(GalaxyRow {
                name: row.get("name"),
                config_file: row.get("config_file"),
                tick: row.get("tick"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update galaxy tick in the database
    pub async fn update_galaxy_tick(
        &self,
        galaxy_name: &str,
        tick: usize,
    ) -> Result<(), PersistenceError> {
        sqlx::query("UPDATE galaxies SET tick = ?, updated_at = CURRENT_TIMESTAMP WHERE name = ?")
            .bind(tick as i64)
            .bind(galaxy_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete a galaxy and all its associated data
    pub async fn delete_galaxy(&self, galaxy_name: &str) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM galaxies WHERE name = ?")
            .bind(galaxy_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_galaxy_crud_operations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "test_galaxy";
        let config_file = "test_config";
        let tick = 1000;

        // Test galaxy doesn't exist initially
        assert!(!db
            .galaxy_exists(galaxy_name)
            .await
            .expect("Failed to check galaxy existence"));

        // Test galaxy creation
        db.create_galaxy(galaxy_name, config_file, tick)
            .await
            .expect("Failed to create galaxy");

        // Test galaxy exists after creation
        assert!(db
            .galaxy_exists(galaxy_name)
            .await
            .expect("Failed to check galaxy existence"));

        // Test getting galaxy
        let galaxy = db
            .get_galaxy(galaxy_name)
            .await
            .expect("Failed to get galaxy")
            .expect("Galaxy should exist");

        assert_eq!(galaxy.name, galaxy_name);
        assert_eq!(galaxy.config_file, config_file);
        assert_eq!(galaxy.tick_as_usize(), tick);

        // Test updating galaxy tick
        let new_tick = 2000;
        db.update_galaxy_tick(galaxy_name, new_tick)
            .await
            .expect("Failed to update galaxy tick");

        let updated_galaxy = db
            .get_galaxy(galaxy_name)
            .await
            .expect("Failed to get galaxy")
            .expect("Galaxy should exist");

        assert_eq!(updated_galaxy.tick_as_usize(), new_tick);

        // Test galaxy deletion
        db.delete_galaxy(galaxy_name)
            .await
            .expect("Failed to delete galaxy");

        // Test galaxy doesn't exist after deletion
        assert!(!db
            .galaxy_exists(galaxy_name)
            .await
            .expect("Failed to check galaxy existence"));

        db.close().await;
    }

    #[tokio::test]
    async fn test_galaxy_not_found() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let result = db
            .get_galaxy("nonexistent")
            .await
            .expect("Failed to query galaxy");
        assert!(result.is_none());

        db.close().await;
    }

    #[tokio::test]
    async fn test_duplicate_galaxy_creation() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "duplicate_test";

        // Create galaxy first time
        db.create_galaxy(galaxy_name, "config", 0)
            .await
            .expect("Failed to create galaxy");

        // Attempt to create same galaxy again should fail
        let result = db.create_galaxy(galaxy_name, "config", 0).await;
        assert!(result.is_err());

        db.close().await;
    }
}
