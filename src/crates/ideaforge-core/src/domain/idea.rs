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
    pub stoke_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// MVP maturity levels: 3 stages (simplified from 7).
///
/// - **Spark**: New idea, just posted. Default state.
/// - **Building**: Validated interest (5+ Stokes), developing.
/// - **InWork**: Active team, executing.
///
/// Only **human Stokes** count toward maturity advancement.
/// Full 7-level state machine is defined in the long-term architecture
/// (docs/architecture/database_schema.md) and will be restored in Phase 2+.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaMaturity {
    /// New idea, just posted
    Spark,
    /// Validated interest, developing (formerly "thought_through")
    Building,
    /// Active team, executing (formerly "in_work")
    InWork,
}

impl Default for IdeaMaturity {
    fn default() -> Self {
        Self::Spark
    }
}

/// Criteria required for a maturity transition.
/// All thresholds count **human Stokes only**.
#[derive(Debug, Clone)]
pub struct TransitionRequirements {
    pub min_stokes: i32,
    pub requires_author_action: bool,
    pub requires_team_member: bool,
}

impl IdeaMaturity {
    /// Get the requirements for transitioning from `self` to `target`.
    /// Returns `None` if the transition is not valid.
    pub fn transition_requirements(&self, target: &IdeaMaturity) -> Option<TransitionRequirements> {
        use IdeaMaturity::*;
        match (self, target) {
            (Spark, Building) => Some(TransitionRequirements {
                min_stokes: 5,
                requires_author_action: false,
                requires_team_member: false,
            }),
            (Building, InWork) => Some(TransitionRequirements {
                min_stokes: 0,
                requires_author_action: true,
                requires_team_member: true,
            }),
            // Regression: InWork -> Building (blockers found)
            (InWork, Building) => Some(TransitionRequirements {
                min_stokes: 0,
                requires_author_action: true,
                requires_team_member: false,
            }),
            _ => None,
        }
    }

    /// Check whether a transition from `self` to `target` is structurally valid.
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
///
/// MVP has 3 modes (no Secret -- deferred to Phase 2+ with encryption infrastructure).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaOpenness {
    /// Fully open, anyone can see and contribute
    Open,
    /// Open but team membership is curated
    Collaborative,
    /// Visible but contributions require approval
    Commercial,
}

impl Default for IdeaOpenness {
    fn default() -> Self {
        Self::Open
    }
}

/// Human "Stoke" -- an upvote/approval of an idea.
/// Only humans can Stoke. Stokes drive maturity advancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stoke {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}
