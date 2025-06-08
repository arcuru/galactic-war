/// User service that handles galaxy account management and system assignment
#[cfg(feature = "db")]
use crate::{
    auth::{AuthError, AuthService},
    db::{Database, PersistenceError},
    models::UserGalaxyAccount,
    Coords,
};

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("Authentication error: {0}")]
    #[cfg(feature = "db")]
    Auth(#[from] AuthError),

    #[error("Database error: {0}")]
    #[cfg(feature = "db")]
    Database(#[from] PersistenceError),

    #[error("Galaxy is full, no available coordinates for new system")]
    GalaxyFull,

    #[error("Account name already taken in this galaxy")]
    AccountNameTaken,

    #[error("User already has an account in this galaxy")]
    UserAlreadyInGalaxy,

    #[error("User does not have an account in this galaxy")]
    UserNotInGalaxy,
}

/// Service for managing users within galaxies
#[cfg(feature = "db")]
pub struct UserService {
    auth: AuthService,
    db: Database,
}

#[cfg(feature = "db")]
impl UserService {
    /// Create a new user service
    pub fn new(db: Database) -> Self {
        let auth = AuthService::new(db.clone());
        Self { auth, db }
    }

    /// Get the auth service reference
    pub fn auth(&self) -> &AuthService {
        &self.auth
    }

    /// Join a galaxy - create a galaxy account and assign a system
    pub async fn join_galaxy(
        &self,
        user_id: i64,
        galaxy_name: &str,
        account_name: &str,
        app_state: &crate::app::AppState,
    ) -> Result<(UserGalaxyAccount, Coords), UserServiceError> {
        // Check if user already has an account in this galaxy
        if (self
            .db
            .get_user_galaxy_account(user_id, galaxy_name)
            .await?)
            .is_some()
        {
            return Err(UserServiceError::UserAlreadyInGalaxy);
        }

        // Check if account name is available
        if !self
            .db
            .is_account_name_available(galaxy_name, account_name)
            .await?
        {
            return Err(UserServiceError::AccountNameTaken);
        }

        // Create galaxy account
        let account_id = self
            .db
            .create_user_galaxy_account(user_id, galaxy_name, account_name)
            .await?;

        // Create a new system for the user using AppState
        let current_tick = crate::tick();
        let (coords, system_info) = app_state
            .create_user_system_in_galaxy(galaxy_name, current_tick)
            .await
            .map_err(|e| {
                if e.contains("full") {
                    UserServiceError::GalaxyFull
                } else {
                    UserServiceError::Database(crate::db::PersistenceError::Migration(e))
                }
            })?;

        // Save the system to database with user ownership
        let system_id = self
            .db
            .save_system(
                galaxy_name,
                coords.x,
                coords.y,
                &system_info.resources,
                current_tick,
                Some(account_id),
            )
            .await?;

        // Assign the system to the user in the database
        self.db.assign_system_to_user(system_id, account_id).await?;

        // Get the created account
        let account = self
            .db
            .get_user_galaxy_account(user_id, galaxy_name)
            .await?
            .expect("Account should exist after creation");

        Ok((UserGalaxyAccount::from(account), coords))
    }

    /// Get user's galaxy account
    pub async fn get_user_galaxy_account(
        &self,
        user_id: i64,
        galaxy_name: &str,
    ) -> Result<Option<UserGalaxyAccount>, UserServiceError> {
        if let Some(account) = self
            .db
            .get_user_galaxy_account(user_id, galaxy_name)
            .await?
        {
            Ok(Some(UserGalaxyAccount::from(account)))
        } else {
            Ok(None)
        }
    }

    /// Get all user's galaxy accounts
    pub async fn get_user_galaxy_accounts(
        &self,
        user_id: i64,
    ) -> Result<Vec<UserGalaxyAccount>, UserServiceError> {
        let accounts = self.db.get_user_galaxy_accounts(user_id).await?;
        Ok(accounts.into_iter().map(UserGalaxyAccount::from).collect())
    }

    /// Get systems owned by a user in a galaxy
    pub async fn get_user_systems_coords(
        &self,
        user_galaxy_account_id: i64,
    ) -> Result<Vec<Coords>, UserServiceError> {
        let systems = self.db.get_user_systems(user_galaxy_account_id).await?;
        Ok(systems
            .into_iter()
            .map(|sys| (sys.x as usize, sys.y as usize).into())
            .collect())
    }

    /// Update user's last active time
    pub async fn update_user_activity(
        &self,
        user_id: i64,
        galaxy_name: &str,
    ) -> Result<(), UserServiceError> {
        self.db
            .update_user_galaxy_last_active(user_id, galaxy_name)
            .await?;
        Ok(())
    }

    /// Check if account name is available in a galaxy
    pub async fn is_account_name_available(
        &self,
        galaxy_name: &str,
        account_name: &str,
    ) -> Result<bool, UserServiceError> {
        let available = self
            .db
            .is_account_name_available(galaxy_name, account_name)
            .await?;
        Ok(available)
    }
}

#[cfg(all(test, feature = "db"))]
#[allow(dead_code)] // TODO: Update tests to use AppState instead of Galaxy directly
mod tests {

    use crate::Database;

    async fn setup_test_galaxy(db: &Database, galaxy_name: &str) {
        db.create_galaxy(galaxy_name, "test_config", 0)
            .await
            .expect("Failed to create test galaxy");
    }

    /* TODO: Update tests to use AppState instead of Galaxy directly
    #[tokio::test]
    async fn test_join_galaxy() {
        let db = Database::new_test().await.expect("Failed to create test database");
        let user_service = UserService::new(db.clone());

        let galaxy_name = "test_galaxy";
        setup_test_galaxy(&db, galaxy_name).await;

        // Create a test galaxy
        let galaxy_config = GalaxyConfig::default();
        let mut galaxy = Galaxy::new(galaxy_config, 0);

        // Create a user
        let user = user_service.auth()
            .register_user("testuser", "test@example.com", "password")
            .await
            .expect("Failed to register user");

        // Join galaxy
        let account_name = "TestAccount";
        let (account, coords) = user_service
            .join_galaxy(user.id, galaxy_name, account_name, &mut galaxy)
            .await
            .expect("Failed to join galaxy");

        assert_eq!(account.user_id, user.id);
        assert_eq!(account.galaxy_name, galaxy_name);
        assert_eq!(account.account_name, account_name);

        // Check that system was created
        assert!(galaxy.systems().contains_key(&coords));

        // Check that user can't join same galaxy twice
        let duplicate_result = user_service
            .join_galaxy(user.id, galaxy_name, "AnotherName", &mut galaxy)
            .await;
        assert!(matches!(duplicate_result, Err(UserServiceError::UserAlreadyInGalaxy)));

        // Check that account name can't be reused
        let user2 = user_service.auth()
            .register_user("testuser2", "test2@example.com", "password")
            .await
            .expect("Failed to register user2");

        let name_taken_result = user_service
            .join_galaxy(user2.id, galaxy_name, account_name, &mut galaxy)
            .await;
        assert!(matches!(name_taken_result, Err(UserServiceError::AccountNameTaken)));
    }

    #[tokio::test]
    #[ignore] // TODO: Update test to use AppState
    async fn test_user_galaxy_accounts() {
        let db = Database::new_test().await.expect("Failed to create test database");
        let user_service = UserService::new(db.clone());

        let galaxy1 = "galaxy1";
        let galaxy2 = "galaxy2";
        setup_test_galaxy(&db, galaxy1).await;
        setup_test_galaxy(&db, galaxy2).await;

        let mut galaxy1_instance = Galaxy::new(GalaxyConfig::default(), 0);
        let mut galaxy2_instance = Galaxy::new(GalaxyConfig::default(), 0);

        // Create a user
        let user = user_service.auth()
            .register_user("multiuser", "multi@example.com", "password")
            .await
            .expect("Failed to register user");

        // Join first galaxy
        let (account1, _) = user_service
            .join_galaxy(user.id, galaxy1, "Account1", &mut galaxy1_instance)
            .await
            .expect("Failed to join galaxy1");

        // Join second galaxy
        let (_account2, _) = user_service
            .join_galaxy(user.id, galaxy2, "Account2", &mut galaxy2_instance)
            .await
            .expect("Failed to join galaxy2");

        // Get all user's galaxy accounts
        let accounts = user_service
            .get_user_galaxy_accounts(user.id)
            .await
            .expect("Failed to get user galaxy accounts");

        assert_eq!(accounts.len(), 2);
        assert!(accounts.iter().any(|a| a.galaxy_name == galaxy1));
        assert!(accounts.iter().any(|a| a.galaxy_name == galaxy2));

        // Get specific account
        let specific_account = user_service
            .get_user_galaxy_account(user.id, galaxy1)
            .await
            .expect("Failed to get specific account")
            .expect("Account should exist");

        assert_eq!(specific_account.id, account1.id);
    }
    */
}
