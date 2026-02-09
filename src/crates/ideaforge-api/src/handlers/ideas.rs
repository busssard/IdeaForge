use axum::{
    extract::{Path, Query},
    routing::{get, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use ideaforge_core::{IdeaMaturity, IdeaOpenness};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_ideas).post(create_idea))
        .route("/{id}", get(get_idea).put(update_idea).delete(archive_idea))
        .route("/{id}/maturity", put(update_maturity))
        // Human Stokes (drive maturity advancement)
        .route("/{id}/stokes", get(list_stokes).post(stoke_idea).delete(withdraw_stoke))
        // Contributions (comments and suggestions)
        .route("/{id}/contributions", get(list_contributions).post(add_contribution))
}

#[derive(Debug, Deserialize)]
pub struct ListIdeasQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub maturity: Option<String>,
    pub openness: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct IdeaResponse {
    pub id: Uuid,
    pub title: String,
    pub summary: String,
    pub maturity: IdeaMaturity,
    pub openness: IdeaOpenness,
    pub stoke_count: i32,
}

async fn list_ideas(Query(_params): Query<ListIdeasQuery>) -> &'static str {
    // TODO: Implement idea listing with filtering and pagination
    "list ideas"
}

async fn create_idea() -> &'static str {
    // TODO: Implement idea creation
    "create idea"
}

async fn get_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement single idea retrieval
    "get idea"
}

async fn update_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement idea update (author only)
    "update idea"
}

async fn archive_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement soft delete
    "archive idea"
}

async fn update_maturity(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement maturity state transition (Spark -> Building -> InWork)
    // Transition validation uses stoke_count (human Stokes only)
    "update maturity"
}

// --- Human Stokes (drive maturity advancement) ---

async fn list_stokes(Path(_id): Path<Uuid>) -> &'static str {
    "list stokes"
}

async fn stoke_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Create a Stoke for this idea (one per user per idea)
    "stoke idea"
}

async fn withdraw_stoke(Path(_id): Path<Uuid>) -> &'static str {
    "withdraw stoke"
}

// --- Contributions ---

async fn list_contributions(Path(_id): Path<Uuid>) -> &'static str {
    "list contributions"
}

async fn add_contribution(Path(_id): Path<Uuid>) -> &'static str {
    "add contribution"
}
