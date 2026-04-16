# Privacy Position — NDA-Encrypted Ideas (Round 1)

**Author:** Privacy specialist
**Stance:** Maximum user sovereignty. Assume the operator is hostile. The ciphertext-at-rest version of IdeaForge must be indistinguishable from a dumb blob store for all NDA-protected content.

---

## 1. Threat Model

**Primary adversaries (must defeat):**
- **Rogue insider / subpoena target:** Anyone with root on the server, read access to Postgres, or the ability to push a server build.
- **Passive DB exfiltration:** Backup tape leaks, SQL injection, ORM bugs, `pg_dump` theft.
- **Network-level adversary with a TLS MITM foothold** (rare but possible via enterprise proxies or CA compromise).

**Secondary (mitigate, don't solve):**
- **Active server that serves malicious WASM:** The server ships the crypto code. If the operator pushes a backdoored WASM bundle, users lose. We mitigate via subresource integrity + eventual reproducible builds + optional user-attested client hashes; we do **not** claim to fully solve this. Call it out honestly in the UX.
- **Endpoint compromise:** Out of scope for crypto; IndexedDB is only as safe as the browser profile.

**Must remain confidential:** idea description, attached files, task descriptions on NDA boards, private comments, title and summary of NDA-protected ideas.

**Acceptable metadata leakage:** the *fact* that user U authored *some* NDA-protected idea; creation timestamp (bucketed); the set of members in the group (needed for MLS membership cert); category only if the author explicitly opts into discoverability.

## 2. Crypto Construction (Recommended)

**Reuse MLS. Don't invent.** One MLS group per NDA-protected idea. MLS already solves the hard parts: asynchronous member addition, forward secrecy, post-compromise security, and a ratcheted group key (the `exporter_secret`) we can derive a content key from.

**Key hierarchy:**
```
PIN + salt --Argon2id--> wrap_key (wraps per-user MLS credential & storage blob)
                                  |
                                  v (MLS client signs KeyPackages)
                                  |
                        per-idea MLS group state
                                  |
                                  v (MlsGroup::export_secret(label, ctx, 32))
                            idea_content_key (DEK, 32B)
                                  |
                                  v AES-256-GCM / XChaCha20-Poly1305
                        ciphertext(description, files, ...)
```

**Encryption:** AES-256-GCM for consistency with existing keystore. Nonces: random 24-byte via XChaCha20-Poly1305 preferred for large files where GCM's 96-bit nonce space invites reuse under concurrent edits. Pick **one** construction per payload type and don't mix.

**Hybrid / PQ:** MLS 1.0 default ciphersuite is classical (X25519 + Ed25519). Stay on the default for now; flag PQ (X25519+MLKEM hybrid ciphersuite, draft) as a **deliberate follow-up** once OpenMLS ships stable support. Ideas are not 30-year secrets; harvest-now-decrypt-later is a real but lower-priority concern.

**Member admission:** Author issues an MLS `Add` proposal → `Commit` → `Welcome` message to the invitee's `KeyPackage`. Server delivers the Welcome blob opaquely. **Founder's sketch is directionally right but underspecified**: don't implement a DIY "share the project key encrypted to user B's public key" — MLS already does this correctly with PCS.

**Member removal (PCS question):** When A kicks B, MLS's next `Commit` rotates the group secret → future content encrypted under a key B cannot derive. **But B can still decrypt any ciphertext they already pulled.** Server must also revoke B's *access* to historical ciphertext rows, but assume they exfiltrated whatever they had. Re-encrypting the full history on every revocation is expensive and leaks little (they already had it). **Do not promise retroactive revocation.** Surface this in the UI.

## 3. Metadata

| Field | Encrypted? | Why |
|---|---|---|
| `description`, files, NDA comments, task bodies | **YES** | Core IP. |
| `title`, `summary` | **Two-tier**: public "sanitized" fields (author-provided, cleartext, for index) + encrypted "true" title shown only to members | Enables discovery without leaking the sensitive original. |
| `category_id` | Cleartext if author opts in | Search utility ≫ leak risk. |
| `author_id`, `created_at` | Cleartext | Required for auth and for MLS membership cert. Timestamps: round to day. |
| Member roster (`idea_id → user_id`) | Cleartext (server-enforced, MLS-verified) | Server must know routing. MLS signatures prove it's not lying about *who else* is in the group. |
| `looking_for_skills`, `stoke_count` | Cleartext | Discoverability; not NDA content. |
| NDA signature records | Cleartext | Legal artefact. |

**Search on encrypted content is out of scope for v1.** No blind indexes, no SSE, no fancy ORAM — they all leak more than people expect. Members search locally after decrypt.

## 4. Footguns & Mitigations

1. **Lost PIN = lost idea.** 6-digit PIN + no recovery = one forgotten PIN nukes the idea for everyone. *Mitigation:* author generates a **one-time recovery KeyPackage** printed/downloaded at idea creation (Shamir 2-of-3 between author, optional trusted co-founder, and an offline printable code). Never escrow to the server.
2. **Server lies about membership.** Operator silently adds an attacker to the MLS group. *Mitigation:* **every member-change commit is displayed in-UI with a cryptographic receipt**; optional out-of-band "safety number" (like Signal) comparison between members. Log all membership changes to an append-only Merkle-logged audit feed the client can verify.
3. **Nonce reuse under concurrent edits.** Two members encrypt a new description version with the same DEK + random nonce within ~2^32 messages → birthday risk. *Mitigation:* derive a fresh DEK per encryption via MLS `export_secret(label="idea-dek", context=version_counter)`; never reuse a DEK across versions.
4. **WASM supply chain.** Operator pushes malicious frontend that exfiltrates PIN. *Mitigation:* publish frontend bundle hashes to a transparency log; users can pin a known-good hash via browser extension or a Tor-style onion build. Document honestly: this is mitigation, not solution.
5. **Device loss with cached keystore.** Unlocked browser tab + stolen laptop = plaintext in memory. *Mitigation:* auto-relock after N minutes of inactivity; keystore only unlocked during an authenticated session; offer "panic wipe" that re-generates the MLS client (kicks the lost device via Remove proposal).

## 5. Questions for Round 2

**For the engineer:**
1. Can OpenMLS 0.8 `export_secret` be called cheaply per-edit, or does each call require a group commit round-trip? If the latter, what's our derivation strategy for a high-churn description?
2. IndexedDB-backed MLS storage provider status — are we blocked on upstream, or is a local shim viable for v1?
3. Server-side: can we evolve `ideas.description: String` into `ideas.description_ciphertext: Option<bytea>` + a separate `idea_encrypted_blobs` table without breaking the existing public-idea flow? I want NDA ideas on a different write path entirely.

**For the intel specialist:**
1. What's the realistic insider threat profile — single rogue SRE, or nation-state with multi-party access? This decides whether we need m-of-n operator-controls for even cleartext metadata.
2. For the "server silently adds a member" attack: is **user-visible cryptographic receipt** acceptable, or do you want a hard out-of-band check (QR code, phone call) before first share?
3. Legal/discovery angle: if IdeaForge receives a subpoena for a specific idea, are we comfortable being *technically unable* to comply (author must voluntarily decrypt), or does the product need a supervised disclosure path? My strong preference is the former — say so in marketing.

---

**TL;DR:** MLS-group-per-idea with author-held recovery share, encrypted description + true title, cleartext roster/category/timestamps, no server-side search on NDA content, no retroactive revocation promises. Founder's sketch is right in spirit; tighten it by using MLS instead of ad-hoc per-user key wrapping.

---

## Round 2

### 1. Answers to questions posed

**To intel (§6.1-3):**

(a) **MLS forward secrecy vs re-encrypt-on-removal:** MLS's PCS protects *future* epochs; a cached pre-removal ciphertext stays readable to anyone who had the old epoch secret. Re-encrypting the full idea body on every removal is expensive and **leaks nothing new** — the removed member already pulled it. Don't do it. Accept cached retention, state it loudly in UI, rotate DEK per new version so *post-removal edits* are protected. This is the honest MLS posture.

(b) **Merkle log vs hash chain for signed ledger:** Merkle log. Hash chain gives append-only but no efficient inclusion/consistency proofs — every verifier replays the whole chain. A Merkle log (RFC 6962 style) lets any client verify inclusion in O(log n) and detect split-view with consistency proofs. Cost is one extra hash per entry. Ship Merkle.

(c) **Plain Shamir vs verifiable (Feldman/Pedersen):** Feldman. Plain SSS lets a malicious trustee submit a garbage share and nuke reconstruction with no attribution. Feldman adds a commitment-per-coefficient; trustees can validate their own share at issue time and the reconstructor can detect bad shares. Pedersen is stronger (unconditionally hiding) but overkill for a 2-of-3 family recovery. Feldman is the sweet spot.

**To engineering (§6.1-3):**

(a) **HPKE X25519+HKDF-SHA256+AES-256-GCM vs MLS-exporter day 1:** HPKE-RFC9180 with that suite is a legitimate primitive; it's what MLS itself uses internally. Acceptable **only if** the v1 includes the hardening items below. Not acceptable as "naked per-user wrap forever."

(b) **Removed-member retains old fetched content:** Acceptable for v1. That's identical to MLS's real guarantee; anyone who claims stronger is lying. Not a day-one blocker.

(c) **AAD binding to `idea_id||epoch`:** Worth it. Costs zero runtime, prevents ciphertext-swap attacks from a hostile server. Non-negotiable v1, not over-engineering.

### 2. Engineering's HPKE-first path — concede, conditionally

Re-reading engineering §2 Architecture B honestly: HPKE-wrap-per-member with <20 members, rewrap on rotation, is **cryptographically sound** as an incremental step. MLS-per-idea state explosion across thousands of ideas is a real concern I underweighted. The two-phase migration is plausible *if* the v1 wire format is designed for it.

**I concede HPKE-first, with non-negotiable hardening:**

1. **AEAD AAD = `"ideaforge-nda-v1" || idea_id || version_counter`** on every ciphertext. Prevents swap and version-rollback.
2. **Per-edit DEK:** fresh 32 B random key per `description` write, HPKE-wrapped to every current member. Never reuse a DEK across edits. This gives us "PCS-lite" without MLS.
3. **Wrap format must be forward-compatible with MLS epoch migration:** the `idea_key_wraps` row must carry a `source: "hpke-v1" | "mls-exporter-v2"` tag so Phase 2 can dual-write then cut over without re-encrypting history.
4. **X25519 identity key rotation on device loss:** when keystore is lost, old pubkey stays in `idea_key_wraps` forever — we need a revocation marker + author-driven rewrap.
5. **Ciphertext padding to size buckets** (intel §5). Cheap, stops size-fingerprinting of ideas.

If engineering signs up to all five, HPKE-first is fine. If any get cut, I'd push back to MLS day 1.

### 3. Membership/signer graph — intel is right

They are. Private set intersection, anonymous credentials (BBS+, idemix), server-blind membership proofs — **none are shippable in a 2026-Q2 sprint.** BBS+ would let a signer prove NDA-signed status without revealing identity, but integration cost is multi-month and our server still needs user_id for routing NDA notifications.

**Ship-able countermeasures for v1:** (i) constant-time responses regardless of signer status (intel §4.3), (ii) fetch-all-own-NDAs-per-session to blur access logs (intel §5), (iii) **warrant canary + public threat-model doc naming the signer-graph leak explicitly.** Anonymous-credential signer proofs go on the Phase 3 research roadmap, not v1.

This is a product/policy problem we **name publicly** and a cryptographic problem we defer honestly.

### 4. Updated recommendation

**Softened:** HPKE-wrap-per-member in v1 (conceding to engineering), **with** the five hardening items above as non-negotiable. MLS-group-per-idea moves to Phase 2 with a concrete wire-format-compatible migration plan.

**Held firm:** encrypted **true title** for NDA-tier ideas, with a separate author-provided sanitized public title for discovery. Engineering wants title cleartext for /browse. I'm still against this — the sanitized-public + encrypted-true two-tier costs almost nothing and defeats the "subpoena-gets-the-title" failure intel flagged (§5). Author chooses what the public-facing sanitized title leaks.

**Held firm:** no server-side search on NDA content, no retroactive revocation promises, no server-side escrow.

### 5. Residual disagreements for Round 3

- **vs engineering:** titles. They want cleartext title+summary for discovery; I want two-tier (public sanitized + encrypted true). Needs resolution.
- **vs engineering:** whether the five HPKE hardening items are *all* in v1 scope or some slip to v1.1. I say all five or we do MLS day 1.
- **vs intel:** PIN length. They want 8 digits default for NDA tier; I want 6 digits + mandatory Feldman-Shamir recovery since PIN length alone doesn't defeat offline grind from a DB leak.
- **vs both:** bot/AI-agent enrollment in NDA groups — neither paper took a position. Round 3 should settle whether bots get first-class MLS identities or must stay out of NDA scope.
