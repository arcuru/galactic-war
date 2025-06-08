use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};

#[cfg(feature = "db")]
use crate::db::{Database, PersistenceError};
#[cfg(feature = "db")]
use crate::models::{User, UserSession};

/// Authentication service for user management
#[cfg(feature = "db")]
pub struct AuthService {
    db: Database,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Invalid session token")]
    InvalidSessionToken,
    
    #[error("Database error: {0}")]
    #[cfg(feature = "db")]
    Database(#[from] PersistenceError),
    
    #[error("Password hashing error")]
    PasswordHashing,
}

#[cfg(feature = "db")]
impl AuthService {
    /// Create a new auth service with database connection
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Hash a password using SHA-256 with salt
    pub fn hash_password(password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a random salt for password hashing
    pub fn generate_salt() -> String {
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect()
    }

    /// Generate a secure session token
    pub fn generate_session_token() -> String {
        let mut rng = rand::thread_rng();
        (0..64)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect()
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, AuthError> {
        // Check if username or email already exists
        if let Some(_) = self.db.get_user_by_username(username).await? {
            return Err(AuthError::UserAlreadyExists);
        }
        
        if let Some(_) = self.db.get_user_by_email(email).await? {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash the password
        let salt = Self::generate_salt();
        let password_hash = format!("{}:{}", salt, Self::hash_password(password, &salt));

        // Create the user
        let user_id = self.db.create_user(username, email, &password_hash).await?;
        
        // Return the user without password hash
        Ok(User {
            id: user_id,
            username: username.to_string(),
            email: email.to_string(),
        })
    }

    /// Authenticate a user with username/email and password
    pub async fn authenticate_user(
        &self,
        login: &str, // Can be username or email
        password: &str,
    ) -> Result<User, AuthError> {
        // Try to find user by username first, then by email
        let user_row = if let Some(user) = self.db.get_user_by_username(login).await? {
            user
        } else if let Some(user) = self.db.get_user_by_email(login).await? {
            user
        } else {
            return Err(AuthError::InvalidCredentials);
        };

        // Verify password
        if self.verify_password(password, &user_row.password_hash) {
            Ok(User::from(user_row))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    /// Verify a password against a stored hash
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        if let Some((salt, hash)) = stored_hash.split_once(':') {
            let computed_hash = Self::hash_password(password, salt);
            computed_hash == hash
        } else {
            false
        }
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: i64) -> Result<UserSession, AuthError> {
        let session_token = Self::generate_session_token();
        let expires_at = Utc::now() + Duration::hours(24); // 24 hour sessions

        self.db.create_user_session(&session_token, user_id, expires_at).await?;

        Ok(UserSession {
            token: session_token,
            user_id,
            expires_at,
        })
    }

    /// Validate a session token and return the user
    pub async fn validate_session(&self, session_token: &str) -> Result<User, AuthError> {
        let session = self.db
            .get_user_session(session_token)
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        // Check if session is expired
        if session.is_expired() {
            // Clean up expired session
            let _ = self.db.delete_user_session(session_token).await;
            return Err(AuthError::SessionExpired);
        }

        // Get the user
        let user_row = self.db
            .get_user_by_id(session.user_id)
            .await?
            .ok_or(AuthError::InvalidSessionToken)?;

        Ok(User::from(user_row))
    }

    /// Logout by deleting the session
    pub async fn logout(&self, session_token: &str) -> Result<(), AuthError> {
        self.db.delete_user_session(session_token).await?;
        Ok(())
    }

    /// Clean up expired sessions (should be called periodically)
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, AuthError> {
        let deleted_count = self.db.cleanup_expired_sessions().await?;
        Ok(deleted_count)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, AuthError> {
        if let Some(user_row) = self.db.get_user_by_id(user_id).await? {
            Ok(Some(User::from(user_row)))
        } else {
            Ok(None)
        }
    }
}

/// Login request data
#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub login: String, // username or email
    pub password: String,
}

/// Registration request data
#[derive(Debug, Clone)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub user: User,
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;
    use crate::db::Database;

    #[tokio::test]
    async fn test_password_hashing() {
        let password = "test_password_123";
        let salt = AuthService::generate_salt();
        let hash1 = AuthService::hash_password(password, &salt);
        let hash2 = AuthService::hash_password(password, &salt);
        
        // Same password and salt should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different salt should produce different hash
        let different_salt = AuthService::generate_salt();
        let hash3 = AuthService::hash_password(password, &different_salt);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_user_registration_and_authentication() {
        let db = Database::new_test().await.expect("Failed to create test database");
        let auth = AuthService::new(db);

        let username = "testuser";
        let email = "test@example.com";
        let password = "secure_password_123";

        // Test user registration
        let user = auth
            .register_user(username, email, password)
            .await
            .expect("Failed to register user");

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
        assert!(user.id > 0);

        // Test duplicate registration fails
        let duplicate_result = auth.register_user(username, email, password).await;
        assert!(matches!(duplicate_result, Err(AuthError::UserAlreadyExists)));

        // Test authentication with username
        let auth_user = auth
            .authenticate_user(username, password)
            .await
            .expect("Failed to authenticate with username");
        assert_eq!(auth_user.id, user.id);

        // Test authentication with email
        let auth_user_email = auth
            .authenticate_user(email, password)
            .await
            .expect("Failed to authenticate with email");
        assert_eq!(auth_user_email.id, user.id);

        // Test authentication with wrong password
        let wrong_auth = auth.authenticate_user(username, "wrong_password").await;
        assert!(matches!(wrong_auth, Err(AuthError::InvalidCredentials)));

        // Test authentication with non-existent user
        let no_user_auth = auth.authenticate_user("nonexistent", password).await;
        assert!(matches!(no_user_auth, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_session_management() {
        let db = Database::new_test().await.expect("Failed to create test database");
        let auth = AuthService::new(db);

        // Create a user first
        let user = auth
            .register_user("sessionuser", "session@example.com", "password123")
            .await
            .expect("Failed to register user");

        // Create a session
        let session = auth
            .create_session(user.id)
            .await
            .expect("Failed to create session");

        assert!(session.token.len() == 64); // Check token length
        assert_eq!(session.user_id, user.id);
        assert!(session.expires_at > Utc::now());

        // Validate the session
        let validated_user = auth
            .validate_session(&session.token)
            .await
            .expect("Failed to validate session");
        assert_eq!(validated_user.id, user.id);

        // Logout (delete session)
        auth.logout(&session.token)
            .await
            .expect("Failed to logout");

        // Session should no longer be valid
        let invalid_session = auth.validate_session(&session.token).await;
        assert!(matches!(invalid_session, Err(AuthError::SessionNotFound)));
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let db = Database::new_test().await.expect("Failed to create test database");
        let auth = AuthService::new(db);

        // Create a user
        let user = auth
            .register_user("expireuser", "expire@example.com", "password123")
            .await
            .expect("Failed to register user");

        // Create an expired session manually
        let expired_token = AuthService::generate_session_token();
        let expired_time = Utc::now() - Duration::hours(1); // 1 hour ago
        
        auth.db
            .create_user_session(&expired_token, user.id, expired_time)
            .await
            .expect("Failed to create expired session");

        // Cleanup expired sessions before validation
        let cleanup_count = auth
            .cleanup_expired_sessions()
            .await
            .expect("Failed to cleanup expired sessions");
        assert!(cleanup_count >= 1);

        // Validation should fail (session was cleaned up)
        let expired_result = auth.validate_session(&expired_token).await;
        assert!(matches!(expired_result, Err(AuthError::SessionNotFound)));
    }
}