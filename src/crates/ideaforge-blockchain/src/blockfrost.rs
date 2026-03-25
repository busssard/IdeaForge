//! Blockfrost API client for querying and submitting to the Cardano blockchain.
//!
//! Blockfrost provides a REST gateway so we don't need to run a full Cardano node.
//! See: <https://docs.blockfrost.io/>

use crate::types::{AssetAmount, Network, ProtocolParameters, TxHash, TxStatus, UTxO};
use serde::Deserialize;
use tracing::{debug, warn};

/// Client for the Blockfrost Cardano API.
#[derive(Debug, Clone)]
pub struct BlockfrostClient {
    base_url: String,
    project_id: String,
    client: reqwest::Client,
    network: Network,
}

#[derive(Debug, thiserror::Error)]
pub enum BlockfrostError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Resource not found")]
    NotFound,

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Raw UTxO response from the Blockfrost API.
#[derive(Debug, Deserialize)]
struct BlockfrostUtxo {
    tx_hash: String,
    output_index: u32,
    #[allow(dead_code)]
    address: Option<String>,
    amount: Vec<BlockfrostAmount>,
    data_hash: Option<String>,
    inline_datum: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct BlockfrostAmount {
    unit: String,
    quantity: String,
}

/// Raw transaction response from Blockfrost.
#[derive(Debug, Deserialize)]
struct BlockfrostTx {
    block_height: Option<u64>,
    block: Option<String>,
    #[allow(dead_code)]
    valid_contract: Option<bool>,
}

/// Raw error response from Blockfrost.
#[derive(Debug, Deserialize)]
struct BlockfrostApiError {
    #[allow(dead_code)]
    status_code: u16,
    message: String,
}

/// Raw protocol parameters from Blockfrost.
#[derive(Debug, Deserialize)]
struct BlockfrostProtocolParams {
    min_fee_a: serde_json::Value,
    min_fee_b: serde_json::Value,
    max_tx_size: serde_json::Value,
    key_deposit: serde_json::Value,
    pool_deposit: serde_json::Value,
    max_val_size: serde_json::Value,
    collateral_percent: serde_json::Value,
    price_step: Option<serde_json::Value>,
    price_mem: Option<serde_json::Value>,
    coins_per_utxo_size: serde_json::Value,
    #[allow(dead_code)]
    min_utxo: Option<serde_json::Value>,
}

impl BlockfrostClient {
    /// Create a new Blockfrost client for the given network.
    ///
    /// # Arguments
    /// * `project_id` - Blockfrost API key (starts with network prefix, e.g., "previewXXX...")
    /// * `network` - Which Cardano network to connect to
    pub fn new(project_id: &str, network: Network) -> Self {
        let base_url = network.blockfrost_base_url().to_string();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url,
            project_id: project_id.to_string(),
            client,
            network,
        }
    }

    /// Get the network this client is connected to.
    pub fn network(&self) -> Network {
        self.network
    }

