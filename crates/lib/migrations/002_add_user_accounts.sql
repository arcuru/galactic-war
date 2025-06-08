-- Add user account support to Galactic War
-- This migration adds user authentication and galaxy-specific accounts

-- Server-level user accounts
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- User accounts within specific galaxies (one per galaxy per user)
CREATE TABLE user_galaxy_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    galaxy_name TEXT NOT NULL,
    account_name TEXT NOT NULL, -- Display name in this galaxy
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (galaxy_name) REFERENCES galaxies(name) ON DELETE CASCADE,
    UNIQUE(user_id, galaxy_name), -- One account per galaxy per user
    UNIQUE(galaxy_name, account_name) -- Unique account names within galaxy
);

-- User authentication sessions
CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY, -- Session token
    user_id INTEGER NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add user ownership to systems (SQLite doesn't support adding foreign keys via ALTER TABLE)
ALTER TABLE systems ADD COLUMN user_galaxy_account_id INTEGER;

-- Indexes for performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_user_galaxy_accounts_user_galaxy ON user_galaxy_accounts(user_id, galaxy_name);
CREATE INDEX idx_user_galaxy_accounts_galaxy ON user_galaxy_accounts(galaxy_name);
CREATE INDEX idx_user_sessions_user ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_expires ON user_sessions(expires_at);
CREATE INDEX idx_systems_user_account ON systems(user_galaxy_account_id);