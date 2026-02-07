use serde::{Deserialize, Serialize};

/// A Cardano transaction hash (hex-encoded, 64 characters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxHash(pub String);

/// A Cardano address (bech32-encoded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardanoAddress(pub String);

/// An unsigned transaction in CBOR hex format, ready for client-side signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub cbor_hex: String,
    pub estimated_fee_lovelace: u64,
}

/// Status of a submitted transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Pending,
    Confirmed { confirmations: u32 },
    Failed { reason: String },
}

/// A UTxO (Unspent Transaction Output) from the Cardano chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTxO {
    pub tx_hash: String,
    pub output_index: u32,
    pub amount_lovelace: u64,
    pub address: String,
}
