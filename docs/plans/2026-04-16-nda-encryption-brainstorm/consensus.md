# NDA Encryption — Consensus Design (Round 3)

**Authors:** Engineering (lead), Privacy, Intel
**Date:** 2026-04-16
**Status:** Draft for founder sign-off
**Sprint target:** v1 in 2026-Q2 (~2 weeks eng, see §9)

---

## 1. Executive summary

NDA-tier ideas encrypt description, long-form body, and board-task text end-to-end; only the author and admitted members can read plaintext. **v1** ships HPKE-wrap-per-member (X25519 + HKDF-SHA256 + AES-256-GCM, RFC 9180) with per-edit fresh DEKs, AAD bound to `idea_id||version_counter`, size-bucket padding, and opt-in 2-of-3 Shamir share generation. Titles are cleartext by default with an opt-in "Also hide the title" toggle. **v2** migrates to MLS-group-per-idea (automatic PCS, free rotation) reusing the v1 wire format via `source_tag` — no re-encryption needed. **v3** adds anonymous-credential signer proofs, cover traffic, encrypted attachments, bot keyholders, Shamir reconstruction desktop tool, reproducible builds. **Security claim:** ciphertext-at-rest is indistinguishable from a dumb blob store against a rogue SRE with Postgres root and one-shot WASM push. Out of scope: nation-state multi-party access, compelled authors, endpoint compromise.

## 2. Threat model

**Defended:** malicious insider / rogue SRE with Postgres root + one-shot WASM push; subpoena against IdeaForge ("technically unable to produce plaintext" is the product); passive DB exfiltration (backups, SQLi, `pg_dump`); disgruntled former signer seeking *future* content; hostile server forging roster additions.

**Acknowledged, not defended:** nation-state multi-party operator access (no m-of-n ceremonies at Series A); compelled authors (a court can force decryption); endpoint compromise (keyloggers, unlocked laptops); WASM supply-chain push (partial mitigation via SRI + hash transparency; reproducible builds v3); membership/signer graph correlation (blurred via session-bulk fetch, named publicly); removed-member cached-ciphertext retention (crypto cannot un-read bytes).

**Confidential:** description, body, NDA task text, private NDA comments, optional title. **Acceptable leakage:** author ID, day-rounded timestamp, roster, stoke count, opt-in category, ciphertext size bucket.

## 3. Architecture — v1

### 3.1 Key hierarchy

```
PIN (6 digits default; 8 for NDA tier)
   └─Argon2id(19 MiB, t=3, salted)─> wrap_key (32 B)
                                         └─AES-256-GCM unseal─> keystore blob
                                              { X25519 identity (new), MLS cred,
                                                cached DEKs }
                                         └─HPKE unwrap───────> idea_DEK_v{n} (32 B, fresh per edit)
                                              └─AES-256-GCM(AAD)──> ciphertext(description|title|task)
```

PIN unlock is client-side; server never sees PIN or wrap key. Adding the X25519 identity keypair extends the existing keystore blob by ~32 B plaintext / ~48 B wrapped — the only format change.

### 3.2 Data model

**New tables:**

- `idea_key_wraps(idea_id, user_id, key_version, wrapped_dek BYTEA, source_tag TEXT DEFAULT 'hpke-v1', wrapped_at)` — PK `(idea_id, user_id, key_version)`. `source_tag` ∈ {`hpke-v1`, `mls-exporter-v2`} enables Phase 2 dual-write.
- `idea_membership_log(idea_id, seq BIGINT, event TEXT /* admit|remove */, subject_user_id, actor_user_id, actor_signature BYTEA, created_at)` — append-only, signed by actor's identity key. Non-Merkle in v1; RFC 6962 is v2.
- `idea_shamir_shares(user_id, created_at, download_confirmed BOOL)` — **metadata only**, never shares. Lets us warn users who skipped recovery.
- `user_identity_keys(user_id, x25519_pub BYTEA, created_at, revoked_at NULL)` — `revoked_at` marks device loss; author rewraps to the new identity on next edit.

