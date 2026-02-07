use crate::types::{TxHash, TxStatus, UTxO};

/// Client for the Blockfrost Cardano API.
///
/// Blockfrost provides a REST gateway to the Cardano blockchain,
/// eliminating the need to run a full node.
pub struct BlockfrostClient {
    base_url: String,
    project_id: String,
    // TODO: Add reqwest::Client once reqwest is added to dependencies
}

impl BlockfrostClient {
    pub fn new(base_url: String, project_id: String) -> Self {
        Self {
            base_url,
            project_id,
        }
    }

    /// Get all UTxOs for a given address.
    pub async fn get_utxos(&self, _address: &str) -> Result<Vec<UTxO>, BlockfrostError> {
        todo!("Implement Blockfrost UTxO query")
    }

    /// Submit a signed transaction to the Cardano network.
    pub async fn submit_tx(&self, _signed_tx_cbor: &[u8]) -> Result<TxHash, BlockfrostError> {
        todo!("Implement Blockfrost TX submission")
    }

    /// Check the confirmation status of a transaction.
    pub async fn get_tx_status(&self, _tx_hash: &str) -> Result<TxStatus, BlockfrostError> {
        todo!("Implement Blockfrost TX status check")
    }

    /// Get UTxOs at a script address (for checking pledged amounts).
    pub async fn get_script_utxos(
        &self,
        _script_hash: &str,
    ) -> Result<Vec<UTxO>, BlockfrostError> {
        todo!("Implement Blockfrost script UTxO query")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockfrostError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}
