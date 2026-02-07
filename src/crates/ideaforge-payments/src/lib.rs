//! IdeaForge Payments - Fiat payment processing via Stripe.
//!
//! Handles subscription billing (Builder/Venture/Enterprise tiers),
//! fiat on-ramp for users who prefer not to use crypto directly,
//! and marketplace commission processing.
//!
//! PCI DSS compliance is achieved via Stripe tokenization --
//! no card data is stored on the IdeaForge platform.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription tiers available on the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// Free tier for open-source ideas
    Spark,
    /// $12/mo - commercial ideas, priority support
    Builder,
    /// $39/mo - IP-protected ideas, advanced analytics
    Venture,
    /// $499/mo - organization accounts, SSO, dedicated support
    Enterprise,
}

/// A subscription record for a paying user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: SubscriptionTier,
    pub stripe_subscription_id: Option<String>,
    pub status: SubscriptionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Canceled,
    Trialing,
}

/// Trait for payment processing. Allows testing with mock payment providers.
pub trait PaymentProvider: Send + Sync {
    fn create_subscription(
        &self,
        user_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<Subscription, PaymentError>;

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
