//! MLS client wrapper. Owns the per-user OpenMLS provider plus every group
//! this user is currently a member of.
//!
//! Groups live in an in-memory `HashMap` for now; they'll migrate to an
//! IndexedDB-backed storage provider in the keystore-persistence slice.

use std::collections::HashMap;

use openmls::prelude::tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use serde::{Deserialize, Serialize};

use super::DEFAULT_CIPHERSUITE;

/// Serialized form of a client, wrapped under the PIN-derived key.
///
/// Everything OpenMLS needs lives in its MemoryStorage (including the
/// signer private key, which we wrote during `new()`). We serialize the
/// storage as a flat list of `(key, value)` pairs rather than a `HashMap`
/// because JSON object keys have to be strings — the OpenMLS keys are raw
/// bytes and won't round-trip through `serde_json` as a map.
///
/// `sent_messages` is the client-side record of Application messages we
/// sent in each group. MLS refuses to decrypt own sends (`CannotDecryptOwn
/// Message`), so we must keep our own history here; otherwise sent messages
/// disappear on refresh.
#[derive(Serialize, Deserialize)]
pub struct SerializedClient {
    pub identity: Vec<u8>,
    pub signer_public: Vec<u8>,
    pub storage: Vec<(Vec<u8>, Vec<u8>)>,
    pub group_ids: Vec<Vec<u8>>,
    /// Both sent and received plaintext, kept for display across refreshes.
    #[serde(default, alias = "sent_messages")]
    pub messages: Vec<StoredSentMessage>,
    /// Per-group highest server message id we've already processed. Prevents
    /// re-feeding consumed ratchet keys (SecretReuseError) on rehydrate.
    #[serde(default)]
    pub cursors: Vec<(Vec<u8>, i64)>,
}

/// A message (sent or received) in plaintext form. Lives only on the owner's
/// device, wrapped under their PIN in the keystore blob.
///
/// We persist RECEIVED messages too — not just sent — because each MLS
/// Application message can only be decrypted ONCE. The ratchet tree advances
/// after every decrypt, and re-feeding the same ciphertext later raises a
/// `SecretReuseError`. So if we didn't store the plaintext, refreshing the
/// page would either lose history (treat error as skip) or crash the
/// decryption (feed the ciphertext back in). Persisting plaintext + the
/// poll cursor avoids both.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSentMessage {
    pub mls_group_id: Vec<u8>,
    /// User id of the sender — either ourselves or a peer.
    #[serde(default)]
    pub sender_user_id: String,
    pub plaintext: String,
    pub created_at: String,
    /// Server sequence number if known. `None` for optimistic sends before
    /// the poll catches up.
    #[serde(default)]
    pub server_id: Option<i64>,
}

pub struct MlsClient {
    provider: OpenMlsRustCrypto,
    signer: SignatureKeyPair,
    credential: CredentialWithKey,
    identity: Vec<u8>,
    groups: HashMap<Vec<u8>, MlsGroup>,
    messages: Vec<StoredSentMessage>,
    cursors: HashMap<Vec<u8>, i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum MlsClientError {
    #[error("key generation failed: {0}")]
    KeyGen(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("mls protocol error: {0}")]
    Protocol(String),
    #[error("serialization error: {0}")]
    Serde(String),
    #[error("unknown group")]
    UnknownGroup,
    #[error("received unexpected MLS message body")]
    UnexpectedBody,
    #[error("key package validation failed: {0}")]
    KeyPackageValidation(String),
    /// OpenMLS refuses to decrypt a message this client just sent — expected
    /// when polling the group history and seeing our own ciphertext come back.
    #[error("own message")]
    OwnMessage,
    /// MLS's hash ratchet already advanced past this message — meaning we've
    /// decrypted it before. Expected if the client was restored from a
    /// snapshot that already processed the message.
    #[error("already processed")]
    AlreadyProcessed,
}

/// The trio of blobs the server needs when Alice creates a group with Bob:
/// the MLS group ID (for lookup), the Welcome (to hand to Bob), and the Commit
/// (which future members would apply — stored for consistency even in a 1:1
/// case).
pub struct GroupCreationPayload {
    pub mls_group_id: Vec<u8>,
    pub welcome: Vec<u8>,
    #[allow(dead_code)]
    pub commit: Vec<u8>,
}

impl MlsClient {
    /// Create a fresh client identity. `identity_bytes` is an opaque label;
    /// we use the IdeaForge user id so incoming messages can be attributed
    /// and so the credential survives a round-trip with the server.
    pub fn new(identity_bytes: Vec<u8>) -> Result<Self, MlsClientError> {
        let provider = OpenMlsRustCrypto::default();

        let basic = BasicCredential::new(identity_bytes.clone());
        let signer = SignatureKeyPair::new(DEFAULT_CIPHERSUITE.signature_algorithm())
            .map_err(|e| MlsClientError::KeyGen(format!("{e:?}")))?;
        signer
            .store(provider.storage())
            .map_err(|e| MlsClientError::Storage(format!("{e:?}")))?;

        let credential = CredentialWithKey {
            credential: basic.into(),
            signature_key: signer.public().into(),
        };

        Ok(Self {
            provider,
            signer,
            credential,
            identity: identity_bytes,
            groups: HashMap::new(),
            messages: Vec::new(),
            cursors: HashMap::new(),
        })
    }

