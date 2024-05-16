use core::panic;
use serde::Deserialize;
use std::collections::HashMap;

use crate::{BuildingInfo, Cost, Details, IslandInfo, IslandProduction, WorldConfig};
use std::fmt;
use std::str::FromStr;

/// An Island in the world
pub struct Island {
    /// List of events that are happening on the island
    events: Vec<Event>,
    gold: usize,
    lumber: usize,
    stone: usize,
    buildings: Vec<Building>,
}

struct Building {
    name: BuildingType,
    level: usize,
}

#[derive(Clone)]
pub struct Event {
    completion: usize,
    callback: EventCallback,
    building: Option<BuildingType>,
}

#[derive(Clone)]
enum EventCallback {
    Gold,
    Lumber,
    Stone,
    Build,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Fortress,
    GoldPit,
    StoneBasin,
    Sawmill,
    Garrison,
    Warehouse,
    Barricade,
    WatchTower,
}

impl fmt::Display for BuildingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for BuildingType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fortress" => Ok(BuildingType::Fortress),
            "goldpit" => Ok(BuildingType::GoldPit),
            "stonebasin" => Ok(BuildingType::StoneBasin),
            "sawmill" => Ok(BuildingType::Sawmill),
            "garrison" => Ok(BuildingType::Garrison),
            "warehouse" => Ok(BuildingType::Warehouse),
            "barricade" => Ok(BuildingType::Barricade),
            "watchtower" => Ok(BuildingType::WatchTower),
            _ => Err(()),
        }
    }
}

