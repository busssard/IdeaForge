//! MLS (RFC 9420) end-to-end encrypted messaging — browser client.
//!
//! The delivery service in `ideaforge-api::handlers::mls` never sees plaintext;
//! all crypto happens here in WASM. See
//! `docs/architecture/simplex_messaging_spike.md` §§13–15 for the design.
//!
//! **Phase-1 scope:** identity creation, KeyPackage generation, and the API
//! glue to publish/consume them. Group creation, message send/receive, and
//! persistent keystore land in the next slice.

pub mod api;
pub mod client;
pub mod crypto;
pub mod keystore;

/// Default ciphersuite for Phase 1 — the standard, widely-supported
/// X25519 + Ed25519 + ChaCha20-Poly1305 combination.
///
/// We'll swap this for `MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519`
/// (ML-KEM + X25519 post-quantum hybrid, code point `0x004D`) once we swap
/// the crypto provider from `openmls_rust_crypto` to `openmls_libcrux_crypto`
/// in a later slice. Tracked in tasks.md.
pub const DEFAULT_CIPHERSUITE: openmls::prelude::Ciphersuite =
    openmls::prelude::Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
