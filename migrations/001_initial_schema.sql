-- Initial schema for Galactic War database persistence
-- This creates the core tables needed to store galaxy state

-- Galaxy metadata and configuration
CREATE TABLE galaxies (
    name TEXT PRIMARY KEY,
    config_file TEXT NOT NULL,
    tick INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- System state within galaxies
CREATE TABLE systems (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    galaxy_name TEXT NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    metal INTEGER NOT NULL DEFAULT 0,
    crew INTEGER NOT NULL DEFAULT 0,
    water INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (galaxy_name) REFERENCES galaxies(name) ON DELETE CASCADE,
    UNIQUE(galaxy_name, x, y)
);

-- Structures within systems
CREATE TABLE structures (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    system_id INTEGER NOT NULL,
    structure_type TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (system_id) REFERENCES systems(id) ON DELETE CASCADE,
    UNIQUE(system_id, structure_type)
);

-- Events (production, building, etc.)
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    system_id INTEGER NOT NULL,
    completion_tick INTEGER NOT NULL,
    action_type TEXT NOT NULL, -- 'Metal', 'Water', 'Crew', 'Build'
    structure_type TEXT, -- NULL for production events, structure name for build events
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (system_id) REFERENCES systems(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_systems_galaxy_coords ON systems(galaxy_name, x, y);
CREATE INDEX idx_events_system_completion ON events(system_id, completion_tick);
CREATE INDEX idx_structures_system ON structures(system_id); 