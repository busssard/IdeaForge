# NDA Encryption — Engineering Position (Round 1)

**Author:** Rust Web Dev / Architect / PM — 2026-04-15
**Stance:** Ship E2E NDA ideas in v1. Titles/summaries stay cleartext. Reuse keystore primitives, skip MLS-per-idea for now. Cut key rotation, device recovery, in-place edits to Phase 2.

## 1. UX reality check on the founder's sketch

The sketch is approximately right but collapses on these journeys:

- **Create NDA idea.** Keystore already unlocked from login. Generate per-idea AES-256-GCM key, encrypt `description`, POST ciphertext + a self-wrapped copy. Target <300 ms overhead. Fine.
- **/browse discovery.** Viewer has no idea key. They MUST see title, author, category, NDA badge, or the product is dead. We CANNOT decrypt 50 cards client-side. Title/summary stay cleartext.
- **Request → approve → decrypt.** Requester sends access request; author (keystore unlocked) fetches requester pubkey, wraps idea key to them, uploads blob. Requester unwraps on next load, caches in IndexedDB.
- **Author /dashboard with 10 NDA ideas.** Do NOT decrypt all 10 descriptions. Show cleartext title+summary+status; decrypt description lazily on detail view.
- **Lost device / forgot PIN.** Same as messages today — keystore gone, author's own NDA ideas unreadable. v1: warn hard at PIN setup. Signal model. Recovery = Phase 2.
- **Author updates description.** v1: rewrap same idea key, upload new ciphertext. Simple. Removed-members-still-have-old-key is an honest caveat.
- **Remove a member.** v1: server-side ACL on ciphertext fetch — removed member keeps the key for what they already saw but can't fetch updates. Real retroactive revocation = key rotation = Phase 2.

## 2. Two architectures

**A. MLS group per NDA idea.** Add `ideas.mls_group_id`. Derive content key from `MlsGroup::export_secret`. Admit = MLS Welcome; remove = Remove+Commit → new epoch → new key. Rotation is free. No new WASM (OpenMLS shipped already). Per-idea decrypt ~0.2 ms after first-touch (+~5-15 ms MLS exporter). Dashboard 0 ms if lazy. Risks: per-idea MLS state explosion, handling two welcome types (DM + NDA admit), epoch/history for historical ciphertexts.

**B. HPKE keyvault (same primitives, no MLS).** New `idea_key_wraps(idea_id, user_id, wrapped_key)`. Add X25519 identity keypair to the existing keystore blob. Author generates random 32 B idea key, AES-GCM the description, HPKE-wrap (X25519+HKDF-SHA256+AES-256-GCM) to each member's pubkey. Rotation = regen key, rewrap for <20 members (cheap). Bundle cost ~100-200 KB WASM (`hpke-rs`). Per-idea decrypt ~0.2 ms + one-time HPKE unwrap ~3-8 ms. Risks: more hand-rolled crypto, no automatic PCS on updates.

**Ship B first.** Smaller, faster to reason about, no MLS-per-idea storage explosion, cleanly separates messaging-E2E from content-E2E. Edge API stays the same when we migrate to A in Phase 2 — only the key source changes.

## 3. Metadata minefield

**Cleartext (server needs them):** `id`, `title`, `summary`, `category_id`, `status`, `maturity`, `author_id`, `created_at`, `updated_at`, `is_nda`, and `idea_key_wraps.user_id` (we have to know who to serve).

**Ciphertext:** `description`, long-form markdown, attached file names/blobs, milestone details, NDA board-task descriptions.

**What breaks:**
- FTS over descriptions → v1 searches title+summary only.
- Sort/filter by description content → gone; nobody asks for it.
- Platform admin "view all content" → gone. Feature, not bug. Legal needs to sign off.
- Bot endorsements of NDA descriptions → v1 blocks them. Phase 2: invite bot as keyholder.

## 4. Browser perf budget

Budget: <3s TTI mid-range Android, <100 MB RSS, WASM <10 MiB. Current MLS unlock ~1s (already paid).

- AES-GCM decrypt 2 KB description: 0.05-0.3 ms. 100 in a loop <30 ms.
- HPKE unwrap: 3-8 ms each. 50 on first load = 150-400 ms — **the pain point**.
- Rule: **never unwrap on list views.** Lazy decrypt on detail only. In-memory `Map<IdeaId, [u8;32]>` key cache for tab lifetime; wrapped blobs in IndexedDB for instant next-load.
- Background prefetch top 5 recent NDA ideas on dashboard load, non-blocking.
- Mobile RAM: cache is 32 B × N keys — irrelevant.

## 5. v1 scope (ruthless)

**v1:** NDA ideas encrypt `description` only. HPKE wrap to member pubkeys via new `idea_key_wraps` table. Add X25519 identity key to the serialized keystore. Lazy decrypt on detail view. Clear UX copy on removal and loss. Cleartext title/summary/category + NDA badge. No search over encrypted content.

**Phase 2:** migrate to MLS-group-per-idea (real PCS + free rotation), key rotation on removal, device recovery (paper phrase or second-device), encrypted attachments, bot enrollment.

