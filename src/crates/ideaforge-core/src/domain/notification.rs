use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A DB-backed notification for a user.
///
/// MVP uses database-backed notifications instead of NATS.
/// The notifications table is polled via the REST API.
/// Upgrade path: add WebSocket push + NATS in Phase 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub payload: serde_json::Value,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Someone Stoked your idea
    IdeaStoked,
    /// Your idea advanced to a new maturity level
    MaturityChanged,
    /// Someone commented on your idea
    NewContribution,
    /// Someone applied to join your team
    TeamApplicationReceived,
    /// Your team application was accepted
    TeamApplicationAccepted,
    /// Your team application was rejected
    TeamApplicationRejected,
    /// A task was assigned to you
    TaskAssigned,
    /// A task you're assigned to was updated
    TaskUpdated,
    /// A new team member joined your idea
    TeamMemberJoined,
}
