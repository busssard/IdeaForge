# IdeaForge Security Framework (MVP)

## Executive Summary

The MVP has a radically simplified security posture. There are no secret ideas, no financial transactions, no smart contracts, no crypto wallets, and no AI agent API. The attack surface is a standard web application with user accounts, public content, and social features (posting, commenting, voting).

**MVP security scope**: Basic JWT authentication, password hashing, HTTPS, input validation, rate limiting. That is it.

Everything else (MFA, HSM/KMS, per-idea encryption, smart contract security, KYC/AML, ABAC, mTLS) is deferred to post-PMF phases when the features that require them are built.

---

## 1. What the MVP Does NOT Have (Deferred Security)

These features and their associated security infrastructure are explicitly deferred:

| Deferred Feature | Associated Security | When to Add |
|-----------------|--------------------|----|
| Secret/IP-protected ideas | Per-idea AES-256-GCM encryption, KMS, HSM, ABAC, NDA-gated access | Phase 2 (if validated) |
| Financial transactions | PCI DSS, escrow, smart contract audits, KYC/AML | Phase 3 |
| Smart contracts (Cardano) | Formal verification, external audits, testnet deployment | Phase 3 |
| Multi-factor authentication | TOTP, MFA step-up for sensitive actions | Phase 2 |
| AI agent API | Bot registration, API key management, HMAC-SHA256 | Phase 3 |
| Crypto wallet auth | Cardano wallet challenge-response | Phase 3 |
| mTLS service-to-service | Certificate management, mutual TLS | Phase 2+ |
| WAF / DDoS protection | Cloud WAF, DDoS mitigation | When traffic justifies it |
| SOC 2 / GDPR compliance | DPO, DPIA, audit trails | Pre-Series A |
| Bug bounty program | HackerOne/Immunefi setup | Post-launch, when attack surface grows |

**Rationale**: Security should be proportional to what you are protecting. The MVP protects user accounts and public idea content. It does not handle money, secrets, or regulated data. Over-engineering security at MVP delays launch and burns runway on infrastructure no one uses yet.

---

## 2. MVP Authentication

### 2.1 Authentication Methods

**Supported at MVP**:
1. **Email + password** (primary)
2. **OAuth 2.0 social login** (Google, GitHub) via OIDC

**Deferred**:
- Cardano wallet authentication
- AI agent API key authentication
- MFA / TOTP

### 2.2 Password Handling

- Passwords hashed with **Argon2id** (memory-hard, resistant to GPU attacks)
- Minimum password length: 10 characters
- No maximum length restriction (up to reasonable limit, e.g., 128 chars)
- Checked against HaveIBeenPwned breached password API on registration
- Account lockout after 5 failed login attempts (15-minute progressive delay)

### 2.3 JWT Token Flow

```
User logs in (email/password or OAuth)
        |
        v
Server verifies credentials
        |
        v
JWT Access Token issued
  - Expiry: 15 minutes
  - Payload: user_id, role, issued_at
  - Signed with HS256 or RS256
        |
        v
Refresh Token issued
  - Expiry: 7 days
  - Stored server-side (database)
  - Rotated on each use (old token invalidated)
  - HTTP-only, Secure, SameSite=Strict cookie
```

### 2.4 Session Security

- Access tokens: short-lived (15 min), stored in memory (not localStorage)
- Refresh tokens: HTTP-only cookie, Secure flag, SameSite=Strict
- Session invalidated on password change
- Maximum 5 concurrent sessions per user
- No "remember me" with indefinite sessions

---

## 3. MVP Authorization

### 3.1 Role-Based Access Control (MVP)

Three roles only:

```
Roles: Curious < Maker/Entrepreneur < Admin
```

### 3.2 Permissions Matrix (MVP)

| Action | Curious | Entrepreneur | Maker | Admin |
|--------|---------|-------------|-------|-------|
| Browse ideas | Y | Y | Y | Y |
| Stoke (vote) ideas | Y | Y | Y | Y |
| Comment on ideas | Y | Y | Y | Y |
| Create ideas | N | Y | Y | Y |
| Edit own ideas | N | Y | N | Y |
| Offer to help build | N | N | Y | Y |
| Delete any content | N | N | N | Y |
| Manage users | N | N | N | Y |

### 3.3 Authorization Implementation

- Server-side role check on every API endpoint (no client-side-only enforcement)
- Deny-by-default: new endpoints require explicit authorization rules
- Idea ownership enforced: only the creator can edit/delete their idea
- Admin actions audit-logged (simple log entry, not full SIEM)

---

## 4. MVP API Security

### 4.1 Basic Protections

- All API endpoints require authentication (except: public idea listing, registration, login)
- HTTPS enforced everywhere (HSTS header)
- CORS restricted to platform domain only
- HTTP security headers: CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy

### 4.2 Rate Limiting

Simple rate limiting via middleware (Tower + in-memory or Redis counter):

| Endpoint | Limit | Window | Action on Exceed |
|----------|-------|--------|-----------------|
| Login | 5 attempts | 15 min | 429 + progressive delay |
| Registration | 3 accounts | 1 hour per IP | 429 |
| Idea creation | 5 ideas | 24 hours per user | 429 |
| Stoking (voting) | 100 votes | 1 hour per user | 429 |
| Comments | 30 comments | 1 hour per user | 429 |
| General API | 300 requests | 5 min per user | 429 |

