use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::state::AppState;
use ideaforge_db::entities::user;
use ideaforge_db::repositories::bot_endorsement_repo::BotEndorsementRepository;
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::user_repo::UserRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_bot))
        .route("/", get(list_bots))
        .route("/:id/endorse/:idea_id", post(endorse_idea))
}

// --- Bot API Key Auth Extractor ---

/// Authenticated bot user extracted from X-Bot-Api-Key header.
/// Hashes the provided API key with SHA-256 and looks up the bot user.
#[derive(Debug, Clone)]
pub struct BotAuth {
    pub bot_user: user::Model,
}

pub struct BotAuthRejection {
    status: StatusCode,
    message: &'static str,
}

impl IntoResponse for BotAuthRejection {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({
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
impl FromRequestParts<AppState> for BotAuth {
    type Rejection = BotAuthRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the X-Bot-Api-Key header
        let api_key = parts
            .headers
            .get("X-Bot-Api-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or(BotAuthRejection {
                status: StatusCode::UNAUTHORIZED,
                message: "Missing X-Bot-Api-Key header",
            })?;

        if api_key.is_empty() {
            return Err(BotAuthRejection {
                status: StatusCode::UNAUTHORIZED,
                message: "Empty API key",
            });
        }

        // SHA-256 hash the provided key
        let key_hash = hash_api_key(api_key);

        // Look up bot user by hash
        let repo = UserRepository::new(state.db.connection());
        let bot_user = repo
            .find_bot_by_api_key_hash(&key_hash)
            .await
            .map_err(|_| BotAuthRejection {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Database error during authentication",
            })?
            .ok_or(BotAuthRejection {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid API key",
            })?;

        Ok(BotAuth { bot_user })
    }
}

// --- Helpers ---

/// Hash an API key using SHA-256, returning the hex-encoded digest.
fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a random API key: 32 random bytes, hex-encoded (64 chars).
fn generate_api_key() -> String {
    let key: [u8; 32] = rand::random();
    hex::encode(key)
}

fn err(status: StatusCode, code: &str, message: &str) -> impl IntoResponse {
    (
        status,
        Json(serde_json::json!({
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct RegisterBotRequest {
    pub username: String,
    pub email: String,
    pub operator: String,
    pub description: String,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct RegisterBotResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub operator: String,
    pub description: String,
    pub api_key: String, // Only returned once at registration
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct BotProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub operator: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct BotListResponse {
    pub data: Vec<BotProfileResponse>,
}

#[derive(Debug, Deserialize)]
pub struct EndorseRequest {
    pub reasoning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EndorsementResponse {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub idea_id: Uuid,
    pub reason: String,
    pub created_at: String,
}

// --- Handlers ---

/// POST /api/v1/bots/register
///
/// Register a new bot account. For now, this is a public endpoint
/// (in production, it would require an admin API key or registration token).
async fn register_bot(
    State(state): State<AppState>,
    Json(body): Json<RegisterBotRequest>,
) -> impl IntoResponse {
    // Validate username
    let username = body.username.trim();
    if username.is_empty() || username.len() > 100 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Username is required (max 100 chars)",
        )
        .into_response();
    }

    // Validate email
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Valid email is required",
        )
        .into_response();
    }

    // Validate operator
    let operator = body.operator.trim();
    if operator.is_empty() || operator.len() > 200 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Operator name is required (max 200 chars)",
        )
        .into_response();
    }

    // Validate description
    let description = body.description.trim();
    if description.is_empty() || description.len() > 2000 {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Description is required (max 2000 chars)",
        )
        .into_response();
    }

    let repo = UserRepository::new(state.db.connection());

    // Check if email already taken
    match repo.find_by_email(&email).await {
        Ok(Some(_)) => {
            return err(
                StatusCode::CONFLICT,
                "CONFLICT",
                "Email already registered",
            )
            .into_response();
        }
        Err(e) => {
            tracing::error!("Database error checking email: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(None) => {}
    }

    // Generate API key and hash it
    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&api_key);

    // Create bot user
    let bot_id = Uuid::new_v4();
    match repo
        .create_bot(
            bot_id,
            &email,
            username,
            operator,
            description,
            &api_key_hash,
        )
        .await
    {
        Ok(bot) => (
            StatusCode::CREATED,
            Json(RegisterBotResponse {
                id: bot.id,
                username: bot.display_name,
                email: bot.email,
                operator: operator.to_string(),
                description: description.to_string(),
                api_key, // Only returned once!
                created_at: bot.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create bot account: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// GET /api/v1/bots
///
/// List all registered bot accounts (public, no auth required).
async fn list_bots(State(state): State<AppState>) -> impl IntoResponse {
    let repo = UserRepository::new(state.db.connection());
    match repo.list_bots().await {
        Ok(bots) => Json(BotListResponse {
            data: bots
                .iter()
                .map(|b| BotProfileResponse {
                    id: b.id,
                    username: b.display_name.clone(),
                    operator: b.bot_operator.clone(),
                    description: b.bot_description.clone(),
                    created_at: b.created_at.to_rfc3339(),
                })
                .collect(),
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list bots: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

/// POST /api/v1/bots/:id/endorse/:idea_id
///
/// Bot endorses an idea. Requires bot API key auth via X-Bot-Api-Key header.
/// Bot endorsements are separate from human stokes.
async fn endorse_idea(
    State(state): State<AppState>,
    bot_auth: BotAuth,
    Path((bot_id, idea_id)): Path<(Uuid, Uuid)>,
    body: Option<Json<EndorseRequest>>,
) -> impl IntoResponse {
    // Verify the authenticated bot matches the path bot ID
    if bot_auth.bot_user.id != bot_id {
        return err(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "API key does not match the specified bot ID",
        )
        .into_response();
    }

    // Verify idea exists
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(idea_id).await {
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea for endorsement: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(Some(_)) => {}
    }

    let endorsement_repo = BotEndorsementRepository::new(state.db.connection());

    // Check if already endorsed
    match endorsement_repo.exists(bot_id, idea_id).await {
        Ok(true) => {
            return err(
                StatusCode::CONFLICT,
                "CONFLICT",
                "Bot has already endorsed this idea",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check endorsement existence: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    let reasoning = body
        .and_then(|b| b.reasoning.clone())
        .unwrap_or_default();

    // Create endorsement
    match endorsement_repo
        .create(Uuid::new_v4(), bot_id, idea_id, &reasoning)
        .await
    {
        Ok(endorsement) => (
            StatusCode::CREATED,
            Json(EndorsementResponse {
                id: endorsement.id,
                bot_id: endorsement.bot_id,
                idea_id: endorsement.idea_id,
                reason: endorsement.reason,
                created_at: endorsement.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create bot endorsement: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
