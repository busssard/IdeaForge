//! PIN-derived key material for the keystore.
//!
//! Given a 6-digit PIN and a per-user random salt, we derive two independent
//! 32-byte keys via Argon2id:
//!   - `wrap_key`    → AES-256-GCM key used to seal/open the serialized MLS state
//!   - `verifier`   → opaque bytes shipped to the server as proof of PIN knowledge
//!
//! Keeping the two derivations independent (distinct `info` strings) means a
//! server holding the verifier learns nothing about the wrap_key beyond the
//! underlying PIN.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};

/// Argon2id parameters. **Matches OWASP's 2024 minimum recommendation**
/// (m=19 MiB, t=2, p=1) rounded up on t-cost. 64 MiB was causing Chrome to
/// OOM-crash when WASM's linear memory couldn't grow contiguously. Change
/// carefully — different parameters produce different keys from the same PIN.
///
/// Offline attack cost for a 6-digit PIN (~10^6 guesses) at these params:
/// roughly 80-100 core-hours on current hardware. Not uncrackable, but
/// gated behind the server-enforced 3-attempts-per-hour rate limit for
/// online attacks.
const ARGON2_MEM_KIB: u32 = 19 * 1024; // 19 MiB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;
const OUTPUT_LEN: usize = 32;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("invalid argon2 parameters: {0}")]
    Params(String),
    #[error("argon2 derivation failed: {0}")]
    Derive(String),
    #[error("aes-gcm seal/open failed: {0}")]
    Aead(String),
    #[error("ciphertext too short")]
    ShortCiphertext,
}

#[derive(Debug, Clone)]
pub struct DerivedKeys {
    pub wrap_key: [u8; OUTPUT_LEN],
    pub verifier: [u8; OUTPUT_LEN],
}

fn argon2() -> Result<Argon2<'static>, CryptoError> {
    let params = Params::new(
        ARGON2_MEM_KIB,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(OUTPUT_LEN),
    )
    .map_err(|e| CryptoError::Params(e.to_string()))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Derive `wrap_key` and `verifier` from a PIN. Uses the salt verbatim; a
/// short domain-separation tag is appended to the PIN bytes before each
/// derivation so the two outputs are guaranteed independent even when the
/// PIN and salt are identical.
pub fn derive_keys(pin: &str, salt: &[u8]) -> Result<DerivedKeys, CryptoError> {
    if salt.len() < SALT_LEN {
        return Err(CryptoError::Params(format!(
            "salt must be >={} bytes",
            SALT_LEN
        )));
    }
    let a2 = argon2()?;

    let mut wrap_key = [0u8; OUTPUT_LEN];
    let mut verifier = [0u8; OUTPUT_LEN];

    let mut wrap_input = Vec::with_capacity(pin.len() + 8);
    wrap_input.extend_from_slice(pin.as_bytes());
    wrap_input.extend_from_slice(b":wrap");
    a2.hash_password_into(&wrap_input, salt, &mut wrap_key)
        .map_err(|e| CryptoError::Derive(e.to_string()))?;

    let mut ver_input = Vec::with_capacity(pin.len() + 8);
    ver_input.extend_from_slice(pin.as_bytes());
    ver_input.extend_from_slice(b":verify");
    a2.hash_password_into(&ver_input, salt, &mut verifier)
        .map_err(|e| CryptoError::Derive(e.to_string()))?;

    Ok(DerivedKeys { wrap_key, verifier })
}

/// Generate a random 16-byte salt using the browser's secure RNG.
pub fn fresh_salt() -> [u8; SALT_LEN] {
    use rand::RngCore;
    let mut buf = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

/// AES-256-GCM seal. Output format: `nonce || ciphertext || tag` (nonce is
/// the first `NONCE_LEN` bytes; aes-gcm puts the tag at the end of
/// ciphertext implicitly).
pub fn seal(wrap_key: &[u8; OUTPUT_LEN], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use rand::RngCore;
    let cipher =
        Aes256Gcm::new_from_slice(wrap_key).map_err(|e| CryptoError::Aead(format!("key: {e}")))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Aead(format!("encrypt: {e}")))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn open(wrap_key: &[u8; OUTPUT_LEN], wrapped: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if wrapped.len() < NONCE_LEN + 16 {
        return Err(CryptoError::ShortCiphertext);
    }
    let cipher =
        Aes256Gcm::new_from_slice(wrap_key).map_err(|e| CryptoError::Aead(format!("key: {e}")))?;
    let (nonce_bytes, ct) = wrapped.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ct)
        .map_err(|e| CryptoError::Aead(format!("decrypt: {e}")))
}
