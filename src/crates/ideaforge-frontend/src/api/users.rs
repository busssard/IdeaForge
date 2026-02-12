use super::client;
use super::types::{PublicUserResponse, UpdateMeRequest, UserResponse};

pub async fn get_me() -> Result<UserResponse, client::ApiError> {
    client::get("/api/v1/users/me").await
}

pub async fn update_me(req: UpdateMeRequest) -> Result<UserResponse, client::ApiError> {
    client::put("/api/v1/users/me", &req).await
}

pub async fn get_user(id: &str) -> Result<PublicUserResponse, client::ApiError> {
    client::get(&format!("/api/v1/users/{id}")).await
}
