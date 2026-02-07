# IdeaForge - Cardano Blockchain Integration

## Overview

IdeaForge integrates with the Cardano blockchain for pledge-to-buy mechanics. Smart contracts hold pledged funds in escrow until either the idea's product is delivered (funds release to creators) or the pledge window expires (funds return to pledgers).

---

## 1. Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────────────┐
│  IdeaForge       │     │  Blockfrost API   │     │  Cardano Blockchain  │
│  Backend (Rust)  │────▶│  (REST gateway)   │────▶│                      │
│                  │     │                   │     │  ┌────────────────┐  │
│  - Pledge intent │     │  - Submit TX      │     │  │ Pledge Escrow  │  │
│  - TX building   │     │  - Query UTxOs    │     │  │ Validator      │  │
│  - TX monitoring │     │  - Monitor addrs  │     │  │ (Aiken)        │  │
│                  │     │                   │     │  └────────────────┘  │
└────────┬────────┘     └──────────────────┘     └──────────────────────┘
         │
         │ Wallet interaction
         ▼
┌──────────────────┐
│  User's Browser  │
│                  │
│  CIP-30 Wallet   │
│  (Nami, Eternl,  │
│   Lace, etc.)    │
└──────────────────┘
```

### Key Design Decisions

- **No private keys on server**: The backend never holds user funds. All transactions are signed client-side via CIP-30 wallet connectors.
- **Blockfrost as chain gateway**: Avoids running a full Cardano node. Provides REST API for UTxO queries, transaction submission, and address monitoring.
- **Aiken for smart contracts**: Rust-inspired language that compiles to Plutus Core (UPLC). Better developer experience than raw Plutus/Haskell.

---

## 2. Smart Contract Design (Aiken)

### Pledge Escrow Validator

The core smart contract is a **parameterized escrow validator** that locks pledged ADA until release conditions are met.

#### Datum (on-chain state per pledge)

```aiken
type PledgeDatum {
    idea_id: ByteArray,          // IdeaForge idea UUID
    pledger: VerificationKeyHash, // pledger's payment key
    creator: VerificationKeyHash, // idea author's payment key
    deadline: POSIXTime,          // expiration timestamp
    min_target: Int,              // minimum pledge target (lovelace)
}
```

#### Redeemer (actions)

```aiken
type PledgeAction {
    /// Creator claims funds after successful delivery
    Claim
    /// Pledger reclaims funds after deadline passes
    Refund
    /// Platform-initiated refund (e.g., idea abandoned)
    PlatformRefund
}
```

#### Validator Logic

```aiken
validator pledge_escrow(platform_key: VerificationKeyHash) {
    spend(datum: PledgeDatum, redeemer: PledgeAction, ctx: ScriptContext) {
        when redeemer is {
            Claim -> {
                // 1. Must be signed by creator
                // 2. Current time must be before deadline
                // 3. Total pledged at script address >= min_target
                // 4. Platform co-signature required (fraud prevention)
                must_be_signed_by(ctx, datum.creator)
                    && before_deadline(ctx, datum.deadline)
                    && target_met(ctx, datum.min_target)
                    && must_be_signed_by(ctx, platform_key)
            }
            Refund -> {
                // 1. Must be signed by original pledger
                // 2. Deadline must have passed OR target not met
                must_be_signed_by(ctx, datum.pledger)
                    && (after_deadline(ctx, datum.deadline)
                        || !target_met(ctx, datum.min_target))
            }
            PlatformRefund -> {
                // 1. Must be signed by platform
                // 2. Funds return to pledger
                must_be_signed_by(ctx, platform_key)
                    && pays_to(ctx, datum.pledger)
            }
        }
    }
}
```

### Platform Multisig

A platform co-signature is required for fund release (Claim) to prevent:
- Creators claiming before actual delivery
- Fraudulent projects

The platform key is a multisig requiring 2-of-3 platform administrators to sign.

---

## 3. Wallet Integration (CIP-30)

### Browser-Side Wallet Connection

The Leptos frontend integrates with Cardano wallets via the CIP-30 standard (dApp connector API).

```
// Leptos component (simplified)
Supported wallets: Nami, Eternl, Lace, Flint, GeroWallet, Typhon

