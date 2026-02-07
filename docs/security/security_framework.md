# IdeaForge Security Framework

## Executive Summary

IdeaForge handles sensitive intellectual property, financial transactions (fiat and crypto), and personal data across multiple user roles. This document defines the security architecture, covering OWASP Top 10 mitigation, authentication/authorization, encryption, smart contract security, and infrastructure protection.

---

## 1. OWASP Top 10:2025 Mitigation

The OWASP Top 10:2025 is the latest industry standard for web application security risks. Each risk is addressed with IdeaForge-specific mitigations.

### A01:2025 -- Broken Access Control

**Risk**: Unauthorized access to secret ideas, other users' financial data, admin functions, or cross-role privilege escalation.

**Mitigations**:
- Role-based access control (RBAC) with the seven platform roles as base roles
- Idea-level access control lists (ACLs) enforced server-side
  - Open ideas: readable by all
  - Commercial ideas: readable by all, contribution terms enforced
  - Secret ideas: readable only by NDA-signed, approved users
- API authorization checks on every endpoint (no client-side-only enforcement)
- Deny-by-default policy: new endpoints require explicit authorization rules
- Automated access control testing in CI/CD pipeline
- Session-based and token-based access with short-lived JWTs (15-minute expiry, rotating refresh tokens)
- Admin actions require multi-factor authentication (MFA) and are audit-logged

### A02:2025 -- Security Misconfiguration

**Risk**: Default credentials, unnecessary services, overly permissive CORS, exposed debug endpoints.

**Mitigations**:
- Infrastructure-as-Code (IaC) with security baselines (Terraform/Pulumi)
- Automated security scanning in CI/CD (SAST, DAST, container scanning)
- No default credentials in any environment (secrets injected at runtime)
- Minimal container images (distroless/Alpine-based)
- HTTP security headers enforced: CSP, X-Frame-Options, X-Content-Type-Options, HSTS
- CORS restricted to platform domains only
- Debug/diagnostic endpoints disabled in production
- Regular configuration audits (quarterly)

### A03:2025 -- Software Supply Chain Failures

**Risk**: Compromised dependencies, malicious packages, tampered build artifacts.

**Mitigations**:
- Dependency lock files committed to version control (package-lock.json, poetry.lock)
- Automated vulnerability scanning (Dependabot, Snyk, or Grype)
- Software Bill of Materials (SBOM) generated for every release
- Signed container images and build artifacts
- Minimal dependency policy: evaluate necessity before adding any package
- Pin dependency versions; no floating version ranges in production
- Private artifact registry with vulnerability scanning

### A04:2025 -- Cryptographic Failures

**Risk**: Exposure of passwords, payment data, secret idea content, API keys.

**Mitigations**:
- Passwords hashed with Argon2id (memory-hard, not bcrypt)
- TLS 1.3 enforced for all connections (TLS 1.2 minimum)
- AES-256-GCM for data at rest encryption
- No custom cryptography -- use well-audited libraries only
- API keys and secrets stored in vault (HashiCorp Vault or AWS Secrets Manager)
- Automatic secret rotation (90-day maximum lifetime)
- No sensitive data in URLs, logs, or error messages
- PCI DSS compliance for fiat payment handling (via Stripe tokenization)

### A05:2025 -- Injection

**Risk**: SQL injection, NoSQL injection, OS command injection, LDAP injection.

**Mitigations**:
- Parameterized queries / prepared statements exclusively (no string concatenation)
- ORM usage with query builder (SQLAlchemy, Prisma, or similar)
- Input validation on all user-facing fields (allowlist approach)
- Content Security Policy (CSP) to prevent XSS
- Output encoding for all user-generated content
- Server-side markdown rendering with sanitization (no raw HTML)
- Regular penetration testing focusing on injection vectors

### A06:2025 -- Insecure Design

**Risk**: Architectural flaws that cannot be fixed by implementation alone.

