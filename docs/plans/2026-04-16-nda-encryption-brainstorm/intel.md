# Intel — NDA Encryption Threat Model (Round 1)

Bias: ops-sec over elegance. Crypto is easy; humans, endpoints, metadata burn you.

## 1. Adversary Catalogue (ranked)

1. **Malicious insider / rogue operator** — very high. **Must defend.**
2. **Subpoena vs IdeaForge** — inevitable. **Must defend.** "We physically can't produce it" is the product.
3. **DB leak / SQL injection** — **Must defend.**
4. **Disgruntled former signer** — the actual founder-fear. **Must defend** (revocation).
5. **Compromised author endpoint** — crypto can't save a keylogged laptop. **Partial**: per-device keys, session TTL.
6. **Nation-state DB access** — **Out of scope**; scheme shouldn't actively help them.
7. **Subpoena vs author** — **out of scope**. Compelled authors decrypt. Feature, not bug.
8. **Traffic / membership graph** — **acknowledge, partial** (§5).

## 2. Compartmentalisation

**Per-idea keys wrapped to per-user identity. Only sane model.**

- Per-user only: one revoked member burns every project. No.
- MLS group = idea. Add signer = `Welcome`. Remove = `Remove` + rekey. Forward secrecy automatic.
- Per-idea content key (CEK) wrapped by current MLS epoch secret; rotates on removal.

**Author-signed access ledger**: append-only `(user_id, key_package_hash, admitted_at, removed_at)`, signed client-side, server stores as opaque blob. Author reconciles server-claimed membership against ledger — catches silent injection.

## 3. Key Custody / Recovery

**Author-chosen Shamir (2-of-3 default), opt-in. "Paranoid" mode = no recovery.**

- Hard "no recovery" sounds pure; guarantees the founder DMs support at 2am after losing a $2M deck's PIN. Enterprise demands break-glass or walks.
- **Never server-side escrow** — recreates the threat we're defeating.
- Shares held by author-chosen trustees, each encrypted under trustee's own MLS identity. Recombination client-side in author's browser.
- UI forces: "Trustees: Alice, Bob, Carol. Any 2 can read this."

## 4. Operational Landmines

1. **Server forges member-add** → MLS Welcomes signed by existing member; server can't forge. Signed ledger is belt-and-braces.
2. **Silent Welcome omission** → DS not trusted. Recipient receipts via second channel (email hash / separate-epoch notification). V2 if tight; document gap in v1.
3. **Timing "does this idea exist"** → constant-time 404 vs 403; NDA responses identical regardless of signer status. Cheap.
4. **New device** → new MLS client re-keys via `DeviceAdd` signed by existing device. No SMS-OTP. All devices lost → §3 escrow, else idea dies.
5. **PIN brute force** → 6 digits = 10^6. DB leak enables offline grinding. Server rate-limit unlock attempts; 8-digit default for NDA tier.
6. **Revoked signer kept plaintext** → crypto can't un-read. Legal recourse only.

## 5. Legal vs Technical Posture

**"We can't produce it" is 80% true. Be honest.**

Under subpoena IdeaForge still produces: idea existence, title (unless encrypted — we should), author, timestamps, signer list, ciphertext size, access logs.

Tighten:
- Encrypt **titles and tags** for NDA-tier, not just bodies.
- Pad ciphertexts to size buckets (1/10/100KB, 1MB).
- Clients fetch all own NDA ideas per session to blur access logs.
- Warrant canary.

**Honest weakness: signer graph.** "Alice signed Bob's NDA" is a cleartext edge leaking strategic intent — who's partnering, what space. Correlators win without touching plaintext. Name this publicly.

## 6. Questions for Round 2

**For privacy**:
1. MLS forward secrecy advances the epoch, but anyone who cached the old epoch secret retains old ciphertext. Re-encrypt idea body on every removal, or accept cached retention?
2. Signed ledger — Merkle log vs hash chain? Cleanest "server can't lie about membership" without a separate transparency service?
3. Shamir — plain SSS, or verifiable (Feldman/Pedersen) so trustees prove share validity without reconstructing?

**For engineering**:
1. OpenMLS 0.8 WASM — Welcome sizes for 50+ member groups? Upper bound before UX degrades?
2. Argon2id 19 MiB / t=3 WASM on mid-range phone — unlock latency? >3s and users disable NDA.
3. Access ledger — client-side Merkle log in v1, or punt to v2 with server-replicated tamper-evident blob?

---

**Stance**: MLS group per idea, CEK wrapped by epoch secret, opt-in author-chosen Shamir, encrypt titles/tags, honest public threat model. Founder's sketch directionally right; real work is membership integrity and metadata minimisation.

**Threat the other two will underweight**: **the membership graph itself.** Privacy will obsess over content post-compromise; engineering over PIN UX. Neither notices signer-edges leak strategic intent without touching plaintext. That's what I push in Round 2.

---

## Round 2

### 1. Answers to questions posed

**From privacy (§5):**

