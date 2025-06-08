use axum::{
    extract::{Extension, Form},
    http::{header, StatusCode},
    response::{Html, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use galactic_war::{app::AppState, User};
use serde::Deserialize;
use std::sync::Arc;

/// Cookie name for storing session tokens
const SESSION_COOKIE: &str = "galactic_war_session";

/// Login form data
#[derive(Deserialize)]
pub struct LoginForm {
    pub login: String, // username or email
    pub password: String,
}

/// Registration form data
#[derive(Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

/// Galaxy join form data
#[derive(Deserialize)]
pub struct JoinGalaxyForm {
    pub galaxy_name: String,
    pub account_name: String,
}

/// Authentication middleware to extract user from session
pub async fn get_current_user(
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Option<User> {
    let session_token = jar.get(SESSION_COOKIE)?.value();

    if let Some(db) = app_state.database() {
        let auth_service = galactic_war::AuthService::new(db.clone());
        if let Ok(user) = auth_service.validate_session(session_token).await {
            return Some(user);
        }
    }

    None
}

/// Show login page
pub async fn login_page() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Galactic War - Login</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 500px; margin: 50px auto; }
        .form-group { margin-bottom: 15px; }
        label { display: block; margin-bottom: 5px; }
        input[type="text"], input[type="email"], input[type="password"] { 
            width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; 
        }
        button { padding: 10px 20px; background: #007cba; color: white; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
        .error { color: red; margin-bottom: 15px; }
        .nav-links { margin-bottom: 20px; }
        .nav-links a { margin-right: 15px; text-decoration: none; color: #007cba; }
    </style>
</head>
<body>
    <div class="nav-links">
        <a href="/">← Back to Galaxy List</a>
        <a href="/register">Register New Account</a>
    </div>
    
    <h1>Login to Galactic War</h1>
    
    <form method="post" action="/login">
        <div class="form-group">
            <label for="login">Username or Email:</label>
            <input type="text" id="login" name="login" required>
        </div>
        
        <div class="form-group">
            <label for="password">Password:</label>
            <input type="password" id="password" name="password" required>
        </div>
        
        <button type="submit">Login</button>
    </form>
</body>
</html>
    "#.to_string())
}

/// Show registration page
pub async fn register_page() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Galactic War - Register</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 500px; margin: 50px auto; }
        .form-group { margin-bottom: 15px; }
        label { display: block; margin-bottom: 5px; }
        input[type="text"], input[type="email"], input[type="password"] { 
            width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; 
        }
        button { padding: 10px 20px; background: #007cba; color: white; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
        .error { color: red; margin-bottom: 15px; }
        .nav-links { margin-bottom: 20px; }
        .nav-links a { margin-right: 15px; text-decoration: none; color: #007cba; }
    </style>
</head>
<body>
    <div class="nav-links">
        <a href="/">← Back to Galaxy List</a>
        <a href="/login">Login to Existing Account</a>
    </div>
    
    <h1>Register for Galactic War</h1>
    
    <form method="post" action="/register">
        <div class="form-group">
            <label for="username">Username:</label>
            <input type="text" id="username" name="username" required minlength="3" maxlength="50">
        </div>
        
        <div class="form-group">
            <label for="email">Email:</label>
            <input type="email" id="email" name="email" required>
        </div>
        
        <div class="form-group">
            <label for="password">Password:</label>
            <input type="password" id="password" name="password" required minlength="6">
        </div>
        
        <div class="form-group">
            <label for="confirm_password">Confirm Password:</label>
            <input type="password" id="confirm_password" name="confirm_password" required>
        </div>
        
        <button type="submit">Register</button>
    </form>
    
    <script>
        document.querySelector('form').addEventListener('submit', function(e) {
            const password = document.getElementById('password').value;
            const confirmPassword = document.getElementById('confirm_password').value;
            
            if (password !== confirmPassword) {
                e.preventDefault();
                alert('Passwords do not match!');
            }
        });
    </script>
</body>
</html>
    "#.to_string())
}

/// Handle login form submission
pub async fn handle_login(
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), Response> {
    if let Some(db) = app_state.database() {
        let auth_service = galactic_war::AuthService::new(db.clone());

        match auth_service
            .authenticate_user(&form.login, &form.password)
            .await
        {
            Ok(user) => match auth_service.create_session(user.id).await {
                Ok(session) => {
                    let cookie = Cookie::build((SESSION_COOKIE, session.token))
                        .http_only(true)
                        .path("/")
                        .build();

                    return Ok((jar.add(cookie), Redirect::to("/dashboard")));
                }
                Err(_) => {
                    return Err(create_error_response("Failed to create session"));
                }
            },
            Err(_) => {
                return Err(create_error_response("Invalid username/email or password"));
            }
        }
    }

    Err(create_error_response("Authentication not available"))
}

/// Handle registration form submission
pub async fn handle_register(
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
    Form(form): Form<RegisterForm>,
) -> Result<(CookieJar, Redirect), Response> {
    // Validate passwords match
    if form.password != form.confirm_password {
        return Err(create_error_response("Passwords do not match"));
    }

    // Validate password length
    if form.password.len() < 6 {
        return Err(create_error_response(
            "Password must be at least 6 characters",
        ));
    }

    if let Some(db) = app_state.database() {
        let auth_service = galactic_war::AuthService::new(db.clone());

        match auth_service
            .register_user(&form.username, &form.email, &form.password)
            .await
        {
            Ok(user) => match auth_service.create_session(user.id).await {
                Ok(session) => {
                    let cookie = Cookie::build((SESSION_COOKIE, session.token))
                        .http_only(true)
                        .path("/")
                        .build();

                    return Ok((jar.add(cookie), Redirect::to("/dashboard")));
                }
                Err(_) => {
                    return Err(create_error_response("Failed to create session"));
                }
            },
            Err(galactic_war::AuthError::UserAlreadyExists) => {
                return Err(create_error_response("Username or email already exists"));
            }
            Err(_) => {
                return Err(create_error_response("Registration failed"));
            }
        }
    }

    Err(create_error_response("Registration not available"))
}

/// Handle logout
pub async fn handle_logout(
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> (CookieJar, Redirect) {
    if let Some(db) = app_state.database() {
        let auth_service = galactic_war::AuthService::new(db.clone());

        if let Some(session_cookie) = jar.get(SESSION_COOKIE) {
            let _ = auth_service.logout(session_cookie.value()).await;
        }
    }

    let cookie = Cookie::build((SESSION_COOKIE, "")).path("/").build();

    (jar.remove(cookie), Redirect::to("/"))
}

/// Show user dashboard with galaxy accounts
pub async fn user_dashboard(
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, Response> {
    let user = get_current_user(jar, Extension(app_state.clone()))
        .await
        .ok_or_else(|| {
            Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/login")
                .body("Redirecting to login".into())
                .unwrap()
        })?;

    if let Some(db) = app_state.database() {
        let user_service = galactic_war::UserService::new(db.clone());

        match user_service.get_user_galaxy_accounts(user.id).await {
            Ok(accounts) => {
                let mut page = format!(
                    r#"
<!DOCTYPE html>
<html>
<head>
    <title>Galactic War - Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; }}
        .galaxy-list {{ margin-bottom: 30px; }}
        .galaxy-item {{ 
            border: 1px solid #ddd; padding: 15px; margin-bottom: 10px; 
            border-radius: 5px; background: #f9f9f9; 
        }}
        .galaxy-name {{ font-weight: bold; font-size: 1.2em; }}
        .join-form {{ 
            border: 1px solid #ddd; padding: 20px; border-radius: 5px; 
            background: #f0f8ff; margin-top: 20px; 
        }}
        .form-group {{ margin-bottom: 15px; }}
        label {{ display: block; margin-bottom: 5px; }}
        input[type="text"] {{ 
            width: 200px; padding: 8px; border: 1px solid #ddd; border-radius: 4px; 
        }}
        select {{
            width: 220px; padding: 8px; border: 1px solid #ddd; border-radius: 4px; 
        }}
        button {{ 
            padding: 10px 20px; background: #007cba; color: white; 
            border: none; border-radius: 4px; cursor: pointer; 
        }}
        button:hover {{ background: #005a8a; }}
        .logout {{ background: #dc3545; }}
        .logout:hover {{ background: #c82333; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Welcome, {}</h1>
        <a href="/logout"><button class="logout">Logout</button></a>
    </div>
    
    <div class="galaxy-list">
        <h2>Your Galaxy Accounts</h2>
    "#,
                    user.username
                );

                if accounts.is_empty() {
                    page.push_str("<p>You haven't joined any galaxies yet. Join one below!</p>");
                } else {
                    for account in &accounts {
                        page.push_str(&format!(
                            r#"
                                <div class="galaxy-item">
                                    <div class="galaxy-name">{}</div>
                                    <div>Account: {}</div>
                                    <div>Joined: {}</div>
                                    <div>Last Active: {}</div>
                                    <a href="/galaxy/{}/dashboard"><button>Enter Galaxy</button></a>
                                </div>
                                "#,
                            account.galaxy_name,
                            account.account_name,
                            account.joined_at.format("%Y-%m-%d %H:%M UTC"),
                            account.last_active.format("%Y-%m-%d %H:%M UTC"),
                            account.galaxy_name
                        ));
                    }
                }

                // Get list of available galaxies
                let galaxies = app_state.list_galaxies().await;

                page.push_str(
                    r#"
    </div>
    
    <div class="join-form">
        <h3>Join a Galaxy</h3>
        <form method="post" action="/join-galaxy">
            <div class="form-group">
                <label for="galaxy_name">Select Galaxy:</label>
                <select id="galaxy_name" name="galaxy_name" required>
                    <option value="">Choose a galaxy...</option>
    "#,
                );

                for galaxy in galaxies {
                    // Check if user already has account in this galaxy
                    let already_joined = accounts.iter().any(|acc| acc.galaxy_name == galaxy);
                    if !already_joined {
                        page.push_str(&format!(
                            r#"<option value="{}">{}</option>"#,
                            galaxy, galaxy
                        ));
                    }
                }

                page.push_str(
                    r#"
                </select>
            </div>
            
            <div class="form-group">
                <label for="account_name">Account Name in Galaxy:</label>
                <input type="text" id="account_name" name="account_name" 
                       required minlength="3" maxlength="50" 
                       placeholder="Your display name in this galaxy">
            </div>
            
            <button type="submit">Join Galaxy</button>
        </form>
    </div>
    
    <div style="margin-top: 30px;">
        <h3>Available Public Galaxies</h3>
        <p>Browse public galaxies without joining:</p>
    "#,
                );

                for galaxy in app_state.list_galaxies().await {
                    page.push_str(&format!(
                            r#"<a href="/{}"><button style="margin-right: 10px; margin-bottom: 10px;">{}</button></a>"#,
                            galaxy, galaxy
                        ));
                }

                page.push_str(
                    r#"
    </div>
</body>
</html>
                    "#,
                );

                return Ok(Html(page));
            }
            Err(_) => {
                return Err(create_error_response("Failed to load user accounts"));
            }
        }
    }

    Err(create_error_response("User service not available"))
}

/// Handle joining a galaxy
pub async fn handle_join_galaxy(
    Extension(app_state): Extension<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<JoinGalaxyForm>,
) -> Result<(CookieJar, Redirect), Response> {
    let user = get_current_user(jar.clone(), Extension(app_state.clone()))
        .await
        .ok_or_else(|| create_error_response("Not logged in"))?;

    if let Some(db) = app_state.database() {
        let user_service = galactic_war::UserService::new(db.clone());

        // Check if account name is available
        match user_service
            .is_account_name_available(&form.galaxy_name, &form.account_name)
            .await
        {
            Ok(false) => {
                return Err(create_error_response(
                    "Account name is already taken in this galaxy",
                ));
            }
            Err(_) => {
                return Err(create_error_response(
                    "Failed to check account name availability",
                ));
            }
            Ok(true) => {} // Continue
        }

        // Join the galaxy using the new thread-safe method
        match user_service
            .join_galaxy(user.id, &form.galaxy_name, &form.account_name, &app_state)
            .await
        {
            Ok(_) => {
                return Ok((
                    jar,
                    Redirect::to(&format!("/galaxy/{}/dashboard", form.galaxy_name)),
                ));
            }
            Err(galactic_war::UserServiceError::AccountNameTaken) => {
                return Err(create_error_response("Account name is already taken"));
            }
            Err(galactic_war::UserServiceError::UserAlreadyInGalaxy) => {
                return Err(create_error_response(
                    "You already have an account in this galaxy",
                ));
            }
            Err(galactic_war::UserServiceError::GalaxyFull) => {
                return Err(create_error_response(
                    "Galaxy is full - no space for new systems",
                ));
            }
            Err(_) => {
                return Err(create_error_response("Failed to join galaxy"));
            }
        }
    }

    Err(create_error_response("Galaxy service not available"))
}

/// Show galaxy-specific dashboard for a user
pub async fn galaxy_dashboard(
    axum::extract::Path(galaxy_name): axum::extract::Path<String>,
    jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Html<String>, Response> {
    let user = get_current_user(jar, Extension(app_state.clone()))
        .await
        .ok_or_else(|| {
            Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/login")
                .body("Redirecting to login".into())
                .unwrap()
        })?;

    if let Some(db) = app_state.database() {
        let user_service = galactic_war::UserService::new(db.clone());

        // Get user's account in this galaxy
        match user_service
            .get_user_galaxy_account(user.id, &galaxy_name)
            .await
        {
            Ok(Some(account)) => {
                // Update last active time
                let _ = user_service
                    .update_user_activity(user.id, &galaxy_name)
                    .await;

                // Get user's systems
                match user_service.get_user_systems_coords(account.id).await {
                    Ok(systems_coords) => {
                        let mut page = format!(
                            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Galactic War - {} Galaxy</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 1000px; margin: 20px auto; }}
        .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
        .nav-links {{ margin-bottom: 20px; }}
        .nav-links a {{ margin-right: 15px; text-decoration: none; color: #007cba; }}
        .account-info {{ background: #f0f8ff; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        .systems-list {{ margin-bottom: 20px; }}
        .system-item {{ 
            border: 1px solid #ddd; padding: 10px; margin-bottom: 10px; 
            border-radius: 5px; background: #f9f9f9; display: flex; 
            justify-content: space-between; align-items: center;
        }}
        button {{ 
            padding: 8px 16px; background: #007cba; color: white; 
            border: none; border-radius: 4px; cursor: pointer; 
        }}
        button:hover {{ background: #005a8a; }}
        .logout {{ background: #dc3545; }}
        .logout:hover {{ background: #c82333; }}
    </style>
</head>
<body>
    <div class="nav-links">
        <a href="/dashboard">← Back to Dashboard</a>
        <a href="/{0}">View Public Galaxy Stats</a>
    </div>
    
    <div class="header">
        <h1>{0} Galaxy - {1}</h1>
        <a href="/logout"><button class="logout">Logout</button></a>
    </div>
    
    <div class="account-info">
        <strong>Account:</strong> {1}<br>
        <strong>Joined:</strong> {2}<br>
        <strong>Last Active:</strong> {3}
    </div>
    
    <div class="systems-list">
        <h2>Your Systems</h2>
    "#,
                            galaxy_name,
                            account.account_name,
                            account.joined_at.format("%Y-%m-%d %H:%M UTC"),
                            account.last_active.format("%Y-%m-%d %H:%M UTC")
                        );

                        if systems_coords.is_empty() {
                            page.push_str("<p>No systems found. This might be an error - please contact support.</p>");
                        } else {
                            for coords in systems_coords {
                                // Get system info
                                match app_state.system_info(&galaxy_name, coords).await {
                                    Ok(system_info) => {
                                        page.push_str(&format!(
                                                r#"
                                                <div class="system-item">
                                                    <div>
                                                        <strong>System ({}, {})</strong><br>
                                                        💰 Metal: {} | 🧑 Crew: {} | 💧 Water: {} | Score: {}
                                                    </div>
                                                    <a href="/{}/{}/{}"><button>Manage System</button></a>
                                                </div>
                                                "#,
                                                coords.x, coords.y,
                                                system_info.resources.metal,
                                                system_info.resources.crew,
                                                system_info.resources.water,
                                                system_info.score,
                                                galaxy_name, coords.x, coords.y
                                            ));
                                    }
                                    Err(_) => {
                                        page.push_str(&format!(
                                                r#"
                                                <div class="system-item">
                                                    <div><strong>System ({}, {})</strong><br>Error loading system data</div>
                                                    <a href="/{}/{}/{}"><button>View System</button></a>
                                                </div>
                                                "#,
                                                coords.x, coords.y, galaxy_name, coords.x, coords.y
                                            ));
                                    }
                                }
                            }
                        }

                        page.push_str(
                            r#"
    </div>
</body>
</html>
                            "#,
                        );

                        return Ok(Html(page));
                    }
                    Err(_) => {
                        return Err(create_error_response("Failed to load user systems"));
                    }
                }
            }
            Ok(None) => {
                return Err(create_error_response(
                    "You don't have an account in this galaxy",
                ));
            }
            Err(_) => {
                return Err(create_error_response("Failed to load galaxy account"));
            }
        }
    }

    Err(create_error_response("Galaxy service not available"))
}

/// Helper function to create error responses
fn create_error_response(message: &str) -> Response {
    let body = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Error - Galactic War</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 500px; margin: 100px auto; text-align: center; }}
        .error {{ color: #dc3545; font-size: 1.2em; margin-bottom: 20px; }}
        a {{ color: #007cba; text-decoration: none; }}
    </style>
</head>
<body>
    <h1>Error</h1>
    <div class="error">{}</div>
    <a href="/">← Back to Home</a>
</body>
</html>
    "#,
        message
    );

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "text/html")
        .body(body.into())
        .unwrap()
}
