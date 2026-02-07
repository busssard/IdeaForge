use axum::{routing::post, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

async fn register() -> &'static str {
    // TODO: Implement user registration
    "register"
}

async fn login() -> &'static str {
    // TODO: Implement login with JWT issuance
    "login"
}

async fn refresh() -> &'static str {
    // TODO: Implement token refresh
    "refresh"
}

async fn logout() -> &'static str {
    // TODO: Implement refresh token invalidation
    "logout"
}
