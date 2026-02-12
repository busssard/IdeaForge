use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::{AuthUser, OptionalAuth};
use crate::state::AppState;
use ideaforge_db::entities::enums::{ContributionKind, IdeaMaturity, IdeaOpenness};
use ideaforge_db::entities::stoke;
use ideaforge_db::repositories::contribution_repo::ContributionRepository;
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::stoke_repo::StokeRepository;
use sea_orm::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_ideas).post(create_idea))
        .route("/my-stokes", get(list_my_stoked_ideas))
        .route("/:id", get(get_idea).put(update_idea).delete(archive_idea))
        .route("/:id/stokes", get(list_stokes).post(stoke_idea))
        .route("/:id/stokes/mine", delete(withdraw_stoke))
        .route("/:id/contributions", get(list_contributions).post(create_contribution))
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
    pub has_stoked: bool,
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

#[derive(Debug, Deserialize)]
pub struct CreateContributionRequest {
    pub contribution_type: String,
    pub title: Option<String>,
    pub body: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ContributionResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub contribution_type: String,
    pub title: Option<String>,
    pub body: String,
    pub parent_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListContributionsQuery {
    pub r#type: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ContributionListResponse {
    pub data: Vec<ContributionResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct MyStokedIdeasResponse {
    pub data: Vec<StokedIdeaEntry>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct StokedIdeaEntry {
    pub idea_id: Uuid,
    pub stoked_at: String,
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

fn idea_response(m: &ideaforge_db::entities::idea::Model, has_stoked: bool) -> IdeaResponse {
    IdeaResponse {
        id: m.id,
        author_id: m.author_id,
        title: m.title.clone(),
        summary: m.summary.clone(),
        description: m.description.clone(),
        maturity: m.maturity.to_string(),
        openness: m.openness.to_string(),
        category_id: m.category_id,
        stoke_count: m.stoke_count,
        has_stoked,
        created_at: m.created_at.to_rfc3339(),
        updated_at: m.updated_at.to_rfc3339(),
    }
}

fn parse_contribution_kind(s: &str) -> Option<ContributionKind> {
    match s {
        "comment" => Some(ContributionKind::Comment),
        "suggestion" => Some(ContributionKind::Suggestion),
        "design" => Some(ContributionKind::Design),
        "code" => Some(ContributionKind::Code),
        "research" => Some(ContributionKind::Research),
        "other" => Some(ContributionKind::Other),
        _ => None,
    }
}

// --- Handlers ---

async fn list_ideas(
    State(state): State<AppState>,
    opt_auth: OptionalAuth,
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
            // Batch-check which ideas the user has stoked
            let stoked_set = if let OptionalAuth(Some(ref auth)) = opt_auth {
                let idea_ids: Vec<Uuid> = ideas.iter().map(|i| i.id).collect();
                if !idea_ids.is_empty() {
                    match stoke::Entity::find()
                        .filter(stoke::Column::UserId.eq(auth.user_id))
                        .filter(stoke::Column::IdeaId.is_in(idea_ids))
                        .all(state.db.connection())
                        .await
                    {
                        Ok(stokes) => stokes.iter().map(|s| s.idea_id).collect::<std::collections::HashSet<_>>(),
                        Err(e) => {
                            tracing::warn!("Failed to check user stokes: {e}");
                            std::collections::HashSet::new()
                        }
                    }
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            };

            let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
            Json(IdeaListResponse {
                data: ideas.iter().map(|i| idea_response(i, stoked_set.contains(&i.id))).collect(),
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
        Ok(idea) => (StatusCode::CREATED, Json(idea_response(&idea, false))).into_response(),
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
    opt_auth: OptionalAuth,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = IdeaRepository::new(state.db.connection());
    match repo.find_by_id(id).await {
        Ok(Some(idea)) => {
            let has_stoked = if let OptionalAuth(Some(ref auth)) = opt_auth {
                let stoke_repo = StokeRepository::new(state.db.connection());
                stoke_repo.exists(auth.user_id, id).await.unwrap_or(false)
            } else {
                false
            };
            Json(idea_response(&idea, has_stoked)).into_response()
        }
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
        Ok(idea) => Json(idea_response(&idea, false)).into_response(),
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
            let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
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

// --- Contributions ---

async fn create_contribution(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateContributionRequest>,
) -> impl IntoResponse {
    // Validate body
    let body_text = body.body.trim();
    if body_text.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Body is required",
        )
        .into_response();
    }
    if body_text.len() > 10000 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Body must be at most 10000 characters",
        )
        .into_response();
    }

    // Validate contribution_type
    let contribution_type = match parse_contribution_kind(&body.contribution_type) {
        Some(ct) => ct,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid contribution_type. Must be: comment, suggestion, design, code, research, or other",
            )
            .into_response();
        }
    };

    // Verify idea exists
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea for contribution: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(Some(_)) => {}
    }

    let repo = ContributionRepository::new(state.db.connection());
    match repo
        .create(
            Uuid::new_v4(),
            id,
            auth.user_id,
            contribution_type,
            body.title.clone(),
            body_text,
            body.parent_id,
        )
        .await
    {
        Ok(c) => (
            StatusCode::CREATED,
            Json(ContributionResponse {
                id: c.id,
                idea_id: c.idea_id,
                user_id: c.user_id,
                contribution_type: c.contribution_type.to_string(),
                title: c.title,
                body: c.body,
                parent_id: c.parent_id,
                created_at: c.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create contribution: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_contributions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListContributionsQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let type_filter = params.r#type.as_deref().and_then(parse_contribution_kind);

    let repo = ContributionRepository::new(state.db.connection());
    match repo.list_for_idea(id, type_filter, page, per_page).await {
        Ok((contributions, total)) => {
            let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
            Json(ContributionListResponse {
                data: contributions
                    .iter()
                    .map(|c| ContributionResponse {
                        id: c.id,
                        idea_id: c.idea_id,
                        user_id: c.user_id,
                        contribution_type: c.contribution_type.to_string(),
                        title: c.title.clone(),
                        body: c.body.clone(),
                        parent_id: c.parent_id,
                        created_at: c.created_at.to_rfc3339(),
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
            tracing::error!("Failed to list contributions: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

// --- My Stoked Ideas ---

async fn list_my_stoked_ideas(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListIdeasQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let query = stoke::Entity::find()
        .filter(stoke::Column::UserId.eq(auth.user_id))
        .order_by_desc(stoke::Column::CreatedAt);

    let paginator = query.paginate(state.db.connection(), per_page);
    match paginator.num_items().await {
        Ok(total) => match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(stokes) => {
                let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
                Json(MyStokedIdeasResponse {
                    data: stokes
                        .iter()
                        .map(|s| StokedIdeaEntry {
                            idea_id: s.idea_id,
                            stoked_at: s.created_at.to_rfc3339(),
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
                tracing::error!("Failed to list my stoked ideas: {e}");
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error",
                )
                .into_response()
            }
        },
        Err(e) => {
            tracing::error!("Failed to count my stoked ideas: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