**Mitigations**:
- Threat modeling for each major feature before development (STRIDE)
- Abuse case documentation alongside use cases
- Security architecture review for:
  - Secret idea access flow
  - Smart contract funding flow
  - AI agent registration and interaction flow
  - Cross-role privilege transitions
- Rate limiting as an architectural requirement (not an afterthought)
- Defense in depth: multiple security layers, no single point of failure

### A07:2025 -- Authentication Failures

**Risk**: Credential stuffing, brute force, session hijacking, weak passwords.

**Mitigations**:
- Multi-factor authentication (MFA) required for:
  - Investors (mandatory for financial transactions)
  - Entrepreneurs with secret ideas (mandatory)
  - All users (optional but encouraged)
- Password requirements: minimum 12 characters, checked against breached password databases (HaveIBeenPwned API)
- Account lockout after 5 failed attempts (progressive delays, not permanent lockout)
- Session management:
  - HTTP-only, Secure, SameSite=Strict cookies
  - Session invalidation on password change
  - Concurrent session limits (max 5 active sessions)
- OAuth 2.0 / OIDC for social login (Google, GitHub, Cardano wallet)
- API authentication via short-lived Bearer tokens with scoped permissions

### A08:2025 -- Software or Data Integrity Failures

**Risk**: Deserialization attacks, CI/CD pipeline compromise, unsigned updates.

**Mitigations**:
- Signed and verified deployments (GPG-signed commits, verified container images)
- CI/CD pipeline hardened with least-privilege service accounts
- Immutable infrastructure (containers rebuilt, not patched in place)
- Integrity checks on all downloaded dependencies and artifacts
- Webhook verification for all external integrations (signature validation)

### A09:2025 -- Security Logging & Alerting Failures

**Risk**: Attacks go undetected due to insufficient logging or monitoring.

**Mitigations**:
- Structured logging (JSON format) with correlation IDs
- Security-relevant events logged:
  - Authentication successes and failures
  - Authorization failures
  - Secret idea access attempts
  - Financial transactions
  - Admin actions
  - API rate limit hits
  - Bot activity patterns
- Log aggregation with alerting (ELK/Loki + Grafana or equivalent)
- Real-time alerting for:
  - Brute force detection (> 10 failed logins from same IP/account)
  - Unusual secret idea access patterns
  - Smart contract anomalies
  - Bot registration spikes
- Log retention: 90 days hot, 1 year cold, 7 years for financial records
- Logs encrypted at rest; access to logs requires elevated permissions

### A10:2025 -- Mishandling of Exceptional Conditions

**Risk**: Unhandled errors revealing system internals, failing open instead of closed.

**Mitigations**:
- Global error handler that returns generic messages to users
- No stack traces, internal paths, or database errors in production responses
- Fail-closed design: if authorization check fails or is unavailable, deny access
- Circuit breaker pattern for external service calls (Cardano node, payment processor)
- Graceful degradation: if smart contract service is unavailable, queue transactions, do not skip verification
- Dead letter queues for failed financial transactions with alerting

---

## 2. Authentication & Authorization Architecture

### 2.1 Authentication Flow

```
User
  |
  v
[Login Page / API]
  |
  +-- Email/Password --> Argon2id verification --> MFA challenge (if enabled)
  |
  +-- OAuth 2.0 (Google, GitHub) --> OIDC token verification
  |
  +-- Cardano Wallet --> Challenge-response signature verification
  |
  +-- AI Agent API Key --> HMAC-SHA256 signature verification
  |
  v
[JWT Issued]
  |
  +-- Access Token (15 min, contains: user_id, role, permissions)
  +-- Refresh Token (7 days, stored server-side, rotated on use)
```

### 2.2 Authorization Model

#### Role-Based Access Control (RBAC)

```
Roles:        Curious < Consumer < Maker/Freelancer < Entrepreneur < Investor < Admin
                                                    AI-Agent (parallel track)
```

#### Permissions Matrix

