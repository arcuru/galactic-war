/// Defines a struct that builds the web interface.
///
/// It includes convenience methods and wrappers to make it easier to build the web interface.
///
use axum::response::Html;

use galactic_war::{app::AppState, Coords, Resources, SystemProduction};

use std::sync::Arc;

/// Return a standardized HTML table for displaying resources
pub fn resource_table(resources: &Resources, production: &SystemProduction) -> String {
    format!("<table width=600 border=1 cellspacing=0 cellpadding=3><tr><td>💰 {}</td><td>🧑 {}</td><td>💧 {}</td><td>🏃 {}/{}/{}</td></tr></table>",
resources.metal, resources.crew, resources.water, production.metal, production.crew, production.water)
}

#[derive(Debug)]
pub struct GalacticWeb {
    pub galaxy: String,
    pub coords: Coords,
    pub body: String,
    pub linkbacks: Vec<(String, String)>,
    pub app_state: Arc<AppState>,
}

impl GalacticWeb {
    /// Create a new GalacticWeb instance.
    ///
    /// This corresponds to a single web page.
    pub fn new(galaxy: &str, coords: Coords, app_state: Arc<AppState>) -> Self {
        Self {
            galaxy: galaxy.to_string(),
            coords,
            body: String::new(),
            linkbacks: Vec::new(),
            app_state,
        }
    }

    /// Add a string to the body of the page.
    pub fn add(&mut self, content: &str) {
        self.body.push_str(content);
    }

    /// Add a string to the body of the page.
    pub fn push_str(&mut self, string: &str) {
        self.body.push_str(string);
    }

    /// Adds a new level of linkback.
    pub fn add_linkback(&mut self, name: &str, link: &str) {
        self.linkbacks.push((name.to_string(), link.to_string()));
    }

    /// Create the linkback structure for the page.
    ///
    /// This adds an expanding header to show you what level you are in the navigation.
    /// e.g.
    ///   Galaxy -> System -> Build -> Structure
    fn get_linkback(&self) -> String {
        let mut linkbuilder = format!("/{}", self.galaxy);
        let mut links = format!("<a href=\"{}\">{}</a>", linkbuilder, self.galaxy);
        for (name, link) in &self.linkbacks {
            linkbuilder.push_str(&format!("/{}", link));
            links.push_str(&format!(" -> <a href=\"{}\">{}</a>", linkbuilder, name));
        }
        links
    }

    /// Return the full HTML page.
    pub async fn get(&self) -> Result<Html<String>, String> {
        let mut page: String = r#"
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
"#
        .to_string();
        page.push_str(&self.get_linkback());
        page.push_str("<br><br>");
        let system_info = self
            .app_state
            .system_info(&self.galaxy, self.coords)
            .await?;
        page.push_str(&resource_table(
            &system_info.resources,
            &system_info.production,
        ));
        page.push_str(self.body.as_str());
        page.push_str("</body>");
        Ok(Html::from(page))
    }
}
