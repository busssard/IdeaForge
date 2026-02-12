use ideaforge_auth::jwt::JwtConfig;
use ideaforge_db::Database;
use std::sync::Arc;

/// Shared application state, available to all Axum handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub jwt: Arc<JwtConfig>,
}
