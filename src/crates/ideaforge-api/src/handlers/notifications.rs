use axum::{routing::{get, put}, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/unread-count", get(unread_count))
        .route("/read-all", put(read_all))
        .route("/:id/read", put(mark_read))
}

async fn list_notifications() -> &'static str {
    "list notifications"
}

async fn unread_count() -> &'static str {
    "unread count"
}

async fn read_all() -> &'static str {
    "read all"
}

async fn mark_read() -> &'static str {
    "mark read"
}