**Modified `ideas` columns:** `description_ciphertext BYTEA NULL`, `title_ciphertext BYTEA NULL` (populated only when "Also hide the title" is checked), `is_nda_encrypted BOOL`, `version_counter BIGINT` (bumped on every description edit; feeds AAD), `padding_bucket SMALLINT` (0=1 KiB, 1=10 KiB, 2=100 KiB, 3=1000 KiB). Public ideas keep `description TEXT`; NDA ideas set `description=''` and write `description_ciphertext`. Zero-downtime.

### 3.3 Endpoints

Axum 0.7 `:param` syntax. Author-only endpoints validate author's MLS-signed credential.

**Author-only:**
- `POST /api/v1/ideas` — create. Body: `is_nda_encrypted`, optional `hide_title`, ciphertexts, `padding_bucket`, `wrapped_dek_for_self`, `aad`.
- `POST /api/v1/ideas/:id/keys` — admit user. Body: `{user_id, wrapped_dek, key_version}`.
- `DELETE /api/v1/ideas/:id/keys/:user_id` — revoke; removes rows, appends signed `remove` log event.
- `POST /api/v1/ideas/:id/membership-log` — actor-signed entry; server verifies sig + monotonic `seq`.
- `PUT /api/v1/ideas/:id` — description update with fresh DEK + new wraps for current members (one transaction).
- `POST /api/v1/ideas/:id/rotate-key` — manual force-rotate (intel's addition); rewraps to current roster, no history re-encryption.

**Member-scoped** (caller must appear in `idea_key_wraps`):
- `GET /api/v1/ideas/:id/keys/mine` — caller's wrapped DEK(s).
- `GET /api/v1/ideas/:id` — ciphertext + version; 403 if not member (constant-time with 404).
- `GET /api/v1/ndas/mine` — **session-bulk fetch** (intel's non-negotiable). Every NDA ciphertext the caller can see, one response. Collapses per-idea timing.
- `GET /api/v1/ideas/:id/membership-log` — signed log for client-side verification.

**Public:** `GET /api/v1/ideas` — browse; cleartext `title`/`summary`/`category_id`/`author_id`/`is_nda_encrypted`/`stoke_count`. When `title_ciphertext` is set, `title` is replaced by tombstone `"🔒 Locked — members only"`. Description never included for NDA ideas. `GET /.well-known/warrant-canary` — static, signed weekly.

### 3.4 Ciphertext format

**AEAD layout** (per encrypted field: description, title, or board-task body):

```
AAD   = "ideaforge-nda-v1"  (16 B literal)
      || field_tag           (16 B ASCII, null-padded: "description"|"title"|"task")
      || idea_id             (16 B UUID)
      || version_counter     (8 B big-endian u64)
nonce = 12 B random, stored with ciphertext
ct    = AES-256-GCM(DEK, nonce, pad(plaintext), AAD) || 16 B tag
```

**source_tag** on every `idea_key_wraps` row: `'hpke-v1'` in v1, `'mls-exporter-v2'` in Phase 2; handler accepts both during migration. MLS-compatible from day 1 (privacy hardening #3). **Padding buckets:** 1 / 10 / 100 / 1000 KiB; u32-length-prefixed, zero-filled, then AEAD'd. >1000 KiB rejected (attachments in v3). **Nonce:** 12 B random; fresh-DEK-per-edit means disjoint keyspace per version, so nonce reuse is impossible — GCM is safe.

### 3.5 The title question — resolved compromise

**Decision:** single title field, cleartext by default, with an **"Also hide the title"** checkbox at NDA-idea creation.

```
Title:        [                              ]    Shown on /browse, search, emails, NDA docs
Summary:      [                              ]
Description:  (encrypted — members only)

☑ This is an NDA-protected idea
☐ Also hide the title (recommended for very sensitive projects)
     Non-members see "🔒 Locked — members only" in place of the title.
```

**Why this compromise honours all three positions:** Privacy gets the two-tier option (checked → title HPKE-wrapped with `field_tag="title"` AAD, non-members see the tombstone; defeats the "subpoena produces the title" failure intel named). Engineering avoids schema bifurcation (one `title` column cleartext, one `title_ciphertext` null unless hidden — every surface reads one field). Intel's posture is preserved (authors who care tick the box; default stays searchable for the 90% case). The tier is nullability, not schema bifurcation. **Recommended resolution.**

## 4. User journeys

- **Create NDA idea.** Keystore unlocked from login. Frontend: random 32 B DEK → AES-GCM description (+optional title) with AAD `...||idea_id||0` → pad → HPKE-wrap to own pubkey → POST envelope. Target <300 ms.
- **Admit user.** Fetch Alice's `x25519_pub` → HPKE-wrap current DEK → POST `/keys` + signed `admit` log entry. Alice's next session-bulk fetch includes the wrap.
- **Viewer decrypts.** `GET /keys/mine` → HPKE-unwrap (3-8 ms) → AES-GCM decrypt (0.05-0.3 ms) → strip padding → render. DEK cached in-memory `Map<IdeaId,[u8;32]>` for tab lifetime; wrapped blobs persisted in IndexedDB for fast next-load.
- **Author updates description.** Fresh 32 B DEK → bump `version_counter` → new AAD → HPKE-wrap to every current member → transactional `PUT` with ciphertext + one wrap row per member at new `key_version`. Removed members get no wrap at this version.
- **Author removes a member.** `DELETE /keys/:user_id` drops wrap rows across all versions + signed `remove` log entry. UI: "Alice kept a copy of what she already read. Future edits are hidden from her." Optional **Rotate now** force-bumps the DEK without an edit.
- **Author loses device, has Shamir.** Shares were downloaded at setup; **reconstruction is v2 desktop tool**. Honest tradeoff of minimal-Shamir; shares use SSS-standard format so motivated users can reconstruct with third-party tools.
- **Author loses device, no Shamir.** NDA ideas gone. UI warns hard at PIN setup and NDA creation: "Signal model — your PIN is your key."
- **Author enables Shamir.** At PIN setup or `/settings/recovery`, frontend generates 3 Feldman-verifiable shares (commitments per coefficient, privacy R2 §1c), downloads them as three `.txt` files. Server stores only `download_confirmed`, never shares.

## 5. Out-of-scope for v1

- **MLS-group-per-idea** — ~120 MB/client state at 10 K ideas × 3 years vs 750 KB HPKE wraps (v2).
- **Bot NDA keyholders** — manually add bot's X25519 pubkey as a regular member; first-class bot identities v2.
- **Cover traffic / decoy fetches** — 1 week + battery cost; session-bulk fetch buys 80% for 10% (v3).
- **Encrypted attachments** — v1 uploads are plaintext (v2). **Encrypted FTS** — members search locally (v3).
- **Automatic key rotation on every removal** — manual rotate-key only; automatic via MLS epochs in v2.
- **Merkle membership log** — v1 signed append-only JSON; RFC 6962 proofs v2.
- **Reproducible WASM builds** — documented gap (v3). **Shamir reconstruction UI** — v2 desktop.
- **Anonymous-credential signer proofs (BBS+/idemix)** — signer graph stays a named leak (v3).
- **Post-quantum hybrid ciphersuite** — X25519+MLKEM once OpenMLS ships stable support.

## 6. Migration path to v2 (MLS-per-idea)

1. **Schema already compatible** — `idea_key_wraps.source_tag` ships in v1; v2 just adds rows with `source_tag='mls-exporter-v2'`. No schema change.
2. **Dual-write window** — post-v2 edits write both HPKE (v1 clients) and MLS-exporter (v2 clients) wraps. AAD prefix + `version_counter` unchanged (privacy R2 hardening #3).
3. **MLS group bootstrap** — at first v2 edit of an existing idea, author creates an MLS group, sends Welcomes to current roster (from `idea_key_wraps.user_id`), thereafter derives DEKs via `MlsGroup::export_secret(label="idea-dek", context=version_counter, 32)`.
4. **History stays readable** — old HPKE-wrapped ciphertexts need no re-encryption; v2 clients fall through to v1 wraps for history.
5. **Cutover** — once >95% clients have v2, new ideas stop emitting `hpke-v1` wraps. HPKE-unwrap code stays shipped for history reads at zero cost.

Credibility proof: the v1 wire format is chosen *so that* MLS slots in without a flag day.

## 7. What we're promising users (honest version)

**Actually safe:** Only you and admitted members can read the description (and title, if hidden). Not our engineers, admins, or backups. Subpoena against us produces only existence, cleartext title (unless hidden), author, roster, and ciphertext blobs — not plaintext. Warrant canary signed weekly. Every member change is actor-signed and auditable from your client.

**Leaks (named publicly):** the **roster** (operator sees it — the **signer graph**: "Alice signed Bob's NDA". Anonymous-credential fix in v3); day-rounded creation time; opt-in category; cleartext title; ciphertext size bucket; NDA-idea count per user (row counts).

**Known unsolvable in v1:** **WASM supply chain** — we ship the crypto; a backdoored bundle could exfiltrate your PIN. SRI + published hashes mitigate; reproducible builds in v3. Said plainly on the NDA-creation screen. **Removed-member cached content** — legal recourse only. **Compelled authors** — courts can force decryption.

**Recovery:** no PIN recovery (Signal model — server has nothing to leak because server has nothing). Optional 2-of-3 Shamir at setup: trustees you choose, shares in-browser-generated and downloaded, never seen by IdeaForge. **v1 generates, v2 desktop tool reconstructs.**

## 8. Risks and open questions

1. **Argon2id unlock latency, low-end Android (2.2-2.8 s, Redmi 9A).** Keep 19 MiB / t=3; 8-digit PIN + rate-limit gives 10⁸ offline cost. Raising params crosses the 3 s UX cliff. Fallback: "stay unlocked 8h" toggle.
2. **Operator fakes a member add by swapping the author's identity key.** Log entries signed by actor's key — operator can't forge without keystore access. Residual: operator swaps `user_identity_keys`. v1: clients pin first-seen identity per author, warn on change. v2: Merkle log + inclusion proofs.
3. **Session-bulk fetch scales poorly past ~500 NDA ideas per user.** Not v1 (median <10). When it bites, shard by last-accessed or paginate with randomised order.
4. **Minimal Shamir: shares are unusable in v1.** Desktop reconstruction tool committed within 3 months; standard SSS format allows third-party reconstruction. Named as v1 limit in recovery UI.
5. **Padding buckets too coarse/tight.** Chosen from corpus (median 1.3 KiB, p95 12 KiB, p99 47 KiB). Add a 300 KiB bucket if data says so; cannot remove once shipped.

## 9. Estimated effort

**v1 total: ~2 weeks eng** (one eng full-time), on top of existing keystore + MLS-for-DM infra.

| Component | Est | Notes |
|---|---|---|
| Schema + migrations | 1.5 d | New tables, column additions, SeaORM entities. Backfill trivial (new NDA ideas only). |
| Backend endpoints (Axum 0.7) | 3 d | NDA CRUD split path, key-wrap handlers, membership-log, session-bulk fetch, constant-time 404/403. |
| Frontend key mgmt (WASM) | 2 d | X25519 in keystore, `hpke-rs` wrap/unwrap, per-edit DEK, AAD, padding. |
| NDA creation UX + title toggle | 2 d | Cropper-style flow, "Also hide title" checkbox, tombstone on /browse. |
| Shamir generate-and-download | 2 d | Feldman-verifiable 2-of-3, three `.txt` downloads, `/settings/recovery`. |
| Warrant canary + cron | 0.5 d | Static file, weekly sign-or-alarm. |
| Testing | 2 d | AAD binding, constant-time assertions, padding, log signature verification, session-bulk semantics. |
| **Total** | **~13 d** | Buffer to 2-week calendar; single-eng. |

**Not included:** v2 MLS migration (2-3 weeks), reconstruction desktop tool (~1 week), reproducible builds (multi-week CI), bot enrollment.

---

## 10. Sign-offs

### Privacy sign-off

**SIGNED WITH RESERVATIONS.**

All five Round-2 hardening items are present and faithful: AAD = `"ideaforge-nda-v1" || field_tag || idea_id || version_counter` (§3.4), per-edit fresh 32 B DEK (§3.1, §4), `source_tag` column on `idea_key_wraps` for MLS dual-write (§3.2, §6), `user_identity_keys.revoked_at` with author-driven rewrap (§3.2, §8.2), and size-bucket padding 1/10/100/1000 KiB length-prefixed before AEAD (§3.2, §3.4). Ciphertext format is concrete enough — byte offsets, endianness, nonce strategy, and the fresh-DEK-per-version argument that makes 12 B random nonces safe are all pinned down. The title compromise (single `title` column + nullable `title_ciphertext` gated by a checkbox) honors the two-tier demand: authors who want the subpoena-proof outcome tick the box, default stays searchable. Accepted. The "what we're promising users" section (§7) names the signer-graph leak, WASM supply-chain gap, and no-PIN-recovery tradeoff plainly — not marketing-dishonest. The v2 migration is credible: `source_tag` shipped in v1 means MLS-exporter wraps slot in as new rows without re-encrypting history.

Reservations for founder adjudication:

1. **PIN length vs Shamir default.** The doc locks 8-digit PIN for NDA tier (intel's ask) but leaves Shamir opt-in. 8 digits + 19 MiB Argon2id gives ~10⁸ offline cost — survivable against a DB leak, but I'd rather see Shamir *prompted* (skippable, not buried in `/settings/recovery`) at NDA-idea creation. One extra modal, big win on recovery rate.
2. **Reconstruction tool slips to v2.** Users download shares in v1 with no first-party way to use them. SSS-standard format means motivated users cope with third-party tools; less-technical users won't. The recovery-UI warning must say this in plain language, not engineer-ese.
3. **Identity-key pinning is TOFU-only in v1.** §8.2 says clients pin first-seen and warn on change. Acceptable, but the warning UX needs to block further action until the user acknowledges, not be a dismissable banner.

### Intel sign-off

**SIGNED WITH RESERVATIONS.** Session-bulk fetch (`GET /api/v1/ndas/mine`) is in-spec and labeled as my non-negotiable — good. Signed append-only `idea_membership_log` with actor-sig and monotonic `seq` is present; non-Merkle for v1 is the right call if the identity-key pinning below gets hardened. Warrant canary, constant-time 404/403, opt-in `hide_title`, and size-bucket padding together deliver the "we physically can't produce the content" story for authors who tick the box. §7 names the signer-graph leak and WASM supply-chain gap in plain language a journalist or lawyer can quote back. Acceptable to ship.

Reservations for the founder to adjudicate:

1. **Shamir generate-without-reconstruct is a false-recovery footgun.** Users download three `.txt` files believing they have backup; until the v2 desktop tool lands they effectively don't (third-party SSS tools exist but zero users find them at 2 a.m. after losing a device). Either (a) gate the download behind an explicit "I understand I cannot restore from these files until the desktop tool ships in ~3 months" checkbox and bake the 3-month commitment into the public roadmap, or (b) cut Shamir from v1 entirely. Half a recovery story is worse than Signal-pure.
2. **Identity-key-swap defence is display-only.** §8.2 says clients "pin first-seen identity, warn on change" — warnings get dismissed. The client should *refuse* to wrap a fresh DEK to a changed identity without an explicit author re-confirmation gesture (modal, not toast). Cheap; closes the only residual roster-forgery path before Merkle lands.
3. **Screen-watermark absent.** Author name + day-rounded timestamp overlay on the NDA detail view — one day's work, deters the common screencap-to-Slack leak. Add to §9 or punt explicitly with rationale; right now it's silently dropped.

### Founder sign-off

(pending)
