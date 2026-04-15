//! Thin wrapper around the MLS delivery-service endpoints. All bytes go
//! over the wire as base64 strings so the backend never needs to know the
//! transport representation of MLS primitives.

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::api::client::{self, ApiError};

#[derive(Debug, Clone, Serialize)]
pub struct PublishKeyPackagesRequest {
    pub key_packages: Vec<String>,
    pub ttl_days: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublishKeyPackagesResponse {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeyPackageBundle {
    pub id: String,
    pub user_id: String,
    pub key_package_b64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupSummary {
    pub id: String,
    pub mls_group_id_b64: String,
    pub name: Option<String>,
    pub created_by: String,
    pub created_at: String,
    #[serde(default)]
    pub members: Vec<MemberSummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberSummary {
    pub user_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupList {
    pub data: Vec<GroupSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateGroupRequest {
    pub mls_group_id_b64: String,
    pub name: Option<String>,
    pub initial_members: Vec<String>,
    pub welcomes_b64: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGroupResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostMessageRequest {
    pub ciphertext_b64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageEnvelope {
    pub id: i64,
    pub group_id: String,
    pub sender_user_id: String,
    pub ciphertext_b64: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageList {
    pub data: Vec<MessageEnvelope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WelcomeEnvelope {
    pub id: String,
    pub ciphertext_b64: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WelcomeList {
    pub data: Vec<WelcomeEnvelope>,
}

pub fn encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn decode(s: &str) -> Result<Vec<u8>, ApiError> {
    STANDARD.decode(s).map_err(|e| ApiError {
        status: 0,
        code: "BAD_BASE64".into(),
        message: e.to_string(),
    })
}

/// Publish a batch of KeyPackages for this user.
pub async fn publish_key_packages(
    packages: &[Vec<u8>],
    ttl_days: i64,
) -> Result<PublishKeyPackagesResponse, ApiError> {
    let body = PublishKeyPackagesRequest {
        key_packages: packages.iter().map(|p| encode(p)).collect(),
        ttl_days,
    };
    client::post::<PublishKeyPackagesRequest, PublishKeyPackagesResponse>(
        "/api/v1/mls/keypackages",
        &body,
    )
    .await
}

/// Atomically consume one available KeyPackage for `user_id` so we can add
/// them to a group. Returns the serialized KeyPackage bytes.
pub async fn consume_key_package(user_id: &str) -> Result<Vec<u8>, ApiError> {
    let bundle: KeyPackageBundle =
        client::post_no_body(&format!("/api/v1/mls/keypackages/{user_id}/consume")).await?;
    decode(&bundle.key_package_b64)
}

pub async fn list_my_groups() -> Result<GroupList, ApiError> {
    client::get("/api/v1/mls/groups").await
}

/// Leave a group. Server-side: removes my membership; if I was the last
/// member, cascades the group + its messages.
pub async fn leave_group(group_id: &str) -> Result<(), ApiError> {
    client::delete_req(&format!("/api/v1/mls/groups/{group_id}")).await
}

/// Delete every unconsumed KeyPackage for the authenticated user. Used on
/// client init to stop stale KPs from a previous session being consumed.
pub async fn purge_my_key_packages() -> Result<(), ApiError> {
    client::delete_req("/api/v1/mls/keypackages").await
}

pub async fn create_group(req: &CreateGroupRequest) -> Result<CreateGroupResponse, ApiError> {
    client::post("/api/v1/mls/groups", req).await
}

pub async fn post_message(group_id: &str, ciphertext: &[u8]) -> Result<(), ApiError> {
    let body = PostMessageRequest {
        ciphertext_b64: encode(ciphertext),
    };
    let _: serde_json::Value =
        client::post(&format!("/api/v1/mls/groups/{group_id}/messages"), &body).await?;
    Ok(())
}

pub async fn list_messages(group_id: &str, since: i64) -> Result<MessageList, ApiError> {
    client::get(&format!(
        "/api/v1/mls/groups/{group_id}/messages?since={since}"
    ))
    .await
}

pub async fn list_welcomes() -> Result<WelcomeList, ApiError> {
    client::get("/api/v1/mls/welcomes").await
}

pub async fn ack_welcome(id: &str) -> Result<(), ApiError> {
    client::delete_req(&format!("/api/v1/mls/welcomes/{id}")).await
}
