#[cfg(feature = "bin")]
use crate::utils::GALAXIES;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[cfg(feature = "db")]
use crate::db::Database;

/// Configuration for the persistence manager
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// How often to check for dirty galaxies and persist them (in seconds)
    pub auto_save_interval: u64,
    /// Maximum time to wait for persistence operations during shutdown (in seconds)
    pub shutdown_timeout: u64,
    /// Whether to enable auto-persistence
    pub enabled: bool,
    /// Whether to use write coalescing (delay writes to batch multiple changes)
    pub write_coalescing: bool,
    /// Write coalescing delay in milliseconds
    pub coalescing_delay_ms: u64,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            auto_save_interval: 30, // Auto-save every 30 seconds
            shutdown_timeout: 10,   // Wait up to 10 seconds during shutdown
            enabled: true,
            write_coalescing: true,
            coalescing_delay_ms: 1000, // 1 second coalescing delay
        }
    }
}

/// Manages automatic persistence of galaxy state
#[cfg(feature = "db")]
#[derive(Debug)]
pub struct PersistenceManager {
    database: Database,
    config: PersistenceConfig,
    shutdown_signal: Arc<AtomicBool>,
    _shutdown_handle: tokio::task::JoinHandle<()>,
}

#[cfg(feature = "db")]
impl PersistenceManager {
    /// Create and start a new persistence manager
    pub async fn new(
        database: Database,
        config: PersistenceConfig,
    ) -> Result<Self, crate::db::PersistenceError> {
        let shutdown_signal = Arc::new(AtomicBool::new(false));

        // Start the background persistence worker
        let worker_handle = if config.enabled {
            tokio::spawn(Self::run_persistence_worker(
                database.clone(),
                config.clone(),
                shutdown_signal.clone(),
            ))
        } else {
            // If persistence is disabled, just spawn a dummy task
            tokio::spawn(async {})
        };

        Ok(Self {
            database,
            config,
            shutdown_signal,
            _shutdown_handle: worker_handle,
        })
    }

