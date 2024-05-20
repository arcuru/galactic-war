use axum::response::Html;
use galactic_war::{
    config::GalaxyConfig, Details, Event, EventCallback, Galaxy, StructureType, SystemInfo,
};

use axum::{extract::Path, routing::get, Router};
use std::str::FromStr;
use std::{
    cmp::max,
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Coords for systems
///
/// TODO: Integrate this everywhere
struct Coords {
    x: usize,
    y: usize,
}

lazy_static::lazy_static! {
    // Safely share the galaxies between threads
    static ref GALAXIES: Arc<Mutex<HashMap<String, Galaxy>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = include_str!("../galaxies/classic.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();

    // Create a new galaxy named "one"
    {
        let mut galaxies = GALAXIES.lock().unwrap();
        galaxies.insert("one".to_string(), Galaxy::new(galaxy_config, tick()));
    }

    serve().await
}

/// Serve the Galaxy(s) over HTTP
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // All the entry points expose both a GET and POST interface
    // The difference is just in presentation
    // A GET request will just display an HTML page, and a POST request will return the data being used to display
    // Internally all GET requests use the same data as returned from the POST request
    let app = Router::new()
        .route("/:galaxy", get(galaxy_get).post(galaxy_post))
        .route(
            "/:galaxy/stats",
            get(galaxy_stats_get).post(galaxy_stats_post),
        )
        .route(
            "/:galaxy/create",
            get(galaxy_create_get).post(galaxy_create_post),
        )
        .route("/:galaxy/:x/:y", get(system_get).post(system_post))
        .route("/:galaxy/:x/:y/", get(system_get).post(system_post))
        .route("/:galaxy/:x/:y/fortress", get(fortress_get))
        .route(
            "/:galaxy/:x/:y/:structure",
            get(structure_get).post(structure_post),
        )
        .route(
            "/:galaxy/:x/:y/:structure/build",
            get(build_structure_get).post(build_structure_post),
        )
        .route("/", get(base_get));

    // Hardcode serve on port 3050
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
    <title>Galactic War</title>
    <script>
        function navigate() {
            var selectedGalaxy = document.getElementById("galaxies").value;
            window.location.href = "/" + selectedGalaxy;
        }
        window.onload = function() {
            document.getElementById("createGalaxy").onsubmit = function() {
                var galaxyName = document.getElementById("newGalaxy").value;
                this.action = "/" + galaxyName + "/create";
            }
        }
    </script>
</head>
<body>
    <h1>Galactic War</h1>
    <select id="galaxies">
    "#
    .to_string();
    for galaxy in GALAXIES.lock().unwrap().keys() {
        page.push_str(&format!("<option value=\"{}\">{}</option>", galaxy, galaxy));
    }
    page.push_str(
        r#"</select>
    <button onclick="navigate()">Go to galaxy</button>
    <br><br>
    <h1>Create a new galaxy</h1>
    <form id="createGalaxy" method="post">
        <label for="newGalaxy">Enter New Galaxy Name:</label>
        <input type="text" id="newGalaxy" name="newGalaxy" required>
        <input type="submit" value="Submit">
    </form>
</body>
"#,
    );
    Html::from(page)
}

/// Handler for GET requests to /:galaxy/create
async fn galaxy_create_get(Path(galaxy): Path<String>) -> String {
    galaxy_create_post(Path(galaxy)).await
}

/// Handler for POST requests to /:galaxy/create
async fn galaxy_create_post(Path(galaxy): Path<String>) -> String {
    let mut galaxies = GALAXIES.lock().unwrap();
    if galaxies.contains_key(&galaxy) {
        return format!("Galaxy {} already exists", galaxy);
    }
    let contents = include_str!("../galaxies/classic.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();
    galaxies.insert(galaxy.clone(), Galaxy::new(galaxy_config, tick()));
    format!("Galaxy {} created", galaxy)
}

/// Handler for GET requests to /:galaxy/:x/:y/fortress
async fn fortress_get(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
) -> Result<Html<String>, String> {
    let dets = structure_info(&galaxy, (x, y), "Fortress");

    let system_info = system_info(&galaxy, (x, y)).unwrap();
    let mut page = resource_table(system_info.gold, system_info.stone, system_info.lumber);

    // Push the table header
    page.push_str("<p><table width=600 border=0 cellspacing=1 cellpadding=3>");

    let structure_costs = match dets.unwrap() {
        Details::Structure(info) => info.builds.unwrap(),
        _ => {
            return Err("Unexpected Details type".to_string());
        }
    };

    for (structure, cost) in structure_costs.iter() {
        let level = system_info.structures.get(structure).unwrap_or(&0);
        page.push_str(&format!(
            "<tr><td bgcolor=dddddd>🛖 
            <a href=/{}/{}/{}/{}>{} (level {})</a>",
            galaxy,
            x,
            y,
            structure.to_string().to_lowercase(),
            structure,
            level
        ));

        page.push_str(&format!(
            "<br>Cost: 💰{}/🪨{}/🪵{}   Duration: {}</td>",
            cost.gold,
            cost.stone,
            cost.lumber,
            seconds_to_readable(cost.ticks)
        ));

        if system_info.gold >= cost.gold
            && system_info.stone >= cost.stone
            && system_info.lumber >= cost.lumber
        {
            page.push_str(&format!(
                "<td bgcolor=dddddd width=200><a href=/{}/{}/{}/{}/build>Upgrade to level {}</a></td></tr>",
                galaxy, x, y, structure.to_string().to_lowercase(), level + 1));
        } else {
            // Figure out how long it will take to produce the missing resources at the current rate
            let gold_time = {
                let gold = cost.gold as isize - system_info.gold as isize;
                if gold > 0 {
                    gold as usize * system_info.production.gold
                } else {
                    0
                }
            };
            let stone_time = {
                let stone = cost.stone as isize - system_info.stone as isize;
                if stone > 0 {
                    stone as usize * system_info.production.stone
                } else {
                    0
                }
            };
            let lumber_time = {
                let lumber = cost.lumber as isize - system_info.lumber as isize;
                if lumber > 0 {
                    lumber as usize * system_info.production.lumber
                } else {
                    0
                }
            };
            // Find the longest time to produce the missing resources, and the name of the typpe
            let time = max(gold_time, max(stone_time, lumber_time));
            let resource = if time == gold_time {
                "💰"
            } else if time == stone_time {
                "🪨"
            } else {
                "🪵"
            };

            page.push_str(&format!(
                "<td bgcolor=dddddd width=200>Upgrade available in<br>~{} (Need {})</td></tr>",
                seconds_to_readable(time),
                resource
            ));
        }
    }
    Ok(Html::from(page.to_string()))
}

/// Answer a GET request to the structure endpoint
///
/// TODO: This should respond with an actual HTML page
async fn structure_get(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    println!("StructureGet: Galaxy: {}, x: {}, y: {}", galaxy, x, y);
    let dets = structure_info(&galaxy, (x, y), &structure);
    if let Ok(dets) = dets {
        Ok(format!("{:?}", dets))
    } else {
        Err(dets.unwrap_err())
    }
}

/// Answer a POST request to the structure endpoint
///
/// TODO: This should respond with JSON
async fn structure_post(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let dets = structure_info(&galaxy, (x, y), &structure);
    if let Ok(dets) = dets {
        Ok(format!("{:?}", dets))
    } else {
        Err(dets.unwrap_err())
    }
}

/// Retrieve the details of a structure on an system
fn structure_info(
    galaxy: &str,
    (x, y): (usize, usize),
    structure: &str,
) -> Result<Details, String> {
    println!("StructurePost: Galaxy: {}, x: {}, y: {}", galaxy, x, y);
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let structure_type = StructureType::from_str(structure);
        if let Ok(structure) = structure_type {
            galaxy.get_details(tick(), (x, y), Some(structure))
        } else {
            Err("Structure not found".to_string())
        }
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Return a standardized HTML table for displaying resources
fn resource_table(gold: usize, stone: usize, lumber: usize) -> String {
    format!("<table width=600 border=1 cellspacing=0 cellpadding=3><tr><td width=33%>💰 {}</td><td width=33%>🪨 {}</td><td>🪵 {}</td></tr></table>",
gold, stone, lumber)
}

/// Handler for GET requests to /:galaxy/:x/:y
async fn system_get(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
) -> Result<Html<String>, String> {
    let system_info = system_info(&galaxy, (x, y))?;

    let mut page = format!("{}<br>
<table width=600 border=0 cellSpacing=1 cellPadding=3><tbody><tr><td vAlign=top width=50%><B>Structures</b><br><font color=#CCCCC><b>",
resource_table(system_info.gold, system_info.stone, system_info.lumber));

    for (structure, level) in system_info.structures.iter() {
        page.push_str(&format!(
            "🛖 <a href=/{}/{}/{}/{}>{} (level {})</a><br>",
            galaxy,
            x,
            y,
            structure.to_string().to_lowercase(),
            structure,
            level
        ));
    }

    page.push_str(&format!(
        "<td vAlign=top><b>Score</b><br>{}</td></tr>",
        system_info.score
    ));
    Ok(Html::from(page.to_string()))
}

/// Handler for POST requests to /:galaxy/:x/:y
///
/// TODO: This should respond with JSON
async fn system_post(Path((galaxy, x, y)): Path<(String, usize, usize)>) -> Result<String, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(&galaxy) {
        if let Ok(dets) = galaxy.get_details(tick(), (x, y), None) {
            Ok(format!("{:?}", dets))
        } else {
            Err("System not found".to_string())
        }
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Retrieve the details of an system
fn system_info(galaxy: &str, (x, y): (usize, usize)) -> Result<SystemInfo, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let dets = galaxy.get_details(tick(), (x, y), None);
        if let Ok(dets) = dets {
            match dets {
                Details::System(info) => Ok(info),
                _ => Err("Unexpected Details type".to_string()),
            }
        } else {
            Err(dets.unwrap_err())
        }
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Handler for GET requests to /:galaxy/:x/:y/:structure/build
async fn build_structure_get(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let event = build_structure(&galaxy, (x, y), &structure);
    if let Ok(event) = event {
        Ok(format!("{:?}", event))
    } else {
        Err(event.unwrap_err())
    }
}

/// Handler for POST requests to /:galaxy/:x/:y/:structure/build
///
/// TODO: This should respond with JSON
async fn build_structure_post(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
) -> Result<String, String> {
    let event = build_structure(&galaxy, (x, y), &structure);
    if let Ok(event) = event {
        Ok(format!("{:?}", event))
    } else {
        Err(event.unwrap_err())
    }
}

/// Send the structure request and return the internal type
fn build_structure(galaxy: &str, (x, y): (usize, usize), structure: &str) -> Result<Event, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let structure_type = StructureType::from_str(structure);
        if let Ok(structure) = structure_type {
            galaxy.build(tick(), (x, y), structure)
        } else {
            Err("Structure not found".to_string())
        }
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Handler for GET requests to /:galaxy/stats
async fn galaxy_stats_get(Path(galaxy): Path<String>) -> Result<Html<String>, String> {
    let mut page = "
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td align=center><b>
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td bgcolor=dddddd><b>Isle</b></td><td bgcolor=dddddd width=15%><b>💰Gold</b></td>
    <td bgcolor=dddddd width=15%><b>🪨Stones</b></td><td bgcolor=dddddd width=15%><b>🪵Lumber</b></td><td bgcolor=dddddd width=15%><b>Activity</b></td><td width=2%></td></tr>
".to_string();
    for (addr, dets) in galaxy_info(&galaxy).unwrap() {
        match dets {
            Details::System(info) => {
                // Build an activity string
                // Scan through the activities to check if we're building something, and if so add a hover with the details
                let mut activity = String::new();
                let mut activity_hover = String::new();
                for event in info.events.iter() {
                    if let EventCallback::Build = event.action {
                        activity.push_str("🏗️");
                        let eta = event.completion - tick();

                        activity_hover.push_str(&format!(
                            "Structure {}: {} remaining",
                            event.structure.unwrap(),
                            seconds_to_readable(eta)
                        ));
                    }
                }
                page.push_str(&format!(
                "<tr><td bgcolor=#ffffff><a href=/{}/{}/{}>{} ({}:{})</a></td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff>{}</td><td bgcolor=#ffffff title=\"{}\">{}</td></tr>",
                galaxy, addr.x, addr.y, "System", addr.x, addr.y, info.gold, info.stone, info.lumber, activity_hover, activity
            ));
            }
            _ => {
                return Err("Unexpected Details type".to_string());
            }
        }
    }
    Ok(Html::from(page.to_string()))
}

/// Handler for POST requests to /:galaxy/stats
///
/// TODO: This should respond with JSON
async fn galaxy_stats_post(Path(galaxy): Path<String>) -> Result<String, String> {
    println!("galaxy_stats_post: {}", galaxy);
    galaxy_stats(&galaxy)
}

fn galaxy_stats(galaxy: &str) -> Result<String, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        Ok(galaxy.stats(tick()).unwrap())
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Handler for GET requests to /:galaxy
///
/// Serves the Galaxy Dashboard page
async fn galaxy_get(Path(galaxy): Path<String>) -> Result<Html<String>, String> {
    galaxy_stats_get(Path(galaxy)).await
}

/// Handler for POST requests to /:galaxy
async fn galaxy_post(Path(galaxy): Path<String>) -> String {
    format!("Welcome to Galaxy {}!", galaxy).to_string()
}

/// Returns all the visible info for the galaxy
fn galaxy_info(galaxy: &str) -> Result<Vec<(Coords, Details)>, String> {
    let mut system_info = Vec::new();
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let addresses = galaxy.systems().keys().cloned().collect::<Vec<_>>();
        for addr in addresses {
            system_info.push((
                Coords {
                    x: addr.0,
                    y: addr.1,
                },
                galaxy.get_details(tick(), addr, None).unwrap(),
            ));
        }
        Ok(system_info)
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Convert seconds into a human readable format
fn seconds_to_readable(seconds: usize) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:2}:{:02}", minutes, seconds)
    }
}
