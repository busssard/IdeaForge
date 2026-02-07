use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A financial pledge toward an idea, backed by Cardano.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pledge {
    pub id: Uuid,
    pub idea_id: Uuid,
    pub user_id: Uuid,
    pub amount_lovelace: i64,
    pub tx_hash: Option<String>,
    pub script_address: Option<String>,
    pub status: PledgeStatus,
    pub pledge_message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PledgeStatus {
    /// Pledge intent recorded, awaiting on-chain confirmation
    Pending,
    /// On-chain TX confirmed
    Confirmed,
    /// Product delivered, funds released to creator
    Fulfilled,
    /// Funds returned to pledger
    Refunded,
    /// Pledge window closed without fulfillment
    Expired,
}

impl Pledge {
    /// Amount in ADA (1 ADA = 1,000,000 lovelace).
    pub fn amount_ada(&self) -> f64 {
        self.amount_lovelace as f64 / 1_000_000.0
    }
}
