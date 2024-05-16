use islandfight::{BuildingType, World, WorldConfig};

fn main() {
    //let mut file = File::open("../worlds/classic.yaml").unwrap();
    let contents = include_str!("../worlds/classic.yaml");
    let world_config: WorldConfig = serde_yaml::from_str(contents).unwrap();
    let mut world = World::new(world_config, 0);

    println!("{}", world.stats(29));
    world
        .build(
            30,
            *world.islands().keys().next().unwrap(),
            BuildingType::GoldPit,
        )
        .unwrap_or_default();
    println!("{}", world.stats(30));
    println!("{}", world.stats(59));
}