| Action | Curious | Consumer | Maker | Freelancer | Entrepreneur | Investor | AI-Agent | Admin |
|--------|---------|----------|-------|------------|--------------|----------|----------|-------|
| Browse open ideas | Y | Y | Y | Y | Y | Y | Y | Y |
| Vote on ideas | Y | Y | Y | Y | Y | Y | N* | Y |
| Comment | Y | Y | Y | Y | Y | Y | Y** | Y |
| Pledge/pre-buy | N | Y | Y | Y | Y | Y | N | Y |
| Create ideas | N | N | Y | N | Y | N | Y** | Y |
| Apply to tasks | N | N | Y | Y | N | N | Y** | Y |
| Fund ideas | N | N | N | N | N | Y | N | Y |
| Access secret ideas | N | N | NDA*** | NDA*** | Own | NDA*** | N | Y |
| Register as expert | N | N | Y | Y | N | N | N | Y |
| Admin functions | N | N | N | N | N | N | N | Y |

\* AI-Agents cannot vote (prevents manipulation); they can endorse via separate mechanism
\** AI-Agent contributions always labeled as AI-generated
\*** NDA required; access granted per-idea by entrepreneur

#### Attribute-Based Access Control (ABAC) for Secret Ideas

Secret ideas use fine-grained ABAC:
- `idea.openness_level == "secret"` AND `user.has_signed_nda(idea.id)` AND `idea.owner.approved(user.id)`
- Access logged and auditable
- Time-limited access windows (renewable)
- Watermarked content delivery for leak detection

### 2.3 API Security

**Architecture alignment note**: The architect has designed the API as RESTful JSON via Axum (not GraphQL), with `/api/v1/` prefix. Bot authentication uses `X-Api-Key` header resolved to a user context with `is_bot: true`. Rate limiting is implemented via Tower middleware with Redis-backed sliding window counters.

- All API endpoints require authentication (except public idea listings)
- Rate limiting per user, per IP, and per API key (implemented via `tower` middleware + Redis)
- Bot API keys: long-lived, stored hashed in database (`bot_api_key_hash` field)
- API versioning via URL prefix (`/api/v1/`)
- Pagination enforced on all list endpoints (cursor-based for real-time feeds)
- WebSocket connections authenticated via JWT token in query parameter (`/ws?token={jwt}`)

### 2.4 Architecture Security Alignment & Gaps

Cross-referencing with the software architecture (`docs/architecture/`), the following alignment and gap analysis applies:

**Aligned**:
- JWT with 15-min access tokens + 7-day refresh tokens (architecture matches security spec)
- Argon2 for password hashing (confirmed in ADR-008)
- Bot/human distinction via `is_bot` boolean and `bot_owner_id` foreign key in users table
- Separate `human_approvals` and `bot_approvals` counters denormalized on ideas table
- Rate limits differentiated for human vs. bot traffic

**Gaps resolved in Cross-Review Round 2** (architect has updated `docs/architecture/`):

1. **Secret idea isolation**: **Resolved.** The architect has added a `secret_ideas` schema (`docs/architecture/database_schema.md` Section 5b) with `secret_ideas.idea_content` (encrypted BYTEA), `secret_ideas.access_grants` (NDA tracking with expiry), and `secret_ideas.access_log` (audit trail). Access routed through a dedicated Secret Idea Access Proxy with mTLS. This matches our security recommendation precisely.
2. **Per-idea encryption**: **Resolved.** The `secret_ideas.idea_content` table stores `encrypted_content` (BYTEA, AES-256-GCM) with `encryption_key_id` referencing per-idea DEKs in KMS. The architect's system overview diagram now shows the isolated Secret Ideas Zone with HSM-backed KMS keys.
3. **KMS integration**: **Resolved.** The architecture now references HSM-backed KMS (AWS KMS or HashiCorp Vault) for master keys wrapping per-idea DEKs. The `CLAUDE.md` project file confirms: "Secret ideas use per-idea AES-256-GCM encryption with HSM-backed KMS."
4. **MFA**: **Partially resolved.** The `CLAUDE.md` references `ideaforge-auth` crate with "JWT, OAuth2, Argon2 password hashing, MFA (TOTP)". The auth crate exists in the workspace structure but MFA flow details are not yet in the API design doc. **Remaining gap**: The API design (`docs/architecture/api_design.md`) should specify MFA endpoints (`/api/v1/auth/mfa/setup`, `/api/v1/auth/mfa/verify`) and which actions require MFA step-up (secret idea access, financial transactions, admin actions).
5. **Audit logging**: **Partially resolved.** The architect added `secret_ideas.access_log` for secret idea access tracking. The maturity state machine logs transitions in `idea_events`. **Remaining gap**: A unified security audit log covering auth failures, admin actions, API rate limit hits, and bot activity anomalies should be specified. Recommend a dedicated `security_audit_log` table or integration with an external SIEM (ELK/Loki) for centralized security event correlation.

