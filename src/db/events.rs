#[cfg(feature = "db")]
use super::{Database, PersistenceError};
#[cfg(feature = "db")]
use crate::models::EventRow;
#[cfg(feature = "db")]
use sqlx::Row;

#[cfg(feature = "db")]
impl Database {
    /// Save an event to the database
    pub async fn save_event(
        &self,
        system_id: i64,
        completion_tick: usize,
        action_type: &str,
        structure_type: Option<&str>,
    ) -> Result<(), PersistenceError> {
        sqlx::query("INSERT INTO events (system_id, completion_tick, action_type, structure_type) VALUES (?, ?, ?, ?)")
            .bind(system_id)
            .bind(completion_tick as i64)
            .bind(action_type)
            .bind(structure_type)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get all events for a system
    pub async fn get_events(&self, system_id: i64) -> Result<Vec<EventRow>, PersistenceError> {
        let rows = sqlx::query("SELECT id, system_id, completion_tick, action_type, structure_type, created_at FROM events WHERE system_id = ? ORDER BY completion_tick")
            .bind(system_id)
            .fetch_all(&self.pool)
            .await?;

        let mut events = Vec::new();
        for row in rows {
            events.push(EventRow {
                id: row.get("id"),
                system_id: row.get("system_id"),
                completion_tick: row.get("completion_tick"),
                action_type: row.get("action_type"),
                structure_type: row.get("structure_type"),
                created_at: row.get("created_at"),
            });
        }

        Ok(events)
    }

    /// Delete completed events (events with completion_tick <= current_tick)
    pub async fn cleanup_completed_events(
        &self,
        current_tick: usize,
    ) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM events WHERE completion_tick <= ?")
            .bind(current_tick as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete all events for a system
    pub async fn delete_events(&self, system_id: i64) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM events WHERE system_id = ?")
            .bind(system_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
