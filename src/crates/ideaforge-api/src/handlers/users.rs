use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::{AuthUser, OptionalAuth};
use crate::state::AppState;
use ideaforge_db::entities::enums::UserRole;
use ideaforge_db::repositories::user_repo::UserRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(get_me).put(update_me))
        .route("/me/avatar", post(upload_avatar))
        .route("/:id", get(get_user))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub role: Option<String>,
    pub skills: Option<String>, // comma-separated
    pub sort: Option<String>,
    pub include_bots: Option<bool>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub skills: serde_json::Value,
    pub looking_for: Option<String>,
    pub availability: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PublicUserResponse {
    pub id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub skills: serde_json::Value,
    pub looking_for: Option<String>,
    pub availability: Option<String>,
    pub idea_count: u64,
    pub stoke_count: u64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub data: Vec<PublicUserResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub skills: Option<Vec<String>>,
    pub looking_for: Option<Option<String>>,
    pub availability: Option<String>,
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

fn parse_user_role(s: &str) -> Option<UserRole> {
    match s {
        "entrepreneur" => Some(UserRole::Entrepreneur),
        "maker" => Some(UserRole::Maker),
        "curious" => Some(UserRole::Curious),
        "admin" => Some(UserRole::Admin),
        _ => None,
    }
}

async fn list_users(
    State(state): State<AppState>,
    _opt_auth: OptionalAuth,
    Query(params): Query<ListUsersQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let sort = params.sort.as_deref().unwrap_or("recently_joined");
    let include_bots = params.include_bots.unwrap_or(false);

    // Parse role filter
    let role_filter = params.role.as_deref().and_then(parse_user_role);

    // Parse skills filter (comma-separated)
    let skills_filter = params.skills.as_deref().map(|s| {
        s.split(',')
            .map(|skill| skill.trim().to_string())
            .filter(|skill| !skill.is_empty())
            .collect::<Vec<_>>()
    });

    let repo = UserRepository::new(state.db.connection());
    match repo
        .list(
            role_filter,
            skills_filter,
            include_bots,
            sort,
            page,
            per_page,
        )
        .await
    {
        Ok((users, total)) => {
            let total_pages = if total == 0 {
                0
            } else {
                total.div_ceil(per_page)
            };
            Json(UserListResponse {
                data: users
                    .iter()
                    .map(|u| PublicUserResponse {
                        id: u.user.id,
                        display_name: u.user.display_name.clone(),
                        bio: u.user.bio.clone(),
                        avatar_url: u.user.avatar_url.clone(),
                        role: u.user.role.to_string(),
                        skills: u.user.skills.clone(),
                        looking_for: u.user.looking_for.clone(),
                        availability: u.user.availability.clone(),
                        idea_count: u.idea_count,
                        stoke_count: u.stoke_count,
                        created_at: u.user.created_at.to_rfc3339(),
                    })
                    .collect(),
                meta: PaginationMeta {
                    total,
                    page,
                    per_page,
                    total_pages,
                },
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list users: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_me(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    let repo = UserRepository::new(state.db.connection());
    match repo.find_by_id(auth.user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            role: user.role.to_string(),
            skills: user.skills,
            looking_for: user.looking_for,
            availability: user.availability,
            created_at: user.created_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "User not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateMeRequest>,
) -> impl IntoResponse {
    // Validate
    if let Some(ref name) = body.display_name
        && (name.trim().is_empty() || name.len() > 100)
    {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Display name must be 1-100 chars",
        )
        .into_response();
    }
    if let Some(ref bio) = body.bio
        && bio.len() > 2000
    {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Bio too long (max 2000 chars)",
        )
        .into_response();
    }

    // Validate skills
    let skills_json = if let Some(ref skills) = body.skills {
        if skills.len() > 10 {
            return err(
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Maximum 10 skills allowed",
            )
            .into_response();
        }
        Some(serde_json::json!(skills))
    } else {
        None
    };

    // Validate looking_for
    if let Some(Some(ref lf)) = body.looking_for
        && lf.len() > 500
    {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Looking for text too long (max 500 chars)",
        )
        .into_response();
    }

    // Validate availability
    if let Some(ref av) = body.availability
        && av.len() > 100
    {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Availability text too long (max 100 chars)",
        )
        .into_response();
    }

    let repo = UserRepository::new(state.db.connection());
    match repo
        .update(
            auth.user_id,
            body.display_name.as_deref(),
            body.bio.as_deref(),
            body.avatar_url.as_ref().map(|opt| opt.as_deref()),
            skills_json.as_ref(),
            body.looking_for.as_ref().map(|opt| opt.as_deref()),
            body.availability.as_deref(),
        )
        .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            bio: user.bio,
            avatar_url: user.avatar_url,
            role: user.role.to_string(),
            skills: user.skills,
            looking_for: user.looking_for,
            availability: user.availability,
            created_at: user.created_at.to_rfc3339(),
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to update user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_user(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    use ideaforge_db::entities::{idea, stoke};
    use sea_orm::*;

    let repo = UserRepository::new(state.db.connection());
    match repo.find_by_id(id).await {
        Ok(Some(user)) => {
            // Fetch stats
            let idea_count = idea::Entity::find()
                .filter(idea::Column::AuthorId.eq(id))
                .filter(idea::Column::ArchivedAt.is_null())
                .count(state.db.connection())
                .await
                .unwrap_or(0);

            let stoke_count = stoke::Entity::find()
                .filter(stoke::Column::UserId.eq(id))
                .count(state.db.connection())
                .await
                .unwrap_or(0);

            Json(PublicUserResponse {
                id: user.id,
                display_name: user.display_name,
                bio: user.bio,
                avatar_url: user.avatar_url,
                role: user.role.to_string(),
                skills: user.skills,
                looking_for: user.looking_for,
                availability: user.availability,
                idea_count,
                stoke_count,
                created_at: user.created_at.to_rfc3339(),
            })
            .into_response()
        }
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "User not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get user: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

const AVATAR_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Minimal magic-byte sniffing so a caller can't POST an HTML payload labelled
/// as `image/png` and get it served back via `/uploads/...`.
fn sniff_image_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else {
        None
    }
}

async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut bytes: Option<Vec<u8>> = None;
    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                if field.name() == Some("avatar") {
                    match field.bytes().await {
                        Ok(b) => {
                            bytes = Some(b.to_vec());
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to read avatar field bytes: {e}");
                            return err(
                                StatusCode::BAD_REQUEST,
                                "BAD_UPLOAD",
                                "Failed to read uploaded file",
                            )
                            .into_response();
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::warn!("Multipart parse error: {e}");
                return err(
                    StatusCode::BAD_REQUEST,
                    "BAD_UPLOAD",
                    "Invalid multipart body",
                )
                .into_response();
            }
        }
    }

    let Some(bytes) = bytes else {
        return err(
            StatusCode::BAD_REQUEST,
            "MISSING_FIELD",
            "Expected a file in the `avatar` field",
        )
        .into_response();
    };

    if bytes.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "EMPTY_FILE",
            "Uploaded file is empty",
        )
        .into_response();
    }

    if bytes.len() > AVATAR_MAX_BYTES {
        return err(
            StatusCode::PAYLOAD_TOO_LARGE,
            "FILE_TOO_LARGE",
            "Avatar must be under 5 MB",
        )
        .into_response();
    }

    let Some(ext) = sniff_image_ext(&bytes) else {
        return err(
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_FORMAT",
            "Avatar must be PNG, JPEG, WebP, or GIF",
        )
        .into_response();
    };

    // Run blocking fs work off the reactor so slow disks don't stall requests.
    let upload_dir = std::path::PathBuf::from("uploads/avatars");
    let user_id = auth.user_id;
    let new_filename = format!("{user_id}.{ext}");
    let write_filename = new_filename.clone();
    let write_result = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        std::fs::create_dir_all(&upload_dir)?;
        for old_ext in ["png", "jpg", "jpeg", "webp", "gif"] {
            if old_ext != ext {
                let _ = std::fs::remove_file(upload_dir.join(format!("{user_id}.{old_ext}")));
            }
        }
        std::fs::write(upload_dir.join(&write_filename), &bytes)
    })
    .await;

    let write_outcome = match write_result {
        Ok(r) => r,
        Err(e) => Err(std::io::Error::other(e)),
    };
    if let Err(e) = write_outcome {
        tracing::error!("Failed to write avatar to disk: {e}");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "WRITE_FAILED",
            "Failed to save avatar",
        )
        .into_response();
    }

    let avatar_url = format!(
        "/uploads/avatars/{new_filename}?v={}",
        chrono::Utc::now().timestamp_millis()
    );

    let repo = UserRepository::new(state.db.connection());
    match repo
        .update(
            auth.user_id,
            None,
            None,
            Some(Some(&avatar_url)),
            None,
            None,
            None,
        )
        .await
    {
        Ok(_) => Json(serde_json::json!({ "avatar_url": avatar_url })).into_response(),
        Err(e) => {
            tracing::error!("Failed to persist avatar_url: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Failed to save avatar URL",
            )
            .into_response()
        }
    }
}
