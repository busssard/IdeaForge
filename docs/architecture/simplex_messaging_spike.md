# SimpleX E2E Messaging — Design Spike

> **Status:** Spike / pre-implementation. Decisions not yet locked in.
> **Goal of this doc:** surface the architectural choices before any code ships,
> since the naïve implementation silently breaks the cryptographic claim we'd be
> making to users. The user-facing feature request was: "users should be able to
> send end-to-end-encrypted messages between each other; create a SimpleX user
> and use their platform for safe communication." (Feedback submission
> 2026-04-14.)

## 1. Summary and recommendation

- **Recommended approach:** **"Matchmaker, not middleman"** (Option B below).
  IdeaForge brokers an invitation-link handshake between two users; their
  SimpleX clients (mobile or desktop app) do the actual messaging. IdeaForge
  never holds either user's SimpleX keys, so the cryptographic claim holds.
- **Do not start with** server-hosted per-user SimpleX identities (Option A).
  It feels convenient but silently downgrades the feature from "true E2E" to
  "encrypted to the platform," which contradicts the stated requirement and
  exposes us to a trust / liability problem if misdescribed.
- **Infrastructure:** self-host one SMP relay and at least one XFTP relay for
  metadata privacy. E2E encryption holds even on public relays, but operating
  our own protects timing / queue-activity metadata from third parties. Both
  run cleanly on a small VPS.
- **Primary risk:** SimpleX has no official browser/WASM client. We can't
  implement the messaging surface inside the Leptos WASM bundle — the user has
  to have a SimpleX client installed. UX costs real design work.
- **Scope for Phase 1:** ship the matchmaker (handshake generation + in-app
  "Connect on SimpleX" flow). Do **not** ship platform-hosted messaging in
  Phase 1. If we later decide we also want an in-browser convenience mode,
  it's an additive change, not a retrofit.

## 2. Why this needs a spike at all

The feedback reads like a small feature ("let users DM each other privately"),
but "end-to-end encrypted" is a contractual phrase, not a UI affordance. The
difference between E2E and "encrypted in transit" is where the private keys
live:

| Key holder      | Called E2E? | What the platform can read |
|------------------|-------------|----------------------------|
| Only the two users' devices | Yes | Nothing — only ciphertext in transit |
| Platform server (on user's behalf) | No | Everything, regardless of TLS/transport encryption |

Most chat integrations ship the second model because it's easier. If we ship
the second model and call it E2E, we mislead the user. Before writing code, we
need to pick a model that matches the claim.

## 3. SimpleX, briefly

SimpleX Chat is a decentralised E2E messenger with three notable properties
relevant here:

1. **No global user identities.** There is no "IdeaForge user 42 ↔ SimpleX
   user 42" directory. Two parties bootstrap a conversation by exchanging a
   *SimpleX queue URI* (invitation link / QR code) out of band. Shape:
   `smp://<serverFingerprint>@<host>[:port]/<queueId>#/?v=<version>&dh=<recipientKey>`.
2. **Relay servers see only ciphertext.** Messages flow through "SMP" relays
   as encrypted blobs. E2E uses a double ratchet, so relay operators — even
   malicious ones — can't read content. Relays *do* see metadata like queue
   activity and timing, which is why self-hosting is a privacy (not secrecy)
   measure.
3. **Messages live on device.** There is no long-horizon inbox. If a recipient
   is offline for a long time, delivery is not guaranteed.

Consequence for integration: **there's no "send as user" API we could call
from the IdeaForge backend without first provisioning a SimpleX identity for
that user and holding their keys.**

Primary sources:
- <https://simplex.chat/docs/server.html> — server architecture
- <https://simplex.chat/docs/cli.html> — CLI / WebSocket control API
- <https://github.com/simplex-chat/simplexmq/blob/stable/protocol/simplex-messaging.md> — protocol
- <https://simplex.chat/faq/> — threat model

## 4. Architecture options

### Option A — Platform-hosted identities ("bot-per-user")

IdeaForge provisions one SimpleX identity per user, running inside a shared
CLI process on our infrastructure. The browser talks to our backend, our
backend talks to SimpleX.

- **Pros:** works with plain browser tabs, no external app required. Identical
  UX to in-app chat.
- **Cons:** the platform holds every user's SimpleX keys. **This is not E2E
  from the user's point of view.** We'd have to either:
  - truthfully describe it as "encrypted in transit, hosted by IdeaForge"
    (at which point SimpleX buys us little over any hosted chat stack), or
  - misdescribe it, which erodes trust the first time someone looks closely.
- **Liability:** we become the custodian of private messages. Subpoena
  exposure, insider-access risk, data-breach blast radius all grow. The
  original feedback's stated intent was "safe communication" — Option A
  silently fails that bar.

**Verdict: reject for Phase 1.** Revisit only if paired with a user-visible
disclosure that clearly distinguishes it from true E2E, and only as a
convenience tier alongside the real thing.

### Option B — Matchmaker with bring-your-own-client (recommended)

Each user has their own SimpleX client (official mobile/desktop app). In
IdeaForge, a "Connect on SimpleX" button on another user's profile triggers
an invitation-link handshake brokered by IdeaForge.

- User A clicks "Message on SimpleX" on user B's IdeaForge profile.
- IdeaForge backend asks user A's already-paired SimpleX identity (which
  IdeaForge knows about only as an address, not as keys) to emit a one-time
  invitation link.
