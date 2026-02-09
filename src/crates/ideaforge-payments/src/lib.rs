//! IdeaForge Payments - Fiat payment processing via Stripe.
//!
//! **STATUS: DEFERRED TO PHASE 2**
//!
//! This crate is a stub. Stripe payment integration (subscriptions, fiat
//! on-ramp for pledges, marketplace commissions) is deferred until after
//! MVP launch. Everyone is on the free tier at launch.
//!
//! Phase 2: Stripe subscriptions + fiat pledge escrow
//!
//! See: docs/architecture/mvp_architecture.md (current scope)
//! See: docs/business/business_model.md (pricing tiers -- Phase 2)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription tiers available on the platform (Phase 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// Free tier (MVP: everyone is on this)
    Free,
    /// Paid tier with commercial features (Phase 2)
    Pro,
}

/// Trait for payment processing. Allows testing with mock payment providers.
pub trait PaymentProvider: Send + Sync {
    fn create_subscription(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<(), PaymentError>;

    fn cancel_subscription(&self, subscription_id: Uuid) -> Result<(), PaymentError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Stripe API error: {0}")]
    StripeError(String),

    #[error("Subscription not found: {0}")]
    NotFound(Uuid),

    #[error("Payment declined: {0}")]
    Declined(String),
}
