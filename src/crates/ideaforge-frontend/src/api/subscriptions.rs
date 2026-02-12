use super::client;
use super::types::SubscriptionResponse;

pub async fn subscribe(idea_id: &str) -> Result<SubscriptionResponse, client::ApiError> {
    client::post_no_body(&format!("/api/v1/ideas/{idea_id}/subscribe")).await
}

pub async fn unsubscribe(idea_id: &str) -> Result<(), client::ApiError> {
    client::delete_req(&format!("/api/v1/ideas/{idea_id}/subscribe")).await
}
