use serde::Deserialize;
use std::collections::HashMap;

use crate::WorldConfig;

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
}

#[derive(Clone)]
enum EventCallback {
    Gold,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl Island {
    /// Create a new island
    pub fn new(tick: usize, config: &IslandConfig) -> Self {
        let gold = *config.resources.get("gold").unwrap_or(&0);
        let lumber = *config.resources.get("lumber").unwrap_or(&0);
        let stone = *config.resources.get("stone").unwrap_or(&0);
        let events = vec![Event {
            completion: tick + 10,
            callback: EventCallback::Gold,
        }];
        let buildings = vec![Building {
            name: BuildingType::GoldPit,
            level: config
                .buildings
                .get("goldpit")
                .unwrap()
                .starting_level
                .unwrap_or(0),
        }];
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
                let time = world_config
                    .islands
                    .buildings
                    .get("goldpit")
                    .unwrap()
                    .production
                    .as_ref()
                    .unwrap()[self.building_level(BuildingType::GoldPit)];
                self.register_event(Event {
                    completion: event.completion + time,
                    callback: EventCallback::Gold,
                });
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
        if let Some(index) = self.building(building) {
            // TODO: Add a build time, this effectively builds instantly and ignores in-flight production events
            self.buildings[index].level += 1;
            Ok(())
        } else {
            // Building does not exist
            Err("Building not found".to_string())
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
}

/// Configuration for the creation of an island
#[derive(Deserialize)]
pub struct IslandConfig {
    /// List of buildings that will be built on the island
    pub buildings: HashMap<String, BuildingConfig>,

    /// Starting resources for the island
    pub resources: HashMap<String, usize>,
}