impl Island {
    /// Create a new island
    ///
    /// This takes an IslandConfig because there may be multiple island types in future
    pub fn new(tick: usize, island_config: &IslandConfig, world_config: &WorldConfig) -> Self {
        let gold = *island_config.resources.get("gold").unwrap_or(&0);
        let lumber = *island_config.resources.get("lumber").unwrap_or(&0);
        let stone = *island_config.resources.get("stone").unwrap_or(&0);
        let mut buildings = Vec::new();
        for (name, building) in island_config.buildings.iter() {
            buildings.push(Building {
                name: BuildingType::from_str(name).unwrap(),
                level: building.starting_level.unwrap_or(0),
            });
        }
        // Kick off events for the buildings
        let mut events = Vec::new();
        for building in buildings.iter() {
            match building.name {
                BuildingType::GoldPit => {
                    let config = Island::get_building_config(world_config, BuildingType::GoldPit);
                    let time = config.production.as_ref().unwrap()[building.level];
                    events.push(Event {
                        completion: tick + time,
                        callback: EventCallback::Gold,
                        building: None,
                    });
                }
                BuildingType::Sawmill => {
                    let config = Island::get_building_config(world_config, BuildingType::Sawmill);
                    let time = config.production.as_ref().unwrap()[building.level];
                    events.push(Event {
                        completion: tick + time,
                        callback: EventCallback::Lumber,
                        building: None,
                    });
                }
                BuildingType::StoneBasin => {
                    let config =
                        Island::get_building_config(world_config, BuildingType::StoneBasin);
                    let time = config.production.as_ref().unwrap()[building.level];
                    events.push(Event {
                        completion: tick + time,
                        callback: EventCallback::Stone,
                        building: None,
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
            buildings,
        }
    }

    /// Get the index of the building by type
    ///
    /// The building may not exist, so it returns an Option
    fn building(&self, building: BuildingType) -> Option<usize> {
        self.buildings.iter().position(|b| b.name == building)
    }

    /// Get the level of a building
    fn building_level(&self, building: BuildingType) -> usize {
        if let Some(index) = self.building(building) {
            self.buildings[index].level
        } else {
            0
        }
    }

    /// Get the building configuration from the WorldConfig
    fn get_building_config(world_config: &WorldConfig, building: BuildingType) -> &BuildingConfig {
        world_config
            .islands
            .buildings
            .get(&building.to_string().to_lowercase())
            .unwrap()
    }

    /// Get the resource production of a single building
    fn get_building_production(
        &mut self,
        tick: usize,
        world_config: &WorldConfig,
        building: BuildingType,
    ) -> IslandProduction {
        self.process_events(tick, world_config);
        let mut production = IslandProduction {
            gold: 0,
            lumber: 0,
            stone: 0,
        };
        match building {
            BuildingType::GoldPit => {
                let config = Island::get_building_config(world_config, BuildingType::GoldPit);
                production.gold =
                    config.production.as_ref().unwrap()[self.building_level(BuildingType::GoldPit)];
            }
            BuildingType::Sawmill => {
                let config = Island::get_building_config(world_config, BuildingType::Sawmill);
                production.lumber =
                    config.production.as_ref().unwrap()[self.building_level(BuildingType::Sawmill)];
            }
            _ => {}
        };
        production
    }

    /// Get the production of the island
    fn get_production(&mut self, tick: usize, world_config: &WorldConfig) -> IslandProduction {
        self.process_events(tick, world_config);
        // Get the gold production
        let gold = {
            let config = Island::get_building_config(world_config, BuildingType::GoldPit);
            config.production.as_ref().unwrap()[self.building_level(BuildingType::GoldPit)]
        };

        IslandProduction {
            gold,
            lumber: 0,
            stone: 0,
        }
    }

    /// Callback for events
    ///
    /// This will process the event and update the state of the island.
    /// It will also create new events if needed.
    fn event_callback(&mut self, tick: usize, world_config: &WorldConfig, event: Event) {
        // Check the completion time
        if event.completion > tick {
            return;
        }
        match event.callback {
            EventCallback::Gold => {
                self.gold += 1;
                // Create a new event for the next gold piece
                let time = Island::get_building_config(world_config, BuildingType::GoldPit)
                    .production
                    .as_ref()
                    .unwrap()[self.building_level(BuildingType::GoldPit)];
                self.register_event(Event {
                    completion: event.completion + time,
                    callback: EventCallback::Gold,
                    building: None,
                });
            }
            EventCallback::Lumber => {
                self.lumber += 1;
                // Create a new event for the next lumber piece
                let time = Island::get_building_config(world_config, BuildingType::Sawmill)
                    .production
                    .as_ref()
                    .unwrap()[self.building_level(BuildingType::Sawmill)];
                self.register_event(Event {
                    completion: event.completion + time,
                    callback: EventCallback::Lumber,
                    building: None,
                });
            }
            EventCallback::Stone => {
                self.stone += 1;
                // Create a new event for the next stone piece
                let time = Island::get_building_config(world_config, BuildingType::StoneBasin)
                    .production
                    .as_ref()
                    .unwrap()[self.building_level(BuildingType::StoneBasin)];
                self.register_event(Event {
                    completion: event.completion + time,
                    callback: EventCallback::Stone,
                    building: None,
                });
            }
            EventCallback::Build => {
                // Build the building
                if let Some(building) = event.building {
                    let index = self.building(building).unwrap();
                    self.buildings[index].level += 1;
                } else {
                    panic!("Building event without building type");
                }
            }
        }
    }

    /// Get the current gold amount
    pub fn gold(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
        self.gold
    }

    /// Get the current lumber amount
    pub fn lumber(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
        self.lumber
    }

    /// Get the current stone amount
    pub fn stone(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
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
    pub fn process_events(&mut self, tick: usize, world_config: &WorldConfig) {
        let mut dirty = true;
        while dirty && self.event_to_process(tick) {
            dirty = false;
            let events = self.events.clone();
            self.events.clear();
            for event in events.iter() {
                if event.completion <= tick {
                    dirty = true;
                    self.event_callback(tick, world_config, event.clone());
                } else {
                    self.register_event(event.clone());
                }
            }
        }
    }

    /// Get the score of an island
    pub fn score(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
        // Sum the levels of each building
        // This is not the same as IK
        self.buildings.iter().map(|b| b.level).sum()
    }

    /// Build a building
    pub fn build(
        &mut self,
        tick: usize,
        world_config: &WorldConfig,
        building: BuildingType,
    ) -> Result<(), String> {
        self.process_events(tick, world_config);
        if self.building(building).is_some() {
            // Verify if the building can be built
            let cost = &Island::get_building_config(world_config, building).cost
                [self.building_level(building)];
            if self.gold >= *cost.get("gold").unwrap_or(&0)
                && self.lumber >= *cost.get("lumber").unwrap_or(&0)
                && self.stone >= *cost.get("stone").unwrap_or(&0)
            {
                // Deduct the cost
                self.gold -= cost.get("gold").unwrap_or(&0);
                self.lumber -= cost.get("lumber").unwrap_or(&0);
                self.stone -= cost.get("stone").unwrap_or(&0);
                // Increase the level
                self.register_event(Event {
                    completion: tick + cost.get("time").unwrap_or(&1),
                    callback: EventCallback::Build,
                    building: Some(building),
                });
                Ok(())
            } else {
                // Not enough resources
                Err("Not enough resources".to_string())
            }
        } else {
            // Building does not exist
            Err("Building not found".to_string())
        }
    }

    /// Get the details of the island
    pub fn get_details(
        &mut self,
        tick: usize,
        world_config: &WorldConfig,
        building: Option<BuildingType>,
    ) -> Result<Details, String> {
        self.process_events(tick, world_config);
        if let Some(building) = building {
            let mut details = BuildingInfo {
                level: self.building_level(building),
                production: Some(self.get_building_production(tick, world_config, building)),
                builds: None,
            };
            if building == BuildingType::Fortress {
                if details.builds.is_none() {
                    details.builds = Some(HashMap::new());
                }
                let builds = details.builds.as_mut().unwrap();
                for building in self.buildings.iter() {
                    builds.insert(
                        building.name,
                        Cost {
                            gold: *Island::get_building_config(world_config, building.name).cost
                                [building.level]
                                .get("gold")
                                .unwrap_or(&0),
                            lumber: *Island::get_building_config(world_config, building.name).cost
                                [building.level]
                                .get("lumber")
                                .unwrap_or(&0),
                            stone: *Island::get_building_config(world_config, building.name).cost
                                [building.level]
                                .get("stone")
                                .unwrap_or(&0),
                            ticks: *Island::get_building_config(world_config, building.name).cost
                                [building.level]
                                .get("time")
                                .unwrap_or(&0),
                        },
                    );
                }
            }
            Ok(Details::Building(details))
        } else {
            let mut details = IslandInfo {
                score: self.score(tick, world_config),
                gold: self.gold,
                lumber: self.lumber,
                stone: self.stone,
                buildings: HashMap::new(),
                production: self.get_production(tick, world_config),
            };
            for building in self.buildings.iter() {
                details.buildings.insert(building.name, building.level);
            }
            Ok(Details::Island(details))
        }
    }
}

#[derive(Deserialize)]
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

/// Configuration for the creation of an island
#[derive(Deserialize)]
pub struct IslandConfig {
    /// List of buildings that will be built on the island
    pub buildings: HashMap<String, BuildingConfig>,

    /// Starting resources for the island
    pub resources: HashMap<String, usize>,
}