Flow:
1. User clicks "Connect Wallet"
2. Frontend calls window.cardano.{wallet}.enable()
3. Wallet returns API handle
4. Frontend reads wallet address via api.getUsedAddresses()
5. Address is sent to backend and stored in user profile
```

### Transaction Building Flow

```
┌──────┐     ┌─────────┐     ┌─────────────┐     ┌──────────────┐
│ User │     │ Frontend │     │   Backend    │     │  Blockchain  │
└──┬───┘     └────┬─────┘     └──────┬───────┘     └──────┬───────┘
   │              │                   │                    │
   │ Click        │                   │                    │
   │ "Pledge 50   │                   │                    │
   │  ADA"        │                   │                    │
   │─────────────▶│                   │                    │
   │              │  POST /pledges    │                    │
   │              │──────────────────▶│                    │
   │              │                   │ Build unsigned TX  │
   │              │                   │ (via Blockfrost    │
   │              │                   │  UTxO query)       │
   │              │   Unsigned TX     │                    │
   │              │◀──────────────────│                    │
   │  Sign TX     │                   │                    │
   │  prompt      │                   │                    │
   │◀─────────────│                   │                    │
   │              │                   │                    │
   │  Signed TX   │                   │                    │
   │─────────────▶│                   │                    │
   │              │  POST /confirm    │                    │
   │              │──────────────────▶│                    │
   │              │                   │  Submit TX         │
   │              │                   │───────────────────▶│
   │              │                   │                    │
   │              │                   │  TX hash           │
   │              │                   │◀───────────────────│
   │              │  Pledge confirmed │                    │
   │              │◀──────────────────│                    │
   │  Success!    │                   │                    │
   │◀─────────────│                   │                    │
```

---

## 4. Transaction Flows

### 4.1 Creating a Pledge

1. **Pledge intent**: User selects idea and amount. Backend records `pledges` row with status `pending`.
2. **TX construction**: Backend queries Blockfrost for user's UTxOs, builds a transaction that:
   - Takes the pledged amount from user's wallet
   - Sends it to the escrow script address with the `PledgeDatum` attached
   - Includes a change output back to the user
3. **Client-side signing**: Unsigned TX (CBOR) is sent to frontend, which calls `wallet.signTx()`.
4. **Submission**: Signed TX is sent back to backend, which submits it via Blockfrost.
5. **Confirmation**: Backend monitors the TX hash. Once confirmed (2+ block confirmations), updates pledge status to `confirmed`.

### 4.2 Claiming Funds (Idea Completed)

1. Idea author requests fund release through the platform.
2. Platform administrators review and co-sign (2-of-3 multisig).
3. Backend builds a claim transaction that:
   - Spends all pledge UTxOs at the escrow address for this idea
   - Sends total to the creator's wallet address
   - Uses the `Claim` redeemer
4. Transaction is submitted via Blockfrost.

### 4.3 Refunding (Deadline Passed / Idea Abandoned)

**Automatic refund** (deadline passed):
1. A scheduled job monitors pledge deadlines.
2. For expired pledges, backend builds refund transactions for each pledger.
3. Each pledger's funds are returned to their original address.

**Platform-initiated refund** (idea abandoned):
1. Admin marks idea as abandoned.
2. Backend uses `PlatformRefund` redeemer to return all pledged funds.

---

## 5. Rust Backend Implementation

### Blockfrost Client

```rust
// crates/ideaforge-blockchain/src/blockfrost.rs

pub struct BlockfrostClient {
    http: reqwest::Client,
    base_url: String,
    project_id: String,  // Blockfrost API key
}

impl BlockfrostClient {
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<UTxO>>;
    pub async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<TxHash>;
    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<TxStatus>;
    pub async fn get_script_utxos(&self, script_hash: &str) -> Result<Vec<UTxO>>;
}
```

### Pledge Service

```rust
// crates/ideaforge-blockchain/src/pledge.rs

pub struct PledgeService {
    blockfrost: BlockfrostClient,
    db: DatabaseConnection,
}

impl PledgeService {
    /// Build an unsigned pledge transaction
    pub async fn build_pledge_tx(
        &self,
        idea_id: Uuid,
        pledger_address: &str,
        amount_lovelace: u64,
    ) -> Result<UnsignedTransaction>;

    /// Submit a signed pledge transaction
    pub async fn submit_pledge(
        &self,
        pledge_id: Uuid,
        signed_tx_cbor: &[u8],
    ) -> Result<TxHash>;

    /// Monitor pending transactions for confirmation
    pub async fn check_confirmations(&self) -> Result<Vec<ConfirmedPledge>>;
}
```

---

## 6. Security Considerations

| Risk | Mitigation |
|---|---|
| Server compromise | No private keys on server. All signing is client-side. |
| Fraudulent claims | Platform multisig co-signature required for fund release. |
| Smart contract bugs | Aiken's type system + formal property testing + third-party audit before mainnet. |
| Front-running | Pledge amounts are public on-chain by design (transparency). |
| Stale UTxO data | Refresh UTxO set immediately before TX building. Retry on submission failure. |
| Double satisfaction | Validators check all relevant UTxOs; unique datum per pledge. |
| Unbounded datum | Enforce datum size limits in validator (max 256 bytes). |
| Time-based attacks | Use slot ranges conservatively; account for clock skew. |

---

## 7. Smart Contract Audit Pipeline

Per the Security Specialist's framework, all smart contracts undergo a mandatory audit pipeline before mainnet deployment.

### Pre-Deployment (Mandatory)

```
Development (Aiken)
    |
    v