---

## 3. Data Encryption

### 3.1 Encryption at Rest

| Data Type | Encryption | Key Management |
|-----------|-----------|----------------|
| User credentials | Argon2id (hashing, not encryption) | N/A (one-way) |
| Secret idea content | AES-256-GCM | Per-idea encryption key, wrapped by master key |
| Financial records | AES-256-GCM | Dedicated key, rotated quarterly |
| Personal data (PII) | AES-256-GCM | Per-user encryption key |
| Database (full) | Transparent Data Encryption (TDE) | Cloud KMS managed |
| Backups | AES-256-GCM | Offline master key with split custody |
| Logs | AES-256-GCM | Dedicated logging key |

### 3.2 Encryption in Transit

- TLS 1.3 for all client-server communication (TLS 1.2 minimum fallback)
- Certificate pinning for mobile applications (if applicable)
- mTLS for service-to-service communication in backend
- Cardano node communication via authenticated channels
- WebSocket connections encrypted via WSS

### 3.3 Key Management

- Cloud KMS (AWS KMS, GCP Cloud KMS, or HashiCorp Vault) for master keys
- Automatic key rotation: 90 days for data keys, 365 days for master keys
- Key hierarchy: Master Key -> Data Encryption Keys (DEKs) -> wrapped per resource
- Hardware Security Module (HSM) backing for master keys (FIPS 140-2 Level 3)
- Break-glass procedure documented for emergency key access
- Key access audit logging with alerting on anomalous access patterns

---

## 4. Smart Contract Security

### 4.1 Cardano-Specific Considerations

IdeaForge uses Cardano smart contracts for:
- Pledge escrow (milestone-based release)
- Investment funding rounds
- Freelancer task payment escrow
- Blockchain timestamping for IP protection

#### Language and Framework
- Smart contracts written in **Aiken** (Rust-inspired language that compiles to Plutus Core/UPLC)
- Aiken is the confirmed choice per architecture team's ADR-001, aligning with the Rust-first stack
- Aiken advantages over raw Plutus/Haskell: better developer experience, Rust-inspired syntax, lower execution costs, stronger type system
- The architect has already designed the `PledgeEscrow` validator in Aiken with parameterized datum, three redeemer actions (Claim, Refund, PlatformRefund), and platform multisig co-signature
- Follow Cardano CIP-52 audit best practice guidelines

### 4.2 Audit Requirements

#### Pre-Deployment (Mandatory)
1. **Internal code review**: Minimum 2 senior developers, line-by-line
2. **Automated analysis**: Static analysis tools (e.g., Cardano-specific linters)
3. **Property-based testing**: QuickCheck or similar for edge case discovery
4. **Formal verification**: For high-value contracts (escrow > $10K threshold), use formal methods to prove correctness
5. **External audit**: Engage a reputable Cardano audit firm (MLabs, Tweag, Vacuumlabs, or equivalent)
6. **Testnet deployment**: Minimum 30 days on Cardano testnet with bug bounty

