use islandfight::{BuildingType, World, WorldConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = include_str!("../worlds/classic.yaml");
    let world_config: WorldConfig = serde_yaml::from_str(contents).unwrap();
    let mut world = World::new(world_config, 0);

    let island = *world.islands().keys().next().unwrap();

    println!("{}", world.stats(29)?);
    if let Err(err) = world.build(
        30,
        *world.islands().keys().next().unwrap(),
        BuildingType::GoldPit,
    ) {
        println!("Error: {}", err);
    }
    println!("{}", world.stats(30)?);
    println!("{}", world.stats(59)?);
    if let Err(err) = world.build(105, island, BuildingType::Sawmill) {
        println!("Error: {}", err);
    }
    if let Err(err) = world.build(150, island, BuildingType::StoneBasin) {
        println!("Error: {}", err);
    }
    println!("{}", world.stats(301)?);
    world.build(301, island, BuildingType::GoldPit)?;
    println!("{}", world.stats(302)?);
    println!("{:?}", world.get_details(302, island, None)?);
    println!(
        "Fortress: {:?}",
        world.get_details(302, island, Some(BuildingType::Fortress))?
    );
    println!(
        "Gold Pit: {:?}",
        world.get_details(302, island, Some(BuildingType::GoldPit))?
    );
    Ok(())
}
