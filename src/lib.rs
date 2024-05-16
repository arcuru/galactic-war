use rand::Rng;
use std::collections::HashMap; // Add this line to import the `Rng` trait from the `rand` crate

/// An Island in the world
struct Island {
    /// List of events that are happening on the island
    events: Vec<Event>,
    gold: usize,
    _lumber: usize,
    _stone: usize,
    buildings: Buildings,
}

struct Buildings {
    goldpit: usize,
}

#[derive(Clone)]
struct Event {
    completion: usize,
    callback: EventCallback,
}

#[derive(Clone)]
enum EventCallback {
    Gold,
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
        Self {
            events,
            gold,
            _lumber: lumber,
            _stone: stone,
            buildings: Buildings { goldpit: 0 },
        }
    }

    /// Callback for the goldpit, increase the gold amount by one and create a new callback
    fn goldpit_callback(
        tick: usize,
        _world_config: &WorldConfig,
        island: &mut Island,
    ) -> Option<Event> {
        island.gold += 1;
        Some(Event {
            completion: tick + 10,
            callback: EventCallback::Gold,
        })
    }

    pub fn gold(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
        self.gold
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
                    match event.callback {
                        EventCallback::Gold => {
                            if let Some(new_event) =
                                Self::goldpit_callback(event.completion, world_config, self)
                            {
                                self.register_event(new_event);
                            }
                        }
                    }
                } else {
                    self.register_event(event.clone());
                }
            }
        }
    }

    /// Get the score of an island
    pub fn score(&mut self, tick: usize, world_config: &WorldConfig) -> usize {
        self.process_events(tick, world_config);
        self.buildings.goldpit
    }
}

/// Configuration for the creation of an island
pub struct IslandConfig {
    /// List of buildings that will be built on the island
    pub buildings: Vec<BuildingConfig>,

    /// Starting resources for the island
    pub resources: HashMap<String, usize>,
}

pub struct BuildingConfig {
    /// Name of the building
    /// Must be one of the predefined types
    pub name: String,

    /// Starting level for this type of building
    /// If not provided it is 0
    pub starting_level: Option<usize>,
}

pub struct World {
    config: WorldConfig,
    islands: HashMap<(usize, usize), Island>,
}

/// Configuration for the world
pub struct WorldConfig {
    /// Static Island Count
    pub island_count: usize,

    /// World size
    pub size: (usize, usize),

    /// Island Config
    pub islands: IslandConfig,
}

impl World {
    /// Create a new World
    ///
    /// Uses a custom configuration struct to set up all the details
    pub fn new(config: WorldConfig, initial_tick: usize) -> Self {
        let mut islands = HashMap::new();
        for _ in 0..config.island_count {
            // Create a new island at a random location in the 2d space

            let mut rng = rand::thread_rng();
            let x: usize = rng.gen_range(0..=config.size.0);
            let y: usize = rng.gen_range(0..=config.size.1);
            let island = Island::new(initial_tick, &config.islands);
            islands.insert((x, y), island);
        }
        Self { config, islands }
    }

    /// Return basic stats about the World
    pub fn stats(&mut self, tick: usize) -> String {
        let mut stats = format!("Island count: {}\n", self.config.island_count);
        for (coords, island) in self.islands.iter_mut() {
            stats.push_str(&format!(
                "Island at {:?} has score {} and gold {}\n",
                coords,
                island.score(tick, &self.config),
                island.gold(tick, &self.config),
            ));
        }
        stats
    }
}
