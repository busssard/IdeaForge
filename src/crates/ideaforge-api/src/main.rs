use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use ideaforge_api::config::AppConfig;
use ideaforge_api::router::build_router;
use ideaforge_api::state::AppState;
use ideaforge_db::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("ideaforge=debug".parse()?))
        .json()
        .init();

    tracing::info!("Starting IdeaForge API server");

    // Load configuration (defaults for now)
    let config = AppConfig::default();

    // Connect to database
    let db = Database::connect(&config.database.url).await?;
    tracing::info!("Connected to database");

    // Build application state
    let state = AppState { db: Arc::new(db) };

    // Build router
    let app = build_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
