use rand::Rng;
use std::collections::HashMap; // Add this line to import the `Rng` trait from the `rand` crate

/// An Island in the world
struct Island {
    buildings: Vec<Box<dyn Building>>,
}

/// Define the interface for a building
pub trait Building {
    fn name(&self) -> &str;
    fn level(&self) -> usize;
    fn score(&self) -> usize {
        self.level()
    }
}

struct GoldPit {
    level: usize,
}

impl GoldPit {
    pub fn new(level: usize) -> Self {
        Self { level }
    }
}

impl Building for GoldPit {
    fn name(&self) -> &str {
        "goldpit"
    }

    fn level(&self) -> usize {
        self.level
    }
}

impl Island {
    /// Create a new island
    pub fn new(config: &IslandConfig) -> Self {
        let buildings = config
            .buildings
            .iter()
            .map(|b| match b.name.as_str() {
                "goldpit" => {
                    Box::new(GoldPit::new(b.starting_level.unwrap_or(0))) as Box<dyn Building>
                }
                _ => panic!("Building not found"),
            })
            .collect();

        Self { buildings }
    }

    /// Get the score of an island
    pub fn score(&self) -> usize {
        self.buildings.iter().map(|b| b.score()).sum()
    }
}

/// Configuration for the creation of an island
pub struct IslandConfig {
    /// List of buildings that will be built on the island
    pub buildings: Vec<BuildingConfig>,

    /// Starting resources for the island
    pub resources: HashMap<String, usize>,
}

pub struct BuildingConfig {
    /// Name of the building
    /// Must be one of the predefined types
    pub name: String,

    /// Starting level for this type of building
    /// If not provided it is 0
    pub starting_level: Option<usize>,
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

    /// Island Config
    pub islands: IslandConfig,
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
            let island = Island::new(&config.islands);
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
