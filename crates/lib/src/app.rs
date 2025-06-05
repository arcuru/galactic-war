use crate::{config::GalaxyConfig, Coords, Details, Event, Galaxy, SystemInfo};

#[cfg(feature = "db")]
use crate::{
    db::Database,
    persistence::{PersistenceConfig, PersistenceManager},
};

use std::collections::HashMap;
#[cfg(feature = "db")]
use std::env;
use std::sync::{Arc, Mutex};

/// Application state manager that coordinates between in-memory state and persistence
#[derive(Debug)]
pub struct AppState {
    #[cfg(feature = "db")]
    persistence_manager: Option<PersistenceManager>,
    /// Galaxy storage
    galaxies: Arc<Mutex<HashMap<String, Galaxy>>>,
}

impl AppState {
    /// Initialize the application state with optional persistence
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let galaxies = Arc::new(Mutex::new(HashMap::new()));

        #[cfg(feature = "db")]
        {
            // Check if persistence is enabled via environment variable
            let persistence_enabled = env::var("GALACTIC_WAR_PERSISTENCE")
                .unwrap_or_else(|_| "true".to_string())
                .parse::<bool>()
                .unwrap_or(true);

            if persistence_enabled {
                log::info!("Initializing with database persistence");

                // Create database connection
                let database = Database::new().await?;

                // Configure persistence settings from environment variables
                let config = PersistenceConfig {
                    auto_save_interval: env::var("GALACTIC_WAR_AUTO_SAVE_INTERVAL")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30),
                    shutdown_timeout: env::var("GALACTIC_WAR_SHUTDOWN_TIMEOUT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10),
                    enabled: persistence_enabled,
                    write_coalescing: env::var("GALACTIC_WAR_WRITE_COALESCING")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(true),
                    coalescing_delay_ms: env::var("GALACTIC_WAR_COALESCING_DELAY_MS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1000),
                };

                log::info!("Persistence config: auto_save_interval={}s, write_coalescing={}, coalescing_delay={}ms", 
                    config.auto_save_interval, config.write_coalescing, config.coalescing_delay_ms);

                // Create persistence manager with weak reference to galaxies
                let galaxies_weak = Arc::downgrade(&galaxies);
                let persistence_manager =
                    PersistenceManager::new(database, config, galaxies_weak).await?;

                Ok(Self {
                    persistence_manager: Some(persistence_manager),
                    galaxies,
                })
            } else {
                log::info!("Persistence disabled via configuration");
                Ok(Self {
                    persistence_manager: None,
                    galaxies,
                })
            }
        }

        #[cfg(not(feature = "db"))]
        {
            log::info!("Database persistence not available (compiled without 'db' feature)");
            Ok(Self { galaxies })
        }
    }

