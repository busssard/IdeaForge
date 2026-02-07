use axum::{
    extract::{Path, Query},
    routing::{get, post, put, delete},
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
        // Human approvals (drive maturity advancement)
        .route("/{id}/approvals", get(list_approvals).post(approve_idea).delete(withdraw_approval))
        // AI endorsements (informational only, separate track)
        .route("/{id}/endorsements", get(list_endorsements).post(endorse_idea).delete(withdraw_endorsement))
        // Combined approval summary
        .route("/{id}/approval-summary", get(get_approval_summary))
        .route("/{id}/contributions", get(list_contributions).post(add_contribution))
        .route("/{id}/todos", get(list_todos).post(create_todo))
        .route("/{id}/pledges", get(list_pledges).post(create_pledge))
        .route("/{id}/applications", get(list_applications).post(apply_as_expert))
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
    pub human_approvals: i32,
    pub ai_endorsements: i32,
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
    // TODO: Implement idea update (author or admin)
    "update idea"
}

async fn archive_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement soft delete
    "archive idea"
}

async fn update_maturity(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Implement maturity state transition
    // Transition validation uses human_approvals only (via TransitionRequirements)
    "update maturity"
}

// --- Human approvals (drive maturity advancement) ---

async fn list_approvals(Path(_id): Path<Uuid>) -> &'static str {
    "list human approvals"
}

async fn approve_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Reject with 403 if caller is an AI agent (is_bot = true)
    "approve idea (human only)"
}

async fn withdraw_approval(Path(_id): Path<Uuid>) -> &'static str {
    "withdraw human approval"
}

// --- AI endorsements (informational only, separate track) ---

async fn list_endorsements(Path(_id): Path<Uuid>) -> &'static str {
    "list AI endorsements"
}

async fn endorse_idea(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Reject with 403 if caller is a human (is_bot = false)
    // Requires confidence score and reasoning
    "endorse idea (AI agent only)"
}

async fn withdraw_endorsement(Path(_id): Path<Uuid>) -> &'static str {
    "withdraw AI endorsement"
}

// --- Combined approval summary ---

async fn get_approval_summary(Path(_id): Path<Uuid>) -> &'static str {
    // TODO: Return ApprovalSummary with human_approvals + ai_endorsements
    "approval summary"
}

// --- Contributions, Todos, Pledges, Applications ---

async fn list_contributions(Path(_id): Path<Uuid>) -> &'static str {
    "list contributions"
}

async fn add_contribution(Path(_id): Path<Uuid>) -> &'static str {
    "add contribution"
}

async fn list_todos(Path(_id): Path<Uuid>) -> &'static str {
    "list todos"
}

async fn create_todo(Path(_id): Path<Uuid>) -> &'static str {
    "create todo"
}

async fn list_pledges(Path(_id): Path<Uuid>) -> &'static str {
    "list pledges"
}

async fn create_pledge(Path(_id): Path<Uuid>) -> &'static str {
    "create pledge"
}

async fn list_applications(Path(_id): Path<Uuid>) -> &'static str {
    "list applications"
}

async fn apply_as_expert(Path(_id): Path<Uuid>) -> &'static str {
    "apply as expert"
}
