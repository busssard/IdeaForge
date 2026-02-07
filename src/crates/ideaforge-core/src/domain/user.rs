use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A platform user (human or bot).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub wallet_address: Option<String>,
    pub is_bot: bool,
    pub bot_owner_id: Option<Uuid>,
    pub onboarding_role: OnboardingRole,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The role a user selects during onboarding. Controls progressive UI disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingRole {
    Entrepreneur,
    Investor,
    Maker,
    Freelancer,
    AiAgent,
    Consumer,
    Curious,
}

impl Default for OnboardingRole {
    fn default() -> Self {
        Self::Curious
    }
}

impl OnboardingRole {
    /// Returns the disclosure tier for this role, controlling which UI features
    /// are shown by default. Aligned with the UX Philosophy progressive disclosure matrix.
    pub fn disclosure_tier(&self) -> DisclosureTier {
        match self {
            Self::Curious | Self::Consumer => DisclosureTier::T1,
            Self::Maker | Self::Freelancer | Self::AiAgent => DisclosureTier::T2,
            Self::Entrepreneur | Self::Investor => DisclosureTier::T3,
        }
    }
}

/// Progressive disclosure tiers control which features are shown in the UI.
/// This is a UX concern, not a permission gate -- users can always access
/// additional features by changing their role in settings.
///
/// Aligned with the Creative Mind's UX Philosophy document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureTier {
    /// Browse, vote, comment -- minimal UI complexity
    T1,
    /// + Create ideas, apply to tasks, expert profiles
    T2,
    /// + Pledges, IP protection, financial features, AI agent management
    T3,
}

impl DisclosureTier {
    /// Check whether a feature at the given tier should be visible for this user tier.
    pub fn can_see(&self, feature_tier: &DisclosureTier) -> bool {
        self >= feature_tier
    }
}

/// Expert roles that users can apply for on specific ideas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpertRole {
    Maker,
    Programmer,
    Designer,
    Scientist,
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
