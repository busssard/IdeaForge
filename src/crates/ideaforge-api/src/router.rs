use axum::{routing::get, Router};

use crate::handlers;
use crate::middleware;
use crate::state::AppState;

/// Build the complete Axum router with all API routes and middleware.
pub fn build_router(state: AppState) -> Router {
    let mut app = Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(handlers::health::health_check))
        .with_state(state);

    // Apply middleware layers (outermost applied first)
    app = app.layer(middleware::trace_layer());
    app = app.layer(middleware::cors_layer());

    // Apply security headers
    for layer in middleware::security_headers() {
        app = app.layer(layer);
    }

    app
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", handlers::auth::routes())
        .nest("/ideas", handlers::ideas::routes())
        .nest("/ideas", handlers::team::routes())
        .nest("/ideas", handlers::subscriptions::routes())
        .nest("/categories", handlers::categories::routes())
        .nest("/users", handlers::users::routes())
    // Deferred to next iteration:
    // .nest("/notifications", handlers::notifications::routes())
    // .nest("/search", handlers::search::routes())
}