#### Post-Deployment (Ongoing)
- On-chain monitoring for anomalous transactions
- Bug bounty program (reward: 5-10% of potential loss prevented)
- Quarterly re-audit of active contracts
- Upgrade path defined (Cardano reference scripts or proxy patterns)

### 4.3 Common Vulnerability Mitigations

| Vulnerability | Description | Mitigation |
|--------------|-------------|------------|
| **Double satisfaction** | Validator satisfied by unintended transaction | Ensure validators check all relevant UTxOs |
| **Unbounded datum** | Excessively large datum causing DoS | Enforce datum size limits in validator |
| **Missing redeemer validation** | Validator ignores redeemer content | Always validate redeemer structure and content |
| **Token name collision** | Minted tokens with misleading names | Strict minting policy with unique identifiers |
| **Time-based attacks** | Manipulating validity intervals | Use slot ranges conservatively; account for clock skew |
| **Insufficient staking control** | Reward withdrawal attacks | Validate staking credentials in validators |
| **Front-running** | Bots submitting competing transactions | Use commit-reveal schemes for sensitive operations |
| **Re-entrancy (Plutus V1)** | Not applicable to Cardano's EUTXO model | N/A -- EUTXO inherently prevents re-entrancy |

### 4.4 Smart Contract Deployment Process

```
Development
    |
    v
Unit Tests (100% coverage on happy + unhappy paths)
    |
    v
Property-Based Tests (500+ generated test cases)
    |
    v
Internal Audit (2+ reviewers)
    |
    v
External Audit (certified firm)
    |
    v
Testnet Deployment (30 days)
    |
    v
Bug Bounty (public, 14 days minimum)
    |
    v
Mainnet Deployment (staged rollout, value limits)
    |
    v
Monitoring + Ongoing Audits
```

---

## 5. Rate Limiting and DDoS Protection

### 5.1 Rate Limiting Strategy

#### Application-Level Rate Limits

| Endpoint Category | Limit | Window | Action on Exceed |
|------------------|-------|--------|-----------------|
| Authentication (login) | 5 attempts | 15 min | Progressive delay + CAPTCHA |
| Idea creation | 3 ideas | 24 hours | Queue for moderation |
| Voting | 100 votes | 1 hour | Soft block + review |
| Comments | 30 comments | 1 hour | Soft block |
| API (authenticated) | 1,000 requests | 1 hour | 429 response + backoff |
| API (AI agent) | 5,000 requests | 1 hour | 429 response + usage billing |
| Financial transactions | 10 transactions | 1 hour | Human verification |
| Secret idea access | 20 requests | 1 hour | Alert + review |

