use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::enums::{FlagStatus, FlagTargetType, UserRole};
use ideaforge_db::repositories::flag_repo::FlagRepository;

/// Public flag routes (authenticated users).
pub fn routes() -> Router<AppState> {
    Router::new().route("/", axum::routing::post(create_flag))
}

/// Admin-only flag routes.
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/flags", get(list_pending_flags))
        .route("/flags/:id", put(review_flag))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    pub target_type: String,
    pub target_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub id: Uuid,
    pub flagger_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub reason: String,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FlagListResponse {
    pub data: Vec<FlagResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListFlagsQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewFlagRequest {
    pub status: String,
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

fn flag_response(m: &ideaforge_db::entities::flag::Model) -> FlagResponse {
    FlagResponse {
        id: m.id,
        flagger_id: m.flagger_id,
        target_type: m.target_type.to_string(),
        target_id: m.target_id,
        reason: m.reason.clone(),
        status: m.status.to_string(),
        reviewed_by: m.reviewed_by,
        created_at: m.created_at.to_rfc3339(),
    }
}

// --- Handlers ---

/// POST /api/v1/flags - Report content (requires auth).
async fn create_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateFlagRequest>,
) -> impl IntoResponse {
    // Validate target_type
    let target_type = match FlagTargetType::from_str_opt(&body.target_type) {
        Some(tt) => tt,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid target_type. Must be: idea, comment, or user",
            )
            .into_response();
        }
    };

    // Validate reason
    let reason = body.reason.trim();
    if reason.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Reason is required",
        )
        .into_response();
    }
    if reason.len() > 2000 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Reason must be at most 2000 characters",
        )
        .into_response();
    }

    // Prevent self-flagging for user targets
    if target_type == FlagTargetType::User && body.target_id == auth.user_id {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "You cannot flag yourself",
        )
        .into_response();
    }

    let repo = FlagRepository::new(state.db.connection());
    match repo
        .create(Uuid::new_v4(), auth.user_id, target_type, body.target_id, reason)
        .await
    {
        Ok(flag) => (StatusCode::CREATED, Json(flag_response(&flag))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create flag: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// GET /api/v1/admin/flags - List pending flags (admin only).
async fn list_pending_flags(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListFlagsQuery>,
) -> impl IntoResponse {
    // Admin check
    let role = UserRole::from_str_opt(&auth.role);
    if !role.map_or(false, |r| r.is_admin()) {
        return err(StatusCode::FORBIDDEN, "FORBIDDEN", "Admin access required").into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let repo = FlagRepository::new(state.db.connection());
    match repo.list_pending(page, per_page).await {
        Ok((flags, total)) => {
            let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
            Json(FlagListResponse {
                data: flags.iter().map(flag_response).collect(),
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
            tracing::error!("Failed to list pending flags: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// PUT /api/v1/admin/flags/:id - Review a flag (admin only).
async fn review_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ReviewFlagRequest>,
) -> impl IntoResponse {
    // Admin check
    let role = UserRole::from_str_opt(&auth.role);
    if !role.map_or(false, |r| r.is_admin()) {
        return err(StatusCode::FORBIDDEN, "FORBIDDEN", "Admin access required").into_response();
    }

    // Validate status
    let status = match FlagStatus::from_str_opt(&body.status) {
        Some(s) if s != FlagStatus::Pending => s,
        Some(_) => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Status must be: reviewed or dismissed",
            )
            .into_response();
        }
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid status. Must be: reviewed or dismissed",
            )
            .into_response();
        }
    };

    let repo = FlagRepository::new(state.db.connection());

    // Verify flag exists
    match repo.find_by_id(id).await {
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Flag not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find flag: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(Some(_)) => {}
    }

    match repo.review(id, auth.user_id, status).await {
        Ok(flag) => Json(flag_response(&flag)).into_response(),
        Err(e) => {
            tracing::error!("Failed to review flag: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
