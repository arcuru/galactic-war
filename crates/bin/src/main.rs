use axum::response::Html;
use galactic_war::{
    app::AppState, config::GalaxyConfig, tick, Coords, Details, EventCallback, StructureType,
};

use std::sync::Arc;

mod auth;
mod web;

use crate::web::GalacticWeb;

use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Router,
};
use std::cmp::max;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    log::info!("Starting Galactic War server...");

    // Initialize application state with persistence
    let app_state = Arc::new(AppState::new().await?);

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
        // Auth routes
        .route("/login", get(auth::login_page))
        .route("/login", post(auth::handle_login))
        .route("/register", get(auth::register_page))
        .route("/register", post(auth::handle_register))
        .route("/logout", get(auth::handle_logout))
        .route("/dashboard", get(auth::user_dashboard))
        .route("/join-galaxy", post(auth::handle_join_galaxy))
        .route("/galaxy/:galaxy/dashboard", get(auth::galaxy_dashboard))
        // Galaxy routes
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
<!DOCTYPE html>
<html>
<head>
    <title>Galactic War</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; }
        .auth-links { display: flex; gap: 15px; }
        .auth-links a { 
            text-decoration: none; color: white; background: #007cba; 
            padding: 10px 20px; border-radius: 4px; 
        }
        .auth-links a:hover { background: #005a8a; }
        .galaxy-section { margin-bottom: 30px; }
        button { 
            padding: 10px 20px; background: #007cba; color: white; 
            border: none; border-radius: 4px; cursor: pointer; 
        }
        button:hover { background: #005a8a; }
        select, input[type="text"] { 
            padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-right: 10px; 
        }
        .form-group { margin-bottom: 15px; }
        .intro { margin-bottom: 30px; background: #f0f8ff; padding: 20px; border-radius: 5px; }
    </style>
    <script>
        function navigate() {
            var selectedGalaxy = document.getElementById("galaxies").value;
            if (selectedGalaxy) {
                window.location.href = "/" + selectedGalaxy;
            }
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
    <div class="header">
        <h1>🌌 Galactic War</h1>
        <div class="auth-links">
            <a href="/login">Login</a>
            <a href="/register">Register</a>
            <a href="/dashboard">Dashboard</a>
        </div>
    </div>
    
    <div class="intro">
        <h2>Welcome to Galactic War!</h2>
        <p>A space conquest game where you manage stellar systems, build structures, and expand your galactic empire.</p>
        <p><strong>New players:</strong> <a href="/register">Register an account</a> to join galaxies and own systems!</p>
        <p><strong>Existing players:</strong> <a href="/login">Login</a> to access your dashboard and manage your empire.</p>
    </div>
    
    <div class="galaxy-section">
        <h2>Browse Public Galaxies</h2>
        <p>View galaxy statistics and explore systems (read-only):</p>
        <select id="galaxies">
            <option value="">Choose a galaxy...</option>
    "#
    .to_string();

    for galaxy in app_state.list_galaxies().await {
        page.push_str(&format!("<option value=\"{}\">{}</option>", galaxy, galaxy));
    }

    page.push_str(
        r#"</select>
        <button onclick="navigate()">View Galaxy</button>
    </div>
    
    <div class="galaxy-section">
        <h2>Create a New Galaxy</h2>
        <form id="createGalaxy" method="get">
            <div class="form-group">
                <label for="newGalaxy">Enter New Galaxy Name:</label>
                <input type="text" id="newGalaxy" name="newGalaxy" required placeholder="Enter galaxy name">
                <button type="submit">Create Galaxy</button>
            </div>
        </form>
    </div>
</body>
</html>
"#,
    );
    Html::from(page)
}

/// Handler for GET requests to /:galaxy/create
#[axum::debug_handler]
async fn galaxy_create_get(
    Path(galaxy): Path<String>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Html<String> {
    // FIXME: Hardcoded galaxy config
    let contents = include_str!("../galaxies/blitz.yaml");
    let galaxy_config: GalaxyConfig = serde_yaml::from_str(contents).unwrap();

    let result = match app_state
        .create_galaxy(&galaxy, &galaxy_config, tick())
        .await
    {
        Ok(msg) => msg,
        Err(e) => e,
    };

    Html::from(result)
}

/// Handler for GET requests to /:galaxy/:x/:y/build/:structure
async fn system_build_struct(
    Path((galaxy, x, y, structure)): Path<(String, usize, usize, String)>,
    jar: axum_extra::extract::CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<String, String> {
    if structure.is_empty() {
        return Err("No structure specified".to_string());
    }

    // Check if user is authenticated and owns this system
    if let Some(user) = auth::get_current_user(jar, Extension(app_state.clone())).await {
        if let Some(db) = app_state.database() {
            let user_service = galactic_war::UserService::new(db.clone());

            // Check if user has an account in this galaxy
            if let Ok(Some(account)) = user_service.get_user_galaxy_account(user.id, &galaxy).await
            {
                // Check if user owns this system
                if let Ok(user_systems) = user_service.get_user_systems_coords(account.id).await {
                    let coords = (x, y).into();
                    if user_systems.contains(&coords) {
                        // User owns this system, allow the build
                        let structure_type = StructureType::from_str(&structure)
                            .map_err(|_| format!("Invalid structure type: {}", structure))?;
                        let event = app_state
                            .build_structure(&galaxy, tick(), coords, structure_type)
                            .await?;
                        return Ok(format!("{:?}", event));
                    }
                }
            }
        }
        Err("You don't own this system".to_string())
    } else {
        Err("You must be logged in to build structures".to_string())
    }
}

/// Handler for GET requests to /:galaxy/:x/:y/build
async fn system_build(
    Path((galaxy, x, y)): Path<(String, usize, usize)>,
    jar: axum_extra::extract::CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, String> {
    // Check if user is authenticated and owns this system
    if let Some(user) = auth::get_current_user(jar, Extension(app_state.clone())).await {
        if let Some(db) = app_state.database() {
            let user_service = galactic_war::UserService::new(db.clone());

            // Check if user has an account in this galaxy
            if let Ok(Some(account)) = user_service.get_user_galaxy_account(user.id, &galaxy).await
            {
                // Check if user owns this system
                if let Ok(user_systems) = user_service.get_user_systems_coords(account.id).await {
                    let coords = (x, y).into();
                    if !user_systems.contains(&coords) {
                        return Err("You don't own this system".to_string());
                    }
                } else {
                    return Err("Failed to check system ownership".to_string());
                }
            } else {
                return Err("You don't have an account in this galaxy".to_string());
            }
        } else {
            return Err("User authentication not available".to_string());
        }
    } else {
        return Err("You must be logged in to build structures".to_string());
    }
    let dets = structure_info(&galaxy, (x, y).into(), "Colony", &app_state).await;

    let system_info = app_state.system_info(&galaxy, (x, y).into()).await?;
    let mut page = GalacticWeb::new(&galaxy, (x, y).into(), app_state.clone());
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
    let mut page = GalacticWeb::new(&galaxy, (x, y).into(), app_state.clone());
    page.add_linkback(&structure, &structure);

    if let Ok(dets) = dets {
        let dets = match dets {
            Details::Structure(info) => info,
            _ => {
                return Err("Unexpected Details type".to_string());
            }
        };
        {
            let galaxies = app_state.galaxies().lock().unwrap();
            if let Some(galaxy_obj) = galaxies.get(&galaxy) {
                let config = galaxy_obj.get_config();
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
    let system_info = app_state.system_info(&galaxy, (x, y).into()).await?;
    let mut page = GalacticWeb::new(&galaxy, (x, y).into(), app_state.clone());

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
                    // For now we only have 1 event type, but we'll add more later
                    #[allow(unreachable_patterns)]
                    match event.action {
                        EventCallback::Build => {
                            activity.push_str("🏗️");
                            let eta = event.completion - tick();

                            activity_hover.push_str(&format!(
                                "Structure {}: {} remaining",
                                event.structure.unwrap(),
                                seconds_to_readable(eta)
                            ));
                        }
                        _ => {
                            activity.push('🔄');
                            activity_hover.push_str("Something is wrong");
                        }
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
    galaxy_name: &str,
    app_state: &Arc<AppState>,
) -> Result<Vec<(Coords, Details)>, String> {
    let mut system_info = Vec::new();

    // First, get all the addresses
    let addresses = {
        let galaxies = app_state.galaxies().lock().unwrap();
        if let Some(galaxy) = galaxies.get(galaxy_name) {
            galaxy.systems().keys().cloned().collect::<Vec<_>>()
        } else {
            return Err(format!("Galaxy '{}' not found", galaxy_name));
        }
    };

    // Now get details for each address
    for addr in addresses {
        match app_state
            .get_galaxy_details(galaxy_name, tick(), addr, None)
            .await
        {
            Ok(details) => system_info.push((addr, details)),
            Err(e) => return Err(e),
        }
    }

    Ok(system_info)
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
