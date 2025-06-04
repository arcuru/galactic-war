use axum::response::Html;
use galactic_war::{
    app::AppState,
    config::GalaxyConfig,
    utils::{system_info, tick, GALAXIES},
    Coords, Details, EventCallback, StructureType,
};

mod web;
use crate::web::GalacticWeb;

use axum::{extract::Path, routing::get, Extension, Router};
use std::cmp::max;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    #[cfg(feature = "bin")]
    env_logger::init();

    log::info!("Starting Galactic War server...");

    // Initialize application state with persistence
    let app_state = Arc::new(AppState::new().await?);

    // FIXME: Hardcoded galaxy config
    let contents = include_str!("../galaxies/blitz.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();

    // Create a new galaxy named "one" if it doesn't exist
    match app_state.create_galaxy("one", &galaxy_config, tick()).await {
        Ok(msg) => log::info!("{}", msg),
        Err(e) => log::error!("Failed to create initial galaxy: {}", e),
    }

    // Setup graceful shutdown
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
        log::info!("Received shutdown signal");
        if let Err(e) = Arc::try_unwrap(app_state_clone).unwrap().shutdown().await {
            log::error!("Error during shutdown: {}", e);
        }
        std::process::exit(0);
    });

    serve(app_state).await
}

/// Serve the Galaxy(s) over HTTP
async fn serve(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // Only use GET requests
    // We will eventually use an API to allow other ways of interacting with
    // the game, likely with POST requests, but for now we will only
    // expose the game over the web interface for simplicity
    let app = Router::new()
        .route("/favicon.ico", get(favicon_get))
        .route("/robots.txt", get(robots_get))
        .route("/:galaxy", get(galaxy_get))
        .route("/:galaxy/", get(galaxy_get))
        .route("/:galaxy/stats", get(galaxy_stats_get))
        .route("/:galaxy/create", get(galaxy_create_get))
        .route("/:galaxy/:x/:y", get(system_get))
        .route("/:galaxy/:x/:y/", get(system_get))
        .route("/:galaxy/:x/:y/build", get(system_build))
        .route("/:galaxy/:x/:y/build/", get(system_build))
        .route("/:galaxy/:x/:y/build/:structure", get(system_build_struct))
        .route("/:galaxy/:x/:y/:structure", get(structure_get))
        .route("/", get(base_get))
        .layer(Extension(app_state));

    // Hardcode serve on port 3050
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3050").await.unwrap();
    log::info!("Server listening on http://0.0.0.0:3050");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Handler for GET requests to /favicon.ico
async fn favicon_get() -> Html<String> {
    Html::from("".to_string())
}

/// Handler for GET requests to /robots.txt
async fn robots_get() -> Html<String> {
    Html::from("User-agent: *\nDisallow: /".to_string())
}

/// Handler for GET requests to /
async fn base_get(Extension(app_state): Extension<Arc<AppState>>) -> Html<String> {
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

    for galaxy in app_state.list_galaxies().await {
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
async fn galaxy_create_get(
    Path(galaxy): Path<String>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> String {
    // FIXME: Hardcoded galaxy config
    let contents = include_str!("../galaxies/blitz.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();

    match app_state
        .create_galaxy(&galaxy, &galaxy_config, tick())
        .await
    {
        Ok(msg) => msg,
        Err(e) => e,
    }
}

/// Handler for GET requests to /:galaxy/:x/:y/build/:structure
async fn system_build_struct(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<String, String> {
    if structure.is_empty() {
        return Err("No structure specified".to_string());
    }

    let structure_type = StructureType::from_str(&structure)
        .map_err(|_| format!("Invalid structure type: {}", structure))?;
    let event = app_state
        .build_structure(&galaxy, tick(), (x, y).into(), structure_type)
        .await?;
    Ok(format!("{:?}", event))
}

/// Handler for GET requests to /:galaxy/:x/:y/build
async fn system_build(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    let dets = structure_info(&galaxy, (x, y).into(), "Colony", &app_state).await;

    let system_info = system_info(&galaxy, (x, y).into(), &app_state).await?;
    let mut page = GalacticWeb::new(&galaxy, (x, y).into());
    page.add_linkback("Build", "build");

    // Push the table header
    page.add("<p><table width=600 border=0 cellspacing=1 cellpadding=3>");

    let structure_costs = match dets? {
        Details::Structure(info) => info.builds.unwrap(),
        _ => {
            return Err("Unexpected Details type".to_string());
        }
    };

    for (structure, cost) in structure_costs.iter() {
        let level = system_info.structures.get(structure).unwrap_or(&0);
        page.add(&format!(
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
            "<br>Cost: 💰{}/🧑{}/💧{}   Duration: {}</td>",
            cost.resources.metal,
            cost.resources.crew,
            cost.resources.water,
            seconds_to_readable(cost.ticks)
        ));

        if system_info.resources >= cost.resources {
            page.push_str(&format!(
                "<td bgcolor=dddddd width=200><a href=/{}/{}/{}/build/{}>Upgrade to level {}</a></td></tr>",
                galaxy, x, y, structure.to_string().to_lowercase(), level + 1));
        } else {
            // Figure out how long it will take to produce the missing resources at the current rate
            let metal_time = {
                let metal = cost.resources.metal as isize - system_info.resources.metal as isize;
                if metal > 0 {
                    metal as usize * system_info.production.metal
                } else {
                    0
                }
            };
            let crew_time = {
                let crew = cost.resources.crew as isize - system_info.resources.crew as isize;
                if crew > 0 {
                    crew as usize * system_info.production.crew
                } else {
                    0
                }
            };
            let water_time = {
                let water = cost.resources.water as isize - system_info.resources.water as isize;
                if water > 0 {
                    water as usize * system_info.production.water
                } else {
                    0
                }
            };
            // Find the longest time to produce the missing resources, and the name of the type
            let time = max(metal_time, max(crew_time, water_time));
            let resource = if time == metal_time {
                "💰"
            } else if time == crew_time {
                "🧑"
            } else {
                "💧"
            };

            page.push_str(&format!(
                "<td bgcolor=dddddd width=200>Upgrade available in<br>~{} (Need {})</td></tr>",
                seconds_to_readable(time),
                resource
            ));
        }
    }

    page.add("</table>");
    page.get()
}

/// Handler for GET requests to /:galaxy/:x/:y/:structure
///
/// This displays very basic info about the structure
async fn structure_get(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    let dets = structure_info(&galaxy, (x, y).into(), &structure, &app_state).await;
    let mut page = GalacticWeb::new(&galaxy, (x, y).into());
    page.add_linkback(&structure, &structure);

    if let Ok(dets) = dets {
        let dets = match dets {
            Details::Structure(info) => info,
            _ => {
                return Err("Unexpected Details type".to_string());
            }
        };
        {
            let mut galaxies = GALAXIES.try_lock().unwrap();
            if let Some(galaxy) = galaxies.get_mut(&galaxy) {
                let config = galaxy.get_config();
                if let Some(structure_config) = config.systems.structures.get(&structure) {
                    if dets.level > 0 {
                        page.add(&format!("<h2>{} (level {})</h2>", structure, dets.level));
                    } else {
                        page.add(&format!("<h2>{}</h2>", structure));
                    }
                    if structure_config.description.is_some() {
                        page.add(&format!(
                            "<p>{}</p>",
                            structure_config.description.as_ref().unwrap()
                        ));
                    }
                    let production = structure_config.get_production(dets.level);
                    if production.metal > 0 || production.crew > 0 || production.water > 0 {
                        page.add("<h3>Produces:</h3><b>");
                        if production.metal > 0 {
                            page.add(&format!("💰 Metal: {} per hour<br>", production.metal));
                        }
                        if production.crew > 0 {
                            page.add(&format!("🧑 Crew: {} per hour<br>", production.crew));
                        }
                        if production.water > 0 {
                            page.add(&format!("💧 Water: {} per hour<br>", production.water));
                        }
                        page.add("</b>");
                    }
                }
            }
        }
        // This locks the galaxy, so we need to drop the previous lock
        page.get()
    } else {
        Err(dets.unwrap_err())
    }
}

/// Retrieve the details of a structure in a system
async fn structure_info(
    galaxy: &str,
    coords: Coords,
    structure: &str,
    app_state: &Arc<AppState>,
) -> Result<Details, String> {
    let structure_type = StructureType::from_str(structure)
        .map_err(|_| format!("Structure '{}' not found", structure))?;
    app_state
        .get_galaxy_details(galaxy, tick(), coords, Some(structure_type))
        .await
}

/// Handler for GET requests to /:galaxy/:x/:y
async fn system_get(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    let system_info = system_info(&galaxy, (x, y).into(), &app_state).await?;
    let mut page = GalacticWeb::new(&galaxy, (x, y).into());

    page.add("<br><table width=600 border=0 cellSpacing=1 cellPadding=3><tbody><tr><td vAlign=top width=50%><B>Structures</b><br><font color=#CCCCC><b>");

    for (structure, level) in system_info.structures.iter() {
        page.add(&format!(
            "🛖 <a href=/{}/{}/{}/{}>{} (level {})</a><br>",
            galaxy,
            x,
            y,
            structure.to_string().to_lowercase(),
            structure,
            level
        ));
    }

    page.add(&format!(
        "<td vAlign=top><b>Score</b><br>{}</td></tr>",
        system_info.score
    ));

    // Now add a link to the build page
    page.add(&format!(
        "<tr><td vAlign=top><br><a href=/{}/{}/{}/build>Build/Upgrade Structures</a></td></tr>",
        galaxy, x, y
    ));

    page.get()
}

/// Handler for GET requests to /:galaxy/stats
async fn galaxy_stats_get(
    Path(galaxy): Path<String>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    let mut page = "
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td align=center><b>
    <table width=600 border=0 cellspacing=1 cellpadding=3>
    <tr><td bgcolor=dddddd><b>Isle</b></td><td bgcolor=dddddd width=15%><b>💰 Metal</b></td>
    <td bgcolor=dddddd width=15%><b>🧑 Crew</b></td><td bgcolor=dddddd width=15%><b>💧 Water</b></td><td bgcolor=dddddd width=15%><b>Activity</b></td><td width=2%></td></tr>
".to_string();

    for (addr, dets) in galaxy_info(&galaxy, &app_state).await? {
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
async fn galaxy_get(
    Path(galaxy): Path<String>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    galaxy_stats_get(Path(galaxy), Extension(app_state)).await
}

/// Returns all the visible info for the galaxy
async fn galaxy_info(
    galaxy: &str,
    _app_state: &Arc<AppState>,
) -> Result<Vec<(Coords, Details)>, String> {
    let mut system_info = Vec::new();
    let mut galaxies = GALAXIES.try_lock().unwrap();
    if let Some(galaxy) = galaxies.get_mut(galaxy) {
        let addresses = galaxy.systems().keys().cloned().collect::<Vec<_>>();
        for addr in addresses {
            system_info.push((addr, galaxy.get_details(tick(), addr, None).unwrap()));
        }
        Ok(system_info)
    } else {
        Err(format!("Galaxy '{}' not found", galaxy).to_string())
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
