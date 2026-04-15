use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use crate::handlers;
use crate::middleware;
use crate::state::AppState;

/// Build the complete Axum router with all API routes and middleware.
pub fn build_router(state: AppState) -> Router {
    // Ensure upload directory exists so ServeDir has something to serve.
    let _ = std::fs::create_dir_all("uploads/avatars");

    let mut app = Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(handlers::health::health_check))
        .nest_service("/uploads", ServeDir::new("uploads"))
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
        .nest("/ideas", handlers::invites::routes())
        .nest("/ideas", handlers::board::routes())
        .nest("/ideas", handlers::nda::routes())
        .nest("/categories", handlers::categories::routes())
        .nest("/users", handlers::users::routes())
        .nest("/skills", handlers::skills::routes())
        .nest("/bots", handlers::bots::routes())
        .nest("/flags", handlers::flags::routes())
        .nest("/notifications", handlers::notifications::routes())
        .nest("/admin", handlers::flags::admin_routes())
        .nest("/bugs", handlers::bugs::routes())
        .nest("/mls", handlers::mls::routes())
        .nest("/mls/keystore", handlers::mls_keystore::routes())
    // Deferred to next iteration:
    // .nest("/search", handlers::search::routes())
}
