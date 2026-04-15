use serde::Deserialize;

use super::client;
use super::types::SubscriptionResponse;

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionStatus {
    pub subscribed: bool,
}

pub async fn subscribe(idea_id: &str) -> Result<SubscriptionResponse, client::ApiError> {
    client::post_no_body(&format!("/api/v1/ideas/{idea_id}/subscribe")).await
}

pub async fn unsubscribe(idea_id: &str) -> Result<(), client::ApiError> {
    client::delete_req(&format!("/api/v1/ideas/{idea_id}/subscribe")).await
}

pub async fn get_subscription_status(idea_id: &str) -> Result<SubscriptionStatus, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/subscribe/status")).await
}