- Link is delivered to user B through the authenticated IdeaForge channel (a
  notification with an `smp://…` deep link).
- User B clicks; their SimpleX client handles the handshake. The chat from
  here on is peer-to-peer between users A and B via SMP relays.
- IdeaForge has never seen plaintext and has never held keys. The cryptographic
  claim is intact.

- **Pros:** true E2E, matches the stated requirement, aligns with SimpleX's
  threat model, no message custody liability.
- **Cons:** both parties need SimpleX installed. First-time UX includes a
  "get SimpleX" nudge. No in-browser chat window; messaging happens in the
  SimpleX app.

**Verdict: start here.**

### Option C — Hybrid (Option B default, Option A opt-in later)

Ship Option B first. Later, for users who don't want to install another app,
add an opt-in "host my SimpleX identity on IdeaForge" toggle with a clear
disclosure that this tier is *not* E2E. Gives us the maximum-privacy default
and a convenience lane for users who explicitly trade it off.

**Verdict: plausible Phase 2+ direction, not now.**

### How users link their SimpleX address

For Option B, each IdeaForge user needs to tell the platform *what their
SimpleX address is* so we can address notifications / handshake pokes to them.
Two patterns:

1. **Paste-in address:** user opens their SimpleX app, copies their
   long-term contact link, pastes it into IdeaForge settings. IdeaForge
   stores the `smp://…` URI on their user profile. Simplest; no server-side
   SimpleX state needed.
2. **Connect via our bot:** IdeaForge runs a SimpleX bot with a known
   address; the user connects to it once from their app; the bot records
   the user's SimpleX `chatId` and ties it to their IdeaForge user id. More
   robust (we can send actual SimpleX messages to the user, like "new match
   — tap here to accept their invite"); also what the existing
   `simpleXbot` project already does for a different use case.

Pattern 2 is strictly richer. We can ship pattern 1 in Phase 1 and migrate
to pattern 2 when we want the bot to actually message users (rather than
just hand back an invitation link via the IdeaForge UI).

## 5. Infrastructure

- **1x self-hosted SMP relay.** SimpleX provides an install script; runs on a
  small VPS. Ports 80, 443, 5223. State in `/etc/opt/simplex/` (config, TLS)
  and `/var/opt/simplex/` (queues, logs). The CA private key should be
  generated offline and stored off-box so the server fingerprint (embedded in
  queue URIs) survives cert rotation. Source:
  <https://simplex.chat/docs/server.html>.
- **1x self-hosted XFTP relay** for file transfer privacy (optional for Phase
  1; only matters if we expose in-DM file sharing).
- **0 platform-side SimpleX state in Option B.** The IdeaForge backend stores
  users' SimpleX addresses as opaque strings; it does not hold keys, does not
  run a CLI, does not see message bodies.
- **Optional "matchmaker bot"** (for pattern 2 of §4): one long-running
  SimpleX CLI process on the IdeaForge backend host, owning a single bot
  identity that users can connect to so we can notify them. This is exactly
  the shape of our existing `~/Documents/projects/simpleXbot` — see §8.

## 6. Data model changes

Minimal. One new column, one new table.

```sql
-- Users opt in by registering their public SimpleX contact link.
ALTER TABLE users
  ADD COLUMN simplex_address TEXT;  -- nullable; long-term smp://… URI

-- Audit trail of handshake intents brokered by IdeaForge. We do NOT store
-- message content. We store who asked to connect to whom and when, so we can
-- rate-limit abuse and show "pending" state in the UI until the recipient
-- accepts in their SimpleX client.
CREATE TABLE simplex_handshake_intents (
  id UUID PRIMARY KEY,
  initiator_user_id UUID NOT NULL REFERENCES users(id),
  target_user_id    UUID NOT NULL REFERENCES users(id),
  one_time_link     TEXT NOT NULL,         -- generated by initiator's app/bot
  state             TEXT NOT NULL DEFAULT 'sent',  -- sent | consumed | expired | declined
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at        TIMESTAMPTZ NOT NULL,
  UNIQUE (initiator_user_id, target_user_id, created_at)
);
CREATE INDEX idx_handshakes_target ON simplex_handshake_intents (target_user_id, state);
```

Notice: **no `messages` table.** Phase 1 intentionally does not persist any
message content server-side. That's what gives us the "we can't read it" claim.

## 7. UX flow (Option B, pattern 1)

Onboarding:

1. User visits `/settings` → "Private messaging" section.
2. CTA: *"Paste your SimpleX contact link to let other IdeaForge users message
   you privately."* + a hint: *"Don't have SimpleX? [Install it](https://simplex.chat)."*
3. User pastes `smp://…`. Backend validates (regex + optionally a real
   handshake probe later). Stores on `users.simplex_address`.

Initiating a conversation:

1. User A on user B's profile sees "Message privately on SimpleX."
2. Click → backend generates a handshake intent row, returns user B's public
   SimpleX contact link to user A's browser. Frontend opens `smp://…` (which
   the OS hands off to the installed SimpleX app).
