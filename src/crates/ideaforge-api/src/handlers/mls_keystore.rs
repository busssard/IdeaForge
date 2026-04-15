//! MLS keystore backup — password-wrapped blob persistence.
//!
//! Users pick a 6-digit PIN (client-side). The client derives two independent
//! keys from the PIN via Argon2id:
//!   - **verifier**: shipped to the server so it can gate unlock attempts
//!   - **wrap_key**: never leaves the client; used to AES-GCM-wrap the
//!     serialized OpenMLS state
//!
//! The server stores `{ salt, verifier, wrapped_blob }`. It never sees the
//! PIN or the wrap_key or the plaintext MLS state. A 6-digit PIN is
//! brute-forceable offline — mitigated by aggressive rate limiting on
//! `unlock` (max 3 failures per hour → 1-hour lockout) plus Argon2id
//! making per-guess cost non-trivial even if the DB leaks.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};

use crate::extractors::AuthUser;
use crate::state::AppState;
use ideaforge_db::entities::mls_keystore;

const MAX_FAILED_ATTEMPTS: i32 = 3;
const LOCKOUT_WINDOW_MINUTES: i64 = 60;
const LOCKOUT_DURATION_MINUTES: i64 = 60;
const MAX_BLOB_BYTES: usize = 10 * 1024 * 1024; // 10 MiB — generous for MLS state

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(setup_keystore).put(update_keystore))
        .route("/status", get(keystore_status))
        .route("/unlock", post(unlock_keystore))
}

fn b64(bytes: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(bytes)
}

fn unb64(s: &str) -> Result<Vec<u8>, (StatusCode, Json<serde_json::Value>)> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(s).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": { "code": "BAD_BASE64", "message": "Invalid base64" } })),
        )
    })
}

fn err(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": { "code": code, "message": message } })),
    )
}

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    /// Base64 per-user random salt (>= 16 bytes). Client generates, sends once.
    pub salt_b64: String,
    /// Base64 Argon2id-derived verifier (>= 32 bytes).
    pub verifier_b64: String,
    /// Base64 AES-GCM-wrapped MLS keystore blob.
    pub wrapped_blob_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    /// Client must prove it knows the PIN with a fresh verifier.
    pub verifier_b64: String,
    pub wrapped_blob_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    pub verifier_b64: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub exists: bool,
    /// Present only if a keystore exists. The client needs this to re-derive
    /// the same wrap_key from the PIN.
    pub salt_b64: Option<String>,
    /// Unix millis. If non-null and in the future, all unlock attempts are
    /// rejected until then.
    pub locked_until_ms: Option<i64>,
    /// How many failed attempts have counted toward the current lockout
    /// window. Surfaced so the UI can show "2 attempts left".
    pub failed_attempts: i32,
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub wrapped_blob_b64: String,
    pub salt_b64: String,
}

async fn setup_keystore(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<SetupRequest>,
) -> impl IntoResponse {
    let salt = match unb64(&body.salt_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let verifier = match unb64(&body.verifier_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let blob = match unb64(&body.wrapped_blob_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };

    if salt.len() < 16 || verifier.len() < 32 {
        return err(
            StatusCode::BAD_REQUEST,
            "WEAK_PARAMS",
            "salt must be >=16 bytes, verifier >=32 bytes",
        )
        .into_response();
    }
    if blob.is_empty() || blob.len() > MAX_BLOB_BYTES {
        return err(
            StatusCode::BAD_REQUEST,
            "BAD_BLOB",
            "blob out of allowed size range",
        )
        .into_response();
    }

    let db = state.db.connection();
    let now = Utc::now().fixed_offset();

    // If a keystore already exists we refuse — the client should use PUT
    // after a successful unlock to change the PIN.
    match mls_keystore::Entity::find_by_id(auth.user_id).one(db).await {
        Ok(Some(_)) => {
            return err(
                StatusCode::CONFLICT,
                "ALREADY_EXISTS",
                "Keystore already set. Use PUT to rotate after unlocking.",
            )
            .into_response();
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("keystore lookup failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Lookup failed",
            )
            .into_response();
        }
    }

    let model = mls_keystore::ActiveModel {
        user_id: Set(auth.user_id),
        salt: Set(salt),
        verifier: Set(verifier),
        wrapped_blob: Set(blob),
        created_at: Set(now),
        updated_at: Set(now),
        failed_attempts: Set(0),
        first_failed_at: Set(None),
        locked_until: Set(None),
    };
    match model.insert(db).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => {
            tracing::error!("keystore insert failed: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Insert failed",
            )
            .into_response()
        }
    }
}

async fn update_keystore(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateRequest>,
) -> impl IntoResponse {
    let verifier = match unb64(&body.verifier_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let blob = match unb64(&body.wrapped_blob_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    if blob.is_empty() || blob.len() > MAX_BLOB_BYTES {
        return err(
            StatusCode::BAD_REQUEST,
            "BAD_BLOB",
            "blob out of allowed size range",
        )
        .into_response();
    }

    let db = state.db.connection();
    let row = match mls_keystore::Entity::find_by_id(auth.user_id).one(db).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "No keystore to update — use POST to set up first",
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!("keystore lookup failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Lookup failed",
            )
            .into_response();
        }
    };

    if !constant_time_eq(&row.verifier, &verifier) {
        return err(StatusCode::FORBIDDEN, "BAD_PIN", "Verifier does not match").into_response();
    }

    let mut active: mls_keystore::ActiveModel = row.into();
    active.wrapped_blob = Set(blob);
    active.updated_at = Set(Utc::now().fixed_offset());
    match active.update(db).await {
        // Return a small JSON body — the frontend helper parses responses
        // as JSON, which makes 204 No Content trip an EOF-while-parsing.
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("keystore update failed: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Update failed",
            )
            .into_response()
        }
    }
}

