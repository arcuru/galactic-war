use island::EventInfo;
use rand::Rng;
use std::collections::HashMap;

pub mod config;
mod island;
use crate::config::WorldConfig;
use crate::island::Island;

pub use crate::island::{BuildingType, Event, EventCallback};

#[derive(Debug)]
pub struct World {
    /// Cached configuration for the world
    config: WorldConfig,

    /// All the islands in the world
    islands: HashMap<(usize, usize), Island>,

    /// The most recent tick
    ///
    /// This is used to ensure that events can only arrive in order
    tick: usize,
}

/// Production of an island
///
/// Each value is the number of ticks needed to produce each resource
#[derive(Debug, Default)]
pub struct IslandProduction {
    pub gold: usize,
    pub lumber: usize,
    pub stone: usize,
}

#[derive(Debug, Default)]
pub struct IslandInfo {
    pub score: usize,
    pub gold: usize,
    pub lumber: usize,
    pub stone: usize,
    pub production: IslandProduction,
    /// Building levels
    pub buildings: HashMap<BuildingType, usize>,

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

/// Info for a specific Building
///
/// Lots of details are optional, as they don't all apply to all buildings
#[derive(Debug, Default)]
pub struct BuildingInfo {
    pub level: usize,
    pub production: Option<IslandProduction>,
    pub builds: Option<HashMap<BuildingType, Cost>>,
}

/// Info to use in return values
///
/// Stores IslandInfo, and will store details about buildings in the future
/// TODO: Is this really the best approach? If it's just two types, may want to just split the API.
#[derive(Debug)]
pub enum Details {
    Island(IslandInfo),
    Building(BuildingInfo),
}

impl World {
    /// Create a new World
    ///
    /// Uses a custom configuration struct to set up all the details
    pub fn new(config: WorldConfig, initial_tick: usize) -> Self {
        let mut islands = HashMap::new();
        for _ in 0..config.island_count {
            // Create a new island at a random location in the 2d space

            let mut rng = rand::thread_rng();
            let x: usize = rng.gen_range(0..=config.size.x);
            let y: usize = rng.gen_range(0..=config.size.y);
            let island = Island::new(initial_tick, &config.islands, &config);
            islands.insert((x, y), island);
        }
        Self {
            config,
            islands,
            tick: initial_tick,
        }
    }

    /// Retrieve the details of an island, possibly scoped to a specific building
    pub fn get_details(
        &mut self,
        tick: usize,
        (x, y): (usize, usize),
        building: Option<BuildingType>,
    ) -> Result<Details, String> {
        self.update_tick(tick)?;
        let island = self.islands.get_mut(&(x, y)).unwrap();
        island.get_details(tick, &self.config, building)
    }

    /// Return basic stats about the World
    pub fn stats(&mut self, tick: usize) -> Result<String, String> {
        self.update_tick(tick)?;
        let mut stats = format!("Island count: {}\n", self.config.island_count);
        for (coords, island) in self.islands.iter_mut() {
            stats.push_str(&format!(
                "Island at {:?} has score {} and gold {}\n",
                coords,
                island.score(tick, &self.config),
                island.gold(tick, &self.config),
            ));
        }
        Ok(stats)
    }

    /// Retrieve the full list of islands
    pub fn islands(&self) -> &HashMap<(usize, usize), Island> {
        &self.islands
    }

    /// Build a building on an island
    pub fn build(
        &mut self,
        tick: usize,
        (x, y): (usize, usize),
        building: BuildingType,
    ) -> Result<Event, String> {
        self.update_tick(tick)?;
        let island = self.islands.get_mut(&(x, y)).unwrap();
        island.build(tick, &self.config, building)
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
