use gloo_net::http::{Request, RequestBuilder, Response};
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::Serialize;

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

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, ApiError> {
    let resp = build_request("GET", url)
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

pub async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, ApiError> {
    let body_str = serde_json::to_string(body).map_err(|e| ApiError {
        status: 0,
        code: "SERIALIZE_ERROR".into(),
        message: e.to_string(),
    })?;

    let resp = build_request("POST", url)
        .body(body_str)
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

    let resp = build_request("PUT", url)
        .body(body_str)
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
