-- Add current_tick column to systems table to store the tick when each system's resources were calculated
ALTER TABLE systems ADD COLUMN current_tick INTEGER NOT NULL DEFAULT 0; 