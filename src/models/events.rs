use chrono::{DateTime, Utc};

/// Database row representing an event within a system
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "db")]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EventRow {
    pub id: i64,
    pub system_id: i64,
    pub completion_tick: i64,
    pub action_type: String,
    pub structure_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "db")]
impl EventRow {
    pub fn new(
        system_id: i64,
        completion_tick: usize,
        action_type: String,
        structure_type: Option<String>,
    ) -> Self {
        Self {
            id: 0, // Will be set by database
            system_id,
            completion_tick: completion_tick as i64,
            action_type,
            structure_type,
            created_at: Utc::now(),
        }
    }

    pub fn completion_tick_as_usize(&self) -> usize {
        self.completion_tick as usize
    }
}