    /// Get a reference to the database
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Manually save all dirty galaxies
    pub async fn save_all_dirty(&self) -> Result<usize, crate::db::PersistenceError> {
        let dirty_galaxies = self.get_dirty_galaxy_names();
        let mut saved_count = 0;

        for galaxy_name in dirty_galaxies {
            if let Some(galaxy) = self.get_galaxy_for_persistence(&galaxy_name) {
                match self.database.save_galaxy_state(&galaxy_name, &galaxy).await {
                    Ok(_) => {
                        self.clear_galaxy_dirty_flag(&galaxy_name);
                        saved_count += 1;
                        log::debug!("Persisted galaxy: {}", galaxy_name);
                    }
                    Err(e) => {
                        log::error!("Failed to persist galaxy {}: {}", galaxy_name, e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(saved_count)
    }

    /// Manually save a specific galaxy
    pub async fn save_galaxy(
        &self,
        galaxy_name: &str,
    ) -> Result<bool, crate::db::PersistenceError> {
        if let Some(galaxy) = self.get_galaxy_for_persistence(galaxy_name) {
            self.database
                .save_galaxy_state(galaxy_name, &galaxy)
                .await?;
            self.clear_galaxy_dirty_flag(galaxy_name);
            log::debug!("Manually persisted galaxy: {}", galaxy_name);
            Ok(true)
        } else {
            Ok(false) // Galaxy not found or not dirty
        }
    }

    /// Load a galaxy from the database into memory
    pub async fn load_galaxy(
        &self,
        galaxy_name: &str,
    ) -> Result<bool, crate::db::PersistenceError> {
        match self.database.load_galaxy(galaxy_name).await? {
            Some(galaxy) => {
                #[cfg(feature = "bin")]
                {
                    let mut galaxies = (**GALAXIES).lock().unwrap();
                    galaxies.insert(galaxy_name.to_string(), galaxy);
                }
                log::info!("Loaded galaxy from database: {}", galaxy_name);
                Ok(true)
            }
            None => Ok(false), // Galaxy not found in database
        }
    }

    /// Check if a galaxy exists in the database
    pub async fn galaxy_exists_in_db(
        &self,
        galaxy_name: &str,
    ) -> Result<bool, crate::db::PersistenceError> {
        self.database.galaxy_exists(galaxy_name).await
    }

    /// Gracefully shutdown the persistence manager
    pub async fn shutdown(&self) -> Result<(), crate::db::PersistenceError> {
        log::info!("Shutting down persistence manager...");

        // Signal shutdown to the worker
        self.shutdown_signal.store(true, Ordering::SeqCst);

        // Try to save all dirty galaxies one last time
        let timeout = Duration::from_secs(self.config.shutdown_timeout);
        match tokio::time::timeout(timeout, self.save_all_dirty()).await {
            Ok(Ok(count)) => {
                log::info!("Saved {} galaxies during shutdown", count);
            }
            Ok(Err(e)) => {
                log::error!("Error saving galaxies during shutdown: {}", e);
            }
            Err(_) => {
                log::warn!("Timeout while saving galaxies during shutdown");
            }
        }

        Ok(())
    }

    /// Background worker that periodically saves dirty galaxies
    async fn run_persistence_worker(
        database: Database,
        config: PersistenceConfig,
        shutdown_signal: Arc<AtomicBool>,
    ) {
        let mut save_interval = interval(Duration::from_secs(config.auto_save_interval));
        let mut coalescing_tracker: HashSet<String> = HashSet::new();
        let mut last_coalescing_check = tokio::time::Instant::now();

        log::info!(
            "Started persistence worker with {}s interval",
            config.auto_save_interval
        );

        loop {
            tokio::select! {
                _ = save_interval.tick() => {
                    if shutdown_signal.load(Ordering::SeqCst) {
                        log::info!("Persistence worker shutting down");
                        break;
                    }

                    // Get list of dirty galaxies
                    let dirty_galaxies = Self::get_dirty_galaxy_names_static();

                    if config.write_coalescing {
                        // Add newly dirty galaxies to coalescing tracker
                        for galaxy_name in &dirty_galaxies {
                            coalescing_tracker.insert(galaxy_name.clone());
                        }

                        // Check if coalescing delay has passed
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_coalescing_check) >= Duration::from_millis(config.coalescing_delay_ms) {
                            // Persist all galaxies in the coalescing tracker
                            let galaxies_to_save: Vec<_> = coalescing_tracker.drain().collect();
                            Self::persist_galaxies_batch(&database, galaxies_to_save).await;
                            last_coalescing_check = now;
                        }
                    } else {
                        // Persist immediately without coalescing
                        Self::persist_galaxies_batch(&database, dirty_galaxies).await;
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Check for shutdown signal periodically
                    if shutdown_signal.load(Ordering::SeqCst) {
                        log::info!("Persistence worker shutting down");
                        break;
                    }
                }
            }
        }
    }

    /// Persist a batch of galaxies
    async fn persist_galaxies_batch(database: &Database, galaxy_names: Vec<String>) {
        if galaxy_names.is_empty() {
            return;
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for galaxy_name in galaxy_names {
            if let Some(galaxy) = Self::get_galaxy_for_persistence_static(&galaxy_name) {
                match database.save_galaxy_state(&galaxy_name, &galaxy).await {
                    Ok(_) => {
                        Self::clear_galaxy_dirty_flag_static(&galaxy_name);
                        success_count += 1;
                        log::debug!("Auto-persisted galaxy: {}", galaxy_name);
                    }
                    Err(e) => {
                        error_count += 1;
                        log::error!("Failed to auto-persist galaxy {}: {}", galaxy_name, e);
                    }
                }
            }
        }

        if success_count > 0 || error_count > 0 {
            log::info!(
                "Auto-persistence: {} saved, {} errors",
                success_count,
                error_count
            );
        }
    }

    /// Get list of galaxy names that need persistence
    fn get_dirty_galaxy_names(&self) -> Vec<String> {
        Self::get_dirty_galaxy_names_static()
    }

    fn get_dirty_galaxy_names_static() -> Vec<String> {
        #[cfg(feature = "bin")]
        {
            let galaxies = (**GALAXIES).lock().unwrap();
            galaxies
                .iter()
                .filter(|(_, galaxy)| galaxy.needs_persist())
                .map(|(name, _)| name.clone())
                .collect()
        }
        #[cfg(not(feature = "bin"))]
        Vec::new()
    }

    /// Get a galaxy for persistence (returns a clone to avoid holding the lock)
    fn get_galaxy_for_persistence(&self, galaxy_name: &str) -> Option<crate::Galaxy> {
        Self::get_galaxy_for_persistence_static(galaxy_name)
    }

    fn get_galaxy_for_persistence_static(galaxy_name: &str) -> Option<crate::Galaxy> {
        #[cfg(feature = "bin")]
        {
            let galaxies = (**GALAXIES).lock().unwrap();
            galaxies
                .get(galaxy_name)
                .filter(|g| g.needs_persist())
                .cloned()
        }
        #[cfg(not(feature = "bin"))]
        None
    }

    /// Clear the dirty flag for a galaxy
    fn clear_galaxy_dirty_flag(&self, galaxy_name: &str) {
        Self::clear_galaxy_dirty_flag_static(galaxy_name);
    }

    fn clear_galaxy_dirty_flag_static(galaxy_name: &str) {
        #[cfg(feature = "bin")]
        {
            let mut galaxies = (**GALAXIES).lock().unwrap();
            if let Some(galaxy) = galaxies.get_mut(galaxy_name) {
                galaxy.clear_dirty_flag();
            }
        }
    }
}

/// No-op persistence manager for when db feature is disabled
#[cfg(not(feature = "db"))]
pub struct PersistenceManager;

#[cfg(not(feature = "db"))]
impl PersistenceManager {
    pub async fn new(_config: PersistenceConfig) -> Result<Self, ()> {
        Ok(Self)
    }

    pub async fn save_all_dirty(&self) -> Result<usize, ()> {
        Ok(0)
    }

    pub async fn save_galaxy(&self, _galaxy_name: &str) -> Result<bool, ()> {
        Ok(false)
    }

    pub async fn load_galaxy(&self, _galaxy_name: &str) -> Result<bool, ()> {
        Ok(false)
    }

    pub async fn galaxy_exists_in_db(&self, _galaxy_name: &str) -> Result<bool, ()> {
        Ok(false)
    }

    pub async fn shutdown(&self) -> Result<(), ()> {
        Ok(())
    }
}
