//! IdeaForge Events - Domain event publishing and subscribing.
//!
//! **STATUS: DEFERRED TO PHASE 2+**
//!
//! NATS JetStream event bus is over-engineering for MVP. The MVP uses
//! DB-backed notifications (ideaforge-core::domain::notification) instead.
//!
//! Phase 2+: Restore NATS for real-time WebSocket fan-out, search index
//! sync, and future microservice communication.
//!
//! See: docs/architecture/mvp_architecture.md (current scope)
//! See: docs/architecture/tech_decisions.md ADR-007 (long-term rationale)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A domain event (Phase 2+: published to NATS).
/// For MVP, events are recorded directly in the notifications table.
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

    // Stokes (human only)
    StokeCreated,
    StokeWithdrawn,

    // Contributions
    ContributionCreated,
    ContributionUpdated,

    // Team formation (MVP killer feature)
    TaskCreated,
    TaskAssigned,
    TaskStatusChanged,
    TeamApplicationSubmitted,
    TeamApplicationReviewed,
    TeamMemberJoined,
    TeamMemberRemoved,

    // Users
    UserRegistered,
}

/// Trait for publishing domain events. Allows testing with mock publishers.
/// MVP: not used (notifications written directly to DB).
/// Phase 2+: implemented by NATS publisher.
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
