//! IdeaForge Auth - JWT tokens and password hashing.
//!
//! MVP scope: JWT access/refresh tokens + Argon2 password hashing.
//! No MFA, no OAuth2 social login, no bot API keys.
//!
//! Phase 2: Add MFA (TOTP + WebAuthn), OAuth2 (GitHub, Google)
//! Phase 2+: Add bot API key authentication

pub mod jwt;
pub mod password;
pub mod permissions;

// MFA is deferred to Phase 2. The module is preserved as a stub.
// pub mod mfa;

pub use jwt::Claims;
