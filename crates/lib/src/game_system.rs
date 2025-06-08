use core::panic;
use indexmap::IndexMap;

use crate::config::{GalaxyConfig, StructureConfig, SystemConfig};
use crate::{Cost, Details, Resources, StructureInfo, SystemInfo, SystemProduction};
use std::fmt;
use std::str::FromStr;

/// An System in the Galaxy
#[derive(Debug, Default, Clone)]
pub struct System {
    /// Current tick this system state represents
    current_tick: usize,

    /// List of events that are happening in the system
    events: Vec<Event>,

    /// Current resources available in the system.
    resources: Resources,

    /// List of structures in the system.
    structures: Vec<Structure>,
}

#[derive(Debug, Clone)]
struct Structure {
    name: StructureType,
    level: usize,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub completion: usize,
    pub action: EventCallback,
    pub structure: Option<StructureType>,
}

pub type EventInfo = Event;

#[derive(Clone, Debug, PartialEq)]
pub enum EventCallback {
    Build,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Colony,
    AsteroidMine,
    WaterHarvester,
    Hatchery,
    StorageDepot,
}

impl fmt::Display for StructureType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for StructureType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "colony" => Ok(StructureType::Colony),
            "asteroidmine" => Ok(StructureType::AsteroidMine),
            "waterharvester" => Ok(StructureType::WaterHarvester),
            "hatchery" => Ok(StructureType::Hatchery),
            "storagedepot" => Ok(StructureType::StorageDepot),
            _ => Err(()),
        }
    }
}

impl System {
    /// Create a new system
    ///
    /// This takes an SystemConfig because there may be multiple system types in future
    pub fn new(tick: usize, system_config: &SystemConfig, _galaxy_config: &GalaxyConfig) -> Self {
        let resources = Resources {
            metal: *system_config.resources.get("metal").unwrap_or(&0),
            water: *system_config.resources.get("water").unwrap_or(&0),
            crew: *system_config.resources.get("crew").unwrap_or(&0),
        };
        let mut structures = Vec::new();
        for (name, structure) in system_config.structures.iter() {
            structures.push(Structure {
                name: StructureType::from_str(name).unwrap(),
                level: structure.starting_level,
            });
        }

        Self {
            current_tick: tick,
            events: Vec::new(),
            resources,
            structures,
        }
    }

    /// Create a system from database data (only available with db feature)
    #[cfg(feature = "db")]
    pub fn from_database(
        current_tick: usize,
        resources: Resources,
        structures: Vec<(StructureType, usize)>,
        events: Vec<Event>,
    ) -> Self {
        let structures = structures
            .into_iter()
            .map(|(name, level)| Structure { name, level })
            .collect();
        Self {
            current_tick,
            events,
            resources,
            structures,
        }
    }

    /// Get system resources (for database persistence)
    pub fn get_resources(&self) -> Resources {
        self.resources
    }

    /// Get current tick (for database persistence)
    pub fn get_current_tick(&self) -> usize {
        self.current_tick
    }

    /// Get system structures as a list of (type, level) pairs (for database persistence)
    pub fn get_structures(&self) -> Vec<(StructureType, usize)> {
        self.structures.iter().map(|s| (s.name, s.level)).collect()
    }

    /// Get system events (for database persistence)
    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    /// Get the index of the structure by type
    ///
    /// The structure may not exist, so it returns an Option
    fn structure(&self, structure: StructureType) -> Option<usize> {
        self.structures.iter().position(|b| b.name == structure)
    }

    /// Get the level of a structure
    fn structure_level(&self, structure: StructureType) -> usize {
        if let Some(index) = self.structure(structure) {
            self.structures[index].level
        } else {
            0
        }
    }

    /// Get the structure configuration from the GalaxyConfig
    fn get_structure_config(
        galaxy_config: &GalaxyConfig,
        structure: StructureType,
    ) -> &StructureConfig {
        galaxy_config
            .systems
            .structures
            .get(&structure.to_string().to_lowercase())
            .unwrap()
    }

    /// Get the production of the system
    pub fn get_production(
        &mut self,
        _tick: usize,
        galaxy_config: &GalaxyConfig,
    ) -> SystemProduction {
        let mut production = SystemProduction {
            metal: 0,
            crew: 0,
            water: 0,
        };
        for structure in self.structures.iter() {
            let production_config = galaxy_config
                .get_structure_production(&structure.name.to_string(), structure.level);
            production = production + production_config;
        }
        production
    }