3. User A's SimpleX app connects to user B's address. User B gets a connection
   request in their SimpleX app (not in IdeaForge). They accept or decline
   there.
4. IdeaForge shows "SimpleX invitation sent" on the initiator's side until
   the TTL expires (say 72h). No polling of message state — SimpleX is the
   source of truth; IdeaForge just recorded that we sent a poke.

Pattern 2 variant (future): instead of the browser opening `smp://…`, the
IdeaForge bot sends user B a notification *inside SimpleX* that says "User A
on IdeaForge wants to connect — tap here." Higher-trust UX, needs the bot.

## 8. What we can reuse from `~/Documents/projects/simpleXbot`

The existing Python SimpleX bot is a bot-per-platform (pattern 2) deployment
we've already battle-tested against the SimpleX CLI's quirks. Relevant pieces:

- **`bot.py:105–115`** — Raw-JSON override of the python-simplex-bot library's
  Pydantic parsing. Unfixed upstream as of v0.0.1. Any new bot must do the
  same. Already a sunk cost there; we'd redo it in a new language.
- **Systemd topology in `deploy/`** — `simplex-chat.service` (CLI on
  `-p 5225`) + `simplex-rss-bot.service` depending on it, both with
  `Restart=on-failure`. Lift directly.
- **`setup.sh`** — pins the SimpleX CLI to a specific version
  (`v6.4.11`). The wire protocol is not stable across majors; pin explicitly.
- **Reconnection loop (`_connect` / `_message_handler`)** — WebSocket drops
  are routine; the retry-with-backoff pattern in this bot is battle-tested.
- **`ChatTarget` dataclass + `send_to()` abstraction** — encodes the
  `chatType` ("chat" vs "group") split we'd otherwise re-derive.
- **SQLite subscriptions schema** — not directly reusable, but the
  `(chat_id, chat_type)` key shape is exactly what a matchmaker-bot pattern
  also needs to map SimpleX contacts back to IdeaForge users.

**What does not carry over:**

- The RSS-feed monitoring loop — application-specific.
- Systemd deployment for the SimpleX CLI — for IdeaForge we'd want
  docker-compose alongside the rest of the stack. `setup.sh` is useful as a
  reference of what needs to happen, not as a drop-in.
- Python as the implementation language — if we keep IdeaForge Rust-native,
  we'd use [`simploxide`](https://github.com/a1akris/simploxide) (the Rust
  SimpleX client wrapper, third-party, solo-maintainer, v0.9.x as of Mar
  2026). Small risk surface; budget for patching / vendoring.

## 9. Rollout phases

