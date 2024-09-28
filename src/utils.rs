use galactic_war::{Coords, Details, Galaxy, Resources, SystemInfo, SystemProduction};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

lazy_static::lazy_static! {
    // Safely share the galaxies between threads
    pub static ref GALAXIES: Arc<Mutex<HashMap<String, Galaxy>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Return the current second
pub fn tick() -> usize {
    // Return the current second since the epoch
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

/// Retrieve the details of an system
pub fn system_info(galaxy: &str, coords: Coords) -> Result<SystemInfo, String> {
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

/// Return a standardized HTML table for displaying resources
pub fn resource_table(resources: &Resources, production: &SystemProduction) -> String {
    // FIXME: move to web.rs
    format!("<table width=600 border=1 cellspacing=0 cellpadding=3><tr><td>💰 {}</td><td>🧑 {}</td><td>💧 {}</td><td>🏃 {}/{}/{}</td></tr></table>",
resources.metal, resources.crew, resources.water, production.metal, production.crew, production.water)
}
