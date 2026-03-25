//! Pledge service for managing escrow transactions on the Cardano blockchain.
//!
//! This service handles building, submitting, and monitoring pledge transactions
//! that lock ADA in the on-chain escrow smart contract.

use crate::blockfrost::{BlockfrostClient, BlockfrostError};
use crate::types::*;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum PledgeError {
    #[error("Blockfrost error: {0}")]
    Blockfrost(#[from] BlockfrostError),

    #[error("Insufficient funds: need {needed} lovelace, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },

    #[error("Campaign expired at POSIX timestamp {0}")]
    CampaignExpired(i64),

    #[error("Campaign target not met: {pledged} of {target} lovelace")]
    TargetNotMet { pledged: u64, target: u64 },

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Transaction building not yet implemented: {0}")]
    TxBuildNotImplemented(String),
}

/// Service for managing pledge transactions on the Cardano blockchain.
///
/// The pledge service coordinates with the Blockfrost API to:
/// - Build unsigned pledge transactions for client-side signing
/// - Submit signed transactions to the Cardano network
/// - Monitor campaign status and transaction confirmations
pub struct PledgeService {
    blockfrost: BlockfrostClient,
    /// Platform's verification key hash for co-signing claim transactions
    platform_key_hash: String,
}

impl PledgeService {
    /// Create a new pledge service.
    ///
    /// # Arguments
    /// * `blockfrost` - Configured Blockfrost API client
    /// * `platform_key_hash` - Hex-encoded verification key hash of the platform's co-signing key
    pub fn new(blockfrost: BlockfrostClient, platform_key_hash: String) -> Self {
        Self {
            blockfrost,
            platform_key_hash,
        }
    }

    /// Get a reference to the underlying Blockfrost client.
    pub fn blockfrost(&self) -> &BlockfrostClient {
        &self.blockfrost
    }

    /// Get the platform's verification key hash.
    pub fn platform_key_hash(&self) -> &str {
        &self.platform_key_hash
    }

    /// Build an unsigned pledge transaction that locks ADA in the escrow contract.
    ///
    /// The transaction sends `amount_lovelace` from the pledger's wallet to the
    /// escrow script address with the appropriate `PledgeDatum` attached.
    ///
    /// The returned `UnsignedTransaction` should be sent to the frontend for
    /// client-side signing via CIP-30 wallet API.
    ///
    /// # Arguments
    /// * `idea_id` - The IdeaForge idea this pledge is for
    /// * `pledger_address` - Bech32 address of the pledger's wallet
    /// * `amount_lovelace` - Amount to pledge in lovelace (1 ADA = 1,000,000 lovelace)
    /// * `creator_address` - Bech32 address of the idea creator
    /// * `deadline_posix` - POSIX timestamp when the pledge campaign expires
    /// * `min_target` - Minimum total lovelace needed for the campaign to succeed
    pub async fn build_pledge_tx(
        &self,
        idea_id: Uuid,
        pledger_address: &str,
        amount_lovelace: u64,
        creator_address: &str,
        deadline_posix: i64,
        _min_target: u64,
    ) -> Result<UnsignedTransaction, PledgeError> {
        // Validate basic address format (bech32 Cardano addresses start with addr)
        if !pledger_address.starts_with("addr") {
            return Err(PledgeError::InvalidAddress(format!(
                "Pledger address does not look like a Cardano address: {}",
                pledger_address
            )));
        }
        if !creator_address.starts_with("addr") {
            return Err(PledgeError::InvalidAddress(format!(
                "Creator address does not look like a Cardano address: {}",
                creator_address
            )));
        }

        // Check deadline hasn't already passed
        let now = chrono::Utc::now().timestamp();
        if deadline_posix <= now {
            return Err(PledgeError::CampaignExpired(deadline_posix));
        }

        // Query pledger's UTxOs to verify they have sufficient funds
        let utxos = self.blockfrost.get_utxos(pledger_address).await?;
        let total_available: u64 = utxos.iter().map(|u| u.lovelace_amount()).sum();

        // Need pledge amount + estimated fee (~0.2 ADA = 200,000 lovelace) + min UTxO for change
        let estimated_fee: u64 = 200_000;
        let min_utxo: u64 = 1_000_000; // ~1 ADA minimum UTxO
        let total_needed = amount_lovelace + estimated_fee + min_utxo;

        if total_available < total_needed {
            return Err(PledgeError::InsufficientFunds {
                needed: total_needed,
                available: total_available,
            });
        }

        info!(
            idea_id = %idea_id,
            pledger = pledger_address,
            amount = amount_lovelace,
            "Building pledge transaction"
        );

        // TODO: Actual CBOR transaction building requires cardano-serialization-lib
        // or pallas-traverse. The transaction should:
        //
        // 1. Select UTxOs from pledger's wallet using a coin selection algorithm
        // 2. Create a transaction output to the escrow script address with:
        //    - The pledge amount in lovelace
        //    - A PledgeDatum containing: idea_id, pledger_vkh, creator_vkh, deadline, min_target
        // 3. Create a change output back to the pledger
        // 4. Set the fee based on protocol parameters and transaction size
        // 5. Return the unsigned transaction in CBOR hex format
        //
        // For now, we validate inputs and return a placeholder.
        // The actual transaction building will be implemented when we integrate
        // cardano-serialization-lib (Rust) or use cardano-cli as a subprocess.

        Err(PledgeError::TxBuildNotImplemented(format!(
            "CBOR transaction building for pledge of {amount_lovelace} lovelace \
             on idea {idea_id} requires cardano-serialization-lib. \
             Pledger has {total_available} lovelace available across {} UTxOs.",
            utxos.len()
        )))
    }

    /// Submit a client-signed pledge transaction to the Cardano network.
    ///
    /// The `signed_tx_cbor` should be the raw CBOR bytes of the transaction
    /// after the user has signed it with their CIP-30 wallet.
    pub async fn submit_signed_pledge(
        &self,
        signed_tx_cbor: &[u8],
    ) -> Result<TxHash, PledgeError> {
        debug!(
            tx_size = signed_tx_cbor.len(),
            "Submitting signed pledge transaction"
        );

        let tx_hash = self.blockfrost.submit_tx(signed_tx_cbor).await?;

        info!(tx_hash = %tx_hash.0, "Pledge transaction submitted to Cardano network");
        Ok(tx_hash)
    }

    /// Get the current status of a pledge campaign by querying the script address.
    ///
    /// Counts all UTxOs at the script address and sums up the total pledged lovelace.
    pub async fn get_campaign_status(
        &self,
        script_address: &str,
        min_target: u64,
    ) -> Result<PledgeCampaign, PledgeError> {
        debug!(script_address, "Querying campaign status");

        let utxos = self.blockfrost.get_script_utxos(script_address).await?;

        let total_pledged: u64 = utxos.iter().map(|u| u.lovelace_amount()).sum();
        let pledge_count = utxos.len() as u32;

        debug!(
            script_address,
            total_pledged,
            pledge_count,
            min_target,
            target_met = total_pledged >= min_target,
            "Campaign status retrieved"
        );

        // Note: idea_id, creator_address, and deadline would normally be extracted from
        // the datum of the UTxOs. For now we return what we can determine from the UTxOs.
        Ok(PledgeCampaign {
            idea_id: Uuid::nil(), // Would be extracted from datum
            creator_address: CardanoAddress(String::new()), // Would be extracted from datum
            script_address: CardanoAddress(script_address.to_string()),
            min_target_lovelace: min_target,
            deadline_posix: 0, // Would be extracted from datum
            total_pledged_lovelace: total_pledged,
            pledge_count,
        })
    }

    /// Check the confirmation status of a transaction.
    pub async fn check_tx_confirmation(
        &self,
        tx_hash: &str,
    ) -> Result<TxStatus, PledgeError> {
        debug!(tx_hash, "Checking transaction confirmation");

        let status = self.blockfrost.get_tx_status(tx_hash).await?;

        match &status {
            TxStatus::Confirmed {
                block_height,
                block_hash,
            } => {
                info!(
                    tx_hash,
                    block_height,
                    block_hash,
                    "Transaction confirmed on-chain"
                );
            }
            TxStatus::Pending => {
                debug!(tx_hash, "Transaction still pending");
            }
            TxStatus::Failed { reason } => {
                warn!(tx_hash, reason, "Transaction failed");
            }
        }

        Ok(status)
    }
}
