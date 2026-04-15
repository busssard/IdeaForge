use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use uuid::Uuid;

/// Authenticated user extracted from Bearer token.
/// Use this in handlers that require authentication.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

/// Rejection type for auth extraction failures.
pub struct AuthRejection {
    message: &'static str,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": self.message,
                }
            })),
        )
            .into_response()
    }
}

#[async_trait]
impl FromRequestParts<crate::state::AppState> for AuthUser {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts).ok_or(AuthRejection {
            message: "Missing or invalid Authorization header",
        })?;

        let claims = state
            .jwt
            .validate_token(&token)
            .map_err(|_| AuthRejection {
                message: "Invalid or expired token",
            })?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
        })
    }
}

/// Optional authentication — returns `Some(AuthUser)` if a valid token
/// is present, `None` otherwise. Never rejects — invalid/expired tokens
/// are silently treated as unauthenticated so public endpoints keep working.
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

#[async_trait]
impl FromRequestParts<crate::state::AppState> for OptionalAuth {
    type Rejection = AuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = match extract_bearer_token(parts) {
            Some(t) => t,
            None => return Ok(OptionalAuth(None)),
        };

        match state.jwt.validate_token(&token) {
            Ok(claims) => Ok(OptionalAuth(Some(AuthUser {
                user_id: claims.sub,
                email: claims.email,
                role: claims.role,
            }))),
            Err(_) => Ok(OptionalAuth(None)),
        }
    }
}

fn extract_bearer_token(parts: &Parts) -> Option<String> {
    let header = parts.headers.get(AUTHORIZATION)?;
    let value = header.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}
