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
    #[serde(default)]
    pub nda_required: Option<bool>,
    #[serde(default)]
    pub nda_signed: Option<bool>,
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
    #[serde(default)]
    pub skills: serde_json::Value,
    pub looking_for: Option<String>,
    pub availability: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicUserResponse {
    pub id: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    #[serde(default)]
    pub skills: serde_json::Value,
    pub looking_for: Option<String>,
    pub availability: Option<String>,
    #[serde(default)]
    pub idea_count: u64,
    #[serde(default)]
    pub stoke_count: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserListResponse {
    pub data: Vec<PublicUserResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub looking_for: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<String>,
}

// --- Skills ---

#[derive(Debug, Clone, Deserialize)]
pub struct SkillCategory {
    pub category: String,
    pub skills: Vec<String>,
}

// --- Invite Links ---

#[derive(Debug, Clone, Deserialize)]
pub struct InviteLinkResponse {
    pub token: String,
    pub idea_id: String,
    pub permission: String,
    pub access_count: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateInviteLinkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
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
    pub role_label: Option<String>,
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

// --- Bots ---

#[derive(Debug, Clone, Deserialize)]
pub struct BotProfileResponse {
    pub id: String,
    pub username: String,
    pub operator: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotListResponse {
    pub data: Vec<BotProfileResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndorsementResponse {
    pub id: String,
    pub bot_id: String,
    pub idea_id: String,
    pub reason: String,
    pub created_at: String,
}

// --- Notifications ---

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub link_url: Option<String>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationListResponse {
    pub data: Vec<NotificationResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnreadCountResponse {
    pub unread_count: u64,
}

// --- NDA ---

#[derive(Debug, Clone, Deserialize)]
pub struct NdaTemplateResponse {
    pub id: String,
    pub idea_id: String,
    pub title: String,
    pub body: String,
    pub confidentiality_period_days: i32,
    pub jurisdiction: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignNdaRequest {
    pub signer_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NdaStatusResponse {
    pub has_signed: bool,
    pub signed_at: Option<String>,
    pub expires_at: Option<String>,
}

// --- Board Tasks ---

#[derive(Debug, Clone, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub idea_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<String>,
    pub created_by: String,
    pub skill_tags: Vec<String>,
    pub due_date: Option<String>,
    pub position: i32,
    #[serde(default)]
    pub budget_cents: i64,
    #[serde(default)]
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoardResponse {
    pub idea_id: String,
    pub columns: BoardColumns,
    pub total_tasks: u64,
    #[serde(default)]
    pub total_budget_cents: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoardColumns {
    pub open: Vec<TaskResponse>,
    pub assigned: Vec<TaskResponse>,
    pub in_review: Vec<TaskResponse>,
    pub done: Vec<TaskResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_cents: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTaskStatusRequest {
    pub status: String,
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