    /// Create a test instance without persistence (for testing)
    #[cfg(test)]
    pub async fn new_test() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(feature = "db")]
        {
            Ok(Self {
                persistence_manager: None,
                galaxies: Arc::new(Mutex::new(HashMap::new())),
            })
        }

        #[cfg(not(feature = "db"))]
        {
            Ok(Self {
                galaxies: Arc::new(Mutex::new(HashMap::new())),
            })
        }
    }

    /// Create a new galaxy
    pub async fn create_galaxy(
        &self,
        galaxy_name: &str,
        config: &GalaxyConfig,
        initial_tick: usize,
    ) -> Result<String, String> {
        // Check if galaxy already exists in memory
        let galaxy_exists = {
            let galaxies = self.galaxies.lock().unwrap();
            galaxies.contains_key(galaxy_name)
        };

        if galaxy_exists {
            return Err(format!("Galaxy {} already exists in memory", galaxy_name));
        }

        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            // Check if galaxy exists in database
            match pm.galaxy_exists_in_db(galaxy_name).await {
                Ok(true) => {
                    // Try to load from database
                    match pm.load_galaxy(galaxy_name).await {
                        Ok(Some(galaxy)) => {
                            {
                                let mut galaxies = self.galaxies.lock().unwrap();
                                galaxies.insert(galaxy_name.to_string(), galaxy);
                            }
                            return Ok(format!("Galaxy {} loaded from database", galaxy_name));
                        }
                        Ok(None) => {
                            return Err(format!(
                                "Galaxy {} exists in database but failed to load",
                                galaxy_name
                            ))
                        }
                        Err(e) => {
                            return Err(format!(
                                "Database error while loading galaxy {}: {}",
                                galaxy_name, e
                            ))
                        }
                    }
                }
                Ok(false) => {
                    // Galaxy doesn't exist, create it
                    let config_yaml = serde_yaml::to_string(config)
                        .map_err(|e| format!("Failed to serialize config: {}", e))?;

                    match pm
                        .database()
                        .create_galaxy(galaxy_name, &config_yaml, initial_tick)
                        .await
                    {
                        Ok(_) => log::info!("Created galaxy {} in database", galaxy_name),
                        Err(e) => log::error!(
                            "Failed to create galaxy {} in database: {}",
                            galaxy_name,
                            e
                        ),
                    }
                }
                Err(e) => log::error!("Database error checking galaxy existence: {}", e),
            }
        }

        // Create galaxy in memory
        let galaxy = Galaxy::new(config.clone(), initial_tick);

        {
            let mut galaxies = self.galaxies.lock().unwrap();
            galaxies.insert(galaxy_name.to_string(), galaxy);
        }

        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            // Save initial state to database - we need to clone the galaxy to avoid holding lock across await
            let galaxy_clone = {
                let galaxies = self.galaxies.lock().unwrap();
                galaxies.get(galaxy_name).cloned()
            };

            if let Some(mut galaxy) = galaxy_clone {
                if let Err(e) = pm.save_galaxy(galaxy_name, &mut galaxy).await {
                    log::error!("Failed to persist new galaxy {}: {}", galaxy_name, e);
                }
            }
        }

        Ok(format!("Galaxy {} created", galaxy_name))
    }

    /// Get galaxy details with auto-loading from database if not in memory
    pub async fn get_galaxy_details(
        &self,
        galaxy_name: &str,
        tick: usize,
        coords: Coords,
        structure: Option<crate::StructureType>,
    ) -> Result<Details, String> {
        // Try to get from memory first
        {
            let mut galaxies = self.galaxies.lock().unwrap();
            if let Some(galaxy) = galaxies.get_mut(galaxy_name) {
                return galaxy.get_details(tick, coords, structure);
            }
        }

        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            // Try to load from database
            match pm.load_galaxy(galaxy_name).await {
                Ok(Some(galaxy)) => {
                    let mut galaxies = self.galaxies.lock().unwrap();
                    galaxies.insert(galaxy_name.to_string(), galaxy);
                    // Now try again to get the details
                    if let Some(galaxy) = galaxies.get_mut(galaxy_name) {
                        galaxy.get_details(tick, coords, structure)
                    } else {
                        Err("Failed to insert loaded galaxy into memory".to_string())
                    }
                }
                Ok(None) => Err(format!("Galaxy '{}' not found in database", galaxy_name)),
                Err(e) => Err(format!("Database error loading galaxy: {}", e)),
            }
        } else {
            Err(format!(
                "Galaxy '{}' not found and persistence not available",
                galaxy_name
            ))
        }

        #[cfg(not(feature = "db"))]
        Err(format!(
            "Galaxy '{}' not found and persistence not available",
            galaxy_name
        ))
    }

    /// Build a structure with auto-persistence
    pub async fn build_structure(
        &self,
        galaxy_name: &str,
        tick: usize,
        coords: Coords,
        structure: crate::StructureType,
    ) -> Result<Event, String> {
        // Ensure galaxy is loaded
        self.ensure_galaxy_loaded(galaxy_name).await?;

        let result = {
            let mut galaxies = self.galaxies.lock().unwrap();
            if let Some(galaxy) = galaxies.get_mut(galaxy_name) {
                galaxy.build(tick, coords, structure)
            } else {
                return Err(format!("Galaxy '{}' not found", galaxy_name));
            }
        };

        // Auto-persist on successful builds (the persistence manager will handle this automatically)
        result
    }

    /// Get galaxy stats with auto-loading
    pub async fn get_galaxy_stats(&self, galaxy_name: &str, tick: usize) -> Result<String, String> {
        // Ensure galaxy is loaded
        self.ensure_galaxy_loaded(galaxy_name).await?;

        let mut galaxies = self.galaxies.lock().unwrap();
        if let Some(galaxy) = galaxies.get_mut(galaxy_name) {
            galaxy.stats(tick)
        } else {
            Err(format!("Galaxy '{}' not found", galaxy_name))
        }
    }

    /// Manually save all dirty galaxies
    pub async fn save_all(&self) -> Result<usize, String> {
        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            let mut galaxies = self.galaxies.lock().unwrap();
            match pm.save_all_dirty(&mut galaxies).await {
                Ok(count) => {
                    log::info!("Manually saved {} galaxies", count);
                    Ok(count)
                }
                Err(e) => Err(format!("Failed to save galaxies: {}", e)),
            }
        } else {
            Ok(0)
        }

        #[cfg(not(feature = "db"))]
        Ok(0)
    }

    /// List all galaxies (from memory and database)
    pub async fn list_galaxies(&self) -> Vec<String> {
        let mut galaxies = Vec::new();

        // Get galaxies from memory
        {
            let memory_galaxies = self.galaxies.lock().unwrap();
            galaxies.extend(memory_galaxies.keys().cloned());
        }

        #[cfg(feature = "db")]
        if let Some(ref _pm) = self.persistence_manager {
            // TODO: Implement database galaxy listing
            // For now, we only show galaxies in memory
        }

        galaxies.sort();
        galaxies.dedup();
        galaxies
    }

    /// Ensure a galaxy is loaded into memory
    async fn ensure_galaxy_loaded(&self, galaxy_name: &str) -> Result<(), String> {
        // Check if already in memory
        {
            let galaxies = self.galaxies.lock().unwrap();
            if galaxies.contains_key(galaxy_name) {
                return Ok(());
            }
        }

        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            // Try to load from database
            match pm.load_galaxy(galaxy_name).await {
                Ok(Some(galaxy)) => {
                    let mut galaxies = self.galaxies.lock().unwrap();
                    galaxies.insert(galaxy_name.to_string(), galaxy);
                    Ok(())
                }
                Ok(None) => Err(format!("Galaxy '{}' not found in database", galaxy_name)),
                Err(e) => Err(format!("Database error loading galaxy: {}", e)),
            }
        } else {
            Err(format!("Galaxy '{}' not found in memory", galaxy_name))
        }

        #[cfg(not(feature = "db"))]
        Err(format!("Galaxy '{}' not found in memory", galaxy_name))
    }

    /// Gracefully shutdown the application
    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "db")]
        if let Some(persistence_manager) = self.persistence_manager {
            log::info!("Shutting down persistence manager...");
            persistence_manager.shutdown().await?;
        }

        #[cfg(not(feature = "db"))]
        log::info!("No persistence to shut down");

        Ok(())
    }

    /// Gracefully shutdown without consuming self (for signal handlers)
    pub async fn shutdown_gracefully(&self) -> Result<(), String> {
        // Save all dirty galaxies first
        self.save_all().await?;

        #[cfg(feature = "db")]
        if let Some(ref pm) = self.persistence_manager {
            log::info!("Shutting down persistence manager...");
            if let Err(e) = pm.shutdown().await {
                log::error!("Error shutting down persistence manager: {}", e);
                return Err(format!("Persistence shutdown error: {}", e));
            }
        }

        #[cfg(not(feature = "db"))]
        log::info!("No persistence to shut down");

        Ok(())
    }

    /// Retrieve the details of a system (async version)
    pub async fn system_info(&self, galaxy: &str, coords: Coords) -> Result<SystemInfo, String> {
        let dets = self
            .get_galaxy_details(galaxy, crate::tick(), coords, None)
            .await?;
        match dets {
            Details::System(info) => Ok(info),
            _ => Err("Unexpected Details type".to_string()),
        }
    }

    /// Retrieve the details of a system (synchronous version for web interface)
    pub fn system_info_sync(&self, galaxy: &str, coords: Coords) -> Result<SystemInfo, String> {
        let mut galaxies = self.galaxies.lock().unwrap();
        if let Some(galaxy) = galaxies.get_mut(galaxy) {
            let dets = galaxy.get_details(crate::tick(), coords, None);
            if let Ok(dets) = dets {
                match dets {
                    Details::System(info) => Ok(info),
                    _ => Err("Unexpected Details type".to_string()),
                }
            } else {
                Err(dets.unwrap_err())
            }
        } else {
            Err("Galaxy not found".to_string())
        }
    }

    /// Get direct access to galaxy storage for binary use (legacy compatibility)
    pub fn galaxies(&self) -> &Arc<Mutex<HashMap<String, Galaxy>>> {
        &self.galaxies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GalaxyConfig;

    fn create_test_config() -> GalaxyConfig {
        GalaxyConfig {
            system_count: 5,
            size: crate::config::GalaxySize { x: 10, y: 10 },
            systems: crate::config::SystemConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let app_state = AppState::new_test().await;
        assert!(app_state.is_ok());
    }

    #[tokio::test]
    async fn test_galaxy_creation_and_listing() {
        let app_state = AppState::new_test().await.unwrap();
        let config = create_test_config();

        // Create a test galaxy
        let result = app_state.create_galaxy("test_galaxy", &config, 0).await;
        assert!(result.is_ok());

        // List galaxies should include our new galaxy
        let galaxies = app_state.list_galaxies().await;
        assert!(galaxies.contains(&"test_galaxy".to_string()));
    }

    #[tokio::test]
    async fn test_galaxy_operations() {
        let app_state = AppState::new_test().await.unwrap();
        let config = create_test_config();

        // Create galaxy
        app_state
            .create_galaxy("ops_test", &config, 0)
            .await
            .unwrap();

        // Get a valid coordinate by checking the galaxy's systems
        let coords = {
            let galaxies = app_state.galaxies.lock().unwrap();
            let galaxy = galaxies.get("ops_test").unwrap();
            *galaxy.systems().keys().next().unwrap()
        };

        // Test getting galaxy details
        let details = app_state
            .get_galaxy_details("ops_test", 100, coords, None)
            .await;
        assert!(details.is_ok());

        // Test getting galaxy stats
        let stats = app_state.get_galaxy_stats("ops_test", 100).await;
        assert!(stats.is_ok());
    }

    #[tokio::test]
    async fn test_persistence_save_all() {
        let app_state = AppState::new_test().await.unwrap();

        // Test manual save (should work even with no galaxies)
        let result = app_state.save_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_app_state_shutdown() {
        let app_state = AppState::new_test().await.unwrap();

        // Test graceful shutdown
        let result = app_state.shutdown().await;
        assert!(result.is_ok());
    }
}
