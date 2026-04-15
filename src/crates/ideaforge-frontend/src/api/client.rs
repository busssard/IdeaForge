use gloo_net::http::{Request, RequestBuilder, Response};
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::types::ApiErrorResponse;

const TOKEN_KEY: &str = "ideaforge_access_token";
const REFRESH_KEY: &str = "ideaforge_refresh_token";

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn get_token() -> Option<String> {
    LocalStorage::get::<String>(TOKEN_KEY).ok()
}

pub fn set_tokens(access: &str, refresh: &str) {
    let _ = LocalStorage::set(TOKEN_KEY, access);
    let _ = LocalStorage::set(REFRESH_KEY, refresh);
}

pub fn clear_tokens() {
    LocalStorage::delete(TOKEN_KEY);
    LocalStorage::delete(REFRESH_KEY);
}

pub fn get_refresh_token() -> Option<String> {
    LocalStorage::get::<String>(REFRESH_KEY).ok()
}

fn build_request(method: &str, url: &str) -> RequestBuilder {
    let req = match method {
        "GET" => Request::get(url),
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        _ => Request::get(url),
    };

    if let Some(token) = get_token() {
        req.header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
    } else {
        req.header("Content-Type", "application/json")
    }
}

async fn parse_error(resp: Response) -> ApiError {
    let status = resp.status();
    match resp.json::<ApiErrorResponse>().await {
        Ok(err) => ApiError {
            status,
            code: err.error.code,
            message: err.error.message,
        },
        Err(_) => ApiError {
            status,
            code: "UNKNOWN".into(),
            message: format!("Request failed with status {status}"),
        },
    }
}

/// Auth endpoints skip the 401-auto-refresh below (would cause infinite loops
/// on an expired refresh token).
fn is_auth_endpoint(url: &str) -> bool {
    url.starts_with("/api/v1/auth/")
}

/// Try a refresh_token roundtrip. Best-effort: returns true if tokens got
/// rotated. We avoid `auth::refresh()` here to keep the dependency graph
/// one-directional (`auth` depends on `client`, not vice versa).
async fn try_refresh_tokens() -> bool {
    let Some(refresh) = get_refresh_token() else {
        return false;
    };
    let body = serde_json::json!({ "refresh_token": refresh });
    let Ok(body_str) = serde_json::to_string(&body) else {
        return false;
    };
    let Ok(req) = Request::post("/api/v1/auth/refresh")
        .header("Content-Type", "application/json")
        .body(body_str)
    else {
        return false;
    };
    let Ok(resp) = req.send().await else {
        return false;
    };
    if !resp.ok() {
        return false;
    }
    let Ok(val) = resp.json::<serde_json::Value>().await else {
        return false;
    };
    let (Some(access), Some(refresh)) = (
        val.get("access_token").and_then(|v| v.as_str()),
        val.get("refresh_token").and_then(|v| v.as_str()),
    ) else {
        return false;
    };
    set_tokens(access, refresh);
    true
}

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let resp = build_request("GET", url)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK_ERROR".into(),
            message: e.to_string(),
        })?;

    // 401 on a non-auth endpoint → the access token is stale. Try a
    // refresh_token roundtrip and retry ONCE before surfacing the error.
    if resp.status() == 401 && !is_auth_endpoint(url) && try_refresh_tokens().await {
        let resp = build_request("GET", url)
            .send()
            .await
            .map_err(|e| ApiError {
                status: 0,
                code: "NETWORK_ERROR".into(),
                message: e.to_string(),
            })?;
        return if resp.ok() {
            resp.json::<T>().await.map_err(|e| ApiError {
                status: resp.status(),
                code: "PARSE_ERROR".into(),
                message: e.to_string(),
            })
        } else {
            Err(parse_error(resp).await)
        };
    }

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: resp.status(),
            code: "PARSE_ERROR".into(),
            message: e.to_string(),
        })
    } else {
        Err(parse_error(resp).await)
    }
}

async fn send_body<T: DeserializeOwned>(
    method: &str,
    url: &str,
    body_str: &str,
) -> Result<T, ApiError> {
    let resp = build_request(method, url)
        .body(body_str.to_string())
        .map_err(|e| ApiError {
            status: 0,
            code: "REQUEST_BUILD_ERROR".into(),
            message: format!("{e:?}"),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK_ERROR".into(),
            message: e.to_string(),
        })?;

    if resp.status() == 401 && !is_auth_endpoint(url) && try_refresh_tokens().await {
        let retry = build_request(method, url)
            .body(body_str.to_string())
            .map_err(|e| ApiError {
                status: 0,
                code: "REQUEST_BUILD_ERROR".into(),
                message: format!("{e:?}"),
            })?
            .send()
            .await
            .map_err(|e| ApiError {
                status: 0,
                code: "NETWORK_ERROR".into(),
                message: e.to_string(),
            })?;
        return if retry.ok() {
            retry.json::<T>().await.map_err(|e| ApiError {
                status: retry.status(),
                code: "PARSE_ERROR".into(),
                message: e.to_string(),
            })
        } else {
            Err(parse_error(retry).await)
        };
    }

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: resp.status(),
            code: "PARSE_ERROR".into(),
            message: e.to_string(),
        })
    } else {
        Err(parse_error(resp).await)
    }
}

pub async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, ApiError> {
    let body_str = serde_json::to_string(body).map_err(|e| ApiError {
        status: 0,
        code: "SERIALIZE_ERROR".into(),
        message: e.to_string(),
    })?;
    send_body("POST", url, &body_str).await
}

pub async fn post_no_body<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let resp = build_request("POST", url)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK_ERROR".into(),
            message: e.to_string(),
        })?;

    if resp.ok() {
        resp.json::<T>().await.map_err(|e| ApiError {
            status: resp.status(),
            code: "PARSE_ERROR".into(),
            message: e.to_string(),
        })
    } else {
        Err(parse_error(resp).await)
    }
}

pub async fn put<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, ApiError> {
    let body_str = serde_json::to_string(body).map_err(|e| ApiError {
        status: 0,
        code: "SERIALIZE_ERROR".into(),
        message: e.to_string(),
    })?;
    send_body("PUT", url, &body_str).await
}

pub async fn delete_req(url: &str) -> Result<(), ApiError> {
    let resp = build_request("DELETE", url)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK_ERROR".into(),
            message: e.to_string(),
        })?;

    if resp.ok() {
        Ok(())
    } else {
        Err(parse_error(resp).await)
    }
}