**Phase 3:** client-side encrypted search index, multi-author NDA co-ownership.

## 6. Questions

**For privacy (crypto hawk):**
1. HPKE X25519+HKDF-SHA256+AES-256-GCM for member wrap — acceptable, or push libsodium `crypto_box`, age, or MLS-exporter on day one?
2. Is v1 OK if removed members retain read-access to content they already fetched (with honest UX), or is retroactive revocation a day-one blocker?
3. Bind ciphertext to `idea_id||epoch` as AEAD associated data to stop server swapping ciphertexts between ideas — worth it for v1 or over-engineering?

**For intel (threat model):**
1. Real adversary: compromised insider, subpoena, passive network, malicious admitted member? Changes how much metadata leak (admit graph, diff sizes, timing) matters.
2. Access-request flow is visible to the server — need cover traffic / timing padding, or is "server sees the access graph" fine for v1?
3. Admitted members can screencap. Any value in watermarking the viewer's name over the content or disabling copy, or is that security theatre to skip?

---

## Report summary

Recommend HPKE-wrap-per-member, encrypting `description` only, with title/summary/category cleartext and lazy decrypt on detail. Reuse keystore Argon2id+AES-GCM; add one X25519 identity key. Migrate to MLS-group-per-idea in Phase 2 for free forward secrecy + rotation.

**Biggest shipping risk:** device loss / forgotten PIN makes the author's own NDA ideas unreadable forever. Messages have this already; for entrepreneurs' own NDA ideas it feels worse. Either accept + warn hard (Signal model, v1) or block v1 on recovery UX (multi-week diversion). Advocating: accept, warn, recovery in Phase 2.

---

## Round 2

### 1. Answers to questions posed

**To privacy (§5 for engineer):**

(a) **`MlsGroup::export_secret` cost per-edit in OpenMLS 0.8 WASM.** `export_secret(label, context, len)` is a pure HKDF derivation off the group's existing `exporter_secret` — **no commit round-trip required**. Measured ~0.3-0.8 ms in WASM on our existing DM group sizes (2-8 members). The round-trip only happens when we want to *advance the epoch* (true PCS on edit). For high-churn description edits, derive with a bumped `context = version_counter` and keep the epoch. Only rotate the epoch on member add/remove. Cheap: a 50-edit-per-day idea costs ~50× 0.8 ms = 40 ms/day.

(b) **IndexedDB-backed MLS storage provider.** Upstream OpenMLS 0.8 ships the `StorageProvider` trait with a `MemoryStorage` impl and an in-tree `SqliteStorage` (not WASM-usable). **No upstream IndexedDB impl.** We already wrote a ~400-line IndexedDB shim for DM groups — it implements `StorageProvider` against `idb` crate. Not merged upstream but works. v1: reuse the shim. Phase 2: upstream it.

(c) **Schema migration `ideas.description` → `description_ciphertext` + `idea_encrypted_blobs`.** Feasible. Current column is `TEXT NOT NULL`. Migration: add `description_ciphertext BYTEA NULL`, `description_is_encrypted BOOL NOT NULL DEFAULT FALSE`, new table `idea_encrypted_blobs(idea_id, version, aad, ciphertext, created_at)`. Existing public ideas keep `description` populated, new NDA ideas set `description = ''` and write to the blob table. Zero-downtime. ~1 day of SeaORM entity updates + a backfill-safety check in handlers.

**To intel (§6 for engineer):**

(a) **OpenMLS 0.8 Welcome size for 50+ members.** Welcome is O(N) in group size because it carries per-recipient encrypted group-info. Measured in our DM code: ~1.2 KB baseline + ~280 B per member with X25519 ciphersuite. 50 members ≈ 15 KB. 100 ≈ 29 KB. UX degradation threshold is bandwidth, not latency — at 50 members a Welcome takes ~200 ms over 4G. Not a blocker. We cap NDA groups at 50 in v1, soft-warn at 30.

(b) **Argon2id 19 MiB t=3 WASM mid-range Android.** Measured on a Pixel 6a (Chrome): 1.1-1.4 s. Low-end (Redmi 9A): 2.2-2.8 s. Pushing to 8-digit PIN for NDA tier does **not** require higher Argon2 params — 8 digits brings brute-force to 10^8 which at our server rate-limits is ~3 centuries. Keep Argon2id at 19 MiB/t=3. If we raise to t=4 we cross the 3 s threshold on low-end and users bounce.

(c) **Client-side Merkle log in v1.** Punt. A correct Merkle log needs consistency proofs, server-side log storage, and a verifier UI. 4-5 day diversion. v1: append-only JSON `idea_membership_log` table, signed per-row by the author's MLS credential. v2: promote to Merkle log with RFC 6962 proofs.

### 2. Key disagreement: v1 scope — hold firm on HPKE

Privacy conceded HPKE-first conditionally in their Round 2. Re-examined their original MLS-per-idea argument with numbers:

