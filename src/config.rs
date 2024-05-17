/// Contains all the config structs
///
/// This is everything used externally to configure the world
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration for the world
#[derive(Debug, Deserialize, Default)]
pub struct WorldConfig {
    /// Static Island Count
    pub island_count: usize,

    /// World size
    pub size: WorldSize,

    /// Island Config
    pub islands: IslandConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorldSize {
    pub x: usize,
    pub y: usize,
}

/// Configuration for the creation of an island
#[derive(Debug, Default, Deserialize)]
pub struct IslandConfig {
    /// List of buildings that will be built on the island
    pub buildings: HashMap<String, BuildingConfig>,

    /// Starting resources for the island
    pub resources: HashMap<String, usize>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BuildingConfig {
    /// Starting level for this type of building
    /// If not provided it is 0
    pub starting_level: Option<usize>,

    /// Used for Gold/Stone/Lumber
    ///
    /// The number of ticks needed to produce the resource at a level
    pub production: Option<Vec<usize>>,

    /// Cost for each level
    /// They are the costs to level up from the current level to the next level
    /// The first element is the cost to level up from 0 to 1
    pub cost: Vec<HashMap<String, usize>>,
}
