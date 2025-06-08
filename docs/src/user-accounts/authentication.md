# Authentication System

The Galactic War authentication system provides secure user registration, login, and session management. It uses industry-standard security practices for password handling and session management.

## Registration Process

### User Registration
New users can register by providing:
- **Username**: 3-50 characters, must be unique server-wide
- **Email**: Valid email address, must be unique server-wide  
- **Password**: Minimum 6 characters with client-side confirmation

```rust
// Registration flow
pub async fn register_user(
    username: &str,
    email: &str, 
    password: &str,
) -> Result<User, AuthError>
```

### Password Security
- Passwords are hashed using **SHA-256** with a random salt
- Each password gets a unique 32-character random salt
- Stored format: `{salt}:{hash}`
- Passwords are never stored in plaintext

```rust
// Password hashing implementation
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

## Login Process

### Authentication Methods
Users can authenticate using either:
- **Username** and password
- **Email** and password

### Session Creation
Upon successful authentication:
1. A secure 64-character session token is generated
2. Session stored in database with 24-hour expiry
3. HTTP-only cookie set in browser
4. User redirected to dashboard

```rust
// Session creation
pub async fn create_session(&self, user_id: i64) -> Result<UserSession, AuthError> {
    let session_token = Self::generate_session_token();
    let expires_at = Utc::now() + Duration::hours(24);
    
    self.db.create_user_session(&session_token, user_id, expires_at).await?;
    
    Ok(UserSession {
        token: session_token,
        user_id,
        expires_at,
    })
}
```

## Session Management

### Session Validation
Every protected request validates the session:
1. Extract session token from HTTP cookie
2. Look up session in database
3. Check if session has expired
4. Return user information if valid

### Session Security
- **HTTP-Only Cookies**: Prevents JavaScript access
- **Path Restriction**: Cookies scoped to root path
- **Automatic Expiry**: Sessions expire after 24 hours
- **Server-Side Validation**: All validation occurs server-side

### Session Cleanup
- Expired sessions are automatically cleaned up
- Manual cleanup can be triggered
- Logout immediately invalidates sessions

```rust
// Session validation
pub async fn validate_session(&self, session_token: &str) -> Result<User, AuthError> {
    let session = self.db.get_user_session(session_token).await?
        .ok_or(AuthError::SessionNotFound)?;

    if session.is_expired() {
        self.db.delete_user_session(session_token).await?;
        return Err(AuthError::SessionExpired);
    }

    let user_row = self.db.get_user_by_id(session.user_id).await?
        .ok_or(AuthError::InvalidSessionToken)?;

    Ok(User::from(user_row))
}
```

## API Endpoints

### Registration
- **GET /register**: Display registration form
- **POST /register**: Process registration submission

### Login  
- **GET /login**: Display login form
- **POST /login**: Process login submission

### Logout
- **GET /logout**: End user session and redirect

### Dashboard
- **GET /dashboard**: User dashboard (requires authentication)

## Error Handling

### Registration Errors
- `UserAlreadyExists`: Username or email already taken
- `PasswordTooShort`: Password doesn't meet requirements
- `ValidationError`: Invalid email format or other validation issues

### Login Errors
- `InvalidCredentials`: Wrong username/email or password
- `SessionCreationFailed`: Unable to create session

### Session Errors
- `SessionNotFound`: Session token not in database
- `SessionExpired`: Session has passed expiry time
- `InvalidSessionToken`: Session exists but user doesn't

## Security Considerations

### Password Requirements
- Minimum 6 characters (configurable)
- Client-side confirmation matching
- Server-side validation before processing

### Session Security
- Random token generation using cryptographically secure methods
- Short session lifetime (24 hours)
- Immediate invalidation on logout
- Server-side session storage only

### Protection Against Attacks
- **Password Hashing**: Protects against database breaches
- **Session Tokens**: Prevent session hijacking
- **HTTP-Only Cookies**: Prevent XSS attacks
- **Server Validation**: All security checks server-side

## Configuration

### Environment Variables
```bash
# Session configuration (optional - has defaults)
GWAR_SESSION_DURATION_HOURS=24
GWAR_SESSION_CLEANUP_INTERVAL=3600

# Database configuration
DATABASE_URL=sqlite:galactic_war.db
```

### Development Mode
```bash
# Use persistent dev database
task dev

# Dev database location
.cache/galactic-war/dev.db
```