use core::panic;
use std::collections::HashMap;

use crate::config::{GalaxyConfig, StructureConfig, SystemConfig};
use crate::{Cost, Details, StructureInfo, SystemInfo, SystemProduction};
use std::fmt;
use std::str::FromStr;

/// An System in the Galaxy
#[derive(Debug, Default)]
pub struct System {
    /// List of events that are happening in the system
    events: Vec<Event>,
    gold: usize,
    lumber: usize,
    stone: usize,
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
    Gold,
    Lumber,
    Stone,
    Build,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Fortress,
    GoldPit,
    StoneBasin,
    Sawmill,
    Garrison,
    Warehouse,
    Barricade,
    WatchTower,
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
            "fortress" => Ok(StructureType::Fortress),
            "goldpit" => Ok(StructureType::GoldPit),
            "stonebasin" => Ok(StructureType::StoneBasin),
            "sawmill" => Ok(StructureType::Sawmill),
            "garrison" => Ok(StructureType::Garrison),
            "warehouse" => Ok(StructureType::Warehouse),
            "barricade" => Ok(StructureType::Barricade),
            "watchtower" => Ok(StructureType::WatchTower),
            _ => Err(()),
        }
    }
}

impl System {
    /// Create a new system
    ///
    /// This takes an SystemConfig because there may be multiple system types in future
    pub fn new(tick: usize, system_config: &SystemConfig, galaxy_config: &GalaxyConfig) -> Self {
        let gold = *system_config.resources.get("gold").unwrap_or(&0);
        let lumber = *system_config.resources.get("lumber").unwrap_or(&0);
        let stone = *system_config.resources.get("stone").unwrap_or(&0);
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
                StructureType::GoldPit => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::GoldPit);
                    let time = config.production.as_ref().unwrap()[structure.level];
                    events.push(Event {
                        completion: tick + time,
                        action: EventCallback::Gold,
                        structure: None,
                    });
                }
                StructureType::Sawmill => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::Sawmill);
                    let time = config.production.as_ref().unwrap()[structure.level];
                    events.push(Event {
                        completion: tick + time,
                        action: EventCallback::Lumber,
                        structure: None,
                    });
                }
                StructureType::StoneBasin => {
                    let config =
                        System::get_structure_config(galaxy_config, StructureType::StoneBasin);
                    let time = config.production.as_ref().unwrap()[structure.level];
                    events.push(Event {
                        completion: tick + time,
                        action: EventCallback::Stone,
                        structure: None,
                    });
                }
                _ => {}
            }
        }
        events.sort_by_key(|e| e.completion);

        Self {
            events,
            gold,
            lumber,
            stone,
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
            gold: 0,
            lumber: 0,
            stone: 0,
        };
        match structure {
            StructureType::GoldPit => {
                let config = System::get_structure_config(galaxy_config, StructureType::GoldPit);
                production.gold = config.production.as_ref().unwrap()
                    [self.structure_level(StructureType::GoldPit)];
            }
            StructureType::Sawmill => {
                let config = System::get_structure_config(galaxy_config, StructureType::Sawmill);
                production.lumber = config.production.as_ref().unwrap()
                    [self.structure_level(StructureType::Sawmill)];
            }
            _ => {}
        };
        production
    }

    /// Get the production of the system
    fn get_production(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> SystemProduction {
        self.process_events(tick, galaxy_config);
        // Get the gold production
        let gold = {
            let config = System::get_structure_config(galaxy_config, StructureType::GoldPit);
            config.production.as_ref().unwrap()[self.structure_level(StructureType::GoldPit)]
        };

        SystemProduction {
            gold,
            lumber: 0,
            stone: 0,
        }
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
            EventCallback::Gold => {
                self.gold += 1;
                // Create a new event for the next gold piece
                let time = System::get_structure_config(galaxy_config, StructureType::GoldPit)
                    .production
                    .as_ref()
                    .unwrap()[self.structure_level(StructureType::GoldPit)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Gold,
                    structure: None,
                });
            }
            EventCallback::Lumber => {
                self.lumber += 1;
                // Create a new event for the next lumber piece
                let time = System::get_structure_config(galaxy_config, StructureType::Sawmill)
                    .production
                    .as_ref()
                    .unwrap()[self.structure_level(StructureType::Sawmill)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Lumber,
                    structure: None,
                });
            }
            EventCallback::Stone => {
                self.stone += 1;
                // Create a new event for the next stone piece
                let time = System::get_structure_config(galaxy_config, StructureType::StoneBasin)
                    .production
                    .as_ref()
                    .unwrap()[self.structure_level(StructureType::StoneBasin)];
                self.register_event(Event {
                    completion: event.completion + time,
                    action: EventCallback::Stone,
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

    /// Get the current gold amount
    pub fn gold(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.process_events(tick, galaxy_config);
        self.gold
    }

    /// Get the current lumber amount
    pub fn lumber(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.process_events(tick, galaxy_config);
        self.lumber
    }

    /// Get the current stone amount
    pub fn stone(&mut self, tick: usize, galaxy_config: &GalaxyConfig) -> usize {
        self.process_events(tick, galaxy_config);
        self.stone
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
        if self.structure(structure).is_some() {
            // Verify if the structure can be built
            let cost = &System::get_structure_config(galaxy_config, structure).cost
                [self.structure_level(structure)];
            if self.gold >= *cost.get("gold").unwrap_or(&0)
                && self.lumber >= *cost.get("lumber").unwrap_or(&0)
                && self.stone >= *cost.get("stone").unwrap_or(&0)
            {
                // Deduct the cost
                self.gold -= cost.get("gold").unwrap_or(&0);
                self.lumber -= cost.get("lumber").unwrap_or(&0);
                self.stone -= cost.get("stone").unwrap_or(&0);
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
            if structure == StructureType::Fortress {
                if details.builds.is_none() {
                    details.builds = Some(HashMap::new());
                }
                let builds = details.builds.as_mut().unwrap();
                for structure in self.structures.iter() {
                    builds.insert(
                        structure.name,
                        Cost {
                            gold: *System::get_structure_config(galaxy_config, structure.name).cost
                                [structure.level]
                                .get("gold")
                                .unwrap_or(&0),
                            lumber: *System::get_structure_config(galaxy_config, structure.name)
                                .cost[structure.level]
                                .get("lumber")
                                .unwrap_or(&0),
                            stone: *System::get_structure_config(galaxy_config, structure.name)
                                .cost[structure.level]
                                .get("stone")
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
                gold: self.gold,
                lumber: self.lumber,
                stone: self.stone,
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
