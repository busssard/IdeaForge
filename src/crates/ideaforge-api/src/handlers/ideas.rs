use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::enums::{IdeaMaturity, IdeaOpenness};
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::stoke_repo::StokeRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_ideas).post(create_idea))
        .route("/:id", get(get_idea).put(update_idea).delete(archive_idea))
        .route("/:id/stokes", get(list_stokes).post(stoke_idea))
        .route("/:id/stokes/mine", delete(withdraw_stoke))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct ListIdeasQuery {
    pub category_id: Option<Uuid>,
    pub maturity: Option<String>,
    pub openness: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIdeaRequest {
    pub title: String,
    pub summary: String,
    pub description: String,
    pub openness: Option<String>,
    pub category_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIdeaRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub openness: Option<String>,
    pub category_id: Option<Option<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct IdeaResponse {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub maturity: String,
    pub openness: String,
    pub category_id: Option<Uuid>,
    pub stoke_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct IdeaListResponse {
    pub data: Vec<IdeaResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct StokeResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub idea_id: Uuid,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct StokeListResponse {
    pub data: Vec<StokeResponse>,
    pub meta: PaginationMeta,
}

fn err(status: StatusCode, code: &str, message: &str) -> impl IntoResponse {
    (
        status,
        Json(serde_json::json!({
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

impl From<&ideaforge_db::entities::idea::Model> for IdeaResponse {
    fn from(m: &ideaforge_db::entities::idea::Model) -> Self {
        Self {
            id: m.id,
            author_id: m.author_id,
            title: m.title.clone(),
            summary: m.summary.clone(),
            description: m.description.clone(),
            maturity: m.maturity.to_string(),
            openness: m.openness.to_string(),
            category_id: m.category_id,
            stoke_count: m.stoke_count,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

// --- Handlers ---

async fn list_ideas(
    State(state): State<AppState>,
    Query(params): Query<ListIdeasQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let maturity_filter = params.maturity.as_deref().and_then(IdeaMaturity::from_str_opt);
    let openness_filter = params.openness.as_deref().and_then(IdeaOpenness::from_str_opt);

    let repo = IdeaRepository::new(state.db.connection());
    match repo
        .list(
            maturity_filter,
            openness_filter,
            params.category_id,
            page,
            per_page,
        )
        .await
    {
        Ok((ideas, total)) => {
            let total_pages = (total + per_page - 1) / per_page;
            Json(IdeaListResponse {
                data: ideas.iter().map(IdeaResponse::from).collect(),
                meta: PaginationMeta {
                    total,
                    page,
                    per_page,
                    total_pages,
                },
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list ideas: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn create_idea(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateIdeaRequest>,
) -> impl IntoResponse {
    // Validate
    let title = body.title.trim();
    if title.is_empty() || title.len() > 200 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Title is required (max 200 chars)",
        )
        .into_response();
    }
    let summary = body.summary.trim();
    if summary.is_empty() || summary.len() > 500 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Summary is required (max 500 chars)",
        )
        .into_response();
    }
    if body.description.trim().is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Description is required",
        )
        .into_response();
    }

    let openness = match body.openness.as_deref() {
        None | Some("open") => IdeaOpenness::Open,
        Some("collaborative") => IdeaOpenness::Collaborative,
        Some("commercial") => IdeaOpenness::Commercial,
        Some(_) => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid openness. Must be: open, collaborative, or commercial",
            )
            .into_response();
        }
    };

    let repo = IdeaRepository::new(state.db.connection());
    match repo
        .create(
            Uuid::new_v4(),
            auth.user_id,
            title,
            summary,
            body.description.trim(),
            IdeaMaturity::Spark,
            openness,
            body.category_id,
        )
        .await
    {
        Ok(idea) => (StatusCode::CREATED, Json(IdeaResponse::from(&idea))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create idea: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_idea(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = IdeaRepository::new(state.db.connection());
    match repo.find_by_id(id).await {
        Ok(Some(idea)) => Json(IdeaResponse::from(&idea)).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get idea: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_idea(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateIdeaRequest>,
) -> impl IntoResponse {
    let repo = IdeaRepository::new(state.db.connection());

    // Check ownership
    match repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(StatusCode::FORBIDDEN, "FORBIDDEN", "Not the idea owner").into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea for update: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate optional fields
    if let Some(ref t) = body.title {
        if t.trim().is_empty() || t.len() > 200 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Title must be 1-200 chars",
            )
            .into_response();
        }
    }
    if let Some(ref s) = body.summary {
        if s.trim().is_empty() || s.len() > 500 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Summary must be 1-500 chars",
            )
            .into_response();
        }
    }
    let openness = match &body.openness {
        Some(o) => match IdeaOpenness::from_str_opt(o) {
            Some(v) => Some(v),
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid openness. Must be: open, collaborative, or commercial",
                )
                .into_response();
            }
        },
        None => None,
    };

    match repo
        .update(
            id,
            body.title.as_deref(),
            body.summary.as_deref(),
            body.description.as_deref(),
            openness,
            body.category_id,
        )
        .await
    {
        Ok(idea) => Json(IdeaResponse::from(&idea)).into_response(),
        Err(e) => {
            tracing::error!("Failed to update idea: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn archive_idea(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = IdeaRepository::new(state.db.connection());

    // Check ownership
    match repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(StatusCode::FORBIDDEN, "FORBIDDEN", "Not the idea owner").into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea for archive: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    match repo.archive(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to archive idea: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

// --- Stokes ---

async fn list_stokes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListIdeasQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let repo = StokeRepository::new(state.db.connection());
    match repo.list_for_idea(id, page, per_page).await {
        Ok((stokes, total)) => {
            let total_pages = (total + per_page - 1) / per_page;
            Json(StokeListResponse {
                data: stokes
                    .iter()
                    .map(|s| StokeResponse {
                        id: s.id,
                        user_id: s.user_id,
                        idea_id: s.idea_id,
                        created_at: s.created_at.to_rfc3339(),
                    })
                    .collect(),
                meta: PaginationMeta {
                    total,
                    page,
                    per_page,
                    total_pages,
                },
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list stokes: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn stoke_idea(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let stoke_repo = StokeRepository::new(state.db.connection());

    // Check if already stoked
    match stoke_repo.exists(auth.user_id, id).await {
        Ok(true) => {
            return err(StatusCode::CONFLICT, "CONFLICT", "Already stoked this idea").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check stoke existence: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    // Verify idea exists
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea for stoke: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(Some(_)) => {}
    }

    // Create stoke
    match stoke_repo.create(Uuid::new_v4(), auth.user_id, id).await {
        Ok(stoke) => {
            // Update denormalized count
            if let Ok(count) = stoke_repo.count_for_idea(id).await {
                let _ = idea_repo.update_stoke_count(id, count as i32).await;
            }

            (
                StatusCode::CREATED,
                Json(StokeResponse {
                    id: stoke.id,
                    user_id: stoke.user_id,
                    idea_id: stoke.idea_id,
                    created_at: stoke.created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create stoke: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn withdraw_stoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let stoke_repo = StokeRepository::new(state.db.connection());
    let idea_repo = IdeaRepository::new(state.db.connection());

    match stoke_repo.delete(auth.user_id, id).await {
        Ok(result) if result.rows_affected == 0 => {
            err(StatusCode::NOT_FOUND, "NOT_FOUND", "Stoke not found").into_response()
        }
        Ok(_) => {
            // Update denormalized count
            if let Ok(count) = stoke_repo.count_for_idea(id).await {
                let _ = idea_repo.update_stoke_count(id, count as i32).await;
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!("Failed to withdraw stoke: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
