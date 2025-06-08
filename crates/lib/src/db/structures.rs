use super::{Database, PersistenceError};

use crate::models::StructureRow;

use sqlx::Row;

impl Database {
    /// Save or update a structure for a system
    pub async fn save_structure(
        &self,
        system_id: i64,
        structure_type: &str,
        level: usize,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO structures (system_id, structure_type, level, updated_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(system_id, structure_type) DO UPDATE SET
                level = excluded.level,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(system_id)
        .bind(structure_type)
        .bind(level as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all structures for a system
    pub async fn get_structures(
        &self,
        system_id: i64,
    ) -> Result<Vec<StructureRow>, PersistenceError> {
        let rows = sqlx::query("SELECT id, system_id, structure_type, level, created_at, updated_at FROM structures WHERE system_id = ?")
            .bind(system_id)
            .fetch_all(&self.pool)
            .await?;

        let mut structures = Vec::new();
        for row in rows {
            structures.push(StructureRow {
                id: row.get("id"),
                system_id: row.get("system_id"),
                structure_type: row.get("structure_type"),
                level: row.get("level"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(structures)
    }

    /// Delete all structures for a system
    pub async fn delete_structures(&self, system_id: i64) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM structures WHERE system_id = ?")
            .bind(system_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
