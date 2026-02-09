use axum::{routing::get, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/{id}", get(get_user))
        .route("/{id}/ideas", get(get_user_ideas))
        // Team membership for current user
        .route("/me/teams", get(get_my_teams))
}

async fn get_me() -> &'static str {
    "get current user"
}

async fn update_me() -> &'static str {
    "update current user"
}

async fn get_user() -> &'static str {
    "get user profile"
}

async fn get_user_ideas() -> &'static str {
    "get user ideas"
}

async fn get_my_teams() -> &'static str {
    // TODO: List all ideas where the current user is an active team member
    "get my teams"
}
