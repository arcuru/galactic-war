use axum::response::Html;
use islandfight::{BuildingType, Details, Event, EventCallback, IslandInfo, World, WorldConfig};

use axum::{extract::Path, routing::get, Router};
use std::str::FromStr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Coords for islands
///
/// TODO: Integrate this everywhere
struct Coords {
    x: usize,
    y: usize,
}

lazy_static::lazy_static! {
    // Safely share the worlds between threads
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
        .route("/", get(base_get))
        .route("/:world/stats", get(world_stats_get).post(world_stats_post))
        .route(
            "/:world/create",
            get(world_create_get).post(world_create_post),
        )
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

/// Handler for GET requests to /
async fn base_get() -> Html<String> {
    let mut page = r#"
<head>
    <title>Welcome to Island Fight</title>
    <script>
        function navigate() {
            var selectedWorld = document.getElementById("worlds").value;
            window.location.href = "/" + selectedWorld;
        }
        window.onload = function() {
            document.getElementById("createWorld").onsubmit = function() {
                var worldName = document.getElementById("newWorld").value;
                this.action = "/" + worldName + "/create";
            }
        }
    </script>
</head>
<body>
    <h1>Welcome to Island Fight!</h1>
    <select id="worlds">
    "#
    .to_string();
    for world in WORLDS.lock().unwrap().keys() {
        page.push_str(&format!("<option value=\"{}\">{}</option>", world, world));
    }
    page.push_str(
        r#"</select>
    <button onclick="navigate()">Go to world</button>
    <br><br>
    <h1>Create a new world</h1>
    <form id="createWorld" method="post">
        <label for="newWorld">Enter New World Name:</label>
        <input type="text" id="newWorld" name="newWorld" required>
        <input type="submit" value="Submit">
    </form>
</body>
"#,
    );
    Html::from(page)
}

/// Handler for GET requests to /:world/create
async fn world_create_get(Path(world): Path<String>) -> String {
    world_create_post(Path(world)).await
}

/// Handler for POST requests to /:world/create
async fn world_create_post(Path(world): Path<String>) -> String {
    let mut worlds = WORLDS.lock().unwrap();
    if worlds.contains_key(&world) {
        return format!("World {} already exists", world);
    }
    let contents = include_str!("../worlds/classic.yaml");
    let world_config: WorldConfig = serde_yaml::from_str(contents).unwrap();
    worlds.insert(world.clone(), World::new(world_config, tick()));
    format!("World {} created", world)
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
async fn island_get(
    Path((world, x, y)): Path<(String, usize, usize)>,
) -> Result<Html<String>, String> {
    let island_info = island_info(&world, (x, y))?;

    let mut page = format!("
<table width=600 border=1 cellspacing=0 cellpadding=3><tr><td width=33%>💰 {}</td>
<td width=33%>🪨 {}</td><td>🪵 {}</td></tr></table>
<br>
<table width=600 border=0 cellSpacing=1 cellPadding=3><tbody><tr><td vAlign=top width=50%><B>Buildings</b><br><font color=#CCCCC><b>
", island_info.gold, island_info.stone, island_info.lumber);

    for (building, level) in island_info.buildings.iter() {
        page.push_str(&format!(
            "🛖 <a href=/{}/{}/{}/{}>{} (level {})</a><br>",
            world,
            x,
            y,
            building.to_string().to_lowercase(),
            building,
            level
        ));
    }

    page.push_str(&format!(
        "<td vAlign=top><b>Score</b><br>{}</td></tr>",
        island_info.score
    ));
    Ok(Html::from(page.to_string()))
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

/// Retrieve the details of an island
fn island_info(world: &str, (x, y): (usize, usize)) -> Result<IslandInfo, String> {
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(world) {
        let dets = world.get_details(tick(), (x, y), None);
        if let Ok(dets) = dets {
            match dets {
                Details::Island(info) => Ok(info),
                _ => Err("Unexpected Details type".to_string()),
            }
        } else {
            Err(dets.unwrap_err())
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
async fn world_stats_get(Path(world): Path<String>) -> Result<Html<String>, String> {
    let mut page = "
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td align=center><b>
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td bgcolor=dddddd><b>Isle</b></td><td bgcolor=dddddd width=15%><b>💰Gold</b></td>
    <td bgcolor=dddddd width=15%><b>🪨Stones</b></td><td bgcolor=dddddd width=15%><b>🪵Lumber</b></td><td bgcolor=dddddd width=15%><b>Activity</b></td><td width=2%></td></tr>
".to_string();
    for (addr, dets) in world_info(&world).unwrap() {
        match dets {
            Details::Island(info) => {
                // Build an activity string
                // Scan through the activities to check if we're building something, and if so add a hover with the details
                let mut activity = String::new();
                let mut activity_hover = String::new();
                for event in info.events.iter() {
                    if let EventCallback::Build = event.action {
                        activity.push_str("🏗️");
                        let eta = event.completion - tick();
                        let eta_h = eta / 3600;
                        let eta_m = (eta % 3600) / 60;
                        let eta_s = eta % 60;
                        let mut eta_str = String::new();
                        if eta_h > 0 {
                            eta_str.push_str(&format!("{}:", eta_h));
                        }
                        if eta_m > 0 || eta_h > 0 {
                            eta_str.push_str(&format!("{:02}:", eta_m));
                        }
                        if eta_s > 0 || eta_m > 0 || eta_h > 0 {
                            eta_str.push_str(&format!("{:02}", eta_s));
                        }

                        activity_hover.push_str(&format!(
                            "Building {}: {} remaining",
                            event.building.unwrap(),
                            eta_str
                        ));
                    }
                }
                page.push_str(&format!(
                "<tr><td bgcolor=#ffffff><a href=/{}/{}/{}>{} ({}:{})</a></td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff></td><td bgcolor=#00FF00 title={}>{}</td></tr>",
                world, addr.x, addr.y, "Island", addr.x, addr.y, info.gold, info.stone, info.lumber, "", ""
            ));
            }
            _ => {
                return Err("Unexpected Details type".to_string());
            }
        }
    }
    Ok(Html::from(page.to_string()))
}

/// Handler for POST requests to /:world/stats
///
/// TODO: This should respond with JSON
async fn world_stats_post(Path(world): Path<String>) -> Result<String, String> {
    println!("world_stats_post: {}", world);
    world_stats(&world)
}

fn world_stats(world: &str) -> Result<String, String> {
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(world) {
        Ok(world.stats(tick()).unwrap())
    } else {
        Err("World not found".to_string())
    }
}

/// Handler for GET requests to /:world
///
/// Serves the World Dashboard page
async fn world_get(Path(world): Path<String>) -> Result<Html<String>, String> {
    world_stats_get(Path(world)).await
}

/// Handler for POST requests to /:world
async fn world_post(Path(world): Path<String>) -> String {
    format!("Welcome to world {}!", world).to_string()
}

/// Returns all the visible info for the world
fn world_info(world: &str) -> Result<Vec<(Coords, Details)>, String> {
    let mut island_info = Vec::new();
    let mut worlds = WORLDS.lock().unwrap();
    if let Some(world) = worlds.get_mut(world) {
        let addresses = world.islands().keys().cloned().collect::<Vec<_>>();
        for addr in addresses {
            island_info.push((
                Coords {
                    x: addr.0,
                    y: addr.1,
                },
                world.get_details(tick(), addr, None).unwrap(),
            ));
        }
        Ok(island_info)
    } else {
        Err("World not found".to_string())
    }
}
