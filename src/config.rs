use indexmap::IndexMap;
/// Contains all the config structs
///
/// This is everything used externally to configure the galaxy
use serde::Deserialize;
use std::collections::HashMap;

use crate::Cost;

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
    pub structures: IndexMap<String, StructureConfig>,

    /// Starting resources for the system
    pub resources: HashMap<String, usize>,
}

/// Production Configuration.
///
/// These are all in production per hour (3600 ticks).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ProductionConfig {
    pub metal: Option<usize>,
    pub crew: Option<usize>,
    pub water: Option<usize>,
}

/// Storage Configuration.
///
/// Stores resource limit for storage.
pub type StorageConfig = ProductionConfig;

#[derive(Debug, Default, Deserialize)]
pub struct StructureConfig {
    /// Description of the structure.
    pub description: Option<String>,

    /// Starting level for this type of structure
    /// If not provided it is 0
    pub starting_level: Option<usize>,

    /// Used to specify how many of each resource is produced
    ///
    /// The number of ticks needed to produce the resource at a level
    pub production: Option<Vec<ProductionConfig>>,

    /// Used as a multiplier for the production.
    ///
    /// The highest given production value will be multiplied by this value.
    pub production_multiplier: Option<f64>,

    /// The amount of storage available at the starting level.
    pub storage: Option<Vec<StorageConfig>>,

    /// Used as a multiplier for the storage.
    pub storage_multiplier: Option<f64>,

    /// Cost for each level
    /// They are the costs to level up from the current level to the next level
    /// The first element is the cost to level up from 0 to 1
    pub cost: Vec<HashMap<String, usize>>,

    /// Used as a multiplier for the cost.
    ///
    /// The highest given cost value will be multiplied by this value.
    pub cost_multiplier: Option<f64>,
}

impl GalaxyConfig {
    /// Get the production for a single structure at a given level.
    pub fn get_structure_production(&self, structure: &str, level: usize) -> ProductionConfig {
        if let Some(structure) = self.systems.structures.get(&structure.to_lowercase()) {
            structure.get_production(level)
        } else {
            ProductionConfig::default()
        }
    }

    /// Get the storage for a single structure at a given level.
    pub fn get_structure_storage(&self, structure: &str, level: usize) -> StorageConfig {
        if let Some(structure) = self.systems.structures.get(&structure.to_lowercase()) {
            structure.get_storage(level)
        } else {
            StorageConfig::default()
        }
    }
}

impl StructureConfig {
    /// Get the cost to build this structure at a given level
    pub fn get_cost(&self, level: usize) -> Cost {
        // Adjust for the starting level
        let index = if let Some(lvl) = self.starting_level {
            level - lvl
        } else {
            level
        };
        if index < self.cost.len() {
            Cost::from_map(&self.cost[index])
        } else if let Some(multiplier) = self.cost_multiplier {
            // Get the last entry in the cost vector and it's index
            let (last_level, last_cost) = self.cost.iter().enumerate().last().unwrap();
            let exponent = index - last_level;
            let multiplier = multiplier.powi(exponent as i32);
            let mut cost = last_cost.clone();
            for (_, value) in cost.iter_mut() {
                *value = (*value as f64 * multiplier) as usize;
            }
            Cost::from_map(&cost)
        } else {
            panic!("No cost found for level {}", level);
        }
    }

    /// Get the production for this structure at a given level.
    pub fn get_production(&self, level: usize) -> ProductionConfig {
        // Adjust for the starting level
        let index = if let Some(lvl) = self.starting_level {
            level - lvl
        } else {
            level
        };
        if let Some(production) = &self.production {
            if index < production.len() {
                production[index].clone()
            } else if let Some(multiplier) = self.production_multiplier {
                // Get the last entry in the production vector and its index
                let (last_level, last_production) = production.iter().enumerate().last().unwrap();
                let exponent = index - last_level;
                let multiplier = multiplier.powi(exponent as i32);
                let mut production = last_production.clone();
                if let Some(metal) = production.metal {
                    production.metal = Some((metal as f64 * multiplier) as usize);
                }
                if let Some(crew) = production.crew {
                    production.crew = Some((crew as f64 * multiplier) as usize);
                }
                if let Some(water) = production.water {
                    production.water = Some((water as f64 * multiplier) as usize);
                }
                production.clone()
            } else {
                panic!("No production found for level {}", level);
            }
        } else {
            ProductionConfig::default()
        }
    }

    /// Get the storage for this structure at a given level.
    pub fn get_storage(&self, level: usize) -> StorageConfig {
        // Adjust for the starting level
        let index = if let Some(lvl) = self.starting_level {
            level - lvl
        } else {
            level
        };
        if let Some(storage) = &self.storage {
            if index < storage.len() {
                storage[index].clone()
            } else if let Some(multiplier) = self.storage_multiplier {
                // Get the last entry in the storage vector and its index
                let (last_level, last_storage) = storage.iter().enumerate().last().unwrap();
                let exponent = index - last_level;
                let multiplier = multiplier.powi(exponent as i32);
                let mut storage = last_storage.clone();
                if let Some(metal) = storage.metal {
                    storage.metal = Some((metal as f64 * multiplier) as usize);
                }
                if let Some(crew) = storage.crew {
                    storage.crew = Some((crew as f64 * multiplier) as usize);
                }
                if let Some(water) = storage.water {
                    storage.water = Some((water as f64 * multiplier) as usize);
                }
                storage.clone()
            } else {
                panic!("No storage found for level {}", level);
            }
        } else {
            StorageConfig::default()
        }
    }
}
