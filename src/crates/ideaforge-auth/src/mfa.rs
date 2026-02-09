//! Multi-Factor Authentication support (TOTP and WebAuthn).
//!
//! **STATUS: DEFERRED TO PHASE 2**
//!
//! MFA is not required for MVP. This module is preserved as a stub
//! for future restoration when security hardening is prioritized.
//!
//! Phase 2: TOTP via authenticator apps
//! Phase 2+: WebAuthn / FIDO2 hardware keys
//!
//! See: docs/security/security_framework.md (long-term requirements)
//! See: docs/architecture/mvp_architecture.md (current scope)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MFA method configured for a user (Phase 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    WebAuthn,
}

/// MFA enrollment status (Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollment {
    pub user_id: Uuid,
    pub method: MfaMethod,
    pub is_active: bool,
    pub credential_id: String,
}

/// Result of an MFA verification attempt (Phase 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfaVerifyResult {
    Valid,
    Invalid,
    NotEnrolled,
}
