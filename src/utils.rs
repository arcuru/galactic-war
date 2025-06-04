use crate::{Coords, Details, Galaxy, Resources, SystemInfo, SystemProduction};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[cfg(feature = "bin")]
use crate::app::AppState;
#[cfg(feature = "bin")]
use std::sync::Arc as StdArc;

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
#[cfg(feature = "bin")]
pub async fn system_info(
    galaxy: &str,
    coords: Coords,
    app_state: &StdArc<AppState>,
) -> Result<SystemInfo, String> {
    let dets = app_state
        .get_galaxy_details(galaxy, tick(), coords, None)
        .await?;
    match dets {
        Details::System(info) => Ok(info),
        _ => Err("Unexpected Details type".to_string()),
    }
}

/// Retrieve the details of an system (synchronous version for web interface)
#[cfg(feature = "bin")]
pub fn system_info_sync(galaxy: &str, coords: Coords) -> Result<SystemInfo, String> {
    let mut galaxies = (**GALAXIES).lock().unwrap();
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
