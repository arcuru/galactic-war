use axum::response::Html;
use galactic_war::{
    config::GalaxyConfig, Coords, Details, Event, EventCallback, Galaxy, Resources, StructureType,
    SystemInfo,
};

use axum::{extract::Path, routing::get, Router};
use std::str::FromStr;
use std::{
    cmp::max,
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    // Only use GET requests
    // We will eventually use an API to allow other ways of interacting with
    // the game, likely with POST requests, but for now we will only
    // expose the game over the web interface for simplicity
    let app = Router::new()
        .route("/:galaxy", get(galaxy_get))
        .route("/:galaxy/stats", get(galaxy_stats_get))
        .route("/:galaxy/create", get(galaxy_create_get))
        .route("/:galaxy/:x/:y", get(system_get))
        .route("/:galaxy/:x/:y/", get(system_get))
        .route("/:galaxy/:x/:y/colony", get(colony_get))
        .route("/:galaxy/:x/:y/:structure", get(structure_get))
        .route("/:galaxy/:x/:y/:structure/build", get(build_structure_get))
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
    <form id="createGalaxy" method="get">
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
    let mut galaxies = GALAXIES.lock().unwrap();
    if galaxies.contains_key(&galaxy) {
        return format!("Galaxy {} already exists", galaxy);
    }
    let contents = include_str!("../galaxies/classic.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();
    galaxies.insert(galaxy.clone(), Galaxy::new(galaxy_config, tick()));
    format!("Galaxy {} created", galaxy)
}

/// Handler for GET requests to /:galaxy/:x/:y/colony
async fn colony_get(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
) -> Result<Html<String>, String> {
    let dets = structure_info(&galaxy, (x, y).into(), "Colony");

    let system_info = system_info(&galaxy, (x, y).into()).unwrap();
    let mut page = resource_table(&system_info.resources);

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
            cost.metal,
            cost.crew,
            cost.water,
            seconds_to_readable(cost.ticks)
        ));

        if system_info.resources.metal >= cost.metal
            && system_info.resources.crew >= cost.crew
            && system_info.resources.water >= cost.water
        {
            page.push_str(&format!(
                "<td bgcolor=dddddd width=200><a href=/{}/{}/{}/{}/build>Upgrade to level {}</a></td></tr>",
                galaxy, x, y, structure.to_string().to_lowercase(), level + 1));
        } else {
            // Figure out how long it will take to produce the missing resources at the current rate
            let metal_time = {
                let metal = cost.metal as isize - system_info.resources.metal as isize;
                if metal > 0 {
                    metal as usize * system_info.production.metal
                } else {
                    0
                }
            };
            let crew_time = {
                let crew = cost.crew as isize - system_info.resources.crew as isize;
                if crew > 0 {
                    crew as usize * system_info.production.crew
                } else {
                    0
                }
            };
            let water_time = {
                let water = cost.water as isize - system_info.resources.water as isize;
                if water > 0 {
                    water as usize * system_info.production.water
                } else {
                    0
                }
            };
            // Find the longest time to produce the missing resources, and the name of the typpe
            let time = max(metal_time, max(crew_time, water_time));
            let resource = if time == metal_time {
                "💰"
            } else if time == crew_time {
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
    let dets = structure_info(&galaxy, (x, y).into(), &structure);
    if let Ok(dets) = dets {
        Ok(format!("{:?}", dets))
    } else {
        Err(dets.unwrap_err())
    }
}

/// Retrieve the details of a structure in a system
fn structure_info(galaxy: &str, coords: Coords, structure: &str) -> Result<Details, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let structure_type = StructureType::from_str(structure);
        if let Ok(structure) = structure_type {
            galaxy.get_details(tick(), coords, Some(structure))
        } else {
            Err("Structure not found".to_string())
        }
    } else {
        Err("Galaxy not found".to_string())
    }
}

/// Return a standardized HTML table for displaying resources
fn resource_table(resources: &Resources) -> String {
    format!("<table width=600 border=1 cellspacing=0 cellpadding=3><tr><td width=33%>💰 {}</td><td width=33%>🪨 {}</td><td>🪵 {}</td></tr></table>",
resources.metal, resources.crew, resources.water)
}

/// Handler for GET requests to /:galaxy/:x/:y
async fn system_get(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
) -> Result<Html<String>, String> {
    let system_info = system_info(&galaxy, (x, y).into())?;

    let mut page = format!("{}<br>
<table width=600 border=0 cellSpacing=1 cellPadding=3><tbody><tr><td vAlign=top width=50%><B>Structures</b><br><font color=#CCCCC><b>",
resource_table(&system_info.resources));

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

/// Retrieve the details of an system
fn system_info(galaxy: &str, coords: Coords) -> Result<SystemInfo, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let dets = galaxy.get_details(tick(), coords, None);
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
    let event = build_structure(&galaxy, (x, y).into(), &structure);
    if let Ok(event) = event {
        Ok(format!("{:?}", event))
    } else {
        Err(event.unwrap_err())
    }
}

/// Send the structure request and return the internal type
fn build_structure(galaxy: &str, coords: Coords, structure: &str) -> Result<Event, String> {
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let structure_type = StructureType::from_str(structure);
        if let Ok(structure) = structure_type {
            galaxy.build(tick(), coords, structure)
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
    <tr><td bgcolor=dddddd><b>Isle</b></td><td bgcolor=dddddd width=15%><b>💰metal</b></td>
    <td bgcolor=dddddd width=15%><b>🪨crews</b></td><td bgcolor=dddddd width=15%><b>🪵water</b></td><td bgcolor=dddddd width=15%><b>Activity</b></td><td width=2%></td></tr>
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
                galaxy, addr.x, addr.y, "System", addr.x, addr.y, info.resources.metal, info.resources.crew, info.resources.water, activity_hover, activity
            ));
            }
            _ => {
                return Err("Unexpected Details type".to_string());
            }
        }
    }
    Ok(Html::from(page.to_string()))
}

/// Handler for GET requests to /:galaxy
///
/// Serves the Galaxy Dashboard page
async fn galaxy_get(Path(galaxy): Path<String>) -> Result<Html<String>, String> {
    galaxy_stats_get(Path(galaxy)).await
}

/// Returns all the visible info for the galaxy
fn galaxy_info(galaxy: &str) -> Result<Vec<(Coords, Details)>, String> {
    let mut system_info = Vec::new();
    let mut galaxies = GALAXIES.lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let addresses = galaxy.systems().keys().cloned().collect::<Vec<_>>();
        for addr in addresses {
            system_info.push((addr, galaxy.get_details(tick(), addr, None).unwrap()));
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