    /// Dump the full client state to a serializable form. Call after any
    /// state-changing operation so the keystore blob stays in sync.
    pub fn to_serialized(&self) -> Result<SerializedClient, MlsClientError> {
        let storage_guard = self
            .provider
            .storage()
            .values
            .read()
            .map_err(|e| MlsClientError::Storage(format!("lock poisoned: {e}")))?;
        let storage: Vec<(Vec<u8>, Vec<u8>)> = storage_guard
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok(SerializedClient {
            identity: self.identity.clone(),
            signer_public: self.signer.public().to_vec(),
            storage,
            group_ids: self.groups.keys().cloned().collect(),
            messages: self.messages.clone(),
            cursors: self.cursors.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        })
    }

    /// Rebuild a client from a previously serialized state.
    pub fn restore(serialized: SerializedClient) -> Result<Self, MlsClientError> {
        let SerializedClient {
            identity,
            signer_public,
            storage,
            group_ids,
            messages,
            cursors,
        } = serialized;
        let cursors: HashMap<Vec<u8>, i64> = cursors.into_iter().collect();

        let provider = OpenMlsRustCrypto::default();
        {
            let mut values = provider
                .storage()
                .values
                .write()
                .map_err(|e| MlsClientError::Storage(format!("lock poisoned: {e}")))?;
            values.clear();
            for (k, v) in storage {
                values.insert(k, v);
            }
        }

        let signer = SignatureKeyPair::read(
            provider.storage(),
            &signer_public,
            DEFAULT_CIPHERSUITE.signature_algorithm(),
        )
        .ok_or_else(|| MlsClientError::Storage("signer not found in restored storage".into()))?;

        let basic = BasicCredential::new(identity.clone());
        let credential = CredentialWithKey {
            credential: basic.into(),
            signature_key: signer.public().into(),
        };

        let mut groups = HashMap::new();
        for gid_bytes in group_ids {
            let gid = GroupId::from_slice(&gid_bytes);
            if let Some(group) = MlsGroup::load(provider.storage(), &gid)
                .map_err(|e| MlsClientError::Storage(format!("load group: {e:?}")))?
            {
                groups.insert(gid_bytes, group);
            }
        }

        Ok(Self {
            provider,
            signer,
            credential,
            identity,
            groups,
            messages,
            cursors,
        })
    }

    /// Record a message this client just sent. `sender_user_id` is the
    /// caller's own id (used for consistent rendering on refresh).
    pub fn remember_sent(
        &mut self,
        mls_group_id: &[u8],
        sender_user_id: String,
        plaintext: String,
    ) {
        self.messages.push(StoredSentMessage {
            mls_group_id: mls_group_id.to_vec(),
            sender_user_id,
            plaintext,
            created_at: chrono::Utc::now().to_rfc3339(),
            server_id: None,
        });
    }

