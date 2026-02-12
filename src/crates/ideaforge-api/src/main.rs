use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use ideaforge_api::config::AppConfig;
use ideaforge_api::router::build_router;
use ideaforge_api::state::AppState;
use ideaforge_auth::jwt::JwtConfig;
use ideaforge_db::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file (ignore if missing — production uses real env vars)
    let _ = dotenvy::dotenv();

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ideaforge=debug,tower_http=debug")),
        )
        .init();

    tracing::info!("Starting IdeaForge API server");

    // Load configuration from environment
    let config = AppConfig::from_env().expect("DATABASE_URL must be set");

    // Connect to database
    let db = Database::connect(&config.database.url).await?;
    tracing::info!("Connected to database");

    // Run migrations automatically
    db.run_migrations().await?;
    tracing::info!("Database migrations applied");

    // Build JWT config
    let jwt_config = JwtConfig {
        secret: config.auth.jwt_secret.clone(),
        access_token_ttl: chrono::Duration::minutes(config.auth.access_token_ttl_minutes),
        refresh_token_ttl: chrono::Duration::days(config.auth.refresh_token_ttl_days),
    };

    // Build application state
    let state = AppState {
        db: Arc::new(db),
        jwt: Arc::new(jwt_config),
    };

    // Build router
    let app = build_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
