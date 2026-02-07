//! Multi-Factor Authentication support (TOTP and WebAuthn).
//!
//! MFA is required for:
//! - Investors (mandatory for financial transactions)
//! - Entrepreneurs with secret ideas (mandatory)
//! - All users (optional but encouraged)
//!
//! Per the Security Framework (docs/security/security_framework.md), Section A07.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MFA method configured for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    /// Time-based One-Time Password (RFC 6238) via authenticator app
    Totp,
    /// WebAuthn / FIDO2 hardware key or biometric
    WebAuthn,
}

/// MFA enrollment status for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollment {
    pub user_id: Uuid,
    pub method: MfaMethod,
    pub is_active: bool,
    /// For TOTP: encrypted shared secret. For WebAuthn: credential ID.
    pub credential_id: String,
}

/// Whether MFA is required for a given action, based on user role and context.
pub fn mfa_required(is_investor: bool, has_secret_ideas: bool, is_financial_action: bool) -> bool {
    is_investor || has_secret_ideas || is_financial_action
}

/// Result of an MFA verification attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfaVerifyResult {
    /// MFA code is valid
    Valid,
    /// MFA code is invalid or expired
    Invalid,
    /// MFA is not configured for this user
    NotEnrolled,
}
