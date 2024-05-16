use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

mod island;
use crate::island::Island;

pub use crate::island::IslandConfig;

pub struct World {
    config: WorldConfig,
    islands: HashMap<(usize, usize), Island>,
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
        Self { config, islands }
    }

    /// Return basic stats about the World
    pub fn stats(&mut self, tick: usize) -> String {
        let mut stats = format!("Island count: {}\n", self.config.island_count);
        for (coords, island) in self.islands.iter_mut() {
            stats.push_str(&format!(
                "Island at {:?} has score {} and gold {}\n",
                coords,
                island.score(tick, &self.config),
                island.gold(tick, &self.config),
            ));
        }
        stats
    }
}
