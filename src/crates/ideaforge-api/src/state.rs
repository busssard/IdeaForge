use ideaforge_db::Database;
use std::sync::Arc;

/// Shared application state, available to all Axum handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    // TODO: Add auth config, search index, event publisher, blockchain client
}
