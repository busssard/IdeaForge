use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::{AuthUser, OptionalAuth};
use crate::state::AppState;
use ideaforge_db::entities::enums::IdeaOpenness;
use ideaforge_db::repositories::idea_repo::IdeaRepository;
use ideaforge_db::repositories::nda_repo::{NdaSignatureRepository, NdaTemplateRepository};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id/nda", get(get_nda_template).post(create_nda_template))
        .route("/:id/nda/sign", axum::routing::post(sign_nda))
        .route("/:id/nda/status", get(check_nda_status))
        .route("/:id/nda/signatures", get(list_nda_signatures))
}

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateNdaTemplateRequest {
    pub title: Option<String>,
    pub body: String,
    pub confidentiality_period_days: Option<i32>,
    pub jurisdiction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignNdaRequest {
    pub signer_name: String,
}

#[derive(Debug, Serialize)]
pub struct NdaTemplateResponse {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub title: String,
    pub body: String,
    pub confidentiality_period_days: i32,
    pub jurisdiction: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NdaStatusResponse {
    pub has_signed: bool,
    pub signed_at: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NdaSignatureResponse {
    pub id: Uuid,
    pub signer_id: Uuid,
    pub signer_name: String,
    pub signed_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NdaSignatureListResponse {
    pub data: Vec<NdaSignatureResponse>,
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
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
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

// --- Handlers ---

async fn get_nda_template(
    State(state): State<AppState>,
    _opt_auth: OptionalAuth,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Verify idea exists and is NDA-protected
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) => {
            if idea.openness != IdeaOpenness::NdaProtected {
                return err(
                    StatusCode::BAD_REQUEST,
                    "NOT_NDA_PROTECTED",
                    "This idea is not NDA-protected",
                )
                .into_response();
            }
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    let nda_repo = NdaTemplateRepository::new(state.db.connection());
    match nda_repo.find_by_idea_id(id).await {
        Ok(Some(template)) => Json(NdaTemplateResponse {
            id: template.id,
            idea_id: template.idea_id,
            title: template.title,
            body: template.body,
            confidentiality_period_days: template.confidentiality_period_days,
            jurisdiction: template.jurisdiction,
            created_at: template.created_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => err(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "No NDA template found for this idea",
        )
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to find NDA template: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn create_nda_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateNdaTemplateRequest>,
) -> impl IntoResponse {
    // Verify idea exists, is NDA-protected, and caller is the author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) => {
            if idea.author_id != auth.user_id {
                return err(
                    StatusCode::FORBIDDEN,
                    "FORBIDDEN",
                    "Only the idea author can create an NDA template",
                )
                .into_response();
            }
            if idea.openness != IdeaOpenness::NdaProtected {
                return err(
                    StatusCode::BAD_REQUEST,
                    "NOT_NDA_PROTECTED",
                    "This idea is not NDA-protected",
                )
                .into_response();
            }
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Enforce free tier limit: max 1 NDA-protected idea per user
    let sig_repo = NdaSignatureRepository::new(state.db.connection());
    match sig_repo.count_nda_ideas_by_user(auth.user_id).await {
        Ok(count) => {
            // Allow if this idea already has an NDA (updating), deny if this is a new NDA idea and count >= 1
            let nda_repo = NdaTemplateRepository::new(state.db.connection());
            let existing = nda_repo.find_by_idea_id(id).await;
            let is_update = matches!(existing, Ok(Some(_)));
            if !is_update && count >= 1 {
                return err(
                    StatusCode::FORBIDDEN,
                    "FREE_TIER_LIMIT",
                    "Free tier allows 1 NDA-protected idea. Upgrade to create more.",
                )
                .into_response();
            }

            // If updating existing template
            if let Ok(Some(tmpl)) = existing {
                match nda_repo
                    .update(
                        tmpl.id,
                        Some(body.title.as_deref().unwrap_or(&tmpl.title)),
                        Some(&body.body),
                        Some(body.confidentiality_period_days.unwrap_or(tmpl.confidentiality_period_days)),
                        Some(body.jurisdiction.as_deref()),
                    )
                    .await
                {
                    Ok(updated) => {
                        return Json(NdaTemplateResponse {
                            id: updated.id,
                            idea_id: updated.idea_id,
                            title: updated.title,
                            body: updated.body,
                            confidentiality_period_days: updated.confidentiality_period_days,
                            jurisdiction: updated.jurisdiction,
                            created_at: updated.created_at.to_rfc3339(),
                        })
                        .into_response();
                    }
                    Err(e) => {
                        tracing::error!("Failed to update NDA template: {e}");
                        return err(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "INTERNAL_ERROR",
                            "Internal server error",
                        )
                        .into_response();
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to count NDA ideas: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    // Validate body text
    let body_text = body.body.trim();
    if body_text.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "NDA body text is required",
        )
        .into_response();
    }

    let title = body
        .title
        .as_deref()
        .unwrap_or("Standard Non-Disclosure Agreement");
    let period = body.confidentiality_period_days.unwrap_or(730);

    let nda_repo = NdaTemplateRepository::new(state.db.connection());
    match nda_repo
        .create(
            Uuid::new_v4(),
            id,
            title,
            body_text,
            period,
            body.jurisdiction.as_deref(),
            auth.user_id,
        )
        .await
    {
        Ok(template) => (
            StatusCode::CREATED,
            Json(NdaTemplateResponse {
                id: template.id,
                idea_id: template.idea_id,
                title: template.title,
                body: template.body,
                confidentiality_period_days: template.confidentiality_period_days,
                jurisdiction: template.jurisdiction,
                created_at: template.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create NDA template: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn sign_nda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SignNdaRequest>,
) -> impl IntoResponse {
    // Verify idea exists and is NDA-protected
    let idea_repo = IdeaRepository::new(state.db.connection());
    let idea = match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) => {
            if idea.openness != IdeaOpenness::NdaProtected {
                return err(
                    StatusCode::BAD_REQUEST,
                    "NOT_NDA_PROTECTED",
                    "This idea is not NDA-protected",
                )
                .into_response();
            }
            idea
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Author cannot sign their own NDA
    if idea.author_id == auth.user_id {
        return err(
            StatusCode::BAD_REQUEST,
            "SELF_SIGN",
            "The idea author cannot sign their own NDA",
        )
        .into_response();
    }

    // Find the NDA template
    let nda_repo = NdaTemplateRepository::new(state.db.connection());
    let template = match nda_repo.find_by_idea_id(id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "No NDA template found for this idea",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find NDA template: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    };

    // Check if user has already signed
    let sig_repo = NdaSignatureRepository::new(state.db.connection());
    match sig_repo.has_signed(auth.user_id, id).await {
        Ok(true) => {
            return err(
                StatusCode::CONFLICT,
                "ALREADY_SIGNED",
                "You have already signed the NDA for this idea",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to check NDA signature: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
        Ok(false) => {}
    }

    // Validate signer name
    let signer_name = body.signer_name.trim();
    if signer_name.is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            "Signer name is required",
        )
        .into_response();
    }

    // Calculate expiration
    let expires_at = if template.confidentiality_period_days > 0 {
        Some(
            chrono::Utc::now().fixed_offset()
                + chrono::Duration::days(template.confidentiality_period_days as i64),
        )
    } else {
        None
    };

    match sig_repo
        .create(
            Uuid::new_v4(),
            template.id,
            id,
            auth.user_id,
            signer_name,
            &auth.email,
            Some("unknown"),
            expires_at,
        )
        .await
    {
        Ok(sig) => (
            StatusCode::CREATED,
            Json(NdaSignatureResponse {
                id: sig.id,
                signer_id: sig.signer_id,
                signer_name: sig.signer_name,
                signed_at: sig.signed_at.to_rfc3339(),
                expires_at: sig.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create NDA signature: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn check_nda_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let sig_repo = NdaSignatureRepository::new(state.db.connection());
    match sig_repo.find_by_idea_and_signer(id, auth.user_id).await {
        Ok(Some(sig)) => Json(NdaStatusResponse {
            has_signed: true,
            signed_at: Some(sig.signed_at.to_rfc3339()),
            expires_at: sig.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
        })
        .into_response(),
        Ok(None) => Json(NdaStatusResponse {
            has_signed: false,
            signed_at: None,
            expires_at: None,
        })
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to check NDA status: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn list_nda_signatures(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Query(params): Query<PaginationQuery>,
) -> impl IntoResponse {
    // Verify caller is idea author
    let idea_repo = IdeaRepository::new(state.db.connection());
    match idea_repo.find_by_id(id).await {
        Ok(Some(idea)) if idea.author_id == auth.user_id => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Only the idea author can view NDA signatures",
            )
            .into_response()
        }
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "NOT_FOUND", "Idea not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to find idea: {e}");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response();
        }
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let sig_repo = NdaSignatureRepository::new(state.db.connection());
    match sig_repo.list_for_idea(id, page, per_page).await {
        Ok((signatures, total)) => {
            let total_pages = if total == 0 {
                0
            } else {
                (total + per_page - 1) / per_page
            };
            Json(NdaSignatureListResponse {
                data: signatures
                    .iter()
                    .map(|s| NdaSignatureResponse {
                        id: s.id,
                        signer_id: s.signer_id,
                        signer_name: s.signer_name.clone(),
                        signed_at: s.signed_at.to_rfc3339(),
                        expires_at: s.expires_at.as_ref().map(|dt| dt.to_rfc3339()),
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
            tracing::error!("Failed to list NDA signatures: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
