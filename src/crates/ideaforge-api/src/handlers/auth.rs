use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use ideaforge_auth::{jwt::JwtConfig, password};
use ideaforge_db::entities::enums::UserRole;
use ideaforge_db::repositories::user_repo::UserRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ErrorJson {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

fn error_response(status: StatusCode, code: &'static str, message: impl Into<String>) -> impl IntoResponse {
    (
        status,
        Json(ErrorJson {
            error: ErrorBody {
                code,
                message: message.into(),
            },
        }),
    )
}

// --- Validation helpers ---

fn validate_email(email: &str) -> Result<String, &'static str> {
    let email = email.trim().to_lowercase();
    if email.is_empty() {
        return Err("Email is required");
    }
    if email.len() > 255 {
        return Err("Email too long");
    }
    // Basic email validation: must contain @ with parts on both sides
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() || !parts[1].contains('.') {
        return Err("Invalid email format");
    }
    Ok(email)
}

fn validate_password(pw: &str) -> Result<(), &'static str> {
    if pw.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    if pw.len() > 128 {
        return Err("Password too long");
    }
    let has_upper = pw.chars().any(|c| c.is_uppercase());
    let has_lower = pw.chars().any(|c| c.is_lowercase());
    let has_digit = pw.chars().any(|c| c.is_ascii_digit());
    if !has_upper || !has_lower || !has_digit {
        return Err("Password must contain uppercase, lowercase, and a digit");
    }
    Ok(())
}

fn validate_display_name(name: &str) -> Result<String, &'static str> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Display name is required");
    }
    if name.len() > 100 {
        return Err("Display name too long (max 100 chars)");
    }
    Ok(name)
}

fn validate_role(role: Option<&str>) -> Result<UserRole, &'static str> {
    match role {
        None | Some("curious") => Ok(UserRole::Curious),
        Some("entrepreneur") => Ok(UserRole::Entrepreneur),
        Some("maker") => Ok(UserRole::Maker),
        _ => Err("Invalid role. Must be: entrepreneur, maker, or curious"),
    }
}

// --- Handlers ---

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate input
    let email = match validate_email(&body.email) {
        Ok(e) => e,
        Err(msg) => return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg).into_response(),
    };
    if let Err(msg) = validate_password(&body.password) {
        return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg).into_response();
    }
    let display_name = match validate_display_name(&body.display_name) {
        Ok(n) => n,
        Err(msg) => return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg).into_response(),
    };
    let role = match validate_role(body.role.as_deref()) {
        Ok(r) => r,
        Err(msg) => return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg).into_response(),
    };

    let repo = UserRepository::new(state.db.connection());

    // Check for existing user
    match repo.find_by_email(&email).await {
        Ok(Some(_)) => {
            return error_response(StatusCode::CONFLICT, "CONFLICT", "Email already registered")
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error checking email: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(None) => {}
    }

    // Hash password
    let password_hash = match password::hash_password(&body.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Password hashing error: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Create user
    let user_id = Uuid::new_v4();
    let user = match repo.create(user_id, &email, &password_hash, &display_name, role).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Database error creating user: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Generate tokens
    match create_token_pair(&state.jwt, user.id, &user.email, &user.role.to_string()) {
        Ok(tokens) => (StatusCode::CREATED, Json(tokens)).into_response(),
        Err(e) => {
            tracing::error!("Token creation error: {e}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    let email = match validate_email(&body.email) {
        Ok(e) => e,
        Err(msg) => return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg).into_response(),
    };

    let repo = UserRepository::new(state.db.connection());

    // Find user (constant-time comparison via Argon2 even on not-found)
    let user = match repo.find_by_email(&email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // Constant-time: still hash to prevent timing attacks
            let _ = password::verify_password("dummy", "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$abc");
            return error_response(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Invalid email or password")
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error during login: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Verify password
    match password::verify_password(&body.password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return error_response(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Invalid email or password")
                .into_response();
        }
        Err(e) => {
            tracing::error!("Password verification error: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Generate tokens
    match create_token_pair(&state.jwt, user.id, &user.email, &user.role.to_string()) {
        Ok(tokens) => (StatusCode::OK, Json(tokens)).into_response(),
        Err(e) => {
            tracing::error!("Token creation error: {e}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Validate the refresh token (same JWT validation for MVP)
    let claims = match state.jwt.validate_token(&body.refresh_token) {
        Ok(c) => c,
        Err(_) => {
            return error_response(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Invalid or expired refresh token")
                .into_response();
        }
    };

    // Re-issue tokens
    match create_token_pair(&state.jwt, claims.sub, &claims.email, &claims.role) {
        Ok(tokens) => (StatusCode::OK, Json(tokens)).into_response(),
        Err(e) => {
            tracing::error!("Token creation error: {e}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

fn create_token_pair(
    jwt: &JwtConfig,
    user_id: Uuid,
    email: &str,
    role: &str,
) -> Result<TokenResponse, jsonwebtoken::errors::Error> {
    let access_token = jwt.create_access_token(user_id, email, role)?;
    // Refresh token uses the refresh_token_ttl (longer-lived)
    let refresh_token = create_refresh_token(jwt, user_id, email, role)?;
    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        user_id,
    })
}

fn create_refresh_token(
    jwt: &JwtConfig,
    user_id: Uuid,
    email: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    // For MVP, refresh token is just a longer-lived JWT
    let now = chrono::Utc::now();
    let claims = ideaforge_auth::Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        iat: now.timestamp(),
        exp: (now + jwt.refresh_token_ttl).timestamp(),
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt.secret.as_bytes()),
    )
}
