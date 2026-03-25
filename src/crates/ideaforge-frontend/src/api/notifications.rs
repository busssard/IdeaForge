use super::client;
use super::types::{NotificationListResponse, UnreadCountResponse};

pub async fn list_notifications(
    page: u64,
    per_page: u64,
    unread_only: bool,
) -> Result<NotificationListResponse, client::ApiError> {
    let url = format!(
        "/api/v1/notifications?page={page}&per_page={per_page}&unread_only={unread_only}"
    );
    client::get(&url).await
}

pub async fn unread_count() -> Result<UnreadCountResponse, client::ApiError> {
    client::get("/api/v1/notifications/unread-count").await
}

pub async fn mark_read(id: &str) -> Result<(), client::ApiError> {
    // PUT returns the notification, but we don't need it
    let _: serde_json::Value = client::put(
        &format!("/api/v1/notifications/{id}/read"),
        &serde_json::json!({}),
    )
    .await?;
    Ok(())
}

pub async fn mark_all_read() -> Result<(), client::ApiError> {
    let _: serde_json::Value = client::put(
        "/api/v1/notifications/read-all",
        &serde_json::json!({}),
    )
    .await?;
    Ok(())
}
