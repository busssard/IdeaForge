use super::client;
use super::types::{BotListResponse, EndorsementResponse};

pub async fn list_bots() -> Result<BotListResponse, client::ApiError> {
    client::get("/api/v1/bots").await
}

pub async fn list_endorsements(idea_id: &str) -> Result<Vec<EndorsementResponse>, client::ApiError> {
    client::get(&format!("/api/v1/bots/endorsements/{idea_id}")).await
}
