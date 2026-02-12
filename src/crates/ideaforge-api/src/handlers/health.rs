use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::ConnectionTrait;
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub database: &'static str,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check DB connectivity
    let db_status = match state
        .db
        .connection()
        .execute_unprepared("SELECT 1")
        .await
    {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let status = if db_status == "connected" {
        "ok"
    } else {
        "degraded"
    };

    let code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(HealthResponse {
            status,
            version: env!("CARGO_PKG_VERSION"),
            database: db_status,
        }),
    )
}
