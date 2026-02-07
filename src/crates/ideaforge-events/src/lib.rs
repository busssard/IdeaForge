//! IdeaForge Events - Domain event publishing and subscribing.
//!
//! Uses NATS JetStream for reliable event delivery.
//! Events are published when domain actions occur (idea created,
//! pledge confirmed, etc.) and consumed by handlers that update
//! search indices, send notifications, and fan out WebSocket updates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A domain event published to the event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub actor_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Types of domain events emitted by the platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // Ideas
    IdeaCreated,
    IdeaUpdated,
    IdeaMaturityChanged,
    IdeaArchived,

    // Approvals (human only)
    ApprovalCreated,
    ApprovalWithdrawn,

    // AI Endorsements (separate track)
    EndorsementCreated,
    EndorsementWithdrawn,

    // Contributions
    ContributionCreated,
    ContributionUpdated,

    // Pledges
    PledgeCreated,
    PledgeConfirmed,
    PledgeFulfilled,
    PledgeRefunded,

    // Todos
    TodoCreated,
    TodoAssigned,
    TodoStatusChanged,

    // Users
    UserRegistered,
    ExpertApplicationSubmitted,
    ExpertApplicationReviewed,

    // Agents
    AgentRegistered,
    AgentDeactivated,
}

impl EventType {
    /// NATS subject for this event type (e.g., "ideaforge.ideas.created").
    pub fn subject(&self) -> &'static str {
        match self {
            Self::IdeaCreated => "ideaforge.ideas.created",
            Self::IdeaUpdated => "ideaforge.ideas.updated",
            Self::IdeaMaturityChanged => "ideaforge.ideas.maturity_changed",
            Self::IdeaArchived => "ideaforge.ideas.archived",
            Self::ApprovalCreated => "ideaforge.approvals.created",
            Self::ApprovalWithdrawn => "ideaforge.approvals.withdrawn",
            Self::EndorsementCreated => "ideaforge.endorsements.created",
            Self::EndorsementWithdrawn => "ideaforge.endorsements.withdrawn",
            Self::ContributionCreated => "ideaforge.contributions.created",
            Self::ContributionUpdated => "ideaforge.contributions.updated",
            Self::PledgeCreated => "ideaforge.pledges.created",
            Self::PledgeConfirmed => "ideaforge.pledges.confirmed",
            Self::PledgeFulfilled => "ideaforge.pledges.fulfilled",
            Self::PledgeRefunded => "ideaforge.pledges.refunded",
            Self::TodoCreated => "ideaforge.todos.created",
            Self::TodoAssigned => "ideaforge.todos.assigned",
            Self::TodoStatusChanged => "ideaforge.todos.status_changed",
            Self::UserRegistered => "ideaforge.users.registered",
            Self::ExpertApplicationSubmitted => "ideaforge.experts.submitted",
            Self::ExpertApplicationReviewed => "ideaforge.experts.reviewed",
            Self::AgentRegistered => "ideaforge.agents.registered",
            Self::AgentDeactivated => "ideaforge.agents.deactivated",
        }
    }
}

/// Trait for publishing domain events. Allows testing with mock publishers.
pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: DomainEvent) -> Result<(), EventError>;
}

/// Trait for subscribing to domain events.
pub trait EventSubscriber: Send + Sync {
    fn subscribe(
        &self,
        subject: &str,
        handler: Box<dyn Fn(DomainEvent) + Send + Sync>,
    ) -> Result<(), EventError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
