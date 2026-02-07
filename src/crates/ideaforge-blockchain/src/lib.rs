//! IdeaForge Blockchain - Cardano integration for pledge-to-buy mechanics.
//!
//! Handles:
//! - Building unsigned pledge transactions
//! - Submitting signed transactions via Blockfrost
//! - Monitoring transaction confirmations
//! - Pledge escrow lifecycle (claim, refund)

pub mod blockfrost;
pub mod pledge;
pub mod types;