    /// Build an authenticated request to the Blockfrost API.
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &url)
            .header("project_id", &self.project_id)
    }

    /// Handle Blockfrost API error responses.
    async fn handle_error(response: reqwest::Response) -> BlockfrostError {
        let status = response.status().as_u16();
        if status == 404 {
            return BlockfrostError::NotFound;
        }
        match response.json::<BlockfrostApiError>().await {
            Ok(err) => BlockfrostError::Api {
                status,
                message: err.message,
            },
            Err(_) => BlockfrostError::Api {
                status,
                message: format!("HTTP {status}"),
            },
        }
    }

    /// Get all UTxOs for a given address.
    ///
    /// Handles pagination automatically (Blockfrost returns max 100 per page).
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<UTxO>, BlockfrostError> {
        let mut all_utxos = Vec::new();
        let mut page = 1u32;

        loop {
            debug!(address, page, "Querying UTxOs from Blockfrost");

            let response = self
                .request(
                    reqwest::Method::GET,
                    &format!("/addresses/{address}/utxos"),
                )
                .query(&[("page", page.to_string())])
                .send()
                .await?;

            if !response.status().is_success() {
                // 404 means no UTxOs (empty address), not an error
                if response.status().as_u16() == 404 {
                    return Ok(all_utxos);
                }
                return Err(Self::handle_error(response).await);
            }

            let utxos: Vec<BlockfrostUtxo> = response.json().await.map_err(|e| {
                BlockfrostError::Deserialization(format!("Failed to parse UTxO response: {e}"))
            })?;

            if utxos.is_empty() {
                break;
            }

            for u in &utxos {
                all_utxos.push(UTxO {
                    tx_hash: u.tx_hash.clone(),
                    output_index: u.output_index,
                    address: address.to_string(),
                    amount: u
                        .amount
                        .iter()
                        .map(|a| AssetAmount {
                            unit: a.unit.clone(),
                            quantity: a.quantity.clone(),
                        })
                        .collect(),
                    data_hash: u.data_hash.clone(),
                    inline_datum: u.inline_datum.clone(),
                });
            }

            // Blockfrost returns max 100 results per page
            if utxos.len() < 100 {
                break;
            }
            page += 1;
        }

        debug!(
            address,
            count = all_utxos.len(),
            "Retrieved UTxOs from Blockfrost"
        );
        Ok(all_utxos)
    }

    /// Submit a signed transaction to the Cardano network.
    ///
    /// The transaction must be in raw CBOR bytes (not hex-encoded).
    pub async fn submit_tx(&self, signed_tx_cbor: &[u8]) -> Result<TxHash, BlockfrostError> {
        debug!(
            tx_size = signed_tx_cbor.len(),
            "Submitting transaction to Blockfrost"
        );

        let response = self
            .request(reqwest::Method::POST, "/tx/submit")
            .header("Content-Type", "application/cbor")
            .body(signed_tx_cbor.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        // Blockfrost returns the tx hash as a JSON string
        let tx_hash: String = response.json().await.map_err(|e| {
            BlockfrostError::Deserialization(format!("Failed to parse submit response: {e}"))
        })?;

        debug!(tx_hash, "Transaction submitted successfully");
        Ok(TxHash(tx_hash))
    }

    /// Check the confirmation status of a transaction.
    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus, BlockfrostError> {
        debug!(tx_hash, "Checking transaction status");

        let response = self
            .request(reqwest::Method::GET, &format!("/txs/{tx_hash}"))
            .send()
            .await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                // Transaction not yet on-chain
                return Ok(TxStatus::Pending);
            }
            return Err(Self::handle_error(response).await);
        }

        let tx: BlockfrostTx = response.json().await.map_err(|e| {
            BlockfrostError::Deserialization(format!("Failed to parse TX response: {e}"))
        })?;

        match (tx.block_height, tx.block) {
            (Some(height), Some(hash)) => Ok(TxStatus::Confirmed {
                block_height: height,
                block_hash: hash,
            }),
            _ => {
                warn!(tx_hash, "Transaction found but missing block info");
                Ok(TxStatus::Pending)
            }
        }
    }

    /// Get UTxOs held at a script address.
    ///
    /// This queries by address (not script hash), using the script's address on-chain.
    /// Handles pagination automatically.
    pub async fn get_script_utxos(
        &self,
        script_address: &str,
    ) -> Result<Vec<UTxO>, BlockfrostError> {
        // Script UTxOs are just UTxOs at the script's address
        self.get_utxos(script_address).await
    }

    /// Get the latest protocol parameters for transaction building.
    pub async fn get_protocol_parameters(
        &self,
    ) -> Result<ProtocolParameters, BlockfrostError> {
        debug!("Fetching latest protocol parameters");

        let response = self
            .request(reqwest::Method::GET, "/epochs/latest/parameters")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        let params: BlockfrostProtocolParams = response.json().await.map_err(|e| {
            BlockfrostError::Deserialization(format!(
                "Failed to parse protocol parameters: {e}"
            ))
        })?;

        // Blockfrost returns some fields as strings, some as numbers.
        // We parse them flexibly.
        let parse_u64 = |v: &serde_json::Value, name: &str| -> u64 {
            match v {
                serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
                serde_json::Value::String(s) => s.parse().unwrap_or_else(|_| {
                    warn!(field = name, value = %s, "Failed to parse protocol param as u64");
                    0
                }),
                _ => 0,
            }
        };

        // Price fields can be floats in Blockfrost responses.
        // We convert to numerator/denominator (multiply by 10^8 for precision).
        let parse_price = |v: &Option<serde_json::Value>| -> (u64, u64) {
            match v {
                Some(serde_json::Value::Number(n)) => {
                    let f = n.as_f64().unwrap_or(0.0);
                    // Express as integer ratio: (f * 10^8) / 10^8
                    ((f * 100_000_000.0) as u64, 100_000_000)
                }
                Some(serde_json::Value::String(s)) => {
                    let f: f64 = s.parse().unwrap_or(0.0);
                    ((f * 100_000_000.0) as u64, 100_000_000)
                }
                _ => (0, 1),
            }
        };

        let (price_step_num, price_step_den) = parse_price(&params.price_step);
        let (price_mem_num, price_mem_den) = parse_price(&params.price_mem);

        Ok(ProtocolParameters {
            min_fee_a: parse_u64(&params.min_fee_a, "min_fee_a"),
            min_fee_b: parse_u64(&params.min_fee_b, "min_fee_b"),
            max_tx_size: parse_u64(&params.max_tx_size, "max_tx_size"),
            min_utxo_value: parse_u64(&params.coins_per_utxo_size, "coins_per_utxo_size"),
            key_deposit: parse_u64(&params.key_deposit, "key_deposit"),
            pool_deposit: parse_u64(&params.pool_deposit, "pool_deposit"),
            max_val_size: parse_u64(&params.max_val_size, "max_val_size"),
            collateral_percent: parse_u64(&params.collateral_percent, "collateral_percent"),
            price_step_numerator: price_step_num,
            price_step_denominator: price_step_den,
            price_mem_numerator: price_mem_num,
            price_mem_denominator: price_mem_den,
            coins_per_utxo_byte: parse_u64(&params.coins_per_utxo_size, "coins_per_utxo_size"),
        })
    }
}