    /// Get the available resource storage in the system.
    fn get_storage(&mut self, _tick: usize, galaxy_config: &GalaxyConfig) -> SystemProduction {
        let mut storage = Resources {
            metal: 0,
            crew: 0,
            water: 0,
        };
        for structure in self.structures.iter() {
            let storage_config =
                galaxy_config.get_structure_storage(&structure.name.to_string(), structure.level);
            storage = storage + storage_config;
        }
        storage
    }

    /// Callback for events
    ///
    /// This will process the event and update the state of the system.
    /// It will also create new events if needed.
    fn event_callback(&mut self, _tick: usize, _galaxy_config: &GalaxyConfig, event: Event) {
        // Check the completion time
        if event.completion > _tick {
            return;
        }
        match event.action {
            EventCallback::Build => {
                // Build the structure
                if let Some(structure) = event.structure {
                    let index = self.structure(structure).unwrap();
                    self.structures[index].level += 1;
                } else {
                    panic!("Structure event without StructureType");
                }
            }
        }
    }

    /// Get the current metal amount
    pub fn metal(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.update_to_tick(tick, galaxy_config);
        self.resources.metal
    }

    /// Get the current water amount
    pub fn water(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.update_to_tick(tick, galaxy_config);
        self.resources.water
    }

    /// Get the current crew count
    pub fn crew(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.update_to_tick(tick, galaxy_config);
        self.resources.crew
    }

