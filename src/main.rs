use islandfight::{World, WorldConfig};

fn main() {
    // For now, we're just running some very basic stuff here to test the lib
    let world = World::new(WorldConfig {
        island_count: 10,
        size: (100, 100),
    });
    println!("{}", world.stats());
}
