use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;
use ideaforge_db::repositories::category_repo::CategoryRepository;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories))
        .route("/:slug", get(get_category))
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
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

async fn list_categories(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CategoryRepository::new(state.db.connection());
    match repo.list_all().await {
        Ok(categories) => Json(
            categories
                .iter()
                .map(|c| CategoryResponse {
                    id: c.id,
                    name: c.name.clone(),
                    slug: c.slug.clone(),
                    description: c.description.clone(),
                    icon: c.icon.clone(),
                    parent_id: c.parent_id,
                    sort_order: c.sort_order,
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to list categories: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}

async fn get_category(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let repo = CategoryRepository::new(state.db.connection());
    match repo.find_by_slug(&slug).await {
        Ok(Some(c)) => Json(CategoryResponse {
            id: c.id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            icon: c.icon,
            parent_id: c.parent_id,
            sort_order: c.sort_order,
        })
        .into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "NOT_FOUND", "Category not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get category: {e}");
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            )
            .into_response()
        }
    }
}
