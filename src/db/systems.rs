#[cfg(feature = "db")]
use super::{Database, PersistenceError};
#[cfg(feature = "db")]
use crate::models::SystemRow;
#[cfg(feature = "db")]
use sqlx::Row;

#[cfg(feature = "db")]
impl Database {
    /// Create or update a system in the database
    pub async fn save_system(
        &self,
        galaxy_name: &str,
        x: usize,
        y: usize,
        metal: usize,
        crew: usize,
        water: usize,
    ) -> Result<i64, PersistenceError> {
        let result = sqlx::query(
            r#"
            INSERT INTO systems (galaxy_name, x, y, metal, crew, water, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(galaxy_name, x, y) DO UPDATE SET
                metal = excluded.metal,
                crew = excluded.crew,
                water = excluded.water,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(galaxy_name)
        .bind(x as i64)
        .bind(y as i64)
        .bind(metal as i64)
        .bind(crew as i64)
        .bind(water as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Get all systems for a galaxy
    pub async fn get_systems(&self, galaxy_name: &str) -> Result<Vec<SystemRow>, PersistenceError> {
        let rows = sqlx::query("SELECT id, galaxy_name, x, y, metal, crew, water, created_at, updated_at FROM systems WHERE galaxy_name = ?")
            .bind(galaxy_name)
            .fetch_all(&self.pool)
            .await?;

        let mut systems = Vec::new();
        for row in rows {
            systems.push(SystemRow {
                id: row.get("id"),
                galaxy_name: row.get("galaxy_name"),
                x: row.get("x"),
                y: row.get("y"),
                metal: row.get("metal"),
                crew: row.get("crew"),
                water: row.get("water"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(systems)
    }

    /// Get a specific system by coordinates
    pub async fn get_system(
        &self,
        galaxy_name: &str,
        x: usize,
        y: usize,
    ) -> Result<Option<SystemRow>, PersistenceError> {
        let result = sqlx::query("SELECT id, galaxy_name, x, y, metal, crew, water, created_at, updated_at FROM systems WHERE galaxy_name = ? AND x = ? AND y = ?")
            .bind(galaxy_name)
            .bind(x as i64)
            .bind(y as i64)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = result {
            Ok(Some(SystemRow {
                id: row.get("id"),
                galaxy_name: row.get("galaxy_name"),
                x: row.get("x"),
                y: row.get("y"),
                metal: row.get("metal"),
                crew: row.get("crew"),
                water: row.get("water"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete all systems for a galaxy (used in cleanup)
    pub async fn delete_systems(&self, galaxy_name: &str) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM systems WHERE galaxy_name = ?")
            .bind(galaxy_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;

    async fn setup_test_galaxy(db: &Database, galaxy_name: &str) {
        db.create_galaxy(galaxy_name, "test_config", 0)
            .await
            .expect("Failed to create test galaxy");
    }

    #[tokio::test]
    async fn test_system_crud_operations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");
        let galaxy_name = "test_galaxy";

        setup_test_galaxy(&db, galaxy_name).await;

        let x = 10;
        let y = 20;
        let metal = 100;
        let crew = 200;
        let water = 300;

        // Test system creation
        let system_id = db
            .save_system(galaxy_name, x, y, metal, crew, water)
            .await
            .expect("Failed to save system");

        assert!(system_id > 0);

        // Test getting specific system
        let system = db
            .get_system(galaxy_name, x, y)
            .await
            .expect("Failed to get system")
            .expect("System should exist");

        assert_eq!(system.id, system_id);
        assert_eq!(system.galaxy_name, galaxy_name);
        assert_eq!(system.coords(), (x, y));
        assert_eq!(system.resources(), (metal, crew, water));

        // Test getting all systems for galaxy
        let systems = db
            .get_systems(galaxy_name)
            .await
            .expect("Failed to get systems");

        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].id, system_id);

        // Test system update (same coordinates, different resources)
        let new_metal = 500;
        let new_crew = 600;
        let new_water = 700;

        let updated_system_id = db
            .save_system(galaxy_name, x, y, new_metal, new_crew, new_water)
            .await
            .expect("Failed to update system");

        // Should return the same ID since we're updating
        assert_eq!(updated_system_id, system_id);

        let updated_system = db
            .get_system(galaxy_name, x, y)
            .await
            .expect("Failed to get updated system")
            .expect("System should exist");

        assert_eq!(updated_system.resources(), (new_metal, new_crew, new_water));

        // Test system deletion
        db.delete_systems(galaxy_name)
            .await
            .expect("Failed to delete systems");

        let systems_after_delete = db
            .get_systems(galaxy_name)
            .await
            .expect("Failed to get systems");

        assert_eq!(systems_after_delete.len(), 0);

        db.close().await;
    }

    #[tokio::test]
    async fn test_multiple_systems() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");
        let galaxy_name = "multi_system_galaxy";

        setup_test_galaxy(&db, galaxy_name).await;

        // Create multiple systems
        let systems_data = vec![
            (10, 20, 100, 200, 300),
            (30, 40, 400, 500, 600),
            (50, 60, 700, 800, 900),
        ];

        for (x, y, metal, crew, water) in &systems_data {
            db.save_system(galaxy_name, *x, *y, *metal, *crew, *water)
                .await
                .expect("Failed to save system");
        }

        // Test getting all systems
        let systems = db
            .get_systems(galaxy_name)
            .await
            .expect("Failed to get systems");

        assert_eq!(systems.len(), 3);

        // Verify each system
        for (x, y, metal, crew, water) in systems_data {
            let system = db
                .get_system(galaxy_name, x, y)
                .await
                .expect("Failed to get system")
                .expect("System should exist");

            assert_eq!(system.coords(), (x, y));
            assert_eq!(system.resources(), (metal, crew, water));
        }

        db.close().await;
    }

    #[tokio::test]
    async fn test_system_not_found() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");
        let galaxy_name = "empty_galaxy";

        setup_test_galaxy(&db, galaxy_name).await;

        let result = db
            .get_system(galaxy_name, 99, 99)
            .await
            .expect("Failed to query system");

        assert!(result.is_none());

        db.close().await;
    }
}
