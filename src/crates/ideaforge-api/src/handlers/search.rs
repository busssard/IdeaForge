use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub r#type: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(search))
}

async fn search(Query(_params): Query<SearchQuery>) -> &'static str {
    // TODO: Implement unified search via Tantivy
    "search results"
}
