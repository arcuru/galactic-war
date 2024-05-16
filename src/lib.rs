use rand::Rng;
use std::collections::HashMap; // Add this line to import the `Rng` trait from the `rand` crate

/// An Island in the world
struct Island {}

impl Island {
    /// Create a new island
    pub fn new() -> Self {
        Self {}
    }

    /// Get the score of an island
    pub fn score(&self) -> usize {
        0
    }
}

pub struct World {
    config: WorldConfig,
    islands: HashMap<(usize, usize), Island>,
}

/// Configuration for the world
pub struct WorldConfig {
    /// Static Island Count
    pub island_count: usize,

    /// World size
    pub size: (usize, usize),
}

impl World {
    /// Create a new World
    ///
    /// Uses a custom configuration struct to set up all the details
    pub fn new(config: WorldConfig) -> Self {
        let mut islands = HashMap::new();
        for _ in 0..config.island_count {
            // Create a new island at a random location in the 2d space

            let mut rng = rand::thread_rng();
            let x: usize = rng.gen_range(0..=config.size.0);
            let y: usize = rng.gen_range(0..=config.size.1);
            let island = Island::new();
            islands.insert((x, y), island);
        }
        Self { config, islands }
    }

    /// Return basic stats about the World
    pub fn stats(&self) -> String {
        let mut stats = format!("Island count: {}\n", self.config.island_count);
        for (pos, island) in self.islands.iter() {
            stats.push_str(&format!(
                "Island at {:?} has score {}\n",
                pos,
                island.score()
            ));
        }
        stats
    }
}
