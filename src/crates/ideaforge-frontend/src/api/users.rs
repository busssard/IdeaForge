use super::client;
use super::types::{PublicUserResponse, UpdateMeRequest, UserListResponse, UserResponse};

pub async fn get_me() -> Result<UserResponse, client::ApiError> {
    client::get("/api/v1/users/me").await
}

pub async fn update_me(req: UpdateMeRequest) -> Result<UserResponse, client::ApiError> {
    client::put("/api/v1/users/me", &req).await
}

pub async fn get_user(id: &str) -> Result<PublicUserResponse, client::ApiError> {
    client::get(&format!("/api/v1/users/{id}")).await
}

pub async fn list_users(
    page: u64,
    per_page: u64,
    role: Option<&str>,
    skills: Option<&str>,
    sort: Option<&str>,
) -> Result<UserListResponse, client::ApiError> {
    let mut url = format!("/api/v1/users?page={page}&per_page={per_page}");
    if let Some(r) = role {
        url.push_str(&format!("&role={r}"));
    }
    if let Some(s) = skills {
        url.push_str(&format!("&skills={s}"));
    }
    if let Some(s) = sort {
        url.push_str(&format!("&sort={s}"));
    }
    client::get(&url).await
}
