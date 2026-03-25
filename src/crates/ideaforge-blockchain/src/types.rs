use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A Cardano transaction hash (hex-encoded, 64 characters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxHash(pub String);

/// A Cardano address (bech32-encoded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardanoAddress(pub String);

/// An unsigned transaction in CBOR hex format, ready for client-side signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    /// Transaction CBOR in hex encoding
    pub tx_cbor_hex: String,
    /// Transaction hash (before signing)
    pub tx_hash: String,
    /// Transaction fee in lovelace
    pub fee: u64,
}

/// A UTxO (Unspent Transaction Output) from the Cardano chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTxO {
    pub tx_hash: String,
    pub output_index: u32,
    pub address: String,
    pub amount: Vec<AssetAmount>,
    pub data_hash: Option<String>,
    pub inline_datum: Option<serde_json::Value>,
}

impl UTxO {
    /// Get the total lovelace amount in this UTxO.
    pub fn lovelace_amount(&self) -> u64 {
        self.amount
            .iter()
            .find(|a| a.unit == "lovelace")
            .and_then(|a| a.quantity.parse::<u64>().ok())
            .unwrap_or(0)
    }
}

/// An asset amount (lovelace or native token).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAmount {
    /// "lovelace" or policy_id + asset_name hex
    pub unit: String,
    /// Quantity as string (to support large values)
    pub quantity: String,
}

/// Status of a submitted transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Pending,
    Confirmed {
        block_height: u64,
        block_hash: String,
    },
    Failed {
        reason: String,
    },
}

/// A pledge that has been confirmed on-chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmedPledge {
    pub pledge_id: Uuid,
    pub tx_hash: String,
    pub block_height: u64,
    pub confirmed_at: chrono::DateTime<chrono::Utc>,
}

/// Summary of a pledge campaign's on-chain status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PledgeCampaign {
    pub idea_id: Uuid,
    pub creator_address: CardanoAddress,
    pub script_address: CardanoAddress,
    pub min_target_lovelace: u64,
    pub deadline_posix: i64,
    pub total_pledged_lovelace: u64,
    pub pledge_count: u32,
}

/// Information about a connected wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: CardanoAddress,
    pub balance_lovelace: u64,
    /// Wallet provider name: "nami", "eternl", "lace", etc.
    pub wallet_name: String,
}

/// Protocol parameters needed for transaction building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParameters {
    /// Linear fee coefficient (fee = min_fee_a * tx_size + min_fee_b)
    pub min_fee_a: u64,
    /// Constant fee component
    pub min_fee_b: u64,
    /// Maximum transaction size in bytes
    pub max_tx_size: u64,
    /// Minimum UTxO value in lovelace
    pub min_utxo_value: u64,
    /// Key deposit in lovelace
    pub key_deposit: u64,
    /// Pool deposit in lovelace
    pub pool_deposit: u64,
    /// Maximum value size in bytes
    pub max_val_size: u64,
    /// Collateral percentage for Plutus scripts
    pub collateral_percent: u64,
    /// Price per Plutus execution step (numerator/denominator)
    pub price_step_numerator: u64,
    pub price_step_denominator: u64,
    /// Price per Plutus memory unit (numerator/denominator)
    pub price_mem_numerator: u64,
    pub price_mem_denominator: u64,
    /// Coins per UTxO byte
    pub coins_per_utxo_byte: u64,
}

/// Cardano network selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    /// Cardano mainnet
    Mainnet,
    /// Pre-production testnet
    Preprod,
    /// Preview testnet (fastest block times, best for development)
    Preview,
}

impl Network {
    /// Get the Blockfrost base URL for this network.
    pub fn blockfrost_base_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://cardano-mainnet.blockfrost.io/api/v0",
            Network::Preprod => "https://cardano-preprod.blockfrost.io/api/v0",
            Network::Preview => "https://cardano-preview.blockfrost.io/api/v0",
        }
    }
}
