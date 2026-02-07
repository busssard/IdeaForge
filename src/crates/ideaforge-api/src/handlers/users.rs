use axum::{routing::get, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/me/onboarding", axum::routing::put(set_onboarding_role))
        .route("/{id}", get(get_user))
        .route("/{id}/ideas", get(get_user_ideas))
}

async fn get_me() -> &'static str {
    "get current user"
}

async fn update_me() -> &'static str {
    "update current user"
}

async fn set_onboarding_role() -> &'static str {
    "set onboarding role"
}

async fn get_user() -> &'static str {
    "get user profile"
}

async fn get_user_ideas() -> &'static str {
    "get user ideas"
}
