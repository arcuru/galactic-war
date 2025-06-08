use super::{Database, PersistenceError};

use crate::models::{UserGalaxyAccountRow, UserRow, UserSessionRow};

use chrono::{DateTime, Utc};

use sqlx::Row;

impl Database {
    /// Create a new user account
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<i64, PersistenceError> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash, updated_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Get a user by username
    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserRow>, PersistenceError> {
        let result = sqlx::query("SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = result {
            Ok(Some(UserRow {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a user by email
    pub async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserRow>, PersistenceError> {
        let result = sqlx::query("SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = result {
            Ok(Some(UserRow {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a user by ID
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserRow>, PersistenceError> {
        let result = sqlx::query("SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = result {
            Ok(Some(UserRow {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a galaxy account for a user
    pub async fn create_user_galaxy_account(
        &self,
        user_id: i64,
        galaxy_name: &str,
        account_name: &str,
    ) -> Result<i64, PersistenceError> {
        let result = sqlx::query(
            r#"
            INSERT INTO user_galaxy_accounts (user_id, galaxy_name, account_name, last_active)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(galaxy_name)
        .bind(account_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Get a user's galaxy account
    pub async fn get_user_galaxy_account(
        &self,
        user_id: i64,
        galaxy_name: &str,
    ) -> Result<Option<UserGalaxyAccountRow>, PersistenceError> {
        let result = sqlx::query(
            "SELECT id, user_id, galaxy_name, account_name, joined_at, last_active FROM user_galaxy_accounts WHERE user_id = ? AND galaxy_name = ?"
        )
        .bind(user_id)
        .bind(galaxy_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            Ok(Some(UserGalaxyAccountRow {
                id: row.get("id"),
                user_id: row.get("user_id"),
                galaxy_name: row.get("galaxy_name"),
                account_name: row.get("account_name"),
                joined_at: row.get("joined_at"),
                last_active: row.get("last_active"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all galaxy accounts for a user
    pub async fn get_user_galaxy_accounts(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserGalaxyAccountRow>, PersistenceError> {
        let rows = sqlx::query(
            "SELECT id, user_id, galaxy_name, account_name, joined_at, last_active FROM user_galaxy_accounts WHERE user_id = ? ORDER BY last_active DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(UserGalaxyAccountRow {
                id: row.get("id"),
                user_id: row.get("user_id"),
                galaxy_name: row.get("galaxy_name"),
                account_name: row.get("account_name"),
                joined_at: row.get("joined_at"),
                last_active: row.get("last_active"),
            });
        }

        Ok(accounts)
    }

    /// Update user's last active time in a galaxy
    pub async fn update_user_galaxy_last_active(
        &self,
        user_id: i64,
        galaxy_name: &str,
    ) -> Result<(), PersistenceError> {
        sqlx::query(
            "UPDATE user_galaxy_accounts SET last_active = CURRENT_TIMESTAMP WHERE user_id = ? AND galaxy_name = ?"
        )
        .bind(user_id)
        .bind(galaxy_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a user session
    pub async fn create_user_session(
        &self,
        session_token: &str,
        user_id: i64,
        expires_at: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        sqlx::query("INSERT INTO user_sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
            .bind(session_token)
            .bind(user_id)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get a user session
    pub async fn get_user_session(
        &self,
        session_token: &str,
    ) -> Result<Option<UserSessionRow>, PersistenceError> {
        let result = sqlx::query(
            "SELECT id, user_id, expires_at, created_at FROM user_sessions WHERE id = ?",
        )
        .bind(session_token)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            Ok(Some(UserSessionRow {
                id: row.get("id"),
                user_id: row.get("user_id"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a user session
    pub async fn delete_user_session(&self, session_token: &str) -> Result<(), PersistenceError> {
        sqlx::query("DELETE FROM user_sessions WHERE id = ?")
            .bind(session_token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, PersistenceError> {
        let now = chrono::Utc::now();
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at < ?")
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Check if an account name is available in a galaxy
    pub async fn is_account_name_available(
        &self,
        galaxy_name: &str,
        account_name: &str,
    ) -> Result<bool, PersistenceError> {
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM user_galaxy_accounts WHERE galaxy_name = ? AND account_name = ?"
        )
        .bind(galaxy_name)
        .bind(account_name)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.get("count");
        Ok(count == 0)
    }
}

mod tests {
    use super::*;
    use crate::Database;
    use chrono::{Duration, Utc};

    #[allow(dead_code)]
    async fn setup_test_galaxy(db: &Database, galaxy_name: &str) {
        db.create_galaxy(galaxy_name, "test_config", 0)
            .await
            .expect("Failed to create test galaxy");
    }

    #[tokio::test]
    async fn test_user_crud_operations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let username = "testuser";
        let email = "test@example.com";
        let password_hash = "hashed_password";

        // Test user creation
        let user_id = db
            .create_user(username, email, password_hash)
            .await
            .expect("Failed to create user");

        assert!(user_id > 0);

        // Test getting user by username
        let user = db
            .get_user_by_username(username)
            .await
            .expect("Failed to get user by username")
            .expect("User should exist");

        assert_eq!(user.id, user_id);
        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);

        // Test getting user by email
        let user_by_email = db
            .get_user_by_email(email)
            .await
            .expect("Failed to get user by email")
            .expect("User should exist");

        assert_eq!(user_by_email.id, user_id);

        // Test getting user by ID
        let user_by_id = db
            .get_user_by_id(user_id)
            .await
            .expect("Failed to get user by ID")
            .expect("User should exist");

        assert_eq!(user_by_id.id, user_id);

        db.close().await;
    }

    #[tokio::test]
    async fn test_galaxy_account_operations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        let galaxy_name = "test_galaxy";
        setup_test_galaxy(&db, galaxy_name).await;

        // Create a user first
        let user_id = db
            .create_user("testuser", "test@example.com", "password")
            .await
            .expect("Failed to create user");

        let account_name = "TestAccount";

        // Test galaxy account creation
        let account_id = db
            .create_user_galaxy_account(user_id, galaxy_name, account_name)
            .await
            .expect("Failed to create galaxy account");

        assert!(account_id > 0);

        // Test getting galaxy account
        let account = db
            .get_user_galaxy_account(user_id, galaxy_name)
            .await
            .expect("Failed to get galaxy account")
            .expect("Account should exist");

        assert_eq!(account.id, account_id);
        assert_eq!(account.user_id, user_id);
        assert_eq!(account.galaxy_name, galaxy_name);
        assert_eq!(account.account_name, account_name);

        // Test getting all galaxy accounts for user
        let accounts = db
            .get_user_galaxy_accounts(user_id)
            .await
            .expect("Failed to get user galaxy accounts");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id, account_id);

        // Test account name availability
        let is_available = db
            .is_account_name_available(galaxy_name, account_name)
            .await
            .expect("Failed to check account name availability");

        assert!(!is_available); // Should not be available since we just created it

        let is_available_new = db
            .is_account_name_available(galaxy_name, "NewAccountName")
            .await
            .expect("Failed to check account name availability");

        assert!(is_available_new); // Should be available

        db.close().await;
    }

    #[tokio::test]
    async fn test_session_operations() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        // Create a user first
        let user_id = db
            .create_user("testuser", "test@example.com", "password")
            .await
            .expect("Failed to create user");

        let session_token = "test_session_token";
        let expires_at = Utc::now() + Duration::hours(24);

        // Test session creation
        db.create_user_session(session_token, user_id, expires_at)
            .await
            .expect("Failed to create session");

        // Test getting session
        let session = db
            .get_user_session(session_token)
            .await
            .expect("Failed to get session")
            .expect("Session should exist");

        assert_eq!(session.id, session_token);
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.expires_at, expires_at);

        // Test session deletion
        db.delete_user_session(session_token)
            .await
            .expect("Failed to delete session");

        let deleted_session = db
            .get_user_session(session_token)
            .await
            .expect("Failed to check session");

        assert!(deleted_session.is_none());

        db.close().await;
    }

    #[tokio::test]
    async fn test_expired_session_cleanup() {
        let db = Database::new_test()
            .await
            .expect("Failed to create test database");

        // Create a user first
        let user_id = db
            .create_user("testuser", "test@example.com", "password")
            .await
            .expect("Failed to create user");

        // Create an expired session
        let expired_token = "expired_token";
        let expired_time = Utc::now() - Duration::hours(1);
        db.create_user_session(expired_token, user_id, expired_time)
            .await
            .expect("Failed to create expired session");

        // Create a valid session
        let valid_token = "valid_token";
        let valid_time = Utc::now() + Duration::hours(1);
        db.create_user_session(valid_token, user_id, valid_time)
            .await
            .expect("Failed to create valid session");

        // Cleanup expired sessions
        let deleted_count = db
            .cleanup_expired_sessions()
            .await
            .expect("Failed to cleanup expired sessions");

        assert_eq!(deleted_count, 1);

        // Check that expired session is gone but valid session remains
        let expired_session = db
            .get_user_session(expired_token)
            .await
            .expect("Failed to check expired session");
        assert!(expired_session.is_none());

        let valid_session = db
            .get_user_session(valid_token)
            .await
            .expect("Failed to check valid session");
        assert!(valid_session.is_some());

        db.close().await;
    }
}
