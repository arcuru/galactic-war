use islandfight::{BuildingConfig, IslandConfig, World, WorldConfig};

fn main() {
    // For now, we're just running some very basic stuff here to test the lib
    let world = World::new(WorldConfig {
        island_count: 10,
        size: (100, 100),
        islands: IslandConfig {
            buildings: vec![BuildingConfig {
                name: "goldpit".to_string(),
                starting_level: None,
            }],
            resources: Default::default(),
        },
    });
    println!("{}", world.stats());
}