    /// Record a decrypted message from a peer. Idempotent: if we already have
    /// this `server_id`, we skip.
    pub fn remember_received(
        &mut self,
        mls_group_id: &[u8],
        sender_user_id: String,
        plaintext: String,
        server_id: i64,
        created_at: String,
    ) {
        let already = self
            .messages
            .iter()
            .any(|m| m.server_id == Some(server_id) && m.mls_group_id == mls_group_id);
        if already {
            return;
        }
        self.messages.push(StoredSentMessage {
            mls_group_id: mls_group_id.to_vec(),
            sender_user_id,
            plaintext,
            created_at,
            server_id: Some(server_id),
        });
    }

    /// All messages for a group in the order they were appended (which, for
    /// received messages, matches server id order; for sent messages, matches
    /// send time).
    pub fn messages_for(&self, mls_group_id: &[u8]) -> impl Iterator<Item = &StoredSentMessage> {
        self.messages
            .iter()
            .filter(move |m| m.mls_group_id == mls_group_id)
    }

    pub fn cursor(&self, mls_group_id: &[u8]) -> i64 {
        self.cursors.get(mls_group_id).copied().unwrap_or(0)
    }

    pub fn set_cursor(&mut self, mls_group_id: &[u8], to: i64) {
        self.cursors.insert(mls_group_id.to_vec(), to);
    }

    /// Generate a fresh KeyPackage ready to serialize and publish.
    pub fn generate_key_package(&self) -> Result<KeyPackageBundle, MlsClientError> {
        KeyPackage::builder()
            .build(
                DEFAULT_CIPHERSUITE,
                &self.provider,
                &self.signer,
                self.credential.clone(),
            )
            .map_err(|e| MlsClientError::Protocol(format!("{e:?}")))
    }

    pub fn serialize_key_package(
        &self,
        bundle: &KeyPackageBundle,
    ) -> Result<Vec<u8>, MlsClientError> {
        bundle
            .key_package()
            .tls_serialize_detached()
            .map_err(|e| MlsClientError::Serde(format!("{e:?}")))
    }

    /// Validate and deserialize a peer's KeyPackage bytes.
    pub fn import_peer_key_package(&self, bytes: &[u8]) -> Result<KeyPackage, MlsClientError> {
        let kp_in = KeyPackageIn::tls_deserialize_exact(bytes)
            .map_err(|e| MlsClientError::Serde(format!("{e:?}")))?;
        kp_in
            .validate(self.provider.crypto(), ProtocolVersion::Mls10)
            .map_err(|e| MlsClientError::KeyPackageValidation(format!("{e:?}")))
    }

    /// Create a new MLS group with `others` as initial members. Returns the
    /// payload the caller ships to the server's `POST /mls/groups` endpoint.
    pub fn create_group_with(
        &mut self,
        others: Vec<KeyPackage>,
    ) -> Result<GroupCreationPayload, MlsClientError> {
        // Keep the ratchet tree inside the Welcome so joiners don't need a
        // separate delivery channel for it.
        let config = MlsGroupCreateConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();

        let mut group = MlsGroup::new(
            &self.provider,
            &self.signer,
            &config,
            self.credential.clone(),
        )
        .map_err(|e| MlsClientError::Protocol(format!("new group: {e:?}")))?;

        let (commit, welcome, _group_info) = group
            .add_members(&self.provider, &self.signer, &others)
            .map_err(|e| MlsClientError::Protocol(format!("add_members: {e:?}")))?;

        group
            .merge_pending_commit(&self.provider)
            .map_err(|e| MlsClientError::Protocol(format!("merge_pending_commit: {e:?}")))?;

        let welcome_bytes = welcome
            .tls_serialize_detached()
            .map_err(|e| MlsClientError::Serde(format!("welcome: {e:?}")))?;
        let commit_bytes = commit
            .tls_serialize_detached()
            .map_err(|e| MlsClientError::Serde(format!("commit: {e:?}")))?;

        let mls_group_id = group.group_id().to_vec();
        self.groups.insert(mls_group_id.clone(), group);

        Ok(GroupCreationPayload {
            mls_group_id,
            welcome: welcome_bytes,
            commit: commit_bytes,
        })
    }

