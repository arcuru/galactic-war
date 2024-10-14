use indexmap::IndexMap;
use rand::Rng;
use std::collections::HashMap;
use system::EventInfo;

pub mod config;
mod system;
use crate::config::GalaxyConfig;
use crate::system::System;

pub use crate::system::{Event, EventCallback, StructureType};

#[derive(Debug)]
pub struct Galaxy {
    /// Cached configuration for the galaxy
    config: GalaxyConfig,

    /// All the systems in the galaxy
    systems: HashMap<Coords, System>,

    /// The most recent tick
    ///
    /// This is used to ensure that events can only arrive in order
    tick: usize,
}

/// Production of a system.
///
/// Each value is the amount of resources produced per 3600 ticks (hour).
pub type SystemProduction = Resources;

/// Resources in a system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Resources {
    pub metal: usize,
    pub crew: usize,
    pub water: usize,
}

impl std::ops::Add for Resources {
    type Output = Resources;

    fn add(self, other: Resources) -> Resources {
        Resources {
            metal: self.metal + other.metal,
            crew: self.crew + other.crew,
            water: self.water + other.water,
        }
    }
}

impl std::ops::Sub for Resources {
    type Output = Resources;

    fn sub(self, other: Resources) -> Resources {
        Resources {
            metal: self.metal - other.metal,
            crew: self.crew - other.crew,
            water: self.water - other.water,
        }
    }
}

impl std::ops::Mul<f64> for Resources {
    type Output = Resources;

    fn mul(self, other: f64) -> Resources {
        Resources {
            metal: (self.metal as f64 * other).round() as usize,
            crew: (self.crew as f64 * other).round() as usize,
            water: (self.water as f64 * other).round() as usize,
        }
    }
}

// Order is defined only if all the values are greater/less than the other.
impl std::cmp::PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.metal > other.metal && self.crew > other.crew && self.water > other.water {
            Some(std::cmp::Ordering::Greater)
        } else if self.metal < other.metal && self.crew < other.crew && self.water < other.water {
            Some(std::cmp::Ordering::Less)
        } else if self.metal == other.metal && self.crew == other.crew && self.water == other.water
        {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SystemInfo {
    /// Computed score of the system
    pub score: usize,

    /// Resources in the system.
    pub resources: Resources,

    /// Production of the system.
    ///
    /// Given in units per hour (3600 ticks).
    pub production: SystemProduction,

    /// Structure levels
    pub structures: IndexMap<StructureType, usize>,

    /// Events in flight
    ///
    /// Next resource, unit builds, incoming attacks, etc.
    pub events: Vec<EventInfo>,
}

/// Struct to hold the cost for a build
#[derive(Clone, Debug, Default)]
pub struct Cost {
    pub resources: Resources,
    pub ticks: usize,
}

impl Cost {
    /// Create a Cost from a HashMap
    ///
    /// This converts the format seen in the config files to an actual Cost struct
    pub fn from_map(cost: &HashMap<String, usize>) -> Cost {
        Cost {
            resources: Resources {
                metal: *cost.get("metal").unwrap_or(&0),
                water: *cost.get("water").unwrap_or(&0),
                crew: *cost.get("crew").unwrap_or(&0),
            },
            ticks: *cost.get("ticks").unwrap_or(&0),
        }
    }
}

impl std::ops::Mul<f64> for Cost {
    type Output = Cost;

    fn mul(self, rhs: f64) -> Self::Output {
        Cost {
            resources: self.resources * rhs,
            ticks: (self.ticks as f64 * rhs).round() as usize,
        }
    }
}

/// Info for a specific structure
///
/// Lots of details are optional, as they don't all apply to all structures
#[derive(Clone, Debug, Default)]
pub struct StructureInfo {
    /// Level of the structure.
    pub level: usize,
    /// Production of the structure, if any.
    pub production: Option<SystemProduction>,
    /// Things that this structure can build, if any.
    pub builds: Option<IndexMap<StructureType, Cost>>,
}

/// Info to use in return values
///
/// Stores SystemInfo and StructureInfo
/// TODO: Is this really the best approach? If it's just two types, may want to just split the API.
#[derive(Clone, Debug)]
pub enum Details {
    System(SystemInfo),
    Structure(StructureInfo),
}

impl Galaxy {
    /// Create a new Galaxy
    ///
    /// Uses a custom configuration struct to set up all the details
    pub fn new(config: GalaxyConfig, initial_tick: usize) -> Self {
        let mut systems = HashMap::new();
        let mut rng = rand::thread_rng();
        for _ in 0..config.system_count {
            // Create a new island at a random location in the 2d space
            let x: usize = rng.gen_range(0..=config.size.x);
            let y: usize = rng.gen_range(0..=config.size.y);
            if systems.contains_key(&(x, y).into()) {
                // Already have a system here, try again
                continue;
            }
            let system = System::new(initial_tick, &config.systems, &config);
            systems.insert((x, y).into(), system);
        }
        Self {
            config,
            systems,
            tick: initial_tick,
        }
    }

    /// Retrieve the details of a system, possibly scoped to a specific structure
    pub fn get_details(
        &mut self,
        tick: usize,
        coords: Coords,
        structure: Option<StructureType>,
    ) -> Result<Details, String> {
        self.update_tick(tick)?;
        let system = self.systems.get_mut(&coords).unwrap();
        system.get_details(tick, &self.config, structure)
    }

    /// Get a pointer to the Config
    pub fn get_config(&self) -> &GalaxyConfig {
        &self.config
    }

    /// Return basic stats about the Galaxy
    pub fn stats(&mut self, tick: usize) -> Result<String, String> {
        self.update_tick(tick)?;
        let mut stats = format!("System count: {}\n", self.config.system_count);
        for (coords, system) in self.systems.iter_mut() {
            stats.push_str(&format!(
                "System at {:?} has score {} and metal {}\n",
                coords,
                system.score(tick, &self.config),
                system.metal(tick, &self.config),
            ));
        }
        Ok(stats)
    }

    /// Retrieve the full list of systems
    pub fn systems(&self) -> &HashMap<Coords, System> {
        &self.systems
    }

    /// Build a structure in a system
    pub fn build(
        &mut self,
        tick: usize,
        coords: Coords,
        structure: StructureType,
    ) -> Result<Event, String> {
        self.update_tick(tick)?;
        let system = self.systems.get_mut(&coords).unwrap();
        system.build(tick, &self.config, structure)
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

/// Coords for systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coords {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Coords {
    fn from(coords: (usize, usize)) -> Self {
        Coords {
            x: coords.0,
            y: coords.1,
        }
    }
}
