use chrono::{DateTime, Utc};

/// Database row representing a system within a galaxy
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "db")]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct SystemRow {
    pub id: i64,
    pub galaxy_name: String,
    pub x: i64,
    pub y: i64,
    pub metal: i64,
    pub crew: i64,
    pub water: i64,
    pub current_tick: i64,
    pub user_galaxy_account_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl SystemRow {
    pub fn new(
        galaxy_name: String,
        coords: crate::Coords,
        resources: crate::Resources,
        current_tick: usize,
        user_galaxy_account_id: Option<i64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            galaxy_name,
            x: coords.x as i64,
            y: coords.y as i64,
            metal: resources.metal as i64,
            crew: resources.crew as i64,
            water: resources.water as i64,
            current_tick: current_tick as i64,
            user_galaxy_account_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn coords(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }

    pub fn resources(&self) -> (usize, usize, usize) {
        (self.metal as usize, self.crew as usize, self.water as usize)
    }

    pub fn current_tick_as_usize(&self) -> usize {
        self.current_tick as usize
    }
}

/// Database row representing a structure within a system
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "db")]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct StructureRow {
    pub id: i64,
    pub system_id: i64,
    pub structure_type: String,
    pub level: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl StructureRow {
    pub fn new(system_id: i64, structure_type: String, level: usize) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            system_id,
            structure_type,
            level: level as i64,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn level_as_usize(&self) -> usize {
        self.level as usize
    }
}
