/// Defines a struct that builds the web interface.
///
/// It includes convenience methods and wrappers to make it easier to build the web interface.
///
use axum::response::Html;

use galactic_war::{
    utils::{resource_table, system_info_sync},
    Coords,
};

#[derive(Debug)]
pub struct GalacticWeb {
    pub galaxy: String,
    pub coords: Coords,
    pub body: String,
    pub linkbacks: Vec<(String, String)>,
}

impl GalacticWeb {
    /// Create a new GalacticWeb instance.
    ///
    /// This corresponds to a single web page.
    pub fn new(galaxy: &str, coords: Coords) -> GalacticWeb {
        GalacticWeb {
            galaxy: galaxy.to_string(),
            coords,
            body: String::new(),
            linkbacks: vec![(
                "systemname".to_string(),
                format!("{}/{}", coords.x, coords.y),
            )],
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
    pub fn get(&self) -> Result<Html<String>, String> {
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
        let system_info = system_info_sync(&self.galaxy, self.coords)?;
        page.push_str(&resource_table(
            &system_info.resources,
            &system_info.production,
        ));
        page.push_str(self.body.as_str());
        page.push_str("</body>");
        Ok(Html::from(page))
    }
}
