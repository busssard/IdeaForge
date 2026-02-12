use super::client;
use super::types::{LoginRequest, RefreshRequest, RegisterRequest, TokenResponse};

pub async fn login(email: String, password: String) -> Result<TokenResponse, client::ApiError> {
    let body = LoginRequest { email, password };
    let resp: TokenResponse = client::post("/api/v1/auth/login", &body).await?;
    client::set_tokens(&resp.access_token, &resp.refresh_token);
    Ok(resp)
}

pub async fn register(
    email: String,
    password: String,
    display_name: String,
    role: Option<String>,
) -> Result<TokenResponse, client::ApiError> {
    let body = RegisterRequest {
        email,
        password,
        display_name,
        role,
    };
    let resp: TokenResponse = client::post("/api/v1/auth/register", &body).await?;
    client::set_tokens(&resp.access_token, &resp.refresh_token);
    Ok(resp)
}

pub async fn refresh() -> Result<TokenResponse, client::ApiError> {
    let refresh_token = client::get_refresh_token().ok_or(client::ApiError {
        status: 0,
        code: "NO_REFRESH_TOKEN".into(),
        message: "No refresh token stored".into(),
    })?;
    let body = RefreshRequest { refresh_token };
    let resp: TokenResponse = client::post("/api/v1/auth/refresh", &body).await?;
    client::set_tokens(&resp.access_token, &resp.refresh_token);
    Ok(resp)
}
