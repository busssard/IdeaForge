use serde::Deserialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::client;
use super::types::{
    IdeaListResponse, PublicUserResponse, UpdateMeRequest, UserListResponse, UserResponse,
};

#[derive(Debug, Clone, Deserialize)]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

/// Upload an avatar image. Uses the browser's native `FormData` + `fetch`
/// because gloo-net doesn't expose multipart ergonomically. Accepts any Blob
/// (including File) — the avatar cropper produces a compressed JPEG Blob that
/// we pass through unchanged.
pub async fn upload_avatar(blob: &web_sys::Blob) -> Result<AvatarUploadResponse, client::ApiError> {
    let form = web_sys::FormData::new().map_err(|e| client::ApiError {
        status: 0,
        code: "FORM_DATA".into(),
        message: format!("Failed to create FormData: {:?}", e),
    })?;
    form.append_with_blob_and_filename("avatar", blob, "avatar.jpg")
        .map_err(|e| client::ApiError {
            status: 0,
            code: "FORM_DATA".into(),
            message: format!("Failed to attach file: {:?}", e),
        })?;

    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&form);

    let headers = web_sys::Headers::new().map_err(|e| client::ApiError {
        status: 0,
        code: "HEADERS".into(),
        message: format!("{:?}", e),
    })?;
    if let Some(token) = client::get_token() {
        headers
            .append("Authorization", &format!("Bearer {token}"))
            .map_err(|e| client::ApiError {
                status: 0,
                code: "HEADERS".into(),
                message: format!("{:?}", e),
            })?;
    }
    // Important: do NOT set Content-Type — the browser has to supply the
    // boundary for multipart/form-data, and setting it manually breaks it.
    init.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init("/api/v1/users/me/avatar", &init)
        .map_err(|e| client::ApiError {
            status: 0,
            code: "REQUEST_BUILD_ERROR".into(),
            message: format!("{:?}", e),
        })?;

    let window = web_sys::window().ok_or_else(|| client::ApiError {
        status: 0,
        code: "NO_WINDOW".into(),
        message: "No window".into(),
    })?;
    let resp_js = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| client::ApiError {
            status: 0,
            code: "NETWORK_ERROR".into(),
            message: format!("{:?}", e),
        })?;
    let resp: web_sys::Response = resp_js.dyn_into().map_err(|_| client::ApiError {
        status: 0,
        code: "RESPONSE_CAST".into(),
        message: "Expected a Response".into(),
    })?;

    let status = resp.status();
    let text_promise = resp.text().map_err(|e| client::ApiError {
        status,
        code: "BODY_READ".into(),
        message: format!("{:?}", e),
    })?;
    let text_js = JsFuture::from(text_promise)
        .await
        .map_err(|e| client::ApiError {
            status,
            code: "BODY_READ".into(),
            message: format!("{:?}", e),
        })?;
    let text = text_js.as_string().unwrap_or_default();

    if !resp.ok() {
        // Try to parse a structured error body; fall back to the raw text.
        let (code, message) = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| {
                let err = v.get("error")?;
                Some((
                    err.get("code")
                        .and_then(|c| c.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_string(),
                    err.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or(&text)
                        .to_string(),
                ))
            })
            .unwrap_or_else(|| ("UPLOAD_FAILED".into(), text.clone()));
        return Err(client::ApiError {
            status,
            code,
            message,
        });
    }

    serde_json::from_str::<AvatarUploadResponse>(&text).map_err(|e| client::ApiError {
        status,
        code: "PARSE_ERROR".into(),
        message: e.to_string(),
    })
}

pub async fn get_me() -> Result<UserResponse, client::ApiError> {
    client::get("/api/v1/users/me").await
}

pub async fn update_me(req: UpdateMeRequest) -> Result<UserResponse, client::ApiError> {
    client::put("/api/v1/users/me", &req).await
}

pub async fn get_user(id: &str) -> Result<PublicUserResponse, client::ApiError> {
    client::get(&format!("/api/v1/users/{id}")).await
}

pub async fn get_user_authored_ideas(
    id: &str,
    page: u64,
    per_page: u64,
) -> Result<IdeaListResponse, client::ApiError> {
    client::get(&format!(
        "/api/v1/users/{id}/ideas?page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn get_user_contributions(
    id: &str,
    page: u64,
    per_page: u64,
) -> Result<IdeaListResponse, client::ApiError> {
    client::get(&format!(
        "/api/v1/users/{id}/contributions?page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn get_user_stoked_ideas(
    id: &str,
    page: u64,
    per_page: u64,
) -> Result<IdeaListResponse, client::ApiError> {
    client::get(&format!(
        "/api/v1/users/{id}/stokes?page={page}&per_page={per_page}"
    ))
    .await
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
