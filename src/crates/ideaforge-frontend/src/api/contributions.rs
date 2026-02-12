use super::client;
use super::types::{ContributionListResponse, ContributionResponse, CreateContributionRequest};

pub async fn list_contributions(
    idea_id: &str,
    contribution_type: Option<&str>,
    page: u64,
    per_page: u64,
) -> Result<ContributionListResponse, client::ApiError> {
    let mut url = format!(
        "/api/v1/ideas/{idea_id}/contributions?page={page}&per_page={per_page}"
    );
    if let Some(t) = contribution_type {
        url.push_str(&format!("&type={t}"));
    }
    client::get(&url).await
}

pub async fn create_contribution(
    idea_id: &str,
    req: CreateContributionRequest,
) -> Result<ContributionResponse, client::ApiError> {
    client::post(&format!("/api/v1/ideas/{idea_id}/contributions"), &req).await
}