    /// Accept a Welcome ciphertext into the client's group state. Returns
    /// the MLS group ID the user just joined.
    pub fn accept_welcome(&mut self, welcome_bytes: &[u8]) -> Result<Vec<u8>, MlsClientError> {
        let msg = MlsMessageIn::tls_deserialize_exact(welcome_bytes)
            .map_err(|e| MlsClientError::Serde(format!("welcome: {e:?}")))?;
        let welcome = match msg.extract() {
            MlsMessageBodyIn::Welcome(w) => w,
            _ => return Err(MlsClientError::UnexpectedBody),
        };
        let staged = StagedWelcome::new_from_welcome(
            &self.provider,
            &MlsGroupJoinConfig::default(),
            welcome,
            None,
        )
        .map_err(|e| MlsClientError::Protocol(format!("StagedWelcome: {e:?}")))?;
        let group = staged
            .into_group(&self.provider)
            .map_err(|e| MlsClientError::Protocol(format!("into_group: {e:?}")))?;

        let mls_group_id = group.group_id().to_vec();
        self.groups.insert(mls_group_id.clone(), group);
        Ok(mls_group_id)
    }

    /// Encrypt a plaintext to an existing group; return bytes to post to the
    /// server.
    pub fn encrypt(
        &mut self,
        mls_group_id: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, MlsClientError> {
        let group = self
            .groups
            .get_mut(mls_group_id)
            .ok_or(MlsClientError::UnknownGroup)?;
        let msg = group
            .create_message(&self.provider, &self.signer, plaintext)
            .map_err(|e| MlsClientError::Protocol(format!("create_message: {e:?}")))?;
        msg.tls_serialize_detached()
            .map_err(|e| MlsClientError::Serde(format!("serialize: {e:?}")))
    }

    /// Process an inbound ciphertext. Returns `Some(plaintext)` for
    /// Application messages, `None` for protocol-internal messages (Commits,
    /// Proposals) that still need to be applied to the group state but don't
    /// surface content to the user.
    pub fn decrypt(
        &mut self,
        mls_group_id: &[u8],
        ciphertext: &[u8],
    ) -> Result<Option<Vec<u8>>, MlsClientError> {
        let group = self
            .groups
            .get_mut(mls_group_id)
            .ok_or(MlsClientError::UnknownGroup)?;

        let msg = MlsMessageIn::tls_deserialize_exact(ciphertext)
            .map_err(|e| MlsClientError::Serde(format!("deserialize: {e:?}")))?;
        let protocol_msg: ProtocolMessage = match msg.extract() {
            MlsMessageBodyIn::PrivateMessage(m) => m.into(),
            MlsMessageBodyIn::PublicMessage(m) => m.into(),
            _ => return Err(MlsClientError::UnexpectedBody),
        };

        // Skip messages that were authored by this client's own epoch —
        // OpenMLS doesn't let you process your own sends.
        if protocol_msg.group_id() != group.group_id() {
            return Err(MlsClientError::UnknownGroup);
        }

        let processed = group
            .process_message(&self.provider, protocol_msg)
            .map_err(|e| {
                let msg = format!("{e:?}");
                if msg.contains("CannotDecryptOwnMessage") {
                    MlsClientError::OwnMessage
                } else if msg.contains("SecretReuseError") || msg.contains("SecretTreeError") {
                    // Hash ratchet has already advanced past this message (we've
                    // seen it before). Silently skip — the stored plaintext
                    // remains from the first processing.
                    MlsClientError::AlreadyProcessed
                } else {
                    MlsClientError::Protocol(format!("process_message: {msg}"))
                }
            })?;

        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(app) => Ok(Some(app.into_bytes())),
            ProcessedMessageContent::StagedCommitMessage(staged) => {
                group
                    .merge_staged_commit(&self.provider, *staged)
                    .map_err(|e| MlsClientError::Protocol(format!("merge_staged_commit: {e:?}")))?;
                Ok(None)
            }
            ProcessedMessageContent::ProposalMessage(_)
            | ProcessedMessageContent::ExternalJoinProposalMessage(_) => Ok(None),
        }
    }

    pub fn has_group(&self, mls_group_id: &[u8]) -> bool {
        self.groups.contains_key(mls_group_id)
    }
}

pub use openmls::prelude::KeyPackageBundle;
