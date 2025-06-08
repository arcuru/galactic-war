use chrono::{DateTime, Utc};

/// Database row representing a server-level user account
#[derive(Debug, Clone, PartialEq)]
#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRow {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Database row representing a user's account within a specific galaxy
#[derive(Debug, Clone, PartialEq)]
#[derive(sqlx::FromRow)]
pub struct UserGalaxyAccountRow {
    pub id: i64,
    pub user_id: i64,
    pub galaxy_name: String,
    pub account_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl UserGalaxyAccountRow {
    pub fn new(user_id: i64, galaxy_name: String, account_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            user_id,
            galaxy_name,
            account_name,
            joined_at: now,
            last_active: now,
        }
    }

    pub fn update_last_active(&mut self) {
        self.last_active = Utc::now();
    }
}

/// Database row representing a user session for authentication
#[derive(Debug, Clone, PartialEq)]
#[derive(sqlx::FromRow)]
pub struct UserSessionRow {
    pub id: String, // Session token
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserSessionRow {
    pub fn new(session_token: String, user_id: i64, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: session_token,
            user_id,
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// User authentication and profile information
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
        }
    }
}

/// User's account information within a specific galaxy
#[derive(Debug, Clone)]
pub struct UserGalaxyAccount {
    pub id: i64,
    pub user_id: i64,
    pub galaxy_name: String,
    pub account_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl From<UserGalaxyAccountRow> for UserGalaxyAccount {
    fn from(row: UserGalaxyAccountRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            galaxy_name: row.galaxy_name,
            account_name: row.account_name,
            joined_at: row.joined_at,
            last_active: row.last_active,
        }
    }
}

/// Authentication session information
#[derive(Debug, Clone)]
pub struct UserSession {
    pub token: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
}

impl From<UserSessionRow> for UserSession {
    fn from(row: UserSessionRow) -> Self {
        Self {
            token: row.id,
            user_id: row.user_id,
            expires_at: row.expires_at,
        }
    }
}
