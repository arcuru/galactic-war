use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

mod island;
use crate::island::Island;

pub use crate::island::{BuildingType, IslandConfig};

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

/// Configuration for the world
#[derive(Deserialize)]
pub struct WorldConfig {
    /// Static Island Count
    pub island_count: usize,

    /// World size
    pub size: WorldSize,

    /// Island Config
    pub islands: IslandConfig,
}

#[derive(Deserialize)]
pub struct WorldSize {
    pub x: usize,
    pub y: usize,
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
            let island = Island::new(initial_tick, &config.islands);
            islands.insert((x, y), island);
        }
        Self {
            config,
            islands,
            tick: initial_tick,
        }
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
    ) -> Result<(), String> {
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
