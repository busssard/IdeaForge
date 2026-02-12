# IdeaForge Intellectual Property Protection

## Table of Contents

1. [Executive Summary](#Executive%20Summary)
2. [1. Idea Openness Levels](#1.%20Idea%20Openness%20Levels)
3. [2. Handling Secret Ideas Securely](#2.%20Handling%20Secret%20Ideas%20Securely)
4. [3. NDA-Like Mechanisms](#3.%20NDA-Like%20Mechanisms)
5. [4. Timestamping and Proof of Priority](#4.%20Timestamping%20and%20Proof%20of%20Priority)
6. [5. IP Lawyer Integration Workflow](#5.%20IP%20Lawyer%20Integration%20Workflow)
7. [6. Data Compartmentalization for Secret Ideas](#6.%20Data%20Compartmentalization%20for%20Secret%20Ideas)
8. [7. AI-Generated IP Considerations](#7.%20AI-Generated%20IP%20Considerations)
9. [8. Architecture Alignment Notes (Updated Cross-Review Round 2)](#8.%20Architecture%20Alignment%20Notes%20(Updated%20Cross-Review%20Round%202))
10. [9. Key Risks & Mitigations](#9.%20Key%20Risks%20&%20Mitigations)

## Executive Summary

Intellectual property protection is a core differentiator for IdeaForge. The platform must handle three openness levels -- open source, commercial, and secret -- with the "secret" level requiring the most rigorous security. This document defines mechanisms for protecting ideas, establishing priority, integrating legal services, and compartmentalizing sensitive data.

---

## 1. Idea Openness Levels

### 1.1 Open Source Ideas

| Aspect | Policy |
|--------|--------|
| **Visibility** | Fully public; indexable by search engines |
| **Contributions** | Anyone can contribute; contributions default to project license |
| **License** | Entrepreneur selects from standard licenses (MIT, Apache 2.0, GPL, CC-BY, etc.) |
| **IP Ownership** | Shared per license terms |
| **Financial** | Pledges and donations; no equity structures |

### 1.2 Commercial Ideas

| Aspect | Policy |
|--------|--------|
| **Visibility** | Publicly visible at summary level; detailed plans may require sign-in |
| **Contributions** | Contributors agree to IP assignment or licensing terms before contributing |
| **License** | Proprietary; terms defined by entrepreneur |
| **IP Ownership** | Entrepreneur retains ownership; contributor agreements define transfer |
| **Financial** | Full financial suite: pledges, investments, freelancer payments |

### 1.3 Secret / IP-Protected Ideas

| Aspect | Policy |
|--------|--------|
| **Visibility** | Title and category only visible publicly; all details encrypted and access-controlled |
| **Contributions** | Requires NDA signature and entrepreneur approval before any access |
| **License** | Proprietary; strictly controlled |
| **IP Ownership** | Entrepreneur retains full ownership; all contributors sign IP assignment |
| **Financial** | Available after NDA; investor access requires additional verification |
| **Legal Review** | Recommended IP lawyer review before any disclosure |

---

## 2. Handling Secret Ideas Securely

### 2.1 Architecture for Secret Idea Storage

```
Entrepreneur creates secret idea
    |
    v
[Client-Side Encryption (optional, for maximum security)]
    |
    v
[Server receives encrypted content]
    |
    v
[Server-side encryption with per-idea key]
    |
    v
[Encrypted content stored in database]
    |
    v
[Per-idea encryption key wrapped by master key in KMS]
```

#### Technical Implementation

1. **Per-idea encryption**: Each secret idea gets a unique AES-256-GCM data encryption key (DEK)
2. **Key wrapping**: DEKs are wrapped (encrypted) by a master key stored in HSM-backed KMS
3. **Access-controlled decryption**: Decryption requires valid session + verified NDA + entrepreneur approval
4. **In-memory processing**: Decrypted content never written to disk; processed in memory only
5. **No caching**: Secret idea content excluded from all caches (CDN, application, browser)

#### Storage Isolation

- Secret idea content stored in a **separate database or schema** from public content
- Database access requires dedicated credentials (not shared with public data service)
- Database connections routed through a dedicated proxy with access logging
- Backup encryption with separate key hierarchy
- Physical isolation (separate cloud account/VPC) recommended for highest-sensitivity ideas

### 2.2 Access Control for Secret Ideas

#### Access Request Flow

```
User requests access to secret idea
    |
    v
[System checks: user authenticated? MFA verified?]
    |-- No --> Deny
    v
[System presents NDA for signature]
    |
    v
[User signs NDA (electronic signature, legally binding)]
    |
    v
[Entrepreneur notified of access request]
    |
    v
[Entrepreneur reviews user profile and approves/denies]
    |-- Deny --> User notified, access denied
    v
[Access granted with time limit and audit logging]
    |
    v
[User can view secret idea content]
    |
    v
[Every access logged: who, when, what sections, duration]
```

#### Access Constraints
- Access is **time-limited** (default: 90 days, renewable by entrepreneur)
- Entrepreneur can **revoke access** at any time (immediate effect)
- **Maximum viewers**: Entrepreneur sets limit on concurrent accessors
- **No download**: Secret idea content rendered server-side; no raw data downloads
- **Watermarking**: Displayed content includes invisible watermarks identifying the viewer
- **Screenshot deterrence**: Visible watermark overlay with viewer identity (similar to financial document platforms)

### 2.3 Content Leak Detection

- **Invisible watermarking**: Each viewer sees unique, imperceptible modifications to text/images
- **If leaked**: Watermark analysis identifies which viewer's version was leaked
- **Monitoring**: Optional service that scans public internet for leaked content patterns
- **Incident response**: Automated NDA breach notification with evidence package

---

## 3. NDA-Like Mechanisms

### 3.1 Platform NDA Framework

IdeaForge provides standardized NDA templates, not custom legal documents. Users should consult their own counsel for high-value ideas.

#### Standard NDA Terms
- **Scope**: Covers all information disclosed through the secret idea interface
- **Duration**: 2-year confidentiality period from last access date
- **Obligations**: No disclosure, no use outside the platform context, no reverse engineering
- **Exceptions**: Publicly available information, independently developed knowledge, legally compelled disclosure
- **Remedies**: Liquidated damages clause, platform account suspension/termination
- **Jurisdiction**: Selectable by entrepreneur (recommended: entrepreneur's home jurisdiction)

#### Electronic Signature
- Legally binding e-signature (compliant with US ESIGN Act, EU eIDAS Regulation)
- Signature includes: full name, date/time, IP address, device fingerprint
- Signed NDA stored immutably (blockchain-anchored hash for tamper-proof record)
- Both parties receive a copy via email

### 3.2 Tiered NDA Levels

| Level | Use Case | Requirements |
|-------|----------|-------------|
| **Standard** | View idea summary and general approach | Sign platform NDA template |
| **Extended** | View full technical details and business plan | Sign extended NDA + identity verification |
| **Custom** | High-value ideas ($100K+ potential) | Upload and counter-sign entrepreneur's custom NDA |

### 3.3 Limitations and Disclaimers

- IdeaForge is **not a law firm** and does not provide legal advice
- Platform NDAs are templates; enforceability depends on jurisdiction
- Entrepreneurs with high-value ideas should engage their own IP counsel
- Platform facilitates NDA signing but does not enforce or litigate breaches
- Users are informed that no technical measure can fully prevent determined leaks

---

## 4. Timestamping and Proof of Priority

### 4.1 Blockchain-Based Timestamping

IdeaForge uses Cardano blockchain to create immutable, timestamped records of idea creation and evolution.

#### How It Works

```
Entrepreneur submits idea
    |
    v
[Platform generates SHA-256 hash of idea content]
    |
    v
[Hash submitted as metadata in Cardano transaction]
    |
    v
[Transaction confirmed on-chain (within ~20 seconds)]
    |
    v
[Entrepreneur receives timestamp certificate]
    |
    +-- Transaction ID (on-chain reference)
    +-- Block number and slot
    +-- Content hash (SHA-256)
    +-- Timestamp (UTC)
    +-- Certificate PDF (downloadable)
```

#### What Gets Timestamped
- **Initial submission**: Full idea content hash at creation
- **Major updates**: New hash for each maturity level advancement
- **Contributions**: Individual contributor submissions timestamped separately
- **Milestones**: Key project milestones recorded on-chain

#### Legal Weight
- France's Tribunal Judiciaire de Marseille (March 2025) recognized blockchain timestamping as reliable proof of IP ownership -- a growing legal precedent
- Blockchain timestamps provide **evidence of prior art** and **proof of existence at a point in time**
- Timestamps are **complementary** to formal patent filings, not a replacement
- The platform clearly communicates: timestamp proves "this content existed at this time," not "this person invented this"

### 4.2 Proof of Concept Priority

#### Priority Chain
Each idea maintains a verifiable priority chain:

```json
{
  "idea_id": "IF-2026-00042",
  "priority_chain": [
    {
      "event": "idea_created",
      "timestamp": "2026-03-15T14:22:00Z",
      "content_hash": "sha256:a1b2c3...",
      "tx_id": "cardano:tx_abc123...",
      "block": 12345678
    },
    {
      "event": "maturity_advanced",
      "from": "half_baked",
      "to": "thought_through",
      "timestamp": "2026-04-01T09:15:00Z",
      "content_hash": "sha256:d4e5f6...",
      "tx_id": "cardano:tx_def456...",
      "block": 12367890
    }
  ]
}
```

#### Verification
- Anyone with the idea content can independently verify the timestamp
- Verification requires: original content + Cardano blockchain access
- Platform provides a public verification tool (does not require account)
- Third-party verification possible using any Cardano block explorer

### 4.3 Cost and Performance

- Cardano transaction fee: approximately 0.17-0.25 ADA per timestamp (~$0.05-0.10 at current prices)
- Batching: Multiple idea hashes can be combined in a single transaction (Merkle tree)
- Cost to platform: Negligible at scale (100 timestamps/day = ~$10/day)
- Confirmation time: ~20 seconds (1 block confirmation), ~60 seconds for higher assurance

---

## 5. IP Lawyer Integration Workflow

### 5.1 When Legal Review Is Recommended

The platform proactively recommends legal consultation in these scenarios:

| Trigger | Recommendation |
|---------|---------------|
| Idea marked as "secret" | "Consider IP counsel before sharing with anyone" |
| Idea advances to "serious proposal" | "Consult a patent attorney about formal protection" |
| First investor interest | "Review investment terms with legal counsel" |
| Cross-border contributors | "Review IP assignment enforceability across jurisdictions" |
| AI agent contribution to secret idea | "Clarify AI-generated IP ownership with counsel" |

### 5.2 Lawyer Marketplace Integration

#### Vetted Lawyer Network
- Partner with IP law firms experienced in:
  - Patent filing (utility and provisional)
  - Trade secret protection
  - Software IP
  - International IP treaties (PCT, Madrid Protocol)
  - Blockchain/crypto-related IP (emerging specialty)
- Lawyers vetted by platform: bar verification, specialization confirmation, client reviews

#### Workflow

```
Entrepreneur requests legal consultation
    |
    v
[Platform presents vetted lawyer directory]
    |
    +-- Filter by: jurisdiction, specialization, language, price range
    |
    v
[Entrepreneur selects lawyer and requests consultation]
    |
    v
[Lawyer receives encrypted idea summary (NDA pre-signed)]
    |
    v
[Consultation scheduled (video call, in-platform messaging)]
    |
    v
[Lawyer provides recommendation]
    |
    +-- "File provisional patent" -> Platform links to USPTO/EPO resources
    +-- "Strengthen trade secret protections" -> Platform adjusts access controls
    +-- "Proceed with current protections" -> Documented in idea record
    |
    v
[Consultation record stored (encrypted, accessible to entrepreneur and lawyer only)]
```

#### Revenue Model
- Platform takes a referral fee (10-15%) on lawyer consultations booked through the platform
- Free initial 15-minute consultations sponsored by partner firms (lead generation for them)
- Subscription tier for entrepreneurs: unlimited basic consultations for $99/month

### 5.3 Self-Service IP Tools

For entrepreneurs who cannot afford full legal counsel:

1. **IP Readiness Assessment**: Questionnaire that evaluates idea's patentability, trade secret suitability, and copyright applicability
2. **NDA Generator**: Customizable NDA template with jurisdiction-specific clauses
3. **Prior Art Search**: Integration with Google Patents, USPTO, EPO databases
4. **IP Timeline**: Visual history of all timestamps, access grants, and contributions
5. **IP Education**: Guides on patent vs. trade secret vs. copyright, written by partner law firms

---

## 6. Data Compartmentalization for Secret Ideas

### 6.1 Compartmentalization Architecture

```
+--------------------------------------------------+
|                  Public Zone                       |
|  - Open ideas (full content)                      |
|  - Commercial ideas (summary + sign-in content)   |
|  - User profiles (public portions)                |
|  - Community activity (votes, comments)           |
+--------------------------------------------------+
                      |
              [API Gateway + WAF]
                      |
+--------------------------------------------------+
|              Authenticated Zone                    |
|  - User private data (encrypted)                  |
|  - Financial transaction records (encrypted)      |
|  - NDA records and signatures                     |
|  - Commercial idea full content                   |
+--------------------------------------------------+
                      |
        [Secret Idea Access Proxy]
        (dedicated service, separate credentials)
                      |
+--------------------------------------------------+
|            Secret Ideas Zone (Isolated)            |
|  - Per-idea encrypted content                     |
|  - Separate database cluster                      |
|  - Separate encryption keys                       |
|  - Access logging to separate audit store         |
|  - No cross-zone data leakage                     |
+--------------------------------------------------+
```

### 6.2 Technical Controls

#### Network Isolation
- Secret ideas zone runs in a separate VPC/subnet
- No direct network path from public zone to secret zone
- Access only through the Secret Idea Access Proxy (mTLS authenticated)
- Proxy enforces: authentication, NDA verification, access approval, rate limiting

#### Data Isolation
- Separate database instance for secret idea content
- No JOINs or queries that span public and secret databases
- Secret idea metadata (title, category) stored separately from content
- Search indexes do not include secret idea content
- Backups stored in separate, encrypted storage with distinct access controls

#### Application Isolation
- Secret idea rendering service is a separate microservice
- Runs with minimal permissions (read-only on secret database)
- No access to public data services
- Memory-safe language recommended for rendering service (Rust, Go)
- Container hardened: read-only filesystem, no network egress except to database

#### Operational Isolation
- Separate on-call rotation for secret idea infrastructure
- Elevated access requires approval from 2 team members (dual control)
- No developer access to production secret idea data
- Debugging uses synthetic data; production issues investigated via logs only
- Quarterly access review: remove any access no longer needed

### 6.3 Secret Idea Lifecycle

```
Created (encrypted, isolated)
    |
    v
Reviewed by IP lawyer (optional, recommended)
    |
    v
Shared with NDA-signed collaborators (access-controlled)
    |
    v
Matured through stages (each stage timestamped)
    |
    v
Decision point:
    |
    +-- Remain secret (ongoing protection)
    +-- Transition to commercial (controlled disclosure)
    +-- Transition to open source (full publication)
    +-- Abandon (content retained for timestamp proof, access revoked)
```

#### Transition from Secret to Commercial/Open
- Irreversible: once disclosed, cannot return to secret
- Entrepreneur explicitly confirms transition with warning
- All NDA-signed viewers notified of openness change
- Blockchain timestamp records the transition event
- Historical access logs retained for audit

---

## 7. AI-Generated IP Considerations

### 7.1 Ownership Ambiguity

AI-generated ideas and contributions create IP ownership challenges:

| Jurisdiction | Current Status (2025-2026) |
|-------------|---------------------------|
| US | USPTO requires human inventor; AI cannot be listed as inventor (Thaler v. Vidal) |
| EU | Similar position; human authorship required for copyright |
| UK | CDPA allows computer-generated works to be owned by the "person who made the arrangements" |
| China | Evolving; some courts have recognized AI-assisted works |

### 7.2 Platform Policy for AI Contributions

1. **Disclosure required**: All AI-generated content must be labeled
2. **Human review**: AI-generated ideas require human co-author/sponsor for IP claims
3. **Contributor agreement**: AI agent operators sign terms acknowledging IP assignment rules
4. **Timestamping**: AI contributions timestamped with both AI agent ID and human operator ID
5. **Legal guidance**: Platform recommends IP counsel for ideas with significant AI contribution

### 7.3 Risk Mitigation
- Platform terms state that IdeaForge does not adjudicate IP ownership
- Users responsible for ensuring their IP claims are valid in their jurisdiction
- AI contributions flagged in IP timeline for legal review
- If AI agent contributes to a secret idea, the AI operator is bound by the same NDA terms
- **Policy decision**: AI agents cannot access secret/IP-protected ideas (see bot_transparency.md Section 4.2.4 "Human-Only Zones"). This eliminates the risk of AI-mediated IP leakage.

---

## 8. Architecture Alignment Notes (Updated Cross-Review Round 2)

Cross-referencing with the software architecture (`docs/architecture/`):

1. **Database schema**: **Resolved.** The architect has added a dedicated `secret_ideas` schema (`docs/architecture/database_schema.md` Section 5b) with three tables: `secret_ideas.idea_content` (encrypted BYTEA with per-idea DEK reference), `secret_ideas.access_grants` (NDA tracking with time-limited expiry), and `secret_ideas.access_log` (full audit trail). The `ideas` table uses an `openness` enum (`open_source`, `open_collaboration`, `commercial`, `secret`). This matches our security recommendations precisely. The system overview (`docs/architecture/system_overview.md`) now shows the isolated Secret Ideas Zone with mTLS proxy and HSM-backed KMS.

2. **Blockchain timestamping**: The architect's Aiken-based `PledgeEscrow` validator handles financial escrow. A separate, lightweight Aiken script (or Cardano metadata transaction) is still needed for IP timestamping. This is simpler than the escrow contract -- it only needs to store a content hash as transaction metadata. The `secret_ideas.idea_content` table includes a `content_hash` field (SHA-256) specifically for this purpose. **Remaining gap**: The Aiken timestamping script should be specified in `docs/architecture/blockchain_integration.md`.

3. **NDA signing flow**: **Partially resolved.** The `secret_ideas.access_grants` table stores `nda_signed_at` and `nda_document_id`, providing the database infrastructure. The product roadmap (`docs/design/roadmap.md`) places NDA automation in Phase 2 (Calling the Guild). For MVP, a click-through NDA acceptance is sufficient. For Phase 4 (The Finished Work), integrate with an e-signature provider (DocuSign, HelloSign) via the lawyer marketplace.

4. **Watermarking**: Not yet in the architecture. For secret idea content rendered in the browser, server-side text watermarking (invisible character substitution per viewer) can be implemented in the Leptos SSR layer without a separate service. **Open question for founder**: Is watermarking a Phase 2 or Phase 4 priority? The security recommendation is Phase 2 (when secret ideas first become accessible to NDA-signed collaborators).

---

## 9. Key Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Secret idea content leaked by NDA-signed viewer | High | Invisible watermarking per viewer, access audit logging, NDA breach notification with evidence, platform ToS enforcement |
| Blockchain timestamp not recognized in court | Medium | Use as supplementary evidence, not sole proof; recommend formal patent filing for high-value ideas; cite France 2025 legal precedent |
| NDA templates not enforceable in all jurisdictions | Medium | Clearly disclaim that platform NDAs are templates; recommend custom NDAs for high-value ideas; offer jurisdiction-specific variants for US, EU, UK, Switzerland |
| IP ownership dispute between entrepreneur and contributor | High | Clear contributor agreements signed before access; IP assignment terms in platform ToS; blockchain-timestamped contribution records |
| AI-generated contributions create IP ownership ambiguity | Medium | Require human co-author for IP claims; disclose AI contribution in IP timeline; recommend legal counsel for AI-heavy ideas |
| Secret idea isolation not implemented early enough | ~~High~~ **Resolved** | The architect has implemented the `secret_ideas` schema with per-idea encryption, access grants, and audit logging in Phase 1 (The Spark). See `docs/architecture/database_schema.md` Section 5b. |
| Cardano network fee volatility makes timestamping expensive | Low | Batch timestamps via Merkle trees (100+ hashes per transaction); current cost ~$0.05-0.10 per timestamp is negligible |
| Platform liability for failed IP protection | Medium | Clear disclaimers in ToS; platform facilitates but does not guarantee IP protection; recommend professional legal counsel |
