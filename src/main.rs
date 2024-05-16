use islandfight::{BuildingType, World, WorldConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = include_str!("../worlds/classic.yaml");
    let world_config: WorldConfig = serde_yaml::from_str(contents).unwrap();
    let mut world = World::new(world_config, 0);

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
    if let Err(err) = world.build(
        100,
        *world.islands().keys().next().unwrap(),
        BuildingType::GoldPit,
    ) {
        println!("Error: {}", err);
    }
    println!("{}", world.stats(199)?);
    println!("{}", world.stats(301)?);
    world.build(
        301,
        *world.islands().keys().next().unwrap(),
        BuildingType::GoldPit,
    )?;
    println!("{}", world.stats(302)?);
    println!(
        "{:?}",
        world.get_details(302, *world.islands().keys().next().unwrap(), None)?
    );
    Ok(())
}
