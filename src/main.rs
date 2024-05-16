use islandfight::{IslandConfig, World, WorldConfig};

fn main() {
    // For now, we're just running some very basic stuff here to test the lib
    let mut world = World::new(
        WorldConfig {
            island_count: 2,
            size: (100, 100),
            islands: IslandConfig {
                resources: Default::default(),
                buildings: Default::default(),
            },
        },
        0,
    );
    println!("{}", world.stats(29));
    println!("{}", world.stats(30));
    println!("{}", world.stats(31));
}
