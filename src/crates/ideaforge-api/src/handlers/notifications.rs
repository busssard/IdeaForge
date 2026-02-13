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
use ideaforge_db::repositories::notification_repo::NotificationRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/unread-count", get(unread_count))
        .route("/read-all", put(read_all))
        .route("/:id/read", put(mark_read))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub data: Vec<NotificationResponse>,
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
pub struct UnreadCountResponse {
    pub unread_count: u64,
}

#[derive(Debug, Serialize)]
pub struct MarkAllReadResponse {
    pub marked_count: u64,
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

fn notification_response(m: &ideaforge_db::entities::notification::Model) -> NotificationResponse {
    NotificationResponse {
        id: m.id,
        user_id: m.user_id,
        kind: m.kind.to_string(),
        title: m.title.clone(),
        message: m.message.clone(),
        link_url: m.link_url.clone(),
        read_at: m.read_at.map(|dt| dt.to_rfc3339()),
        created_at: m.created_at.to_rfc3339(),
    }
}

// --- Handlers ---

/// GET /api/v1/notifications - List current user's notifications.
async fn list_notifications(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListNotificationsQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let unread_only = params.unread_only.unwrap_or(false);

    let repo = NotificationRepository::new(state.db.connection());
    match repo.list_for_user(auth.user_id, unread_only, page, per_page).await {
        Ok((notifications, total)) => {
            let total_pages = if total == 0 { 0 } else { (total + per_page - 1) / per_page };
            Json(NotificationListResponse {
                data: notifications.iter().map(notification_response).collect(),
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
            tracing::error!("Failed to list notifications: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// PUT /api/v1/notifications/:id/read - Mark a notification as read.
async fn mark_read(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = NotificationRepository::new(state.db.connection());
    match repo.mark_read(id, auth.user_id).await {
        Ok(Some(n)) => Json(notification_response(&n)).into_response(),
        Ok(None) => {
            err(StatusCode::NOT_FOUND, "NOT_FOUND", "Notification not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to mark notification as read: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// PUT /api/v1/notifications/read-all - Mark all notifications as read.
async fn read_all(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let repo = NotificationRepository::new(state.db.connection());
    match repo.mark_all_read(auth.user_id).await {
        Ok(count) => Json(MarkAllReadResponse { marked_count: count }).into_response(),
        Err(e) => {
            tracing::error!("Failed to mark all notifications as read: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// GET /api/v1/notifications/unread-count - Get unread notification count.
async fn unread_count(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let repo = NotificationRepository::new(state.db.connection());
    match repo.count_unread(auth.user_id).await {
        Ok(count) => Json(UnreadCountResponse { unread_count: count }).into_response(),
        Err(e) => {
            tracing::error!("Failed to count unread notifications: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