### 4.3 Input Validation

- Parameterized queries / prepared statements exclusively (no string concatenation for SQL)
- Input validation on all user-facing fields (allowlist approach where possible)
- Output encoding for all user-generated content (prevent XSS)
- Server-side markdown rendering with sanitization (no raw HTML allowed)
- Maximum input lengths enforced (idea title: 200 chars, description: 10,000 chars, comment: 5,000 chars)

---

## 5. MVP Data Protection

### 5.1 What We Store

| Data | Sensitivity | Protection |
|------|------------|------------|
| Email addresses | Medium | Database-level access controls |
| Hashed passwords | High | Argon2id (irreversible) |
| Ideas (title, description) | Low (all public in MVP) | Standard database backups |
| Comments | Low | Standard database backups |
| Stoke votes | Low | Standard database backups |
| User profiles | Low-Medium | Database-level access controls |

### 5.2 What We Do NOT Store (MVP)

- Credit card numbers (no payments in MVP)
- Crypto wallet keys (no crypto in MVP)
- Secret idea content (no secret ideas in MVP)
- Government IDs or KYC documents (no financial features in MVP)
- Encrypted content (nothing requires encryption beyond TLS in transit and standard DB at rest)

### 5.3 Database Security

- Database access restricted to application service account only
- No direct database access from public internet
- Database credentials stored in environment variables (not in code)
- Regular automated backups
- Transparent Data Encryption (TDE) if cloud-hosted database supports it (standard cloud default)

---

## 6. MVP Infrastructure Security

### 6.1 Deployment

- Container-based deployment (Docker)
- Minimal container images (Alpine-based or distroless)
- No default credentials in any environment
- Secrets injected via environment variables at runtime
- HTTPS with TLS 1.2+ (TLS 1.3 preferred)

### 6.2 Dependency Management

- Dependency lock files committed to version control (Cargo.lock)
- Automated vulnerability scanning (cargo-audit, Dependabot)
- Pin dependency versions; no floating ranges
- Minimal dependency policy: evaluate necessity before adding packages

### 6.3 Logging (MVP)

- Structured logging (JSON format)
- Log: authentication successes/failures, authorization failures, rate limit hits
- No sensitive data in logs (no passwords, tokens, or PII)
- Log retention: 30 days (sufficient for MVP debugging)
- No full SIEM at MVP -- review logs manually when issues arise

---

## 7. Security Checklist Before Launch

- [ ] All API endpoints authenticated (except public reads and auth endpoints)
- [ ] Passwords hashed with Argon2id
- [ ] HTTPS enforced, HSTS header set
- [ ] CORS restricted to platform domain
- [ ] Rate limiting active on all endpoints
- [ ] Input validation on all user inputs
- [ ] Output encoding on all user-generated content
- [ ] SQL injection prevention (parameterized queries only)
- [ ] XSS prevention (CSP header, sanitized markdown rendering)
- [ ] No secrets in code or version control
- [ ] Container images scanned for vulnerabilities
- [ ] Dependency audit clean (cargo-audit)
- [ ] Session management working (JWT rotation, refresh token rotation)
- [ ] Account lockout after failed login attempts
- [ ] Error responses do not leak internal details

---

## 8. Security Roadmap (Post-MVP)

When features are added, security scales with them:

### Phase 2: Collaboration Features
- Add MFA (TOTP) -- optional for all users
- Add secret idea infrastructure if validated: per-idea encryption, KMS, access grants
- Add NDA-gated access controls
- Begin GDPR compliance work (DPO, privacy policy, data subject rights)

### Phase 3: Financial Infrastructure
- Smart contract security (Aiken audit, formal verification, testnet deployment)
- PCI DSS compliance via Stripe tokenization
- KYC/AML integration for investors
- Dedicated financial transaction logging
- External security audit
- Bug bounty program launch
- WAF and DDoS protection

### Phase 4: Scale
- SOC 2 Type II certification
- HSM-backed KMS for master keys
- mTLS for service-to-service communication
- Full SIEM integration (ELK/Loki + Grafana)
- AI agent API security (bot registration, capability verification, rate limiting)
- Penetration testing (annual)

---

## 9. Key Risks and Mitigations (MVP)

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| Account takeover via credential stuffing | Medium | Medium | Argon2id hashing, account lockout, breached password check, rate limiting |
| XSS in user-generated content | Medium | Medium | CSP header, output encoding, sanitized markdown rendering, no raw HTML |
| SQL injection | High | Low | Parameterized queries only, no string concatenation |
| Spam/bot account creation | Medium | High | Rate limiting on registration, email verification, CAPTCHA if needed |
| Data breach of user emails | Medium | Low | Database access restricted, standard cloud security, no unnecessary PII collection |
| Dependency vulnerability | Medium | Medium | cargo-audit in CI, Dependabot alerts, pinned versions |

**Note**: The MVP risk profile is dramatically lower than the full platform because there is no money, no secrets, and no regulated data. The worst-case breach exposes emails, public ideas, and hashed passwords -- serious but not catastrophic. This is the security posture of a standard content platform, which is exactly what the MVP is.
