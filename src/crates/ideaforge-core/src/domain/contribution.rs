use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A contribution to an idea (comment, suggestion, design, code, research).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub contribution_type: ContributionType,
    pub title: Option<String>,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionType {
    Comment,
    Suggestion,
    Design,
    Code,
    Research,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub url: String,
    pub filename: String,
    pub mime_type: String,
}
