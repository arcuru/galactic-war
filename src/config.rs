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
pub struct Production {
    pub resource: String,
    pub production: Vec<usize>,
}

#[derive(Debug, Default, Deserialize)]
pub struct StructureConfig {
    /// Starting level for this type of structure
    /// If not provided it is 0
    pub starting_level: Option<usize>,

    /// Used to specify how many of each resource is produced
    ///
    /// The number of ticks needed to produce the resource at a level
    pub production: Option<Vec<Production>>,

    /// Used as a multiplier for the production.
    ///
    /// The highest given production value will be multiplied by this value.
    pub production_multiplier: Option<f64>,

    /// Cost for each level
    /// They are the costs to level up from the current level to the next level
    /// The first element is the cost to level up from 0 to 1
    pub cost: Vec<HashMap<String, usize>>,

    /// Used as a multiplier for the cost.
    ///
    /// The highest given cost value will be multiplied by this value.
    pub cost_multiplier: Option<f64>,
}

impl StructureConfig {
    /// Get the cost to build this structure at a given level
    pub fn get_cost(&self, level: usize) -> HashMap<String, usize> {
        // Adjust for the starting level
        let index = if let Some(lvl) = self.starting_level {
            level - lvl
        } else {
            level
        };
        if index < self.cost.len() {
            self.cost[index].clone()
        } else if let Some(multiplier) = self.cost_multiplier {
            // Get the last entry in the cost vector and it's index
            let (last_level, last_cost) = self.cost.iter().enumerate().last().unwrap();
            let exponent = index - last_level;
            let multiplier = multiplier.powi(exponent as i32);
            let mut cost = last_cost.clone();
            for (_, value) in cost.iter_mut() {
                *value = (*value as f64 * multiplier) as usize;
            }
            cost
        } else {
            panic!("No cost found for level {}", level);
        }
    }
}
