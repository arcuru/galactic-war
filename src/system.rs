use core::panic;
use std::collections::HashMap;

use crate::config::{GalaxyConfig, StructureConfig, SystemConfig};
use crate::{Cost, Details, Resources, StructureInfo, SystemInfo, SystemProduction};
use std::fmt;
use std::str::FromStr;

/// An System in the Galaxy
#[derive(Debug, Default)]
pub struct System {
    /// List of events that are happening in the system.
    events: Vec<Event>,

    /// Current resources available in the system.
    resources: Resources,

    /// List of structures in the system.
    structures: Vec<Structure>,
}

#[derive(Debug)]
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

#[derive(Clone, Debug)]
pub enum EventCallback {
    Metal,
    Water,
    Crew,
    Build,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Colony,
    AsteroidMine,
    WaterHarvester,
    Hatchery,
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
            _ => Err(()),
        }
    }
}

impl System {
    /// Create a new system
    ///
    /// This takes an SystemConfig because there may be multiple system types in future
    pub fn new(tick: usize, system_config: &SystemConfig, galaxy_config: &GalaxyConfig) -> Self {
        let resources = Resources {
            metal: *system_config.resources.get("metal").unwrap_or(&0),
            water: *system_config.resources.get("water").unwrap_or(&0),
            crew: *system_config.resources.get("crew").unwrap_or(&0),
        };
        let mut structures = Vec::new();
        for (name, structure) in system_config.structures.iter() {
            structures.push(Structure {
                name: StructureType::from_str(name).unwrap(),
                level: structure.starting_level.unwrap_or(0),
            });
        }
        // Kick off events for the structures
        let mut events = Vec::new();
        for structure in structures.iter() {
            match structure.name {
                StructureType::AsteroidMine => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::AsteroidMine);
                    let metal = &config.production.as_ref().unwrap()[0]; // FIXME: allow different resources?
                    events.push(Event {
                        completion: tick + metal.production[structure.level],
                        action: EventCallback::Metal,
                        structure: None,
                    });
                }
                StructureType::Hatchery => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::Hatchery);
                    let crew = &config.production.as_ref().unwrap()[0]; // FIXME: allow different resources?
                    events.push(Event {
                        completion: tick + crew.production[structure.level],
                        action: EventCallback::Water,
                        structure: None,
                    });
                }
                StructureType::WaterHarvester => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::WaterHarvester);
                    let water = &config.production.as_ref().unwrap()[0]; // FIXME: allow different resources?
                    events.push(Event {
                        completion: tick + water.production[structure.level],
                        action: EventCallback::Crew,
                        structure: None,
                    });
                }
                _ => {}
            }
        }
        events.sort_by_key(|e| e.completion);

        Self {
            events,
            resources,
            structures,
        }
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

    /// Get the resource production of a single structure
    fn get_structure_production(
        &mut self,
        tick: usize,
        galaxy_config: &GalaxyConfig,
        structure: StructureType,
    ) -> SystemProduction {
        self.process_events(tick, galaxy_config);
        let mut production = SystemProduction {
            metal: 0,
            crew: 0,
            water: 0,
        };
        match structure {
            StructureType::AsteroidMine => {
                let config =
                    System::get_structure_config(galaxy_config, StructureType::AsteroidMine);
                production.metal = config.production.as_ref().unwrap()[0].production
                    [self.structure_level(StructureType::AsteroidMine)];
            }
            StructureType::Hatchery => {
                let config = System::get_structure_config(galaxy_config, StructureType::Hatchery);
                production.crew = config.production.as_ref().unwrap()[0].production
                    [self.structure_level(StructureType::Hatchery)];
            }
            StructureType::WaterHarvester => {
                let config = System::get_structure_config(galaxy_config, StructureType::Hatchery);
                production.water = config.production.as_ref().unwrap()[0].production
                    [self.structure_level(StructureType::WaterHarvester)];
            }
            _ => {}
        };
        production
    }

    /// Get the production of the system
    fn get_production(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> SystemProduction {
        self.process_events(tick, galaxy_config);
        // Get the metal production
        let metal = {
            let config = System::get_structure_config(galaxy_config, StructureType::AsteroidMine);
            config.production.as_ref().unwrap()[0].production
                [self.structure_level(StructureType::AsteroidMine)]
        };
        let crew = {
            let config = System::get_structure_config(galaxy_config, StructureType::Hatchery);
            config.production.as_ref().unwrap()[0].production
                [self.structure_level(StructureType::Hatchery)]
        };
        let water = {
            let config = System::get_structure_config(galaxy_config, StructureType::WaterHarvester);
            config.production.as_ref().unwrap()[0].production
                [self.structure_level(StructureType::WaterHarvester)]
        };

        SystemProduction { metal, crew, water }
    }

    /// Callback for events
    ///
    /// This will process the event and update the state of the system.
    /// It will also create new events if needed.
    fn event_callback(&mut self, tick: usize, galaxy_config: &GalaxyConfig, event: Event) {
        // Check the completion time
        if event.completion > tick {
            return;
        }
        match event.action {
            EventCallback::Metal => {
                self.resources.metal += 1;
                // Create a new event for the next metal piece
                let time = System::get_structure_config(galaxy_config, StructureType::AsteroidMine)
                    .production
                    .as_ref()
                    .unwrap()[0]
                    .production[self.structure_level(StructureType::AsteroidMine)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Metal,
                    structure: None,
                });
            }
            EventCallback::Water => {
                self.resources.water += 1;
                // Create a new event for the next water unit
                let time =
                    System::get_structure_config(galaxy_config, StructureType::WaterHarvester)
                        .production
                        .as_ref()
                        .unwrap()[0]
                        .production[self.structure_level(StructureType::WaterHarvester)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Water,
                    structure: None,
                });
            }
            EventCallback::Crew => {
                self.resources.crew += 1;
                // Create a new event for the next crew member
                let time = System::get_structure_config(galaxy_config, StructureType::Hatchery)
                    .production
                    .as_ref()
                    .unwrap()[0]
                    .production[self.structure_level(StructureType::Hatchery)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Crew,
                    structure: None,
                });
            }
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
        self.resources(tick, galaxy_config).water
    }

    /// Get the current water amount
    pub fn water(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.resources(tick, galaxy_config).water
    }

    /// Get the current crew count
    pub fn crew(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.resources(tick, galaxy_config).crew
    }

    /// Get the current resources of the system
    pub fn resources(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> Resources {
        self.process_events(tick, galaxy_config);
        self.resources.clone()
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
        let mut dirty = true;
        while dirty && self.event_to_process(tick) {
            dirty = false;
            let events = self.events.clone();
            self.events.clear();
            for event in events.iter() {
                if event.completion <= tick {
                    dirty = true;
                    self.event_callback(tick, galaxy_config, event.clone());
                } else {
                    self.register_event(event.clone());
                }
            }
        }
    }

    /// Get the score of a system
    pub fn score(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.process_events(tick, galaxy_config);
        // Sum the levels of each structure
        self.structures.iter().map(|b| b.level).sum()
    }

    /// Build a structure
    pub fn build(
        &mut self,
        tick: usize,
        galaxy_config: &GalaxyConfig,
        structure: StructureType,
    ) -> Result<Event, String> {
        self.process_events(tick, galaxy_config);
        // Check if we're already building a structure, we can only build one at a time
        if self.events.iter().any(|e| e.structure.is_some()) {
            return Err("Already building a structure".to_string());
        }
        if self.structure(structure).is_some() {
            // Verify if the structure can be built
            let cost = &System::get_structure_config(galaxy_config, structure).cost
                [self.structure_level(structure)];
            if self.resources.metal >= *cost.get("metal").unwrap_or(&0)
                && self.resources.water >= *cost.get("water").unwrap_or(&0)
                && self.resources.crew >= *cost.get("crew").unwrap_or(&0)
            {
                // Deduct the cost
                self.resources = self.resources.clone()
                    - Resources {
                        metal: *cost.get("metal").unwrap_or(&0),
                        water: *cost.get("water").unwrap_or(&0),
                        crew: *cost.get("crew").unwrap_or(&0),
                    };
                // Increase the level
                let event = Event {
                    completion: tick + cost.get("time").unwrap_or(&1),
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
        self.process_events(tick, galaxy_config);
        if let Some(structure) = structure {
            let mut details = StructureInfo {
                level: self.structure_level(structure),
                production: Some(self.get_structure_production(tick, galaxy_config, structure)),
                builds: None,
            };
            if structure == StructureType::Colony {
                if details.builds.is_none() {
                    details.builds = Some(HashMap::new());
                }
                let builds = details.builds.as_mut().unwrap();
                for structure in self.structures.iter() {
                    builds.insert(
                        structure.name,
                        Cost {
                            metal: *System::get_structure_config(galaxy_config, structure.name)
                                .cost[structure.level]
                                .get("metal")
                                .unwrap_or(&0),
                            water: *System::get_structure_config(galaxy_config, structure.name)
                                .cost[structure.level]
                                .get("water")
                                .unwrap_or(&0),
                            crew: *System::get_structure_config(galaxy_config, structure.name).cost
                                [structure.level]
                                .get("crew")
                                .unwrap_or(&0),
                            ticks: *System::get_structure_config(galaxy_config, structure.name)
                                .cost[structure.level]
                                .get("time")
                                .unwrap_or(&0),
                        },
                    );
                }
            }
            Ok(Details::Structure(details))
        } else {
            let mut details = SystemInfo {
                score: self.score(tick, galaxy_config),
                resources: self.resources.clone(),
                structures: HashMap::new(),
                production: self.get_production(tick, galaxy_config),
                events: self.events.clone(),
            };
            for structure in self.structures.iter() {
                details.structures.insert(structure.name, structure.level);
            }
            Ok(Details::System(details))
        }
    }
}
