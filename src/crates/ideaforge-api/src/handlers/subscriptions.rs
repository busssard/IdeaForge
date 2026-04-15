//! Idea subscription handlers.
//!
//! Users can subscribe to ideas to get notified of updates.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::repositories::subscription_repo::SubscriptionRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/subscribe", post(subscribe).delete(unsubscribe))
        .route("/:id/subscribe/status", get(subscription_status))
}

#[derive(Debug, Serialize)]
pub struct SubscriptionStatusResponse {
    pub subscribed: bool,
}

async fn subscription_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = SubscriptionRepository::new(state.db.connection());
    match repo.exists(auth.user_id, id).await {
        Ok(subscribed) => {
            Json(SubscriptionStatusResponse { subscribed }).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check subscription status: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub idea_id: Uuid,
    pub created_at: String,
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

async fn subscribe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = SubscriptionRepository::new(state.db.connection());

    // Idempotent: if already subscribed, return 200 OK with a minimal response
    // rather than 409. The end state is what the client wanted either way, and a
    // 409 forced the frontend to silently revert its optimistic update — which
    // read as "subscribe does nothing".
    match repo.exists(auth.user_id, id).await {
        Ok(true) => {
            return (
                StatusCode::OK,
                Json(SubscriptionResponse {
                    id: Uuid::nil(),
                    user_id: auth.user_id,
                    idea_id: id,
                    created_at: String::new(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check subscription existence: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    match repo.create(Uuid::new_v4(), auth.user_id, id).await {
        Ok(sub) => (
            StatusCode::CREATED,
            Json(SubscriptionResponse {
                id: sub.id,
                user_id: sub.user_id,
                idea_id: sub.idea_id,
                created_at: sub.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create subscription: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn unsubscribe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = SubscriptionRepository::new(state.db.connection());

    // Idempotent: a missing subscription is treated as success, matching the
    // symmetry of `subscribe`.
    match repo.delete(auth.user_id, id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("Failed to delete subscription: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