Unit Tests (100% coverage on happy + unhappy paths)
    |
    v
Property-Based Tests (500+ generated test cases, QuickCheck-style)
    |
    v
Internal Code Review (minimum 2 senior developers, line-by-line)
    |
    v
Static Analysis (Aiken linter + Cardano-specific security checks)
    |
    v
Formal Verification (for escrow contracts handling > $10K equivalent)
    |
    v
External Audit (certified Cardano audit firm: MLabs, Tweag, or Vacuumlabs)
    |
    v
Testnet Deployment (minimum 30 days on Cardano Preview testnet)
    |
    v
Bug Bounty (public, minimum 14 days; reward: 5-10% of potential loss prevented)
    |
    v
Mainnet Deployment (staged rollout with value limits)
    |
    v
Ongoing: On-chain monitoring + quarterly re-audit
```

### Post-Deployment (Ongoing)

- On-chain monitoring for anomalous transactions
- Bug bounty program via Immunefi
- Quarterly re-audit of active contracts
- Upgrade path via Cardano reference scripts

---

## 8. Pledge as Pre-Order (Securities Law Compliance)

Pledges on IdeaForge are structured as **pre-orders / pledge-to-buy commitments**, NOT investment contracts. This distinction is critical for securities law compliance.

### Legal Framing

| Aspect | IdeaForge Pledge (Pre-Order) | Investment Security |
|---|---|---|
| **What you get** | A future product or service | Equity, dividends, or profit share |
| **Expectation** | Delivery of a specific deliverable | Financial return on investment |
| **Legal test** | Consumer purchase | Howey test (SEC) / prospectus requirement (EU) |
| **Regulatory burden** | Consumer protection law | Securities regulation (heavy) |

### Implementation Safeguards

1. **No equity language**: The platform never uses terms like "invest," "returns," "equity," or "shares" for pledges
2. **Product-centric**: Pledges are tied to specific deliverables described by the idea creator
3. **Refund guarantee**: Smart contract enforces automatic refund if deadline passes or target is not met
4. **No secondary market**: Pledges are non-transferable; no trading of pledge positions
5. **Clear disclosures**: Every pledge page shows: "This is a pre-order commitment, not an investment. You are pledging to receive [deliverable]. Full refund if the project does not deliver by [deadline]."
6. **Jurisdictional awareness**: Platform restricts pledge features in jurisdictions where even pre-orders may trigger securities requirements
7. **Legal review**: Platform terms of service reviewed by counsel in US (SEC), EU (MiFID II), and Switzerland (FINMA)

### Amount Limits

To stay well below securities thresholds:
- Individual pledge: max 500 ADA (~$150) per idea per user at launch
- Total campaign: max 50,000 ADA (~$15,000) per idea at launch
- Limits increase after legal review for each expansion jurisdiction

---

## 9. Testnet-First Strategy

1. **Phase 1 (MVP)**: Cardano Preview testnet only. Pledges use test ADA.
2. **Phase 2**: External security audit of Aiken contracts. Bug bounty program (Immunefi).
3. **Phase 3**: Mainnet deployment with low per-pledge caps (500 ADA per user, 50,000 ADA per campaign).
4. **Phase 4**: Increase caps after legal review and audit cycle.

Configuration:
```toml
[blockchain]
network = "preview"  # "preview" | "preprod" | "mainnet"
blockfrost_url = "https://cardano-preview.blockfrost.io/api/v0"
blockfrost_project_id = "preview..."
escrow_script_hash = "..."
platform_key_hash = "..."
max_pledge_per_user_lovelace = 500_000_000    # 500 ADA
max_campaign_total_lovelace = 50_000_000_000  # 50,000 ADA
```

---

## 10. Cross-References

| Topic | Document |
|---|---|
| System architecture overview | `docs/architecture/system_overview.md` |
| API endpoints for pledges | `docs/architecture/api_design.md` (Section 2.8) |
| Database schema for pledges | `docs/architecture/database_schema.md` (Section 2.8) |
| Smart contract audit pipeline | `docs/security/security_framework.md` |
| Pledge-to-buy fee structure | `docs/business/business_model.md` (Section 4) |
| IP protection (blockchain timestamping) | `docs/security/ip_protection.md` |
| Product roadmap (Phase 3: Fueling the Fire) | `docs/design/roadmap.md` |

---

*Blockchain integration designed February 2026. Revised during cross-review Rounds 1-2 with security, business, and product teams. Smart contracts: Aiken on Cardano (Plutus V3). Fiat on-ramp via Stripe (`ideaforge-payments` crate) for users who prefer not to use Cardano wallets. Pledge-to-buy fees: 3% campaign + 2% milestone (total ~5%, vs Kickstarter's 8-10%). Securities compliance: pledges structured as pre-orders, not investment contracts.*
