use super::client;
use super::types::StokeResponse;

pub async fn stoke_idea(idea_id: &str) -> Result<StokeResponse, client::ApiError> {
    client::post_no_body(&format!("/api/v1/ideas/{idea_id}/stokes")).await
}

pub async fn withdraw_stoke(idea_id: &str) -> Result<(), client::ApiError> {
    client::delete_req(&format!("/api/v1/ideas/{idea_id}/stokes/mine")).await
}