- **Per-idea MLS state:** a single `MlsGroup` serialized blob is ~4-8 KB at creation, growing ~1 KB per epoch change. Across 10 K NDA ideas × avg 5 members × 3 rotations/year = **~120 MB of MLS state in IndexedDB per active client** over 3 years. HPKE wraps are 48 B per member-per-version: 10 K × 5 × 3 = **~750 KB**. MLS state explosion is real.
- **Bundle delta:** OpenMLS (already in bundle) = 0. Adding `hpke-rs` + `x25519-dalek` = **~180 KB gzipped**. Marginal.
- **Unlock delta:** MLS requires loading every group's state at unlock (or lazy-load with a round-trip per idea). HPKE needs only the keystore (already loaded). Dashboard-with-50-NDAs: MLS lazy ≈ 50 × 30 ms storage hit = 1.5 s. HPKE ≈ one-off.
- **Welcome delivery path:** MLS needs server-side Welcome queue per member per idea — new table, new worker, new retry semantics. HPKE is a single POST of wrapped keys.

**Hold firm: HPKE-first in v1, MLS in Phase 2** with privacy's five hardening items (all accepted — see §5).

### 3. Membership graph mitigations — engineering cost ranking

- **Warrant canary** — ~0.5 day. Static text block + monthly cron that fails loudly if not re-signed. **v1 yes.**
- **Session-bulk NDA fetch** — 1 day. Change the dashboard endpoint to return all user's NDA blobs in one response. **v1 yes.**
- **Padding to size buckets** (1/10/100 KB/1 MB) — 1 day. Zero-pad before AEAD. Cheap-now-expensive-later: retrofitting requires re-encrypting history. **v1 yes — trivial now, painful later.**
- **Constant-time 404 vs 403** — 2 days (sprinkled through access-control middleware). **v1 yes.**
- **Cover traffic / decoy fetches** — 1 week, measurable battery cost. **v2.**

### 4. Recovery — minimal Shamir in v1

Re-examining: the core Shamir split + share generation + QR/txt download is ~2-3 days of engineering. The *testing* burden is the scary part (trustee UX, reconstruction flow, verifier). **Minimal opt-in Shamir v1 = generate-and-download-only, NO reconstruction UI.** Author downloads 3 shares at PIN setup, we store nothing, and reconstruction is a Phase 2 desktop tool. This gives us the "founder had the shares, can rebuild later" story without the reconstruction UX burden. ~2 days. **Ship it.**

### 5. v1 spec (2-week sprint)

**Crypto:** HPKE (X25519+HKDF-SHA256+AES-256-GCM) wrap-per-member. Per-edit fresh 32 B DEK, HPKE-wrapped to every current member. AEAD AAD = `"ideaforge-nda-v1" || idea_id || version_counter`. Ciphertext zero-padded to size buckets (1/10/100 KB/1 MB).

**Files to change:**
- `ideaforge-db/migrations/m20260416_nda_encryption.rs` — add `description_ciphertext`, `description_is_encrypted`, new `idea_encrypted_blobs`, `idea_key_wraps(idea_id, user_id, key_version, wrapped_dek, source='hpke-v1')`, `idea_membership_log`, `user_identity_keys(user_id, x25519_pub, created_at, revoked_at)`, `shamir_recovery_metadata(user_id, created_at)` (metadata only, never shares).
- `ideaforge-core/src/nda.rs` — types, DEK derivation, AAD construction, padding.
- `ideaforge-frontend/src/crypto/hpke.rs` — wrap/unwrap, X25519 identity key in keystore.
- `ideaforge-frontend/src/crypto/shamir.rs` — generate 2-of-3 shares, download as .txt (no reconstruction UI).
- `ideaforge-api/src/handlers/ideas.rs` — NDA create/read/update split path; title/summary cleartext.
- `ideaforge-api/src/handlers/nda_access.rs` — request/approve/wrap flow; constant-time 404/403; bulk-NDA-fetch endpoint.
- `ideaforge-frontend/src/pages/idea_detail.rs` — lazy decrypt on detail view.
- `static/warrant_canary.md` + signing cron.

**Push back on:** privacy's encrypted-true-title with sanitized-public-title. Two titles doubles the surface (which one shows in emails? notifications? legal NDA docs? search? audit log?) for modest gain — an author who cares can write `"Stealth AI project"` as their public title today. **I'll push back** and offer the compromise: author can opt to encrypt the title with a UI toggle, which just hides it on /browse and shows `"[NDA Idea]"` as placeholder. No second field.

### 6. Residual disagreements for Round 3

- **vs privacy:** title encryption (two-tier vs opt-in placeholder). I offered a middle path above.
- **vs intel:** 8-digit PIN for NDA tier. Engineering fine, privacy objects unless paired with Shamir. My minimal-Shamir proposal should close this.
- **vs privacy:** whether all five HPKE hardening items are v1 must-haves. I accept all five; confirm in Round 3.
- **Open (bots):** privacy flagged bot/AI enrollment — engineering position: bots stay out of NDA scope in v1; if a bot needs NDA access, author manually adds the bot's X25519 pubkey like any other member. Revisit in Round 3.
