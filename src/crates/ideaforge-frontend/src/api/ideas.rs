use super::client;
use super::types::{CreateIdeaRequest, IdeaListResponse, IdeaResponse, UpdateIdeaRequest};

pub async fn list_ideas(
    page: u64,
    per_page: u64,
    category: Option<&str>,
    maturity: Option<&str>,
    sort: Option<&str>,
) -> Result<IdeaListResponse, client::ApiError> {
    let mut url = format!("/api/v1/ideas?page={page}&per_page={per_page}");
    if let Some(c) = category {
        if !c.is_empty() {
            url.push_str(&format!("&category_id={c}"));
        }
    }
    if let Some(m) = maturity {
        if !m.is_empty() {
            url.push_str(&format!("&maturity={m}"));
        }
    }
    if let Some(s) = sort {
        if !s.is_empty() {
            url.push_str(&format!("&sort={s}"));
        }
    }
    client::get(&url).await
}

pub async fn get_idea(id: &str) -> Result<IdeaResponse, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{id}")).await
}

pub async fn create_idea(req: CreateIdeaRequest) -> Result<IdeaResponse, client::ApiError> {
    client::post("/api/v1/ideas", &req).await
}

pub async fn update_idea(
    id: &str,
    req: UpdateIdeaRequest,
) -> Result<IdeaResponse, client::ApiError> {
    client::put(&format!("/api/v1/ideas/{id}"), &req).await
}

pub async fn archive_idea(id: &str) -> Result<(), client::ApiError> {
    client::delete_req(&format!("/api/v1/ideas/{id}")).await
}

pub async fn list_my_stoked_ideas(
    page: u64,
    per_page: u64,
) -> Result<IdeaListResponse, client::ApiError> {
    client::get(&format!(
        "/api/v1/ideas/my-stokes?page={page}&per_page={per_page}"
    ))
    .await
}
