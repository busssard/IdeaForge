//! IdeaForge Blockchain - Cardano integration for pledge-to-buy mechanics.
//!
//! **STATUS: DEFERRED TO PHASE 2-3**
//!
//! This crate is a stub. Cardano blockchain integration (Aiken smart contracts,
//! Blockfrost API, CIP-30 wallet connectors, pledge escrow) is deferred until
//! after MVP launch and product-market fit validation.
//!
//! Phase 2: Stripe escrow (fiat pledges)
//! Phase 3: Cardano on-chain pledges with Aiken smart contracts
//!
//! See: docs/architecture/blockchain_integration.md (long-term design)
//! See: docs/architecture/mvp_architecture.md (current scope)

// Submodules preserved as stubs for future restoration
pub mod blockfrost;
pub mod pledge;
pub mod types;
