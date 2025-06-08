use chrono::{DateTime, Utc};

/// Database row representing a galaxy
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct GalaxyRow {
    pub name: String,
    pub config_file: String,
    pub tick: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GalaxyRow {
    pub fn new(name: String, config_file: String, tick: usize) -> Self {
        let now = Utc::now();
        Self {
            name,
            config_file,
            tick: tick as i64,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn tick_as_usize(&self) -> usize {
        self.tick as usize
    }
}
