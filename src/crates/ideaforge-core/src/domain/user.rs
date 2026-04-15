use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A platform user. MVP is human-only (no bot accounts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MVP roles: 3 (simplified from 8).
///
/// - **Entrepreneur**: Creates ideas, manages task boards, leads teams.
/// - **Maker**: Applies to join teams, claims tasks, builds.
/// - **Curious**: Browses, Stokes ideas, comments.
///
/// Full role system (Investor, Consumer, Freelancer, AI Agent) is
/// defined in the long-term architecture and restored in Phase 2+.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum UserRole {
    Entrepreneur,
    Maker,
    #[default]
    Curious,
}

impl UserRole {
    /// Returns the default permissions for this role.
    pub fn default_permissions(&self) -> &'static [&'static str] {
        match self {
            Self::Entrepreneur => &[
                "ideas.create",
                "ideas.update_own",
                "ideas.delete_own",
                "ideas.set_maturity",
                "stokes.create",
                "contributions.create",
                "board.create",
                "board.manage_own",
                "team.manage_own",
            ],
            Self::Maker => &[
                "ideas.create",
                "stokes.create",
                "contributions.create",
                "team.apply",
                "board.tasks.claim",
            ],
            Self::Curious => &["stokes.create", "contributions.create"],
        }
    }
}

/// Platform permissions following the `domain.action` pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(pub String);

impl Permission {
    pub fn new(domain: &str, action: &str) -> Self {
        Self(format!("{domain}.{action}"))
    }

    pub fn matches(&self, required: &str) -> bool {
        self.0 == required || self.0 == "admin.*"
    }
}