    /// Get the current resources of the system
    pub fn resources(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> Resources {
        self.update_to_tick(tick, galaxy_config);
        self.resources
    }

    /// Check if there is an event that needs to be processed
    pub fn event_to_process(&mut self, tick: usize) -> bool {
        // Check if the first event is ready to be processed
        if let Some(event) = self.events.first() {
            event.completion <= tick
        } else {
            false
        }
    }

    /// Register a new event
    /// The event will be sorted by the completion time
    pub fn register_event(&mut self, event: Event) {
        self.events.push(event);
        self.events.sort_by_key(|e| e.completion)
    }

    /// Process all events that are expected to happen
    pub fn process_events(&mut self, tick: usize, galaxy_config: &GalaxyConfig) {
        while self.event_to_process(tick) {
            let events = self.events.clone();
            self.events.clear();
            for event in events.iter() {
                if event.completion <= tick {
                    self.event_callback(tick, galaxy_config, event.clone());
                } else {
                    self.register_event(event.clone());
                }
            }
        }
    }

    /// Updates pending events based on the current system state
    ///
    /// It's possible, as in the case of Building new structures, that the ETAs for
    /// events need to be updated. This updates events so their ETA is the minimum of the
    /// current ETA and the ETA with the new system state.
    pub fn update_events(&mut self, _tick: usize, _galaxy_config: &GalaxyConfig) {
        // Only Build events remain, so we don't need to update their timing
        // They have fixed completion times based on structure build costs
    }

    /// Update the system state to a given tick, calculating resource production
    pub fn update_to_tick(&mut self, new_tick: usize, galaxy_config: &GalaxyConfig) {
        if new_tick <= self.current_tick {
            return; // No time has passed or going backwards
        }

        // Process any events that should complete before the new tick
        self.process_events(new_tick, galaxy_config);

        // Calculate resource production for each structure
        let production = self.get_production(new_tick, galaxy_config);
        let storage = self.get_storage(new_tick, galaxy_config);

        // Calculate how many complete production cycles occurred for each resource
        let _tick_diff = new_tick - self.current_tick;

        // Metal production: if production is X per hour, then every (3600/X) ticks we get 1 metal
        if production.metal > 0 {
            let production_interval = 3600 / production.metal;
            let start_cycle = self.current_tick / production_interval;
            let end_cycle = new_tick / production_interval;
            let cycles_completed = end_cycle - start_cycle;
            self.resources.metal = (self.resources.metal + cycles_completed).min(storage.metal);
        }

        // Water production
        if production.water > 0 {
            let production_interval = 3600 / production.water;
            let start_cycle = self.current_tick / production_interval;
            let end_cycle = new_tick / production_interval;
            let cycles_completed = end_cycle - start_cycle;
            self.resources.water = (self.resources.water + cycles_completed).min(storage.water);
        }

        // Crew production
        if production.crew > 0 {
            let production_interval = 3600 / production.crew;
            let start_cycle = self.current_tick / production_interval;
            let end_cycle = new_tick / production_interval;
            let cycles_completed = end_cycle - start_cycle;
            self.resources.crew = (self.resources.crew + cycles_completed).min(storage.crew);
        }

        self.current_tick = new_tick;
    }

    /// Get the score of a system.
    ///
    /// The score is the summation of every level of every structure in the system.
    /// A structure with a level of 4 will contribute 1+2+3+4=10 to the score.
    pub fn score(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.update_to_tick(tick, galaxy_config);
        self.structures
            .iter()
            .map(|b| (1..=b.level).sum::<usize>())
            .sum()
    }

    /// Build a structure
    pub fn build(
        &mut self,
        tick: usize,
        galaxy_config: &GalaxyConfig,
        structure: StructureType,
    ) -> Result<Event, String> {
        self.update_to_tick(tick, galaxy_config);
        // Check if we're already building a structure, we can only build one at a time
        if self.events.iter().any(|e| e.structure.is_some()) {
            return Err("Already building a structure".to_string());
        }
        if self.structure(structure).is_some() {
            // Verify if the structure can be built
            let cost = &System::get_structure_config(galaxy_config, structure)
                .get_cost(self.structure_level(structure) + 1);
            if self.resources >= cost.resources {
                // Deduct the cost
                self.resources = self.resources - cost.resources;
                // Add a callback for the build completion
                let event = Event {
                    completion: tick + cost.ticks,
                    action: EventCallback::Build,
                    structure: Some(structure),
                };
                self.register_event(event.clone());
                Ok(event)
            } else {
                // Not enough resources
                Err("Not enough resources".to_string())
            }
        } else {
            Err("Structure not found".to_string())
        }
    }

    /// Get the details of the system
    pub fn get_details(
        &mut self,
        tick: usize,
        galaxy_config: &GalaxyConfig,
        structure: Option<StructureType>,
    ) -> Result<Details, String> {
        self.update_to_tick(tick, galaxy_config);
        if let Some(structure) = structure {
            let production_config = galaxy_config
                .get_structure_production(&structure.to_string(), self.structure_level(structure));
            let mut details = StructureInfo {
                level: self.structure_level(structure),
                production: Some(production_config),
                builds: None,
            };
            if structure == StructureType::Colony {
                let mut builds: IndexMap<StructureType, Cost> = Default::default();
                for structure in self.structures.iter() {
                    builds.insert(
                        structure.name,
                        System::get_structure_config(galaxy_config, structure.name)
                            .get_cost(structure.level + 1),
                    );
                }
                details.builds = Some(builds);
            }
            Ok(Details::Structure(details))
        } else {
            let mut details = SystemInfo {
                score: self.score(tick, galaxy_config),
                resources: self.resources,
                structures: IndexMap::new(),
                production: self.get_production(tick, galaxy_config),
                events: self.events.clone(),
            };
            for structure in self.structures.iter() {
                details.structures.insert(structure.name, structure.level);
            }
            Ok(Details::System(details.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GalaxyConfig, StructureConfig, SystemConfig};

    fn create_test_galaxy_config() -> GalaxyConfig {
        let mut galaxy_config = GalaxyConfig::default();

        // Set up structure configs for testing
        let mut structure_configs = IndexMap::new();

        // Colony: produces 1 crew per hour, has storage
        let colony_config = StructureConfig {
            starting_level: 1,
            production: Some(crate::config::ProductionConfig {
                multiplier: Some(2.0), // Level 2 produces 2x as much
                metal: 0,
                crew: 1, // 1 crew per hour at level 1
                water: 0,
            }),
            storage: Some(crate::config::StorageConfig {
                multiplier: Some(2.0),
                metal: 200,
                crew: 20,
                water: 200,
            }),
            cost: Some(crate::config::CostConfig {
                multiplier: Some(2.0),
                time: 1800, // 30 minutes
                metal: 10,
                crew: 0,
                water: 5,
            }),
            ..Default::default()
        };
        structure_configs.insert("colony".to_string(), colony_config);

        // Asteroid Mine: produces metal
        let mine_config = StructureConfig {
            starting_level: 1,
            production: Some(crate::config::ProductionConfig {
                multiplier: Some(2.0), // Level 2 produces 2x as much
                metal: 2,              // 2 metal per hour at level 1
                crew: 0,
                water: 0,
            }),
            cost: Some(crate::config::CostConfig {
                multiplier: Some(2.0),
                time: 3600, // 1 hour
                metal: 5,
                crew: 1,
                water: 0,
            }),
            ..Default::default()
        };
        structure_configs.insert("asteroidmine".to_string(), mine_config);

        // Water Harvester: produces water
        let harvester_config = StructureConfig {
            starting_level: 1,
            production: Some(crate::config::ProductionConfig {
                multiplier: Some(2.0), // Level 2 produces 2x as much
                metal: 0,
                crew: 0,
                water: 3, // 3 water per hour at level 1
            }),
            cost: Some(crate::config::CostConfig {
                multiplier: Some(2.0),
                time: 2400, // 40 minutes
                metal: 8,
                crew: 1,
                water: 0,
            }),
            ..Default::default()
        };
        structure_configs.insert("waterharvester".to_string(), harvester_config);

        galaxy_config.systems.structures = structure_configs;
        galaxy_config
    }

    fn create_test_system_config() -> SystemConfig {
        let mut system_config = SystemConfig::default();
        system_config.resources.insert("metal".to_string(), 10);
        system_config.resources.insert("water".to_string(), 5);
        system_config.resources.insert("crew".to_string(), 3);

        // Set up structures with starting levels - these get used by System::new
        let mut structure_configs = IndexMap::new();

        structure_configs.insert(
            "colony".to_string(),
            StructureConfig {
                starting_level: 1,
                ..Default::default()
            },
        );

        structure_configs.insert(
            "asteroidmine".to_string(),
            StructureConfig {
                starting_level: 1,
                ..Default::default()
            },
        );

        structure_configs.insert(
            "waterharvester".to_string(),
            StructureConfig {
                starting_level: 1,
                ..Default::default()
            },
        );

        system_config.structures = structure_configs;
        system_config
    }

    #[test]
    fn test_system_initialization() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let system = System::new(0, &system_config, &galaxy_config);

        assert_eq!(system.current_tick, 0);
        assert_eq!(system.resources.metal, 10);
        assert_eq!(system.resources.water, 5);
        assert_eq!(system.resources.crew, 3);
        assert_eq!(system.structures.len(), 3);
    }

    #[test]
    fn test_resource_production_single_tick() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Production rates: Metal: 2/hour, Water: 3/hour, Crew: 1/hour
        // Intervals: Metal: 3600/2 = 1800 ticks, Water: 3600/3 = 1200 ticks, Crew: 3600/1 = 3600 ticks

        // Test at tick 1200 (should get 1 water)
        system.update_to_tick(1200, &galaxy_config);
        assert_eq!(system.resources.metal, 10); // No metal yet
        assert_eq!(system.resources.water, 6); // 5 + 1
        assert_eq!(system.resources.crew, 3); // No crew yet

        // Test at tick 1800 (should get 1 metal)
        system.update_to_tick(1800, &galaxy_config);
        assert_eq!(system.resources.metal, 11); // 10 + 1
        assert_eq!(system.resources.water, 6); // Still 6
        assert_eq!(system.resources.crew, 3); // No crew yet

        // Test at tick 2400 (should get another water)
        system.update_to_tick(2400, &galaxy_config);
        assert_eq!(system.resources.metal, 11); // Still 11
        assert_eq!(system.resources.water, 7); // 6 + 1
        assert_eq!(system.resources.crew, 3); // No crew yet

        // Test at tick 3600 (should get another metal and first crew and another water)
        system.update_to_tick(3600, &galaxy_config);
        assert_eq!(system.resources.metal, 12); // 11 + 1
        assert_eq!(system.resources.water, 8); // 7 + 1 (at tick 3600)
        assert_eq!(system.resources.crew, 4); // 3 + 1
    }

    #[test]
    fn test_resource_production_large_jump() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Jump to tick 7200 (2 hours)
        system.update_to_tick(7200, &galaxy_config);

        // Expected production in 2 hours:
        // Metal: 2 per hour * 2 hours = 4 total
        // Water: 3 per hour * 2 hours = 6 total
        // Crew: 1 per hour * 2 hours = 2 total

        assert_eq!(system.resources.metal, 14); // 10 + 4
        assert_eq!(system.resources.water, 11); // 5 + 6
        assert_eq!(system.resources.crew, 5); // 3 + 2
    }

    #[test]
    fn test_storage_limits() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Set initial resources close to storage limits
        // Colony level 1 has storage: metal=200, water=200, crew=20
        system.resources = Resources {
            metal: 195,
            water: 195,
            crew: 18,
        };

        // Jump ahead by a lot to test storage caps
        system.update_to_tick(36000, &galaxy_config); // 10 hours

        // Should be capped at storage limits
        assert_eq!(system.resources.metal, 200);
        assert_eq!(system.resources.water, 200);
        assert_eq!(system.resources.crew, 20);
    }

