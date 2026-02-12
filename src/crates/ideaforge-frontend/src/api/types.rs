use serde::{Deserialize, Serialize};

// --- Auth ---

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// --- Ideas ---

#[derive(Debug, Clone, Deserialize)]
pub struct IdeaResponse {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub maturity: String,
    pub openness: String,
    pub category_id: Option<String>,
    pub stoke_count: i32,
    #[serde(default)]
    pub has_stoked: Option<bool>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdeaListResponse {
    pub data: Vec<IdeaResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateIdeaRequest {
    pub title: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateIdeaRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Option<String>>,
}

// --- Stokes ---

#[derive(Debug, Clone, Deserialize)]
pub struct StokeResponse {
    pub id: String,
    pub user_id: String,
    pub idea_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StokeListResponse {
    pub data: Vec<StokeResponse>,
    pub meta: PaginationMeta,
}

// --- Users ---

#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicUserResponse {
    pub id: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

// --- Categories ---

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

// --- Contributions ---

#[derive(Debug, Clone, Deserialize)]
pub struct ContributionResponse {
    pub id: String,
    pub idea_id: String,
    pub user_id: String,
    pub contribution_type: String,
    pub title: Option<String>,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContributionListResponse {
    pub data: Vec<ContributionResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateContributionRequest {
    pub contribution_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
}

// --- Team ---

#[derive(Debug, Clone, Deserialize)]
pub struct TeamApplicationResponse {
    pub id: String,
    pub idea_id: String,
    pub user_id: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamApplicationListResponse {
    pub data: Vec<TeamApplicationResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyToTeamRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewApplicationRequest {
    pub accepted: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamMemberResponse {
    pub id: String,
    pub idea_id: String,
    pub user_id: String,
    pub display_name: String,
    pub role: String,
    pub joined_at: String,
}

// --- Subscriptions ---

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub user_id: String,
    pub idea_id: String,
    pub created_at: String,
}

// --- Errors ---

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}
