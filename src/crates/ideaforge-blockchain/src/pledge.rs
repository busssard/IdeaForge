use crate::blockfrost::BlockfrostClient;
use crate::types::{TxHash, UnsignedTransaction};
use uuid::Uuid;

/// Service for managing pledge transactions on the Cardano blockchain.
pub struct PledgeService {
    blockfrost: BlockfrostClient,
}

impl PledgeService {
    pub fn new(blockfrost: BlockfrostClient) -> Self {
        Self { blockfrost }
    }

    /// Build an unsigned pledge transaction that locks ADA in the escrow contract.
    ///
    /// The transaction sends `amount_lovelace` from the pledger's wallet to the
    /// escrow script address with the appropriate datum attached.
    pub async fn build_pledge_tx(
        &self,
        _idea_id: Uuid,
        _pledger_address: &str,
        _amount_lovelace: u64,
        _deadline_posix: i64,
    ) -> Result<UnsignedTransaction, PledgeError> {
        todo!("Build unsigned pledge TX using Blockfrost UTxO data")
    }

    /// Submit a client-signed pledge transaction to the Cardano network.
    pub async fn submit_signed_pledge(
        &self,
        _signed_tx_cbor: &[u8],
    ) -> Result<TxHash, PledgeError> {
        todo!("Submit signed TX via Blockfrost")
    }

    /// Check all pending pledges for on-chain confirmation.
    pub async fn check_pending_confirmations(&self) -> Result<Vec<ConfirmedPledge>, PledgeError> {
        todo!("Poll Blockfrost for TX confirmations")
    }
}

/// A pledge that has been confirmed on-chain.
pub struct ConfirmedPledge {
    pub pledge_id: Uuid,
    pub tx_hash: TxHash,
    pub confirmations: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum PledgeError {
    #[error("Blockfrost error: {0}")]
    Blockfrost(#[from] crate::blockfrost::BlockfrostError),

    #[error("Insufficient funds: need {needed} lovelace, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },

    #[error("Transaction building error: {0}")]
    TxBuild(String),
}