    #[test]
    fn test_no_production_structures() {
        let galaxy_config = create_test_galaxy_config();
        let mut system_config = create_test_system_config();

        // Create system with only level 0 structures (no production)
        system_config
            .structures
            .get_mut("colony")
            .unwrap()
            .starting_level = 0;
        system_config
            .structures
            .get_mut("asteroidmine")
            .unwrap()
            .starting_level = 0;
        system_config
            .structures
            .get_mut("waterharvester")
            .unwrap()
            .starting_level = 0;

        let mut system = System::new(0, &system_config, &galaxy_config);
        let initial_resources = system.resources;

        // Jump ahead - should not produce anything
        system.update_to_tick(7200, &galaxy_config);

        assert_eq!(system.resources, initial_resources);
    }

    #[test]
    fn test_update_to_same_tick() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(1000, &system_config, &galaxy_config);

        let initial_resources = system.resources;

        // Update to same tick should do nothing
        system.update_to_tick(1000, &galaxy_config);
        assert_eq!(system.resources, initial_resources);

        // Update to earlier tick should do nothing
        system.update_to_tick(500, &galaxy_config);
        assert_eq!(system.resources, initial_resources);
    }

    #[test]
    fn test_production_with_multiple_levels() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Upgrade structures manually for testing
        system.structures[0].level = 2; // Colony level 2: produces 2 crew/hour
        system.structures[1].level = 2; // Mine level 2: produces 4 metal/hour
        system.structures[2].level = 2; // Harvester level 2: produces 6 water/hour

        // Test production for 1 hour
        system.update_to_tick(3600, &galaxy_config);

        // Expected production for level 2 structures (with 2.0 multiplier):
        // Metal: 2 * 2 = 4 per hour * 1 hour = 4 total
        // Water: 3 * 2 = 6 per hour * 1 hour = 6 total
        // Crew: 1 * 2 = 2 per hour * 1 hour = 2 total

        assert_eq!(system.resources.metal, 14); // 10 + 4
        assert_eq!(system.resources.water, 11); // 5 + 6
        assert_eq!(system.resources.crew, 5); // 3 + 2
    }

    #[test]
    fn test_fractional_production_intervals() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Test that production happens at exact intervals
        // Water production: 3 per hour = every 1200 ticks

        // At tick 1199, no water should be produced yet
        system.update_to_tick(1199, &galaxy_config);
        assert_eq!(system.resources.water, 5);

        // At tick 1200, exactly 1 water should be produced
        system.update_to_tick(1200, &galaxy_config);
        assert_eq!(system.resources.water, 6);

        // At tick 2399, still only 1 water
        system.update_to_tick(2399, &galaxy_config);
        assert_eq!(system.resources.water, 6);

        // At tick 2400, 2nd water should be produced
        system.update_to_tick(2400, &galaxy_config);
        assert_eq!(system.resources.water, 7);
    }

    #[test]
    #[cfg(feature = "db")]
    fn test_database_persistence_with_tick() {
        let galaxy_config = create_test_galaxy_config();
        let system_config = create_test_system_config();
        let mut system = System::new(0, &system_config, &galaxy_config);

        // Let some time pass and production occur
        system.update_to_tick(3600, &galaxy_config); // 1 hour

        // Save state for database persistence
        let current_tick = system.get_current_tick();
        let resources = system.get_resources();
        let structures = system.get_structures();
        let events = system.get_events().clone();

        // Create a new system from database data
        let mut restored_system =
            System::from_database(current_tick, resources, structures, events);

        // The restored system should have the same state
        assert_eq!(restored_system.get_current_tick(), 3600);
        assert_eq!(
            restored_system.get_resources(),
            Resources {
                metal: 12,
                water: 8,
                crew: 4
            }
        );

        // When we advance time further, production should continue correctly
        restored_system.update_to_tick(7200, &galaxy_config); // Another hour

        // Should have gained another hour of production
        assert_eq!(restored_system.resources.metal, 14); // 12 + 2
        assert_eq!(restored_system.resources.water, 11); // 8 + 3
        assert_eq!(restored_system.resources.crew, 5); // 4 + 1
    }
}
