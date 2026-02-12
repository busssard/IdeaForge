use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::repositories::user_repo::UserRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/:id", get(get_user))
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PublicUserResponse {
    pub id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<Option<String>>,
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

async fn get_me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let repo = UserRepository::new(state.db.connection());
    match repo.find_by_id(auth.user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "User not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateMeRequest>,
) -> impl IntoResponse {
    // Validate
    if let Some(ref name) = body.display_name {
        if name.trim().is_empty() || name.len() > 100 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Display name must be 1-100 chars",
            )
            .into_response();
        }
    }
    if let Some(ref bio) = body.bio {
        if bio.len() > 2000 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Bio too long (max 2000 chars)",
            )
            .into_response();
        }
    }

    let repo = UserRepository::new(state.db.connection());
    match repo
        .update(
            auth.user_id,
            body.display_name.as_deref(),
            body.bio.as_deref(),
            body.avatar_url
                .as_ref()
                .map(|opt| opt.as_deref()),
        )
        .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to update user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = UserRepository::new(state.db.connection());
    match repo.find_by_id(id).await {
        Ok(Some(user)) => Json(PublicUserResponse {
            id: user.id,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            role: user.role.to_string(),
            created_at: user.created_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "User not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
