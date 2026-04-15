use super::client;
use super::types::{
    ApplyToTeamRequest, ReviewApplicationRequest, TeamApplicationListResponse,
    TeamApplicationResponse, TeamMemberResponse,
};

pub async fn apply_to_team(
    idea_id: &str,
    message: String,
) -> Result<TeamApplicationResponse, client::ApiError> {
    let req = ApplyToTeamRequest { message };
    client::post(&format!("/api/v1/ideas/{idea_id}/team/apply"), &req).await
}

pub async fn list_applications(
    idea_id: &str,
) -> Result<TeamApplicationListResponse, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/team/applications")).await
}

pub async fn review_application(
    idea_id: &str,
    app_id: &str,
    accepted: bool,
) -> Result<TeamApplicationResponse, client::ApiError> {
    let req = ReviewApplicationRequest { accepted };
    client::put(
        &format!("/api/v1/ideas/{idea_id}/team/applications/{app_id}"),
        &req,
    )
    .await
}

pub async fn list_team_members(idea_id: &str) -> Result<Vec<TeamMemberResponse>, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/team")).await
}
