use indexmap::IndexMap;
/// Contains all the config structs
///
/// This is everything used externally to configure the galaxy
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Cost, Resources};

/// Configuration for the Galaxy
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GalaxyConfig {
    /// Static System Count
    pub system_count: usize,

    /// Galaxy size
    pub size: GalaxySize,

    /// System Config
    pub systems: SystemConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GalaxySize {
    pub x: usize,
    pub y: usize,
}

/// Configuration for the creation of an system
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct SystemConfig {
    /// List of structures that will be built on the system
    pub structures: IndexMap<String, StructureConfig>,

    /// Starting resources for the system
    pub resources: HashMap<String, usize>,
}

/// Production Configuration.
///
/// These are all in production per hour (3600 ticks).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProductionConfig {
    /// Used as a multiplier for the production.
    pub multiplier: Option<f64>,

    /// Metal produced by lvl 1
    #[serde(default)]
    pub metal: usize,

    /// Crew produced by lvl 1
    #[serde(default)]
    pub crew: usize,

    /// Water produced by lvl 1
    #[serde(default)]
    pub water: usize,
}

/// Cost Configuration.
///
/// These are the costs for building a lvl 1 structure and a multiplier for higher levels.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CostConfig {
    /// Used as a multiplier for the production.
    pub multiplier: Option<f64>,
    /// Time to build the structure
    #[serde(default)]
    pub time: usize,

    /// Metal cost for lvl 1
    #[serde(default)]
    pub metal: usize,

    /// Crew cost for lvl 1
    #[serde(default)]
    pub crew: usize,

    /// Water cost for lvl 1
    #[serde(default)]
    pub water: usize,
}

/// Storage Configuration.
///
/// Stores resource limit for storage.
pub type StorageConfig = ProductionConfig;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct StructureConfig {
    /// Description of the structure.
    pub description: Option<String>,

    /// Starting level for this type of structure
    #[serde(default)]
    pub starting_level: usize,

    /// Default multiplier for the structure, applying to all multiplier settings (Production, Cost, Storage)
    pub multiplier: Option<f64>,

    /// Used to specify how many of each resource is produced
    pub production: Option<ProductionConfig>,

    /// The amount of storage available at the starting level.
    pub storage: Option<StorageConfig>,

    /// Cost for each level
    /// They are the costs to build the next level
    pub cost: Option<CostConfig>,
}

impl GalaxyConfig {
    /// Get the production for a single structure at a given level.
    pub fn get_structure_production(&self, structure: &str, level: usize) -> Resources {
        if let Some(structure) = self.systems.structures.get(&structure.to_lowercase()) {
            structure.get_production(level)
        } else {
            Resources::default()
        }
    }

    /// Get the storage for a single structure at a given level.
    pub fn get_structure_storage(&self, structure: &str, level: usize) -> Resources {
        if let Some(structure) = self.systems.structures.get(&structure.to_lowercase()) {
            structure.get_storage(level)
        } else {
            Resources::default()
        }
    }
}

impl StructureConfig {
    /// Get the cost to build this structure at a given level
    pub fn get_cost(&self, level: usize) -> Cost {
        if level == 0 || self.cost.is_none() {
            return Cost::default();
        }
        let cost_config = self.cost.as_ref().unwrap();
        let cost = Cost {
            resources: Resources {
                metal: cost_config.metal,
                crew: cost_config.crew,
                water: cost_config.water,
            },
            ticks: cost_config.time,
        };
        let multiplier = cost_config
            .multiplier
            .unwrap_or(self.multiplier.unwrap_or(1.0));
        // The cost is cost * (multiplier ^ (level - 1)))
        cost * multiplier.powi((level - 1) as i32)
    }

    /// Get the production for this structure at a given level.
    pub fn get_production(&self, level: usize) -> Resources {
        if level == 0 || self.production.is_none() {
            return Resources::default();
        }
        let production_config = self.production.as_ref().unwrap();
        let production = Resources {
            metal: production_config.metal,
            crew: production_config.crew,
            water: production_config.water,
        };
        let multiplier = production_config
            .multiplier
            .unwrap_or(self.multiplier.unwrap_or(1.0));
        // The production is production * (multiplier ^ (level - 1)))
        production * multiplier.powi((level - 1) as i32)
    }

    /// Get the storage for this structure at a given level.
    pub fn get_storage(&self, level: usize) -> Resources {
        if level == 0 || self.storage.is_none() {
            return Resources::default();
        }
        let storage_config = self.storage.as_ref().unwrap();
        let storage = Resources {
            metal: storage_config.metal,
            crew: storage_config.crew,
            water: storage_config.water,
        };
        let multiplier = storage_config
            .multiplier
            .unwrap_or(self.multiplier.unwrap_or(1.0));
        // The storage is storage * (multiplier ^ (level - 1)))
        storage * multiplier.powi((level - 1) as i32)
    }
}