async fn keystore_status(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    let db = state.db.connection();
    match mls_keystore::Entity::find_by_id(auth.user_id).one(db).await {
        Ok(Some(row)) => Json(StatusResponse {
            exists: true,
            salt_b64: Some(b64(&row.salt)),
            locked_until_ms: row.locked_until.map(|t| t.to_utc().timestamp_millis()),
            failed_attempts: row.failed_attempts,
        })
        .into_response(),
        Ok(None) => Json(StatusResponse {
            exists: false,
            salt_b64: None,
            locked_until_ms: None,
            failed_attempts: 0,
        })
        .into_response(),
        Err(e) => {
            tracing::error!("keystore status: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Lookup failed",
            )
            .into_response()
        }
    }
}

async fn unlock_keystore(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UnlockRequest>,
) -> impl IntoResponse {
    let verifier = match unb64(&body.verifier_b64) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };

    let db = state.db.connection();
    let row = match mls_keystore::Entity::find_by_id(auth.user_id).one(db).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NO_KEYSTORE", "No keystore set up").into_response();
        }
        Err(e) => {
            tracing::error!("keystore lookup failed: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DB_ERROR",
                "Lookup failed",
            )
            .into_response();
        }
    };

    let now = Utc::now();

    // Hard lock first — refuse immediately if in lockout.
    if let Some(locked_until) = row.locked_until {
        if locked_until.to_utc() > now {
            return locked_response(locked_until, row.failed_attempts).into_response();
        }
    }

    if constant_time_eq(&row.verifier, &verifier) {
        // Correct PIN — reset attempt counters atomically.
        let mut active: mls_keystore::ActiveModel = row.clone().into();
        active.failed_attempts = Set(0);
        active.first_failed_at = Set(None);
        active.locked_until = Set(None);
        if let Err(e) = active.update(db).await {
            tracing::warn!("failed to reset lockout counters (ok): {e}");
        }

        return Json(UnlockResponse {
            wrapped_blob_b64: b64(&row.wrapped_blob),
            salt_b64: b64(&row.salt),
        })
        .into_response();
    }

    // Wrong PIN — update rate-limit state.
    let window_start = row.first_failed_at.map(|t| t.to_utc()).unwrap_or(now);
    let window_active =
        now.signed_duration_since(window_start) <= Duration::minutes(LOCKOUT_WINDOW_MINUTES);

    let mut active: mls_keystore::ActiveModel = row.clone().into();
    let (new_attempts, new_first_failed, new_locked) = if window_active {
        let attempts = row.failed_attempts + 1;
        let first = Some(window_start.fixed_offset());
        let locked = if attempts >= MAX_FAILED_ATTEMPTS {
            Some((now + Duration::minutes(LOCKOUT_DURATION_MINUTES)).fixed_offset())
        } else {
            None
        };
        (attempts, first, locked)
    } else {
        // Stale window → treat this as attempt 1 of a new window.
        (1, Some(now.fixed_offset()), None)
    };
    active.failed_attempts = Set(new_attempts);
    active.first_failed_at = Set(new_first_failed);
    active.locked_until = Set(new_locked);
    let _ = active.update(db).await;

    if let Some(until) = new_locked {
        return locked_response(until, new_attempts).into_response();
    }

    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": {
                "code": "BAD_PIN",
                "message": "Wrong PIN.",
                "attempts_used": new_attempts,
                "attempts_remaining": (MAX_FAILED_ATTEMPTS - new_attempts).max(0),
            }
        })),
    )
        .into_response()
}

fn locked_response(
    locked_until: DateTime<FixedOffset>,
    attempts: i32,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": {
                "code": "LOCKED",
                "message": "Too many failed PIN attempts. Try again later.",
                "locked_until_ms": locked_until.to_utc().timestamp_millis(),
                "attempts_used": attempts,
            }
        })),
    )
}

/// Variable-time `==` leaks timing. The verifier is a ~32-byte Argon2id
/// output, so timing differences are small, but this is the kind of thing
/// that's cheap to get right once.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
