use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::enums::InvitePermission;
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::invite_link_repo::InviteLinkRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/invites", get(list_invites).post(create_invite))
        .route("/:id/invites/:token", delete(revoke_invite))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub permission: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub token: String,
    pub url: String,
    pub permission: String,
    pub created_by: Uuid,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub access_count: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct InviteListResponse {
    pub data: Vec<InviteResponse>,
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

fn invite_response(m: &ideaforge_db::entities::invite_link::Model, base_url: &str) -> InviteResponse {
    InviteResponse {
        id: m.id,
        idea_id: m.idea_id,
        token: m.token.clone(),
        url: format!("{}/ideas/{}?token={}", base_url, m.idea_id, m.token),
        permission: match m.permission {
            InvitePermission::View => "view".to_string(),
            InvitePermission::Comment => "comment".to_string(),
        },
        created_by: m.created_by,
        expires_at: m.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
        revoked_at: m.revoked_at.as_ref().map(|dt| dt.to_rfc3339()),
        access_count: m.access_count,
        created_at: m.created_at.to_rfc3339(),
    }
}

// --- Handlers ---

async fn create_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateInviteRequest>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can create invite links",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
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

    // Parse permission (default to view)
    let permission = match body.permission.as_deref() {
        None | Some("view") => InvitePermission::View,
        Some("comment") => InvitePermission::Comment,
        Some(_) => {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Invalid permission. Must be: view or comment",
            )
            .into_response();
        }
    };

    // Generate random 32-char hex token
    let token = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.r#gen();
        hex::encode(bytes)
    };

    let invite_repo = InviteLinkRepository::new(state.db.connection());
    match invite_repo
        .create(Uuid::new_v4(), id, &token, permission, auth.user_id)
        .await
    {
        Ok(invite) => {
            // TODO: Get base_url from config or request headers
            let base_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
            (
                StatusCode::CREATED,
                Json(invite_response(&invite, &base_url)),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create invite link: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_invites(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can view invite links",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
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

    let invite_repo = InviteLinkRepository::new(state.db.connection());
    match invite_repo.list_for_idea(id).await {
        Ok(invites) => {
            let base_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
            Json(InviteListResponse {
                data: invites.iter().map(|i| invite_response(i, &base_url)).collect(),
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list invite links: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn revoke_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, token)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can revoke invite links",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
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

    let invite_repo = InviteLinkRepository::new(state.db.connection());

    // Verify the invite belongs to this idea
    match invite_repo.find_by_token(&token).await {
        Ok(Some(invite)) if invite.idea_id == id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Invite link not found for this idea",
            )
            .into_response()
        }
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Invite link not found",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find invite link: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    match invite_repo.revoke(&token).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to revoke invite link: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
