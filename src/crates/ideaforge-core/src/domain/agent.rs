use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AI agent verification levels, aligned with the Bot Transparency Framework.
/// Higher levels grant more privileges on the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentVerificationLevel {
    /// Basic registration only. Read-only access; endorsements not counted publicly.
    Unverified,
    /// Operator identity confirmed, agent description reviewed. Full agent privileges.
    Verified,
    /// Capability testing passed, code audit completed. Priority matching, higher rate limits.
    Certified,
    /// Strategic partnership with IdeaForge. Custom integrations, elevated visibility.
    Partner,
}

impl Default for AgentVerificationLevel {
    fn default() -> Self {
        Self::Unverified
    }
}

/// AI agent capability classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCapability {
    Ideation,
    Coding,
    Design,
    Analysis,
}

/// Extended metadata for an AI agent account.
/// Stored alongside the `User` record (where `is_bot = true`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_id: Uuid,
    pub operator_id: Uuid,
    pub model_type: String,
    pub capability_class: AgentCapability,
    pub verification_level: AgentVerificationLevel,
    pub description: String,
    pub webhook_url: Option<String>,
    pub max_endorsements_per_day: i32,
    pub quality_score: Option<f32>,
    pub registration_date: DateTime<Utc>,
    pub last_verification_date: Option<DateTime<Utc>>,
}

impl AgentProfile {
    /// Default endorsement limit per day, based on verification level.
    pub fn default_endorsement_limit(level: AgentVerificationLevel) -> i32 {
        match level {
            AgentVerificationLevel::Unverified => 0,
            AgentVerificationLevel::Verified => 10,
            AgentVerificationLevel::Certified => 25,
            AgentVerificationLevel::Partner => 50,
        }
    }
}
