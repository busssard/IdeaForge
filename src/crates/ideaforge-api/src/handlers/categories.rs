use axum::{routing::get, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories))
        .route("/{slug}", get(get_category))
}

async fn list_categories() -> &'static str {
    "list categories"
}

async fn get_category() -> &'static str {
    "get category"
}
