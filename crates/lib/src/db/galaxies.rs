use super::{Database, PersistenceError};

use crate::models::GalaxyRow;

use crate::{Coords, Event, EventCallback, Galaxy, GalaxyConfig, StructureType, System};

use sqlx::Row;

use std::collections::HashMap;

use std::str::FromStr;

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

    /// Create a new galaxy with full configuration and initial state
    pub async fn create_galaxy_with_config(
        &self,
        galaxy_name: &str,
        config: &GalaxyConfig,
        initial_tick: usize,
    ) -> Result<Galaxy, PersistenceError> {
        // Serialize configuration to YAML
        let config_yaml = serde_yaml::to_string(config)?;

        // Create galaxy in database
        self.create_galaxy(galaxy_name, &config_yaml, initial_tick)
            .await?;

        // Create galaxy struct
        let galaxy = Galaxy::new(config.clone(), initial_tick);

        // Save initial state
        self.save_galaxy_state(galaxy_name, &galaxy).await?;

        Ok(galaxy)
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

    /// List all galaxy names in the database
    pub async fn list_galaxy_names(&self) -> Result<Vec<String>, PersistenceError> {
        let rows = sqlx::query("SELECT name FROM galaxies ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let names = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        Ok(names)
    }

    /// Save complete galaxy state to database
    pub async fn save_galaxy_state(
        &self,
        galaxy_name: &str,
        galaxy: &Galaxy,
    ) -> Result<(), PersistenceError> {
        let mut tx = self.pool.begin().await?;

        // Ensure galaxy exists in database, create if it doesn't
        let galaxy_exists = sqlx::query("SELECT 1 FROM galaxies WHERE name = ? LIMIT 1")
            .bind(galaxy_name)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();

        if !galaxy_exists {
            // Create galaxy record if it doesn't exist
            sqlx::query("INSERT INTO galaxies (name, config_file, tick) VALUES (?, ?, ?)")
                .bind(galaxy_name)
                .bind("") // Empty config file for existing galaxies
                .bind(galaxy.get_tick() as i64)
                .execute(&mut *tx)
                .await?;
        } else {
            // Update galaxy metadata
            sqlx::query(
                "UPDATE galaxies SET tick = ?, updated_at = CURRENT_TIMESTAMP WHERE name = ?",
            )
            .bind(galaxy.get_tick() as i64)
            .bind(galaxy_name)
            .execute(&mut *tx)
            .await?;
        }

        // Get dirty systems to minimize database writes
        let systems_to_save: Vec<_> = if galaxy.needs_persist() {
            if galaxy.get_dirty_systems().is_empty() {
                // If no specific systems are dirty but needs_persist is true, save all
                galaxy.systems().keys().cloned().collect()
            } else {
                galaxy.get_dirty_systems().iter().cloned().collect()
            }
        } else {
            Vec::new()
        };

        // Save dirty systems
        for coords in systems_to_save {
            if let Some(system) = galaxy.systems().get(&coords) {
                let system_id = self
                    .save_system_with_tx(&mut tx, galaxy_name, &coords, system)
                    .await?;

                // Save structures for this system
                let structures = system.get_structures();
                for (structure_type, level) in structures {
                    sqlx::query(
                        r#"
                        INSERT INTO structures (system_id, structure_type, level, updated_at)
                        VALUES (?, ?, ?, CURRENT_TIMESTAMP)
                        ON CONFLICT(system_id, structure_type) DO UPDATE SET
                            level = excluded.level,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(system_id)
                    .bind(structure_type.to_string().to_lowercase())
                    .bind(level as i64)
                    .execute(&mut *tx)
                    .await?;
                }

                // Clear existing events and save new ones
                sqlx::query("DELETE FROM events WHERE system_id = ?")
                    .bind(system_id)
                    .execute(&mut *tx)
                    .await?;

                for event in system.get_events() {
                    let structure_type = event.structure.map(|s| s.to_string().to_lowercase());
                    sqlx::query("INSERT INTO events (system_id, completion_tick, action_type, structure_type) VALUES (?, ?, ?, ?)")
                        .bind(system_id)
                        .bind(event.completion as i64)
                        .bind(format!("{:?}", event.action))
                        .bind(structure_type)
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Helper method to save a system within a transaction
    async fn save_system_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        galaxy_name: &str,
        coords: &Coords,
        system: &System,
    ) -> Result<i64, PersistenceError> {
        let resources = system.get_resources();
        let current_tick = system.get_current_tick();

        // Use RETURNING to get the ID whether it's an insert or update
        let row = sqlx::query(
            r#"
            INSERT INTO systems (galaxy_name, x, y, metal, crew, water, current_tick, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(galaxy_name, x, y) DO UPDATE SET
                metal = excluded.metal,
                crew = excluded.crew,
                water = excluded.water,
                current_tick = excluded.current_tick,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(galaxy_name)
        .bind(coords.x as i64)
        .bind(coords.y as i64)
        .bind(resources.metal as i64)
        .bind(resources.crew as i64)
        .bind(resources.water as i64)
        .bind(current_tick as i64)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row.get("id"))
    }

    /// Load complete galaxy state from database
    pub async fn load_galaxy(&self, galaxy_name: &str) -> Result<Option<Galaxy>, PersistenceError> {
        // Load galaxy metadata
        let galaxy_row = self.get_galaxy(galaxy_name).await?;
        let galaxy_row = match galaxy_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Parse configuration
        let config: GalaxyConfig = serde_yaml::from_str(&galaxy_row.config_file)?;

        // Load all systems for this galaxy
        let current_tick = galaxy_row.tick_as_usize();
        let systems = self
            .load_systems_for_galaxy(galaxy_name, current_tick)
            .await?;

        // Create galaxy with loaded data
        let mut galaxy = Galaxy::new(config, current_tick);
        galaxy.replace_systems(systems);

        Ok(Some(galaxy))
    }

    /// Load all systems for a galaxy from database
    async fn load_systems_for_galaxy(
        &self,
        galaxy_name: &str,
        _galaxy_current_tick: usize,
    ) -> Result<HashMap<Coords, System>, PersistenceError> {
        let system_rows = self.get_systems(galaxy_name).await?;
        let mut systems = HashMap::new();

        for system_row in system_rows {
            let coords = Coords {
                x: system_row.x as usize,
                y: system_row.y as usize,
            };

            let resources = crate::Resources {
                metal: system_row.metal as usize,
                crew: system_row.crew as usize,
                water: system_row.water as usize,
            };

            // Load structures for this system
            let structure_rows = self.get_structures(system_row.id).await?;
            let structures: Vec<(StructureType, usize)> = structure_rows
                .into_iter()
                .filter_map(|row| {
                    StructureType::from_str(&row.structure_type)
                        .ok()
                        .map(|s| (s, row.level as usize))
                })
                .collect();

            // Load events for this system
            let event_rows = self.get_events(system_row.id).await?;
            let events: Vec<Event> = event_rows
                .into_iter()
                .filter_map(|row| {
                    let action = match row.action_type.as_str() {
                        "Build" => Some(EventCallback::Build),
                        _ => None, // Only Build events are supported now
                    }?;

                    let structure = row
                        .structure_type
                        .and_then(|s| StructureType::from_str(&s).ok());

                    Some(Event {
                        completion: row.completion_tick as usize,
                        action,
                        structure,
                    })
                })
                .collect();

            // Use each system's stored current_tick
            let system_current_tick = system_row.current_tick_as_usize();
            let system = System::from_database(system_current_tick, resources, structures, events);
            systems.insert(coords, system);
        }

        Ok(systems)
    }
}

mod tests {
    use super::*;
    use crate::{Database, GalaxyConfig};

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

    #[tokio::test]
    async fn test_galaxy_with_config_creation() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "config_test_galaxy";

        // Create a minimal galaxy config for testing
        let config = GalaxyConfig::default();

        // Test galaxy creation with config
        let galaxy = db
            .create_galaxy_with_config(galaxy_name, &config, 0)
            .await
            .expect("Failed to create galaxy with config");

        assert_eq!(galaxy.get_tick(), 0);

        // Verify galaxy exists in database
        assert!(db
            .galaxy_exists(galaxy_name)
            .await
            .expect("Failed to check galaxy existence"));

        db.close().await;
    }

    #[tokio::test]
    async fn test_galaxy_load_save_cycle() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "load_save_test";

        // Create a galaxy with config
        let config = GalaxyConfig::default();
        let mut galaxy = db
            .create_galaxy_with_config(galaxy_name, &config, 100)
            .await
            .expect("Failed to create galaxy");

        // Mark some systems as dirty to test persistence
        galaxy.mark_all_dirty();

        // Save the galaxy
        db.save_galaxy_state(galaxy_name, &galaxy)
            .await
            .expect("Failed to save galaxy state");

        // Load the galaxy back
        let loaded_galaxy = db
            .load_galaxy(galaxy_name)
            .await
            .expect("Failed to load galaxy")
            .expect("Galaxy should exist");

        // Verify tick is preserved
        assert_eq!(loaded_galaxy.get_tick(), 100);

        // Verify systems count matches
        assert_eq!(loaded_galaxy.systems().len(), galaxy.systems().len());

        db.close().await;
    }

    #[tokio::test]
    async fn test_system_individual_tick_persistence() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "tick_test_galaxy";

        // Create a galaxy with minimal config
        let config = GalaxyConfig::default();
        let mut galaxy = db
            .create_galaxy_with_config(galaxy_name, &config, 1000)
            .await
            .expect("Failed to create galaxy");

        // This test demonstrates the critical bug: systems at different ticks
        // should maintain their individual current_tick values when saved/loaded

        // Add two systems manually with different current_tick values
        use crate::{Coords, Resources, System};
        use std::collections::HashMap;

        let coords1 = Coords { x: 10, y: 20 };
        let coords2 = Coords { x: 30, y: 40 };

        // Create system 1 at tick 500 (behind galaxy tick)
        let system1 = System::from_database(
            500, // current_tick = 500
            Resources {
                metal: 100,
                crew: 50,
                water: 75,
            },
            vec![], // no structures
            vec![], // no events
        );

        // Create system 2 at tick 800 (still behind galaxy tick)
        let system2 = System::from_database(
            800, // current_tick = 800
            Resources {
                metal: 200,
                crew: 100,
                water: 150,
            },
            vec![], // no structures
            vec![], // no events
        );

        // Add systems to galaxy
        let mut systems = HashMap::new();
        systems.insert(coords1, system1);
        systems.insert(coords2, system2);
        galaxy.replace_systems(systems);

        // Mark as dirty and save
        galaxy.mark_all_dirty();
        db.save_galaxy_state(galaxy_name, &galaxy)
            .await
            .expect("Failed to save galaxy state");

        // Load the galaxy back
        let loaded_galaxy = db
            .load_galaxy(galaxy_name)
            .await
            .expect("Failed to load galaxy")
            .expect("Galaxy should exist");

        // Verify the galaxy tick is preserved
        assert_eq!(loaded_galaxy.get_tick(), 1000);

        // CRITICAL TEST: Verify each system maintains its individual current_tick
        let loaded_system1 = loaded_galaxy.systems().get(&coords1).unwrap();
        let loaded_system2 = loaded_galaxy.systems().get(&coords2).unwrap();

        assert_eq!(
            loaded_system1.get_current_tick(),
            500,
            "System 1 should maintain its current_tick of 500"
        );
        assert_eq!(
            loaded_system2.get_current_tick(),
            800,
            "System 2 should maintain its current_tick of 800"
        );

        // Verify resources are preserved
        assert_eq!(
            loaded_system1.get_resources(),
            Resources {
                metal: 100,
                crew: 50,
                water: 75
            }
        );
        assert_eq!(
            loaded_system2.get_resources(),
            Resources {
                metal: 200,
                crew: 100,
                water: 150
            }
        );

        db.close().await;
    }

    #[tokio::test]
    async fn test_list_galaxy_names() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        // Initially should be empty
        let names = db
            .list_galaxy_names()
            .await
            .expect("Failed to list galaxy names");
        assert!(names.is_empty());

        // Create some galaxies
        db.create_galaxy("alpha", "config_alpha", 0)
            .await
            .expect("Failed to create galaxy alpha");
        db.create_galaxy("beta", "config_beta", 100)
            .await
            .expect("Failed to create galaxy beta");
        db.create_galaxy("gamma", "config_gamma", 200)
            .await
            .expect("Failed to create galaxy gamma");

        // List should now contain all galaxies in alphabetical order
        let names = db
            .list_galaxy_names()
            .await
            .expect("Failed to list galaxy names");
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);

        db.close().await;
    }
}
