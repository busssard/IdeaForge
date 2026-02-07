use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// An idea on the IdeaForge platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub maturity: IdeaMaturity,
    pub openness: IdeaOpenness,
    pub metadata: serde_json::Value,
    pub is_archived: bool,
    pub human_approvals: i32,
    pub ai_endorsements: i32,
    pub total_pledged_lovelace: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Maturity levels form a state machine with defined transitions.
/// Only **human approvals** count toward maturity advancement.
/// AI endorsements are informational only and never trigger transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaMaturity {
    UnansweredQuestion,
    HalfBaked,
    ThoughtThrough,
    SeriousProposal,
    InWork,
    AlmostFinished,
    Completed,
}

/// Criteria required for a maturity transition, aligned with the Product Manager's spec.
/// All approval thresholds count **human approvals only**.
#[derive(Debug, Clone)]
pub struct TransitionRequirements {
    pub min_human_approvals: i32,
    pub min_human_comments: i32,
    pub min_contributors: i32,
    pub requires_author_action: bool,
    pub requires_admin_verification: bool,
}

impl IdeaMaturity {
    /// Get the requirements for transitioning from `self` to `target`.
    /// Returns `None` if the transition is not valid.
    pub fn transition_requirements(&self, target: &IdeaMaturity) -> Option<TransitionRequirements> {
        use IdeaMaturity::*;
        match (self, target) {
            (UnansweredQuestion, HalfBaked) => Some(TransitionRequirements {
                min_human_approvals: 5,
                min_human_comments: 0,
                min_contributors: 0,
                requires_author_action: false,
                requires_admin_verification: false,
            }),
            (HalfBaked, ThoughtThrough) => Some(TransitionRequirements {
                min_human_approvals: 15,
                min_human_comments: 3,
                min_contributors: 0,
                requires_author_action: false,
                requires_admin_verification: false,
            }),
            (ThoughtThrough, SeriousProposal) => Some(TransitionRequirements {
                min_human_approvals: 30,
                min_human_comments: 0,
                min_contributors: 3,
                requires_author_action: false,
                requires_admin_verification: false,
            }),
            (SeriousProposal, InWork) => Some(TransitionRequirements {
                min_human_approvals: 0,
                min_human_comments: 0,
                min_contributors: 0,
                requires_author_action: true,
                requires_admin_verification: false,
            }),
            (InWork, AlmostFinished) => Some(TransitionRequirements {
                min_human_approvals: 0,
                min_human_comments: 0,
                min_contributors: 0,
                requires_author_action: true,
                requires_admin_verification: false,
            }),
            (AlmostFinished, Completed) => Some(TransitionRequirements {
                min_human_approvals: 0,
                min_human_comments: 0,
                min_contributors: 0,
                requires_author_action: true,
                requires_admin_verification: true,
            }),
            // Regressions from InWork (blockers found)
            (InWork, SeriousProposal) | (InWork, ThoughtThrough) => Some(TransitionRequirements {
                min_human_approvals: 0,
                min_human_comments: 0,
                min_contributors: 0,
                requires_author_action: true,
                requires_admin_verification: false,
            }),
            _ => None,
        }
    }

    /// Check whether a transition from `self` to `target` is structurally valid
    /// (ignoring threshold checks, which require idea context).
    pub fn can_transition_to(&self, target: &IdeaMaturity) -> bool {
        self.transition_requirements(target).is_some()
    }

    /// Attempt a state transition. Returns an error if the transition is invalid.
    pub fn transition_to(&self, target: IdeaMaturity) -> Result<IdeaMaturity, AppError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(AppError::InvalidStateTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", target),
            })
        }
    }
}

/// How open/protected an idea is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaOpenness {
    /// Fully open, anyone can see and contribute
    OpenSource,
    /// Open but with contributor agreements (community co-creation)
    OpenCollaboration,
    /// Commercially oriented, visible but with contribution controls
    Commercial,
    /// IP-protected, restricted access (requires NDA + entrepreneur approval)
    Secret,
}

/// Human approval of an idea. Only humans can approve.
/// Approvals are the sole signal for maturity advancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// AI agent endorsement of an idea. Completely separate from human approvals.
/// Endorsements are informational only and never count toward maturity.
/// Aligns with EU AI Act Article 50 transparency requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEndorsement {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub agent_id: Uuid,
    pub operator_id: Uuid,
    pub confidence: Option<f32>,
    pub reasoning: Option<String>,
    pub model_version: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Summary of approvals and endorsements for an idea, always displayed separately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalSummary {
    pub human_approvals: i32,
    pub ai_endorsements: i32,
    pub human_comments: i32,
    pub ai_comments: i32,
}
