//! End-to-end smoke test for the MLS delivery service.
//!
//! Runs against a live backend. Exercises the full flow:
//!   1. log in as Alice and Bob (seed accounts from the dev DB)
//!   2. both generate MLS identities and publish KeyPackages
//!   3. Alice consumes Bob's KeyPackage, creates a 1:1 group, and emits a
//!      Welcome + Commit for Bob
//!   4. Bob fetches the Welcome and imports it into his keystore
//!   5. Alice sends an encrypted Application message to the group
//!   6. Bob polls, fetches, decrypts — asserts the plaintext matches
//!
//! If any step fails, exits with a non-zero code and a clear error. If the
//! whole flow succeeds, prints "PASS" and exits 0.
//!
//! Usage (with the dev backend running on :3000):
//!   cargo run --bin mls-smoketest
//!
//! Override defaults with env vars:
//!   IDEAFORGE_API=http://localhost:3000
//!   ALICE_EMAIL=alice@example.com  ALICE_PASSWORD=Test1234!
//!   BOB_EMAIL=bob@example.com      BOB_PASSWORD=Test1234!

use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use openmls::prelude::*;
use openmls::prelude::tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::OpenMlsProvider;
use reqwest::Client as Http;
use serde::{Deserialize, Serialize};

const CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

fn unb64(s: &str) -> Result<Vec<u8>> {
    STANDARD.decode(s).context("invalid base64")
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    #[serde(default)]
    user: Option<UserPayload>,
    #[serde(default)]
    user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserPayload {
    id: String,
    #[allow(dead_code)]
    email: String,
}

struct Session {
    label: &'static str,
    token: String,
    user_id: String,
    http: Http,
    api: String,
}

impl Session {
    async fn login(
        http: Http,
        api: &str,
        label: &'static str,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        #[derive(Serialize)]
        struct Body<'a> {
            email: &'a str,
            password: &'a str,
        }
        let url = format!("{api}/api/v1/auth/login");
        let resp = http
            .post(&url)
            .json(&Body { email, password })
            .send()
            .await
            .with_context(|| format!("login request failed for {label}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("login for {label} returned {status}: {text}");
        }
        let body: LoginResponse = resp.json().await.context("login: bad JSON")?;
        let user_id = body
            .user
            .as_ref()
            .map(|u| u.id.clone())
            .or(body.user_id.clone())
            .ok_or_else(|| anyhow!("login for {label} did not return a user id"))?;
        Ok(Session {
            label,
            token: body.access_token,
            user_id,
            http,
            api: api.to_string(),
        })
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

/// Publish `count` KeyPackages for the given session. Returns the bundles kept
/// by the client so it can consult them after a peer consumes one.
async fn publish_key_packages(
    session: &Session,
    provider: &OpenMlsRustCrypto,
    signer: &SignatureKeyPair,
    credential: &CredentialWithKey,
    count: usize,
) -> Result<Vec<KeyPackageBundle>> {
    let mut bundles = Vec::with_capacity(count);
    let mut serialized = Vec::with_capacity(count);
    for _ in 0..count {
        let bundle = KeyPackage::builder()
            .build(CIPHERSUITE, provider, signer, credential.clone())
            .context("KeyPackage::builder failed")?;
        let bytes = bundle
            .key_package()
            .tls_serialize_detached()
            .context("KeyPackage serialize failed")?;
        serialized.push(b64(&bytes));
        bundles.push(bundle);
    }

    #[derive(Serialize)]
    struct Req {
        key_packages: Vec<String>,
        ttl_days: i64,
    }
    let url = format!("{}/api/v1/mls/keypackages", session.api);
    let resp = session
        .http
        .post(&url)
        .header("Authorization", session.auth())
        .json(&Req {
            key_packages: serialized,
            ttl_days: 7,
        })
        .send()
        .await
        .with_context(|| format!("publish keypackages for {}", session.label))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("publish keypackages for {} → {status}: {text}", session.label);
    }
    println!(
        "  · {} published {count} KeyPackage(s)",
        session.label
    );
    Ok(bundles)
}

async fn consume_key_package(session: &Session, target_user_id: &str) -> Result<Vec<u8>> {
    #[derive(Deserialize)]
    struct Resp {
        key_package_b64: String,
    }
    let url = format!("{}/api/v1/mls/keypackages/{target_user_id}/consume", session.api);
    let resp = session
        .http
        .post(&url)
        .header("Authorization", session.auth())
        .send()
        .await
        .context("consume keypackage request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("consume keypackage → {status}: {text}");
    }
    let r: Resp = resp.json().await.context("consume keypackage JSON")?;
    unb64(&r.key_package_b64)
}

async fn create_group(
    session: &Session,
    mls_group_id: &[u8],
    initial_members: &[String],
    welcomes: &[Vec<u8>],
) -> Result<String> {
    #[derive(Serialize)]
    struct Req<'a> {
        mls_group_id_b64: String,
        name: Option<&'a str>,
        initial_members: &'a [String],
        welcomes_b64: Vec<String>,
    }
    #[derive(Deserialize)]
    struct Resp {
        id: String,
    }
    let url = format!("{}/api/v1/mls/groups", session.api);
    let resp = session
        .http
        .post(&url)
        .header("Authorization", session.auth())
        .json(&Req {
            mls_group_id_b64: b64(mls_group_id),
            name: Some("smoketest"),
            initial_members,
            welcomes_b64: welcomes.iter().map(|w| b64(w)).collect(),
        })
        .send()
        .await
        .context("create group request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("create group → {status}: {text}");
    }
    let r: Resp = resp.json().await.context("create group JSON")?;
    Ok(r.id)
}

async fn list_welcomes(session: &Session) -> Result<Vec<(String, Vec<u8>)>> {
    #[derive(Deserialize)]
    struct Envelope {
        id: String,
        ciphertext_b64: String,
    }
    #[derive(Deserialize)]
    struct Resp {
        data: Vec<Envelope>,
    }
    let url = format!("{}/api/v1/mls/welcomes", session.api);
    let resp = session
        .http
        .get(&url)
        .header("Authorization", session.auth())
        .send()
        .await
        .context("list welcomes request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("list welcomes → {status}: {text}");
    }
    let r: Resp = resp.json().await.context("list welcomes JSON")?;
    r.data
        .into_iter()
        .map(|e| Ok((e.id, unb64(&e.ciphertext_b64)?)))
        .collect()
}

async fn purge_my_keypackages(session: &Session) -> Result<u64> {
    #[derive(Deserialize)]
    struct Resp {
        deleted: u64,
    }
    let url = format!("{}/api/v1/mls/keypackages", session.api);
    let resp = session
        .http
        .delete(&url)
        .header("Authorization", session.auth())
        .send()
        .await
        .context("purge keypackages request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("purge keypackages for {} → {status}: {text}", session.label);
    }
    let r: Resp = resp.json().await.context("purge keypackages JSON")?;
    Ok(r.deleted)
}

async fn ack_welcome(session: &Session, id: &str) -> Result<()> {
    let url = format!("{}/api/v1/mls/welcomes/{id}", session.api);
    let resp = session
        .http
        .delete(&url)
        .header("Authorization", session.auth())
        .send()
        .await
        .context("ack welcome request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("ack welcome → {status}: {text}");
    }
    Ok(())
}

async fn post_message(session: &Session, group_id: &str, ciphertext: &[u8]) -> Result<()> {
    #[derive(Serialize)]
    struct Req {
        ciphertext_b64: String,
    }
    let url = format!("{}/api/v1/mls/groups/{group_id}/messages", session.api);
    let resp = session
        .http
        .post(&url)
        .header("Authorization", session.auth())
        .json(&Req {
            ciphertext_b64: b64(ciphertext),
        })
        .send()
        .await
        .context("post message request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("post message → {status}: {text}");
    }
    Ok(())
}

async fn list_messages(
    session: &Session,
    group_id: &str,
    since: i64,
) -> Result<Vec<(i64, Vec<u8>)>> {
    #[derive(Deserialize)]
    struct Envelope {
        id: i64,
        ciphertext_b64: String,
    }
    #[derive(Deserialize)]
    struct Resp {
        data: Vec<Envelope>,
    }
    let url = format!("{}/api/v1/mls/groups/{group_id}/messages?since={since}", session.api);
    let resp = session
        .http
        .get(&url)
        .header("Authorization", session.auth())
        .send()
        .await
        .context("list messages request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("list messages → {status}: {text}");
    }
    let r: Resp = resp.json().await.context("list messages JSON")?;
    r.data
        .into_iter()
        .map(|e| Ok((e.id, unb64(&e.ciphertext_b64)?)))
        .collect()
}

struct MlsIdentity {
    provider: OpenMlsRustCrypto,
    signer: SignatureKeyPair,
    credential: CredentialWithKey,
}

fn new_identity(identity_bytes: &[u8]) -> Result<MlsIdentity> {
    let provider = OpenMlsRustCrypto::default();
    let signer = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())
        .map_err(|e| anyhow!("signer generation failed: {e:?}"))?;
    signer
        .store(provider.storage())
        .map_err(|e| anyhow!("signer store failed: {e:?}"))?;
    let basic = BasicCredential::new(identity_bytes.to_vec());
    let credential = CredentialWithKey {
        credential: basic.into(),
        signature_key: signer.public().into(),
    };
    Ok(MlsIdentity {
        provider,
        signer,
        credential,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let api = env_or("IDEAFORGE_API", "http://localhost:3000");
    let alice_email = env_or("ALICE_EMAIL", "alice@example.com");
    let alice_password = env_or("ALICE_PASSWORD", "Test1234!");
    let bob_email = env_or("BOB_EMAIL", "bob@example.com");
    let bob_password = env_or("BOB_PASSWORD", "Test1234!");

    println!("MLS smoke test — target: {api}");
    println!("-- Logging in --");
    let http = Http::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let alice = Session::login(http.clone(), &api, "alice", &alice_email, &alice_password).await?;
    let bob = Session::login(http.clone(), &api, "bob", &bob_email, &bob_password).await?;
    println!("  · alice = {}", alice.user_id);
    println!("  · bob   = {}", bob.user_id);

    println!("-- Clearing stale state (idempotency) --");
    let stale = list_welcomes(&bob).await?;
    for (id, _) in &stale {
        ack_welcome(&bob, id).await?;
    }
    if !stale.is_empty() {
        println!("  · acked {} stale Welcome(s)", stale.len());
    }
    // Older KeyPackages from previous runs tie to long-gone provider storage,
    // so each run must start with a clean slate.
    let alice_purged = purge_my_keypackages(&alice).await?;
    let bob_purged = purge_my_keypackages(&bob).await?;
    if alice_purged + bob_purged > 0 {
        println!(
            "  · purged {alice_purged} alice + {bob_purged} bob stale KeyPackages"
        );
    }

    println!("-- Generating MLS identities --");
    let alice_id = new_identity(alice.user_id.as_bytes())?;
    let bob_id = new_identity(bob.user_id.as_bytes())?;

    println!("-- Publishing KeyPackages --");
    let _alice_bundles = publish_key_packages(
        &alice,
        &alice_id.provider,
        &alice_id.signer,
        &alice_id.credential,
        2,
    )
    .await?;
    let bob_bundles = publish_key_packages(
        &bob,
        &bob_id.provider,
        &bob_id.signer,
        &bob_id.credential,
        2,
    )
    .await?;

    println!("-- Alice consumes one of Bob's KeyPackages --");
    let bob_kp_bytes = consume_key_package(&alice, &bob.user_id).await?;

    // Sanity check: assert the bytes that came back from the server match one
    // of the bundles Bob generated locally. If this fails, the server is
    // corrupting KeyPackage bytes and the test should abort here.
    let bob_local_bytes: Vec<Vec<u8>> = bob_bundles
        .iter()
        .map(|b| {
            b.key_package()
                .tls_serialize_detached()
                .map_err(|e| anyhow!("serialize: {e:?}"))
        })
        .collect::<Result<_>>()?;
    let matched = bob_local_bytes.iter().any(|b| b == &bob_kp_bytes);
    if !matched {
        bail!(
            "server-returned KeyPackage ({} bytes) doesn't byte-match any of Bob's \
             locally generated bundles — server is corrupting bytes",
            bob_kp_bytes.len()
        );
    }
    println!(
        "  · Bob's KeyPackage consumed ({} bytes) — byte-identical to local",
        bob_kp_bytes.len()
    );

    let bob_kp_in = KeyPackageIn::tls_deserialize_exact(&bob_kp_bytes)
        .context("deserialize Bob's KeyPackage")?;
    let bob_kp = bob_kp_in
        .validate(alice_id.provider.crypto(), ProtocolVersion::Mls10)
        .map_err(|e| anyhow!("KeyPackage validation failed: {e:?}"))?;
    drop(bob_bundles);

    println!("-- Alice creates a 1:1 group and generates Bob's Welcome --");
    // Include the ratchet tree as an extension inside the Welcome so the
    // recipient can construct the full group state without an out-of-band
    // delivery of the tree. Without this, `StagedWelcome::new_from_welcome`
    // fails with `MissingRatchetTree`.
    let group_config = MlsGroupCreateConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();
    let mut alice_group = MlsGroup::new(
        &alice_id.provider,
        &alice_id.signer,
        &group_config,
        alice_id.credential.clone(),
    )
    .map_err(|e| anyhow!("MlsGroup::new failed: {e:?}"))?;
    let (_commit, welcome_out, _group_info) = alice_group
        .add_members(&alice_id.provider, &alice_id.signer, &[bob_kp])
        .map_err(|e| anyhow!("add_members failed: {e:?}"))?;
    // Alice must merge her own pending commit before the group is usable.
    alice_group
        .merge_pending_commit(&alice_id.provider)
        .map_err(|e| anyhow!("merge_pending_commit failed: {e:?}"))?;

    let welcome_bytes = welcome_out
        .tls_serialize_detached()
        .context("serialize Welcome")?;
    let mls_group_id = alice_group.group_id().to_vec();

    let server_group_id = create_group(
        &alice,
        &mls_group_id,
        &[bob.user_id.clone()],
        &[welcome_bytes],
    )
    .await?;
    println!("  · Server group id = {server_group_id}");

    println!("-- Bob fetches the Welcome and joins --");
    let welcomes = list_welcomes(&bob).await?;
    if welcomes.is_empty() {
        bail!("Bob saw no Welcomes — delivery-service bug");
    }
    println!("  · Bob sees {} pending Welcome(s)", welcomes.len());
    let (welcome_id, welcome_ciphertext) = welcomes.into_iter().next().unwrap();

    let welcome_msg = MlsMessageIn::tls_deserialize_exact(&welcome_ciphertext)
        .context("deserialize Welcome")?;
    let welcome_in = match welcome_msg.extract() {
        MlsMessageBodyIn::Welcome(w) => w,
        other => bail!("Bob received a non-Welcome message: {:?}", other),
    };
    let staged = StagedWelcome::new_from_welcome(
        &bob_id.provider,
        &MlsGroupJoinConfig::default(),
        welcome_in,
        None, // ratchet tree will be extracted from the Welcome
    )
    .map_err(|e| anyhow!("StagedWelcome::new_from_welcome failed: {e:?}"))?;
    let mut bob_group = staged
        .into_group(&bob_id.provider)
        .map_err(|e| anyhow!("StagedWelcome::into_group failed: {e:?}"))?;
    ack_welcome(&bob, &welcome_id).await?;
    println!("  · Bob joined the group, group_id_matches = {}",
        bob_group.group_id().as_slice() == mls_group_id);

    if bob_group.group_id().as_slice() != mls_group_id {
        bail!("group id mismatch between Alice and Bob");
    }

    println!("-- Alice sends an encrypted message --");
    let plaintext = b"Hello, end-to-end world.";
    let app_msg = alice_group
        .create_message(&alice_id.provider, &alice_id.signer, plaintext)
        .map_err(|e| anyhow!("create_message failed: {e:?}"))?;
    let app_bytes = app_msg
        .tls_serialize_detached()
        .context("serialize Application message")?;
    post_message(&alice, &server_group_id, &app_bytes).await?;

    println!("-- Bob polls and decrypts --");
    let messages = list_messages(&bob, &server_group_id, 0).await?;
    if messages.is_empty() {
        bail!("Bob saw no messages — delivery-service bug");
    }
    println!("  · Bob received {} message(s)", messages.len());
    let (_msg_id, msg_bytes) = messages.into_iter().next().unwrap();
    let incoming = MlsMessageIn::tls_deserialize_exact(&msg_bytes)
        .context("deserialize Application")?;
    let protocol_msg = match incoming.extract() {
        MlsMessageBodyIn::PrivateMessage(m) => ProtocolMessage::from(m),
        MlsMessageBodyIn::PublicMessage(m) => ProtocolMessage::from(m),
        other => bail!("Bob got an unexpected message type: {:?}", other),
    };
    let processed = bob_group
        .process_message(&bob_id.provider, protocol_msg)
        .map_err(|e| anyhow!("process_message failed: {e:?}"))?;
    let decrypted = match processed.into_content() {
        ProcessedMessageContent::ApplicationMessage(app) => app.into_bytes(),
        other => bail!("Bob expected Application, got {:?}", other),
    };

    if decrypted != plaintext {
        bail!(
            "plaintext mismatch: sent {:?}, got {:?}",
            std::str::from_utf8(plaintext).unwrap_or("<non-utf8>"),
            std::str::from_utf8(&decrypted).unwrap_or("<non-utf8>")
        );
    }

    println!(
        "  · decrypted: {:?}",
        std::str::from_utf8(&decrypted).unwrap_or("<non-utf8>")
    );
    println!();
    println!("PASS — delivery service is end-to-end healthy.");
    Ok(())
}