(a) **Insider threat profile:** assume **single rogue SRE with occasional root**, not nation-state multi-party. IdeaForge will not be running N-of-M operator ceremonies at Series A — that's security theatre we can't enforce. Design for one hostile admin with read access to Postgres, S3, and the ability to sneak one WASM bundle past review. That bound lets us skip m-of-n operator controls and still sleep. Nation-state capability is out of scope; honest threat doc says so.

(b) **Server-forges-membership countermeasure:** in-UI cryptographic receipt is **necessary but not sufficient** for high-value ideas. Adopt Signal's pattern: ship receipts by default, surface "safety number / fingerprint" comparison as a one-click flow, **require author acknowledgement of the fingerprint before first share**. Out-of-band QR/phone-call forced for every share = UX death. Make it opt-in-per-idea via a "Verify in person" toggle surfaced on high-maturity ideas. Cheap, Signal-proven, audit-friendly.

(c) **Subpoena posture:** technical-inability is the right default — own it publicly, put it in marketing. No supervised disclosure path in v1 (that becomes the subpoena target). Legal must sign off that we respond with idea metadata (existence, title-if-cleartext, timestamps, signer list) and cannot produce plaintext. Advantage: warrant canary bites cleanly; no ambiguous middle ground.

**From engineering (§6):**

(a) **Real adversary → metadata tolerance:** rogue SRE + subpoena (see 1a). Metadata tolerance: ciphertext size leakage = **not tolerable** (pad); access-request timing = **partially tolerable** (v1 session-bulk fetches are enough); admit-graph = **tolerable only if publicly named**.

(b) **Cover traffic / timing padding on access-request:** **out of scope for v1.** Full cover traffic is a rat-hole. Ship session-bulk NDA fetches (see §3 below) — it collapses the access-timing signal 80% for 5% of the effort. Real cover traffic is Phase 3 with the anonymous-credential work.

(c) **Screen-watermarking / copy-disable:** **theatre, ship it anyway.** Doesn't defeat phone-cameras; does deter casual screencap-and-paste-to-Slack leaks (the common case). Costs ~1 day. Add author name + timestamp translucent overlay on NDA detail view. Skip "copy-disable" — it's pure theatre and breaks accessibility.

### 2. V1 scope disagreement — concede, conditionally

Re-reading engineering §2B honestly: HPKE-wrap-per-member for <20 members with per-edit DEK rewrap gives us most of MLS's practical benefit at 20% the complexity. Privacy already conceded with five hardening items — I line up behind those five. The capability left on the table is (i) automatic PCS on member churn and (ii) post-compromise recovery of the group secret. For a v1 where the median idea has 3-8 signers and churn is rare, that window is defensible **if the wire format is migration-ready.**

**Concede, with one addition to privacy's five:** a v1 **key-compromise recovery protocol** — author can force-rotate the idea DEK manually (one button, rewraps to current roster). Cheap, gives us a manual-PCS escape hatch, stops "silent compromise persists across a year of edits."

### 3. Membership graph — push harder, one v1 must-have

Neither paper offered a real cryptographic answer; both (correctly) punted anonymous credentials to Phase 3. My v1 must-haves:

1. **Session-bulk NDA fetch** (privacy conceded this). Client pulls all own NDA ciphertexts on session start, not on detail-view click. Collapses per-idea access timing. **Non-negotiable v1.**
2. **Warrant canary** in footer, auto-signed weekly. Ships in one afternoon.
3. Anonymous-credential signer tokens → **Phase 3 research**, not v1.

The one I refuse to cut: **session-bulk fetch.** Without it, server logs reconstruct the access graph idea-by-idea; with it, server sees "Alice opened the app" and nothing more granular.

### 4. Recovery / escrow — middle path

Signal-model-by-default **with opt-in Shamir at setup** wins. Engineering's "no recovery, warn hard" is right for the median user; my/privacy's Feldman-Shamir is right for the founder who just lost a $2M deck's PIN. Ship both: default is Signal, setup wizard offers "Enable recovery (2-of-3 trustees)" as a prominent optional step. Trustees are author-chosen, shares encrypted under trustee MLS identity, no server-side escrow ever. Entrepreneurs self-select into recovery; paranoid users keep the pure-Signal guarantee.

### 5. Residual disagreements for Round 3

- **vs engineering (still open):** titles. Privacy wants two-tier (sanitized-public + encrypted-true). Engineering wants cleartext. I side with privacy — founder's sanitized public title is the subpoena-resistant posture. Round 3 must resolve.
- **vs engineering:** whether all five of privacy's HPKE hardening items + my force-rotate button are v1 scope or some slip.
- **vs privacy:** PIN length. I still want 8 digits for NDA tier — Feldman-Shamir is opt-in, so PIN is the only line of defense for users who skip recovery. Offline grind from DB leak on 6-digit = 2^20 cheap.
- **vs both:** bot/AI-agent enrollment in NDA groups — unresolved from Round 1. Round 3 settles.
- **vs both (new):** screen-watermarking — I want it, neither paper has a position.
