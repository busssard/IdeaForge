//! Keystore API + setup/unlock/persist flow. Lives on top of `client`
//! (`to_serialized` / `restore`) and `crypto` (PIN → keys, AES-GCM wrap).
//!
//! The server never sees the PIN, the wrap key, or the plaintext MLS state —
//! only the per-user salt, a verifier bytestring, and the opaque wrapped
//! blob.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::api::client::{self, ApiError};
use crate::mls::client::{MlsClient, SerializedClient};
use crate::mls::crypto::{self, DerivedKeys};

#[derive(Debug, Clone, Deserialize)]
pub struct StatusResponse {
    pub exists: bool,
    #[serde(default)]
    pub salt_b64: Option<String>,
    #[serde(default)]
    pub locked_until_ms: Option<i64>,
    #[serde(default)]
    pub failed_attempts: i32,
}

#[derive(Debug, Clone, Serialize)]
struct SetupRequest {
    salt_b64: String,
    verifier_b64: String,
    wrapped_blob_b64: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateRequest {
    verifier_b64: String,
    wrapped_blob_b64: String,
}

#[derive(Debug, Clone, Serialize)]
struct UnlockRequest {
    verifier_b64: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UnlockResponse {
    wrapped_blob_b64: String,
    #[allow(dead_code)]
    salt_b64: String,
}

pub async fn status() -> Result<StatusResponse, ApiError> {
    client::get("/api/v1/mls/keystore/status").await
}

/// First-time setup. Caller supplies a freshly-generated `MlsClient` plus
/// the user's chosen PIN. We generate a salt, derive keys, wrap, and POST.
pub async fn setup(pin: &str, client_state: &MlsClient) -> Result<DerivedKeys, ApiError> {
    let salt = crypto::fresh_salt();
    let keys = crypto::derive_keys(pin, &salt).map_err(crypto_to_api)?;
    let serialized = client_state.to_serialized().map_err(client_to_api)?;
    let plaintext = serde_json::to_vec(&serialized).map_err(serde_to_api)?;
    let wrapped = crypto::seal(&keys.wrap_key, &plaintext).map_err(crypto_to_api)?;

    let req = SetupRequest {
        salt_b64: STANDARD.encode(salt),
        verifier_b64: STANDARD.encode(keys.verifier),
        wrapped_blob_b64: STANDARD.encode(&wrapped),
    };
    let _: serde_json::Value = client::post("/api/v1/mls/keystore", &req).await?;
    Ok(keys)
}

/// Unlock with the PIN. Returns both the decrypted client AND the derived
/// keys (so the caller can cache the wrap_key for subsequent `persist` calls).
pub async fn unlock(pin: &str) -> Result<(MlsClient, DerivedKeys), ApiError> {
    // Step 1: fetch the salt from status.
    let status = status().await?;
    if !status.exists {
        return Err(ApiError {
            status: 404,
            code: "NO_KEYSTORE".into(),
            message: "No keystore has been set up yet.".into(),
        });
    }
    let salt_b64 = status.salt_b64.ok_or_else(|| ApiError {
        status: 500,
        code: "NO_SALT".into(),
        message: "Keystore reported as existing but has no salt.".into(),
    })?;
    let salt = STANDARD.decode(&salt_b64).map_err(|e| ApiError {
        status: 500,
        code: "BAD_SALT".into(),
        message: e.to_string(),
    })?;

    let keys = crypto::derive_keys(pin, &salt).map_err(crypto_to_api)?;

    // Step 2: ship verifier to server; server either returns blob or rejects.
    let req = UnlockRequest {
        verifier_b64: STANDARD.encode(keys.verifier),
    };
    let resp: UnlockResponse = client::post("/api/v1/mls/keystore/unlock", &req).await?;

    // Step 3: decrypt the blob locally.
    let wrapped = STANDARD
        .decode(&resp.wrapped_blob_b64)
        .map_err(|e| ApiError {
            status: 500,
            code: "BAD_BLOB".into(),
            message: e.to_string(),
        })?;
    let plaintext = crypto::open(&keys.wrap_key, &wrapped).map_err(crypto_to_api)?;
    let serialized: SerializedClient =
        serde_json::from_slice(&plaintext).map_err(serde_to_api)?;
    let mls_client = MlsClient::restore(serialized).map_err(client_to_api)?;

    Ok((mls_client, keys))
}

/// Call after any state-changing MLS operation. Requires the already-derived
/// wrap_key + verifier from a prior `setup` or `unlock` (no re-prompt).
pub async fn persist(client_state: &MlsClient, keys: &DerivedKeys) -> Result<(), ApiError> {
    let serialized = client_state.to_serialized().map_err(client_to_api)?;
    let plaintext = serde_json::to_vec(&serialized).map_err(serde_to_api)?;
    let wrapped = crypto::seal(&keys.wrap_key, &plaintext).map_err(crypto_to_api)?;

    let req = UpdateRequest {
        verifier_b64: STANDARD.encode(keys.verifier),
        wrapped_blob_b64: STANDARD.encode(&wrapped),
    };
    client::put::<UpdateRequest, serde_json::Value>("/api/v1/mls/keystore", &req)
        .await
        .map(|_| ())
}

fn crypto_to_api(e: crypto::CryptoError) -> ApiError {
    ApiError {
        status: 0,
        code: "CRYPTO".into(),
        message: e.to_string(),
    }
}

fn client_to_api(e: crate::mls::client::MlsClientError) -> ApiError {
    ApiError {
        status: 0,
        code: "MLS".into(),
        message: e.to_string(),
    }
}

fn serde_to_api(e: serde_json::Error) -> ApiError {
    ApiError {
        status: 0,
        code: "SERDE".into(),
        message: e.to_string(),
    }
}