#### Implementation (aligned with architecture)
- Implemented via `tower` middleware with Redis-backed sliding window counters (per architect's API design)
- Per-user, per-IP, and per-API-key tracking
- Rate limit headers returned (X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset)
- Graceful degradation: return cached responses when possible
- Architect's rate limits are per-minute; the table above uses per-hour for strategic planning. Operational limits should follow the architect's per-minute granularity.

### 5.2 DDoS Protection

#### Layer 3/4 (Network)
- Cloud provider DDoS protection (AWS Shield, Cloudflare, GCP Cloud Armor)
- Anycast network distribution
- Traffic scrubbing for volumetric attacks
- BGP-based black hole routing as last resort

#### Layer 7 (Application)
- Web Application Firewall (WAF) with OWASP Core Rule Set
- Bot detection and CAPTCHA challenge for suspicious traffic
- Geographic rate limiting (higher limits for primary markets)
- Adaptive rate limiting that adjusts based on load
- Circuit breakers to protect backend services

#### Infrastructure
- Auto-scaling groups for web and API servers
- Database connection pooling with limits
- Queue-based processing for non-real-time operations (idea creation, voting aggregation)
- CDN for static assets and cached content
- Separate infrastructure for financial operations (isolated from public-facing services)

### 5.3 Incident Response

#### DDoS Response Runbook
1. **Detection** (< 1 min): Automated alerting on traffic anomalies
2. **Triage** (< 5 min): Classify attack type and vector
3. **Mitigation** (< 15 min): Engage DDoS protection, adjust WAF rules
4. **Communication** (< 30 min): Status page update, team notification
5. **Resolution**: Monitor for attack cessation, gradual rule relaxation
6. **Post-mortem**: Document attack, update mitigation rules, improve detection

---

## 6. Additional Security Measures

### 6.1 Vulnerability Management
- Continuous vulnerability scanning (weekly automated, monthly manual)
- Responsible disclosure program with security.txt
- Bug bounty program (platform: HackerOne or Immunefi for smart contracts)
- Patch management SLA: Critical (24h), High (72h), Medium (7 days), Low (30 days)

### 6.2 Security Development Lifecycle
- Security training for all developers (OWASP, secure coding)
- Pre-commit hooks for secret detection (git-secrets, truffleHog)
- SAST in CI/CD pipeline (Semgrep, CodeQL)
- DAST against staging environment (OWASP ZAP)
- Container scanning (Trivy, Grype)
- Infrastructure scanning (Checkov, tfsec)

### 6.3 Third-Party Security
- Vendor security assessment before integration
- Minimal data sharing with third parties
- Contractual security requirements (DPA, security SLAs)
- Regular review of third-party access and permissions

### 6.4 Compliance Requirements
- GDPR (EU users): DPO appointment, DPIA for high-risk processing, data subject rights
- SOC 2 Type II (target: Year 2): Trust services criteria for security, availability, confidentiality
- PCI DSS (via Stripe): Tokenized payment, no card data stored on platform
- KYC/AML: For investor and high-value transactions; integration with identity verification provider
- WCAG 2.2 Level AA: Accessibility compliance aligned with UX philosophy (see docs/design/ux_philosophy.md)

---

## 7. Key Risks & Mitigations

| Risk | Severity | Likelihood | Mitigation | Owner |
|------|----------|-----------|------------|-------|
| Smart contract vulnerability in pledge escrow | Critical | Medium | External audit (CIP-52 compliant), Aiken type safety, property-based testing, testnet-first deployment, bug bounty | Architecture + Security |
| Secret idea data breach via database access | Critical | Low | **Resolved in architecture**: Separate `secret_ideas` schema with per-idea AES-256-GCM encryption, HSM-backed KMS, mTLS access proxy, and audit logging (`docs/architecture/database_schema.md` Section 5b) | Security + Architecture |
| Bot army manipulation of Stoke signals | High | Medium | Separate human Stokes / AI endorsements (completely separate DB tables per `docs/architecture/database_schema.md` Sections 2.5/2.5b), AI endorsements excluded from maturity advancement, Sybil resistance measures, anomaly detection | Security + Product |
| Pledge-to-buy classified as security offering | High | Medium | Structure as pre-orders not investments, engage securities counsel early, Regulation CF exemption for US users, clear ToS language | Business + Legal |
| Undisclosed AI agent posing as human | High | Medium | Behavioral analysis, CAPTCHA challenges, device fingerprinting, community reporting, permanent ban on discovery | Security |
| Supply chain attack on Rust dependencies | Medium | Low | Dependency lock files, cargo-deny for license/vulnerability checks, SBOM generation, signed container images | Architecture |
| Cardano network congestion delaying pledges | Medium | Medium | Queue-based transaction processing, retry with backoff, clear user communication on confirmation times | Architecture |
| GDPR right-to-erasure conflicting with blockchain timestamps | Medium | Low | Only store hashes on-chain (not PII), platform retains right to delete off-chain data while hash remains as proof | Legal + Security |
| Blockfrost API dependency (single point of failure) | Medium | Low | Circuit breaker pattern, fallback to cached data, plan for self-hosted Cardano node at scale | Architecture |
| Key management failure (KMS unavailable) | High | Low | Break-glass procedure, split custody for master keys, HSM backing, regular DR testing | Security + Ops |
