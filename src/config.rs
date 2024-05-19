/// Contains all the config structs
///
/// This is everything used externally to configure the galaxy
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration for the Galaxy
#[derive(Debug, Deserialize, Default)]
pub struct GalaxyConfig {
    /// Static System Count
    pub system_count: usize,

    /// Galaxy size
    pub size: GalaxySize,

    /// System Config
    pub systems: SystemConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct GalaxySize {
    pub x: usize,
    pub y: usize,
}

/// Configuration for the creation of an system
#[derive(Debug, Default, Deserialize)]
pub struct SystemConfig {
    /// List of structures that will be built on the system
    pub structures: HashMap<String, StructureConfig>,

    /// Starting resources for the system
    pub resources: HashMap<String, usize>,
}

#[derive(Debug, Default, Deserialize)]
pub struct StructureConfig {
    /// Starting level for this type of structure
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