**Phase 1 — "Paste your address" (minimum viable matchmaker):**
- [ ] Migration: `users.simplex_address` column + handshake intents table.
- [ ] Backend: `PUT /api/v1/users/me/simplex` (validate + store address),
  `POST /api/v1/users/:id/simplex-connect` (check target has an address,
  create intent row, return target's address to requester).
- [ ] Frontend: Settings card to paste/change the address. Profile "Message
  privately" button that opens `smp://…` as a deep link.
- [ ] Docs: README section on how to install SimpleX + get your contact
  link. Include a screenshot.
- **Server-side SimpleX state: none.** No CLI, no bot, no SMP relay yet —
  Phase 1 is pure glue.
- **Time estimate:** 2–3 dev-days.

**Phase 2 — Self-hosted SMP relay + optional matchmaker bot:**
- [ ] Deploy `smp-server` on our infra. Serve it as a *recommended relay*
  in SimpleX address suggestions.
- [ ] Run the matchmaker bot on backend host (docker-compose service).
  Pattern 2 from §4: users connect to the bot once; from then on, IdeaForge
  can push notifications to them inside SimpleX.
- [ ] Wrap `simploxide` in a thin Rust client crate under
  `src/crates/ideaforge-simplex` (new).
- [ ] Swap handshake UX: instead of opening `smp://…` in the browser, the
  bot messages the recipient in SimpleX.
- **Time estimate:** 5–7 dev-days once `simploxide` is vetted.

**Phase 3 — Convenience tier (only if clearly demanded):**
- [ ] Opt-in "IdeaForge-hosted" messaging (Option A). **Must** ship behind
  a user-visible disclosure explaining the reduced privacy guarantee. Keep
  Option B as the default and recommended path.

## 10. Open questions for the founder

Before Phase 1 starts, we need your call on:

1. **Privacy framing.** Do you want to ship only true E2E (Option B), or
   also a convenience-tier hosted option (Option C) with a "not E2E"
   disclosure? My recommendation is E2E-only at launch.
2. **Self-hosted SMP relay in Phase 1?** Pure E2E works on SimpleX's public
   relays. Self-hosting adds metadata privacy but also ops burden. I'd
   defer to Phase 2 unless the launch narrative needs "we run our own
   relays" for credibility.
3. **License tolerance.** SimpleX Chat is AGPL-3.0. Running the CLI as a
   separate process (and only talking to it over WebSocket) is the normal
   arm's-length pattern, but I'd run it past legal before we ship a
   commercial offering that depends on SimpleX binaries in our deployment.
4. **Bot identity.** For Phase 2, what's the bot's handle and voice?
   ("IdeaForge" vs some more colourful forge metaphor?) This is the
   persona users see inside their SimpleX app.

## 11. What this spike does NOT cover

- Group messaging (SimpleX supports it; out of scope until we have a clear
  need — per-idea team chats might be a natural second feature).
- Storing or searching DM content on IdeaForge. Intentional: that capability
  is what breaks the E2E claim.
- File transfer via XFTP relays. Defer to Phase 2 with XFTP self-hosting.
- Federation with Matrix / XMPP / other messaging stacks. Not compatible
  with SimpleX's threat model.

## 12. Can SimpleX run inside the browser (WASM)?

**Short answer: no, not realistically in 2026.** Three independent blockers:

1. **SimpleX's client is Haskell.** The GHC WebAssembly backend exists but is a
   tech preview — single-threaded runtime, incomplete WASI on browser JS
   shims, async-FFI restrictions. GHCi-in-browser works; shipping a program
   the size of the SimpleX client does not. No experimental SimpleX-WASM port
   has surfaced.
   Source: <https://ghc.gitlab.haskell.org/ghc/doc/users_guide/wasm.html>,
   <https://www.tweag.io/blog/2025-04-17-wasm-ghci-browser/>
2. **Transport is *not* the blocker.** The SMP protocol is transport-agnostic,
   and SimpleX relays can optionally expose WebSocket on port 443 alongside
   the default TLS/TCP on 5223. A browser client could reach a relay over
   WSS; it just needs the client code to exist.
   Source: <https://github.com/simplex-chat/simplexmq/blob/stable/protocol/simplex-messaging.md>
3. **No Rust SimpleX client exists.** `simploxide` is a *wrapper* around the
   SimpleX CLI's WebSocket control API, not a re-implementation of the
   protocol. It can't run standalone in a browser because it assumes a local
   CLI process on the other end of the socket.

### Cookies as key storage — explicitly wrong

The original question asked about "using cookies." Don't. Cookies are the
wrong primitive for cryptographic material:

- **4 KB size limit** per cookie, ~80 KB per origin — real key material and
  ratchet state won't fit.
- **Automatically sent on every request** to the origin — huge ambient
  exfiltration surface and bandwidth waste.
- **`HttpOnly` prevents JS access** — which is what you'd need to prevent
  XSS theft — **but also prevents WebCrypto/WASM from using the key at
  all.** So either the cookie is usable (and XSS-readable) or unusable.

The right primitives are, in order of preference:

1. **`CryptoKey` objects with `extractable: false`, persisted in IndexedDB.**
   Key material never becomes visible to JS or WASM; only the browser's
   crypto layer can use the handle. Works with WebCrypto's supported
   algorithms (ECDH P-256/P-384/X25519, AES-GCM, HKDF).
2. **WebAuthn PRF extension** to derive a keystore-unwrapping secret from a
   passkey or hardware key. Broadly supported in 2025–26 (Chrome, Safari
   17.4+, iOS 18.4+, Android). Best when you want keys bound to a user's
   hardware.
3. **OPFS (Origin Private File System)** for storing wrapped-key blobs.

Caveat: **WebCrypto has no ML-KEM/Kyber support.** Post-quantum keys
necessarily live in WASM-managed memory and must be wrapped at rest under a
non-extractable AES-GCM key bound to the user.

## 13. Browser-native post-quantum E2E alternatives

Given SimpleX-in-WASM is out, the browser-native options with a credible PQ
story are:

| Option | Status for browser | PQ story | Verdict |
|---|---|---|---|
| **OpenMLS + X-Wing ciphersuite** | `openmls-wasm` crate ships; ≤500 KB gzipped target | MLS ciphersuite `MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519` (ML-KEM + X25519 hybrid, via libcrux's formally verified ML-KEM) | **Strongest candidate** |
| `awslabs/mls-rs` | WASM builds advertised | PQ ciphersuite possible via AWS-LC (ML-KEM available) but not advertised by default | Alternative to OpenMLS |
| Signal Protocol in browser | **No official WASM build.** `libsignal` TypeScript binding is Node-only. A third-party `getmaapp/signal-wasm` exists but is unaffiliated and self-discloses unsafe key handling. | PQXDH (X25519 + Kyber-768 hybrid) in production libsignal | **Reject** — relying on an unaffiliated third-party fork for cryptographic security is not acceptable |
| WebRTC data channel + manual ML-KEM handshake | Viable — run ML-KEM in WASM, derive symmetric key, ChaCha20-Poly1305 over the data channel | Roll-your-own PQ handshake using a Kyber WASM lib (libcrux, pqc_kyber) | Works for 1:1 only; reinvents what MLS specifies; skip unless MLS is too heavy |
| SimpleX via self-hosted WS-to-TCP bridge | Transport possible, but no browser SimpleX client exists to drive it (see §12) | SimpleX v5.6+ uses sntrup761 in the Double Ratchet (deliberately not ML-KEM) | Not tractable for us to build |

Sources: <https://blog.openmls.tech/posts/2024-04-11-pq-openmls/>,
<https://github.com/openmls/openmls>,
<https://github.com/awslabs/mls-rs>,
<https://github.com/signalapp/libsignal/issues/350>,
<https://simplex.chat/blog/20240314-simplex-chat-v5-6-quantum-resistance-signal-double-ratchet-algorithm.html>,
<https://blog.projecteleven.com/posts/guaranteeing-post-quantum-encryption-in-the-browser-ml-kem-over-websockets>

### Why MLS fits IdeaForge specifically

RFC 9420 separates the **Delivery Service (DS)** from the protocol. The DS's
job is to fan out encrypted Welcome/Commit/Application messages and to serve
KeyPackages; **it never needs plaintext access**. This maps cleanly onto an
Axum service we already know how to run, and the cryptographic claim "the
platform cannot read your messages" is enforceable by the protocol — the DS
genuinely does not have the keys. Groups come for free (MLS is
group-messaging-native), so per-idea team chats become an easy follow-up.

Sources: <https://datatracker.ietf.org/doc/rfc9420/>,
<https://www.rfc-editor.org/rfc/rfc9750.html>

## 14. The browser-trust hole — you can't hide from it

**Critical caveat, stated plainly:** any browser-based E2E system, no matter
how good the crypto, shares a fundamental property: **the platform ships the
code that does the crypto**. On the next deploy IdeaForge could push a
malicious WASM bundle that exfiltrates keys, and the user has no
cryptographic defence against that. The platform can always break the
crypto by backdooring itself. This is not an MLS problem or a SimpleX
problem; it is a *the platform ships the client* problem.

Signal's public position (via the *Security Cryptography Whatever* podcast,
2023) is to *not* ship a pure-web Signal client precisely because of this.
TLS compromise ≡ app compromise; same-origin policy does not recover the
gap. Matrix / Element Web accepts this risk in practice. SimpleX has no web
client at all.

The industry-standard mitigations are layered; none are individually
sufficient:

- **Subresource Integrity (SRI)** — every JS/WASM/CSS URL has a
  `sha256/384/512` hash in its `<script>`/`<link>` tag. Browser refuses to
  execute a loaded asset whose hash doesn't match. Requires the entry-point
  HTML to be trusted; usually served from a *separate origin* to narrow the
  attack surface.
  Source: <https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity>
- **Reproducible builds + published signed hashes** of the deployed WASM
  bundle (sigstore / minisign). Independent observers can verify the
  deployed artefact matches the public source tree for each release.
- **Binary transparency log** of deployed client hashes, so any surreptitious
  build swap leaves a verifiable public record.
- **Independent audits** of the reproducible-build pipeline.

Ship all of these and you get "trust through transparency": the platform
*could* backdoor itself, but it cannot do so *silently*. That's a
meaningfully different posture from "trust us," but it is not
cryptographic independence.

**The only way to get genuine cryptographic independence of the platform is a
client the user installs separately** — which is exactly what Option B of
the original spike proposes, using SimpleX's official apps.

## 15. Revised recommendation — two honest paths

The original spike assumed only Option B was on the table. The research
above adds a credible Option D. They serve different audiences:

- **Path 1: Matchmaker → SimpleX (Option B of §4).** Users install SimpleX;
  IdeaForge brokers invitation links; the platform cryptographically
  cannot read messages. PQ via SimpleX's sntrup761.
  - Honest claim: **true E2E, cryptographically independent of the
    platform, post-quantum.**
  - UX cost: users must install SimpleX.
  - Audience: users who actually need the strongest privacy — journalists,
    activists, sensitive-IP founders.

- **Path 2: OpenMLS in WASM with X-Wing ciphersuite.** IdeaForge ships the
  client as WASM, runs an Axum-based Delivery Service that only sees
  ciphertext, and publishes SRI hashes + reproducible builds + a binary
  transparency log.
  - Honest claim: **E2E under the assumption that the platform has not
    backdoored the delivered WASM bundle. Post-quantum hybrid. Verifiable
    through published hashes.** Do *not* say "we can't read your messages"
    unqualified — say "the server does not see plaintext, and you can
    verify we're shipping what we say we're shipping."
  - UX cost: none beyond a settings toggle.
  - Audience: regular users who want good-enough encryption without
    installing another app.

Either is defensible. The bad option is a hosted hybrid that calls itself
E2E without the asterisk — which is where we'd land if we naïvely "put
SimpleX on the server for the user." That's why this spike exists.

### Effort comparison

| Path | Phase 1 effort | Phase 2 effort | Notes |
|---|---|---|---|
| Path 1 (Matchmaker→SimpleX) | 2–3 dev-days (no server SimpleX state) | 5–7 dev-days (self-hosted SMP relay + matchmaker bot) | Reuses `~/Documents/projects/simpleXbot` heavily |
| Path 2 (OpenMLS WASM + DS) | 10–14 dev-days (WASM client integration, DS fan-out service, KeyPackage storage, keystore, SRI pipeline) | 5+ dev-days (WebAuthn PRF unlock, reproducible-build pipeline, public transparency log) | Most risk is in the WASM bundle size and the reproducible-build plumbing |

### Revised decision ask

Before either path starts, four questions to answer:

1. **Which audience are we building for first?** If "everyone," Path 2 is
   the pragmatic default. If "users who need the strongest guarantee,"
   Path 1. If both, ship Path 1 first (smaller scope), add Path 2 as a
   later convenience tier with the transparency story.
2. **Is the "install SimpleX" UX a deal-breaker?** If yes → Path 2. If no
   or not yet clear → Path 1 and gather evidence.
3. **Do we have bandwidth for the reproducible-build / SRI / transparency-log
   plumbing Path 2 needs to be honest?** Without it, Path 2's privacy
   claim collapses to "trust us."
4. **Do we want groups (per-idea team chats) in the same system?** MLS (Path
   2) makes groups native; SimpleX (Path 1) has small groups but the
   matchmaker UX gets clunky at >2 people.

---

*Author: Claude Code, 2026-04-14. Revisit before Phase 1 kickoff.*
