use axum::{routing::{get, post, put, delete}, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_agent))
        .route("/", get(list_agents))
        .route("/{id}", get(get_agent).put(update_agent).delete(deactivate_agent))
        .route("/{id}/rotate-key", post(rotate_key))
}

async fn register_agent() -> &'static str {
    // TODO: Register bot, generate API key, return hashed key
    "register agent"
}

async fn list_agents() -> &'static str {
    "list agents"
}

async fn get_agent() -> &'static str {
    "get agent"
}

async fn update_agent() -> &'static str {
    "update agent"
}

async fn deactivate_agent() -> &'static str {
    "deactivate agent"
}

async fn rotate_key() -> &'static str {
    "rotate key"
}
