use islandfight::{BuildingType, Details, Event, World, WorldConfig};

use axum::{extract::Path, routing::get, Router};
use std::str::FromStr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

lazy_static::lazy_static! {
    // Safely share the world between threads
    static ref WORLDS: Arc<Mutex<HashMap<String, World>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = include_str!("../worlds/classic.yaml");
    let world_config: WorldConfig = serde_yaml::from_str(contents).unwrap();

    // Create a new world named "one"
    {
        let mut worlds = WORLDS.lock().unwrap();
        worlds.insert("one".to_string(), World::new(world_config, tick()));
    }

    serve().await
}

/// Serve the World(s) over HTTP
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // All the entry points expose both a GET and POST interface
    // The difference is just in presentation
    // A GET request will just display an HTML page, and a POST request will return the data being used to display
    // Internally all GET requests use the same data as returned from the POST request
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/:world/stats", get(world_stats_get).post(world_stats_post))
        .route("/:world/:x/:y", get(island_get).post(island_post))
        .route("/:world/:x/:y/", get(island_get).post(island_post))
        .route(
            "/:world/:x/:y/:building",
            get(building_get).post(building_post),
        )
        .route(
            "/:world/:x/:y/:building/build",
            get(build_building_get).post(build_building_post),
        )
        .route("/:world", get(world_get).post(world_post));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3050").await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

/// Return the current second
fn tick() -> usize {
    // Return the current second since the epoch
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

/// Answer a GET request to the building endpoint
///
/// TODO: This should respond with an actual HTML page
async fn building_get(
    Path((world, x, y, building)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    println!("BuildingGet: World: {}, x: {}, y: {}", world, x, y);
    let dets = building_info(&world, (x, y), &building);
    if let Ok(dets) = dets {
        Ok(format!("{:?}", dets))
    } else {
        Err(dets.unwrap_err())
    }
}

/// Answer a POST request to the building endpoint
///
/// TODO: This should respond with JSON
async fn building_post(
    Path((world, x, y, building)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let dets = building_info(&world, (x, y), &building);
    if let Ok(dets) = dets {
        Ok(format!("{:?}", dets))
    } else {
        Err(dets.unwrap_err())
    }
}

/// Retrieve the details of a building on an island
fn building_info(world: &str, (x, y): (usize, usize), building: &str) -> Result<Details, String> {
    println!("BuildingPost: World: {}, x: {}, y: {}", world, x, y);
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(world) {
        let building_type = BuildingType::from_str(building);
        if let Ok(building) = building_type {
            world.get_details(tick(), (x, y), Some(building))
        } else {
            Err("Building not found".to_string())
        }
    } else {
        Err("World not found".to_string())
    }
}

/// Handler for GET requests to /:world/:x/:y
///
/// TODO: This should respond with an actual HTML page
async fn island_get(Path((world, x, y)): Path<(String, usize, usize)>) -> Result<String, String> {
    island_post(Path((world, x, y))).await
}

/// Handler for POST requests to /:world/:x/:y
///
/// TODO: This should respond with JSON
async fn island_post(Path((world, x, y)): Path<(String, usize, usize)>) -> Result<String, String> {
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(&world) {
        if let Ok(dets) = world.get_details(tick(), (x, y), None) {
            Ok(format!("{:?}", dets))
        } else {
            Err("Island not found".to_string())
        }
    } else {
        Err("World not found".to_string())
    }
}

/// Handler for GET requests to /:world/:x/:y/:building/build
async fn build_building_get(
    Path((world, x, y, building)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let event = build_building(&world, (x, y), &building);
    if let Ok(event) = event {
        Ok(format!("{:?}", event))
    } else {
        Err(event.unwrap_err())
    }
}

/// Handler for POST requests to /:world/:x/:y/:building/build
///
/// TODO: This should respond with JSON
async fn build_building_post(
    Path((world, x, y, building)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let event = build_building(&world, (x, y), &building);
    if let Ok(event) = event {
        Ok(format!("{:?}", event))
    } else {
        Err(event.unwrap_err())
    }
}

/// Send the building request and return the internal type
fn build_building(world: &str, (x, y): (usize, usize), building: &str) -> Result<Event, String> {
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(world) {
        let building_type = BuildingType::from_str(building);
        if let Ok(building) = building_type {
            world.build(tick(), (x, y), building)
        } else {
            Err("Building not found".to_string())
        }
    } else {
        Err("World not found".to_string())
    }
}

/// Handler for GET requests to /:world/stats
///
/// TODO: This should respond with an actual HTML page
async fn world_stats_get(Path(world): Path<String>) -> Result<String, String> {
    println!("world_stats_get: {}", world);
    world_stats_post(Path(world)).await
}

/// Handler for POST requests to /:world/stats
///
/// TODO: This should respond with JSON
async fn world_stats_post(Path(world): Path<String>) -> Result<String, String> {
    println!("world_stats_post: {}", world);
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(&world) {
        Ok(world.stats(tick()).unwrap())
    } else {
        Err("World not found".to_string())
    }
}

/// Handler for GET requests to /:world
///
/// Serves the World Dashboard page
async fn world_get(Path(world): Path<String>) -> String {
    world_post(Path(world)).await
}

/// Handler for POST requests to /:world
async fn world_post(Path(world): Path<String>) -> String {
    format!("Welcome to world {}!", world).to_string()
}
