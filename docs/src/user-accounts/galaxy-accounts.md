# Galaxy Accounts

Galaxy accounts represent a user's presence within a specific galaxy. Each user can have one account per galaxy, allowing them to participate in multiple galactic wars simultaneously while maintaining separate identities and empires.

## Galaxy Account Concept

### One Account Per Galaxy
- Users can join multiple galaxies
- Each galaxy requires a separate account
- Account names must be unique within each galaxy
- Users cannot have multiple accounts in the same galaxy

### Account Isolation
- Each galaxy account is independent
- Resources, systems, and progress are separate
- Account names can be different across galaxies
- Activity tracking is per-galaxy

## Joining a Galaxy

### Prerequisites
- Must have a registered user account
- Must be logged in
- Galaxy must exist and be accessible
- Account name must be available in the target galaxy

### Galaxy Join Process
1. User selects galaxy from available list
2. Chooses unique account name for that galaxy
3. System automatically generates a system for the user
4. User gains access to galaxy-specific features

### System Assignment
When joining a galaxy:
- A new system is automatically created
- System placed at random coordinates within galaxy bounds
- User becomes the owner of that system
- System includes default starting structures and resources

```rust
// Galaxy joining workflow
pub async fn join_galaxy(
    &self,
    user_id: i64,
    galaxy_name: &str,
    account_name: &str,
    galaxy: &mut Galaxy,
) -> Result<(UserGalaxyAccount, Coords), UserServiceError>
```

## Account Management

### Account Information
Each galaxy account tracks:
- **Account Name**: Display name within the galaxy
- **Join Date**: When the user first joined the galaxy
- **Last Active**: Most recent activity timestamp
- **System Ownership**: Which systems belong to this account

### Activity Tracking
- Last active time updated on galaxy access
- Used for leaderboards and activity metrics
- Helps identify inactive accounts
- Supports future features like inactivity cleanup

### Account Display
```rust
pub struct UserGalaxyAccount {
    pub id: i64,
    pub user_id: i64,
    pub galaxy_name: String,
    pub account_name: String,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}
```

## Multi-Galaxy Experience

### Unified Dashboard
Users see all their galaxy accounts from one dashboard:
- List of joined galaxies
- Account name and activity for each
- Quick access to each galaxy
- Join new galaxies option

### Context Switching
- Users can switch between galaxies easily
- Each galaxy maintains separate game state
- Progress and resources don't transfer between galaxies
- Account names and reputations are galaxy-specific

### Galaxy-Specific Features
Within each galaxy, users can:
- Manage their assigned systems
- Build and upgrade structures
- View galaxy-wide statistics
- Compete with other players in that galaxy

## Database Schema

### User Galaxy Accounts Table
```sql
CREATE TABLE user_galaxy_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    galaxy_name TEXT NOT NULL,
    account_name TEXT NOT NULL,
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (galaxy_name) REFERENCES galaxies(name) ON DELETE CASCADE,
    UNIQUE(user_id, galaxy_name),  -- One account per galaxy per user
    UNIQUE(galaxy_name, account_name)  -- Unique names within galaxy
);
```

### Key Constraints
- **One Account Per Galaxy**: `UNIQUE(user_id, galaxy_name)`
- **Unique Names**: `UNIQUE(galaxy_name, account_name)`
- **Referential Integrity**: Foreign keys ensure data consistency

## API Operations

### Account Creation
```rust
pub async fn create_user_galaxy_account(
    &self,
    user_id: i64,
    galaxy_name: &str,
    account_name: &str,
) -> Result<i64, PersistenceError>
```

### Account Retrieval
```rust
// Get specific galaxy account
pub async fn get_user_galaxy_account(
    &self,
    user_id: i64,
    galaxy_name: &str,
) -> Result<Option<UserGalaxyAccount>, UserServiceError>

// Get all user's galaxy accounts
pub async fn get_user_galaxy_accounts(
    &self,
    user_id: i64,
) -> Result<Vec<UserGalaxyAccount>, UserServiceError>
```

### Activity Updates
```rust
pub async fn update_user_activity(
    &self,
    user_id: i64,
    galaxy_name: &str,
) -> Result<(), UserServiceError>
```

### Name Availability
```rust
pub async fn is_account_name_available(
    &self,
    galaxy_name: &str,
    account_name: &str,
) -> Result<bool, UserServiceError>
```

## Web Interface

### Dashboard View
The user dashboard shows:
```html
<div class="galaxy-item">
    <div class="galaxy-name">Alpha Centauri</div>
    <div>Account: CommanderSmith</div>
    <div>Joined: 2024-01-15 10:30 UTC</div>
    <div>Last Active: 2024-01-20 14:22 UTC</div>
    <a href="/galaxy/alpha-centauri/dashboard">
        <button>Enter Galaxy</button>
    </a>
</div>
```

### Galaxy-Specific Dashboard
Each galaxy has its own management interface:
- Account information display
- List of owned systems
- Galaxy-specific statistics
- System management links

## Business Rules

### Account Limits
- Maximum one account per galaxy per user
- No limit on number of galaxies a user can join
- Account names must be 3-50 characters
- Account names must be unique within each galaxy

### System Ownership
- Each account owns exactly one system per galaxy
- Systems cannot be transferred between accounts
- System ownership tied to galaxy account, not user account
- Deleting galaxy account removes system ownership

### Naming Rules
- Account names are case-sensitive
- Special characters allowed in account names
- Account names cannot be changed after creation
- Deleted account names may be reused

## Future Enhancements

### Potential Features
- Account name changes (with cooldown)
- Account deletion and system reassignment
- Cross-galaxy messaging or achievements
- Galaxy-specific reputation systems
- Inactive account cleanup policies