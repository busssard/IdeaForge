//! IdeaForge Blockchain - Cardano integration for pledge-to-buy mechanics.
//!
//! This crate provides:
//! - **Blockfrost API client** for querying UTxOs, submitting transactions, and
//!   monitoring confirmations without running a full Cardano node.
//! - **Pledge service** for building, submitting, and tracking escrow pledge
//!   transactions on-chain.
//! - **Type definitions** for Cardano primitives (transactions, UTxOs, addresses,
//!   protocol parameters).
//!
//! The companion Aiken smart contract lives in `contracts/escrow/` and defines the
//! on-chain escrow validator that locks pledged ADA until release conditions are met.
//!
//! ## Architecture
//!
//! ```text
//! Frontend (CIP-30 wallet) <-> Backend (this crate) <-> Blockfrost API <-> Cardano
//! ```
//!
//! The backend never holds private keys. All transaction signing happens client-side
//! via CIP-30 compatible wallets (Nami, Eternl, Lace, etc.).
//!
//! ## Network Configuration
//!
//! Use `Network::Preview` for development and testing, `Network::Preprod` for staging,
//! and `Network::Mainnet` for production (after security audit).
//!
//! See: `docs/architecture/blockchain_integration.md` for full design documentation.

pub mod blockfrost;
pub mod pledge;
pub mod types;

pub use blockfrost::{BlockfrostClient, BlockfrostError};
pub use pledge::{PledgeError, PledgeService};
pub use types::*;
