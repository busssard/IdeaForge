use axum::{routing::get, Router};

use crate::handlers;
use crate::state::AppState;

/// Build the complete Axum router with all API routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(handlers::health::health_check))
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", handlers::auth::routes())
        .nest("/ideas", handlers::ideas::routes())
        // Team formation is nested under /ideas but defined separately
        // because it's the killer feature and deserves its own handler module.
        .nest("/ideas", handlers::team::routes())
        .nest("/categories", handlers::categories::routes())
        .nest("/users", handlers::users::routes())
        .nest("/notifications", handlers::notifications::routes())
        .nest("/search", handlers::search::routes())
    // Deferred to Phase 2+:
    // .nest("/agents", handlers::agents::routes())  -- AI agent management
}
