use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};

use crate::extractors::OptionalAuth;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(report_bug))
}

#[derive(Debug, Deserialize)]
pub struct BugReportRequest {
    pub description: String,
    pub page_url: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BugReportResponse {
    pub message: String,
}

async fn report_bug(
    State(_state): State<AppState>,
    opt_auth: OptionalAuth,
    Json(body): Json<BugReportRequest>,
) -> impl IntoResponse {
    let description = body.description.trim();
    if description.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(BugReportResponse {
                message: "Description is required".to_string(),
            }),
        )
            .into_response();
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let user_info = match &opt_auth.0 {
        Some(auth) => format!("{} ({})", auth.email, auth.user_id),
        None => "anonymous".to_string(),
    };
    let page = body.page_url.as_deref().unwrap_or("unknown");
    let severity = body.severity.as_deref().unwrap_or("normal");

    let entry = format!(
        "\n---\n\n### Bug Report — {now}\n\n\
         - **Severity**: {severity}\n\
         - **Page**: {page}\n\
         - **User**: {user_info}\n\n\
         {description}\n",
    );

    // Write to bugs.md in the project root — try a few common locations
    let paths_to_try = [
        std::path::PathBuf::from("bugs.md"),
        std::path::PathBuf::from("../bugs.md"),
        std::path::PathBuf::from("/home/om/Documents/projects/IdeaForge/bugs.md"),
    ];

    let mut written = false;
    for path in &paths_to_try {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            use std::io::Write;
            // If the file is new/empty, add a header
            if std::fs::metadata(path).map_or(0, |m| m.len()) == 0 {
                let _ = write!(
                    file,
                    "# IdeaForge Bug Reports\n\n\
                     > Reported via in-app bug button. Review and fix these at the start of each session.\n"
                );
            }
            if write!(file, "{entry}").is_ok() {
                written = true;
                tracing::info!("Bug report written to {}", path.display());
                break;
            }
        }
    }

    if !written {
        tracing::error!("Failed to write bug report to any path");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(BugReportResponse {
                message: "Failed to save bug report".to_string(),
            }),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(BugReportResponse {
            message: "Bug report saved. Thanks!".to_string(),
        }),
    )
        .into_response()
}
