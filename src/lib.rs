use rand::Rng;
use std::collections::HashMap;
use system::EventInfo;

pub mod config;
mod system;
use crate::config::GalaxyConfig;
use crate::system::System;

pub use crate::system::{Event, EventCallback, StructureType};

#[derive(Debug)]
pub struct Galaxy {
    /// Cached configuration for the galaxy
    config: GalaxyConfig,

    /// All the systems in the galaxy
    systems: HashMap<(usize, usize), System>,

    /// The most recent tick
    ///
    /// This is used to ensure that events can only arrive in order
    tick: usize,
}

/// Production of a system
///
/// Each value is the number of ticks needed to produce each resource
#[derive(Debug, Default)]
pub struct SystemProduction {
    pub gold: usize,
    pub lumber: usize,
    pub stone: usize,
}

#[derive(Debug, Default)]
pub struct SystemInfo {
    pub score: usize,
    pub gold: usize,
    pub lumber: usize,
    pub stone: usize,
    pub production: SystemProduction,

    /// Structure levels
    pub structures: HashMap<StructureType, usize>,

    /// Events in flight
    ///
    /// Next resource, unit builds, incoming attacks, etc.
    pub events: Vec<EventInfo>,
}

/// Struct to hold the cost for a build
#[derive(Debug, Default)]
pub struct Cost {
    pub gold: usize,
    pub lumber: usize,
    pub stone: usize,
    pub ticks: usize,
}

/// Info for a specific structure
///
/// Lots of details are optional, as they don't all apply to all structures
#[derive(Debug, Default)]
pub struct StructureInfo {
    pub level: usize,
    pub production: Option<SystemProduction>,
    pub builds: Option<HashMap<StructureType, Cost>>,
}

/// Info to use in return values
///
/// Stores SystemInfo and StructureInfo
/// TODO: Is this really the best approach? If it's just two types, may want to just split the API.
#[derive(Debug)]
pub enum Details {
    System(SystemInfo),
    Structure(StructureInfo),
}

impl Galaxy {
    /// Create a new Galaxy
    ///
    /// Uses a custom configuration struct to set up all the details
    pub fn new(config: GalaxyConfig, initial_tick: usize) -> Self {
        let mut systems = HashMap::new();
        for _ in 0..config.system_count {
            // Create a new island at a random location in the 2d space

            let mut rng = rand::thread_rng();
            let x: usize = rng.gen_range(0..=config.size.x);
            let y: usize = rng.gen_range(0..=config.size.y);
            let system = System::new(initial_tick, &config.systems, &config);
            systems.insert((x, y), system);
        }
        Self {
            config,
            systems,
            tick: initial_tick,
        }
    }

    /// Retrieve the details of an island, possibly scoped to a specific structure
    pub fn get_details(
        &mut self,
        tick: usize,
        (x, y): (usize, usize),
        structure: Option<StructureType>,
    ) -> Result<Details, String> {
        self.update_tick(tick)?;
        let island = self.systems.get_mut(&(x, y)).unwrap();
        island.get_details(tick, &self.config, structure)
    }

    /// Return basic stats about the Galaxy
    pub fn stats(&mut self, tick: usize) -> Result<String, String> {
        self.update_tick(tick)?;
        let mut stats = format!("System count: {}\n", self.config.system_count);
        for (coords, system) in self.systems.iter_mut() {
            stats.push_str(&format!(
                "System at {:?} has score {} and gold {}\n",
                coords,
                system.score(tick, &self.config),
                system.gold(tick, &self.config),
            ));
        }
        Ok(stats)
    }

    /// Retrieve the full list of systems
    pub fn systems(&self) -> &HashMap<(usize, usize), System> {
        &self.systems
    }

    /// Build a structure in a system
    pub fn build(
        &mut self,
        tick: usize,
        (x, y): (usize, usize),
        structure: StructureType,
    ) -> Result<Event, String> {
        self.update_tick(tick)?;
        let system = self.systems.get_mut(&(x, y)).unwrap();
        system.build(tick, &self.config, structure)
    }

    /// Update the current tick, and verify we are not going back in time
    fn update_tick(&mut self, tick: usize) -> Result<(), String> {
        if tick < self.tick {
            return Err("Tick is out of order".to_string());
        }
        self.tick = tick;
        Ok(())
    }
}
