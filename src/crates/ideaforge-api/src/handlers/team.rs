//! Team formation handlers -- THE KILLER FEATURE.
//!
//! Provides team member management and application workflows
//! that let Entrepreneurs build teams around their ideas.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::enums::{ApplicationStatus, TeamMemberRole};
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::team_repo::{TeamApplicationRepository, TeamMemberRepository};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/team/apply", axum::routing::post(apply_to_team))
        .route("/:id/team/applications", get(list_applications))
        .route("/:id/team/applications/:aid", put(review_application))
        .route("/:id/team", get(list_team_members))
        .route("/:id/team/:uid", delete(remove_team_member))
        .route("/:id/team/:uid/role", put(update_team_role))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub accepted: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListApplicationsQuery {
    pub status: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub message: String,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationListResponse {
    pub data: Vec<ApplicationResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub role_label: Option<String>,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct TeamListResponse {
    pub data: Vec<TeamMemberResponse>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: Option<String>,
    pub role_label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
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

// --- Handlers ---

async fn apply_to_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ApplyRequest>,
) -> impl IntoResponse {
    // Validate message
    let message = body.message.trim();
    if message.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Message is required",
        )
        .into_response();
    }
    if message.len() > 2000 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Message must be at most 2000 characters",
        )
        .into_response();
    }

    // Check idea exists
    let idea_repo = IdeaRepository::new(state.db.connection());
    let idea = match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) => idea,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Cannot apply to own idea
    if idea.author_id == auth.user_id {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Cannot apply to your own idea",
        )
        .into_response();
    }

    // Check if already applied (pending or accepted)
    let app_repo = TeamApplicationRepository::new(state.db.connection());
    match app_repo.exists(auth.user_id, id).await {
        Ok(true) => {
            return err(
                StatusCode::CONFLICT,
                "CONFLICT",
                "You already have an active application for this idea",
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check application existence: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    // Check if already a team member
    let member_repo = TeamMemberRepository::new(state.db.connection());
    match member_repo.exists(auth.user_id, id).await {
        Ok(true) => {
            return err(
                StatusCode::CONFLICT,
                "CONFLICT",
                "You are already a team member",
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check team membership: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    match app_repo
        .create(Uuid::new_v4(), id, auth.user_id, message)
        .await
    {
        Ok(app) => (
            StatusCode::CREATED,
            Json(ApplicationResponse {
                id: app.id,
                idea_id: app.idea_id,
                user_id: app.user_id,
                message: app.message,
                status: app.status.to_string(),
                reviewed_by: app.reviewed_by,
                created_at: app.created_at.to_rfc3339(),
                updated_at: app.updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create application: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_applications(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<ListApplicationsQuery>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can view applications",
            )
            .into_response();
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let status_filter = params
        .status
        .as_deref()
        .and_then(ApplicationStatus::from_str_opt);

    let app_repo = TeamApplicationRepository::new(state.db.connection());
    match app_repo
        .list_for_idea(id, status_filter, page, per_page)
        .await
    {
        Ok((apps, total)) => {
            let total_pages = if total == 0 {
                0
            } else {
                (total + per_page - 1) / per_page
            };
            Json(ApplicationListResponse {
                data: apps
                    .iter()
                    .map(|a| ApplicationResponse {
                        id: a.id,
                        idea_id: a.idea_id,
                        user_id: a.user_id,
                        message: a.message.clone(),
                        status: a.status.to_string(),
                        reviewed_by: a.reviewed_by,
                        created_at: a.created_at.to_rfc3339(),
                        updated_at: a.updated_at.to_rfc3339(),
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
            tracing::error!("Failed to list applications: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn review_application(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, aid)): Path<(Uuid, Uuid)>,
    Json(body): Json<ReviewRequest>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can review applications",
            )
            .into_response();
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Find the application
    let app_repo = TeamApplicationRepository::new(state.db.connection());
    let application = match app_repo.find_by_id(aid).await {
        Ok(Some(app)) if app.idea_id == id => app,
        Ok(Some(_)) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Application not found for this idea",
            )
            .into_response();
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Application not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find application: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Check application is still pending
    if application.status != ApplicationStatus::Pending {
        return err(
            StatusCode::CONFLICT,
            "CONFLICT",
            "Application has already been reviewed",
        )
        .into_response();
    }

    let new_status = if body.accepted {
        ApplicationStatus::Accepted
    } else {
        ApplicationStatus::Rejected
    };

    // If accepted, create team member first
    if body.accepted {
        let member_repo = TeamMemberRepository::new(state.db.connection());
        if let Err(e) = member_repo
            .create(
                Uuid::new_v4(),
                id,
                application.user_id,
                TeamMemberRole::Builder,
            )
            .await
        {
            tracing::error!("Failed to create team member: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Update application status
    match app_repo.update_status(aid, new_status, auth.user_id).await {
        Ok(app) => Json(ApplicationResponse {
            id: app.id,
            idea_id: app.idea_id,
            user_id: app.user_id,
            message: app.message,
            status: app.status.to_string(),
            reviewed_by: app.reviewed_by,
            created_at: app.created_at.to_rfc3339(),
            updated_at: app.updated_at.to_rfc3339(),
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to update application status: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_team_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let member_repo = TeamMemberRepository::new(state.db.connection());
    match member_repo.list_for_idea(id).await {
        Ok(members) => Json(TeamListResponse {
            data: members
                .iter()
                .map(|m| TeamMemberResponse {
                    id: m.id,
                    idea_id: m.idea_id,
                    user_id: m.user_id,
                    role: m.role.to_string(),
                    role_label: m.role_label.clone(),
                    joined_at: m.joined_at.to_rfc3339(),
                })
                .collect(),
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list team members: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn remove_team_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, uid)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can remove team members",
            )
            .into_response();
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    let member_repo = TeamMemberRepository::new(state.db.connection());
    match member_repo.remove(id, uid).await {
        Ok(result) if result.rows_affected == 0 => {
            err(StatusCode::NOT_FOUND, "NOT_FOUND", "Team member not found").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to remove team member: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_team_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, uid)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    // Only idea author can change team roles
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can change team roles",
            )
            .into_response();
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate role_label length if provided
    if let Some(ref label) = body.role_label {
        if label.trim().is_empty() || label.len() > 100 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Role label must be 1-100 chars",
            )
            .into_response();
        }
    }

    // If both role and role_label are provided, use update_role
    // If only role_label, use update_role_label
    // If only role, use update_role with existing label (or None)
    let member_repo = TeamMemberRepository::new(state.db.connection());

    let result = if let Some(ref role_str) = body.role {
        let role = match TeamMemberRole::from_str_opt(role_str) {
            Some(r) => r,
            None => {
                return err(
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    "Invalid role. Must be: lead, builder, or advisor",
                )
                .into_response();
            }
        };
        member_repo
            .update_role(id, uid, role, body.role_label.as_deref())
            .await
    } else if body.role_label.is_some() {
        member_repo
            .update_role_label(id, uid, body.role_label.as_deref())
            .await
    } else {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "At least one of role or role_label must be provided",
        )
        .into_response();
    };

    match result {
        Ok(member) => Json(TeamMemberResponse {
            id: member.id,
            idea_id: member.idea_id,
            user_id: member.user_id,
            role: member.role.to_string(),
            role_label: member.role_label,
            joined_at: member.joined_at.to_rfc3339(),
        })
        .into_response(),
        Err(sea_orm::DbErr::RecordNotFound(_)) => {
            err(StatusCode::NOT_FOUND, "NOT_FOUND", "Team member not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update team role: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
