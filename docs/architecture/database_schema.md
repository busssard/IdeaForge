# IdeaForge - Database Schema Design

## Database: PostgreSQL 16+

PostgreSQL is chosen for its JSONB support (flexible idea metadata), row-level security, full-text search fallback, and mature Rust driver ecosystem (SeaORM / SQLx).

## Table of Contents

1. [1. Entity-Relationship Diagram](#1.%20Entity-Relationship%20Diagram)
2. [2. Core Tables](#2.%20Core%20Tables)
3. [3. Role & Permission Model](#3.%20Role%20&%20Permission%20Model)
4. [4. Bot/Human Distinction (Separate Approval Tracks)](#4.%20Bot/Human%20Distinction%20(Separate%20Approval%20Tracks))
5. [5. Idea Maturity State Machine](#5.%20Idea%20Maturity%20State%20Machine)
6. [5b. Secret Idea Storage Architecture](#5b.%20Secret%20Idea%20Storage%20Architecture)
7. [6. Indexing Strategy](#6.%20Indexing%20Strategy)
8. [7. Migration Strategy](#7.%20Migration%20Strategy)
9. [8. Cross-References](#8.%20Cross-References)

---

## 1. Entity-Relationship Diagram

```
┌─────────────┐       ┌──────────────┐       ┌──────────────────┐
│   users      │──1:N──│  user_roles   │──N:1──│   roles          │
│              │       │              │       │                  │
│ id (PK)      │       │ user_id (FK) │       │ id (PK)          │
│ email        │       │ role_id (FK) │       │ name             │
│ display_name │       │ verified_at  │       │ description      │
│ is_bot       │       └──────────────┘       │ permissions JSONB│
│ bot_owner_id │                              └──────────────────┘
│ avatar_url   │
│ bio          │       ┌──────────────┐
│ wallet_addr  │──1:N──│  ideas        │
│ onboarding   │       │              │
│ created_at   │       │ id (PK)      │
│ updated_at   │       │ author_id FK │
└──────┬───────┘       │ title        │
       │               │ description  │
       │               │ category_id  │
       │               │ maturity     │──── ENUM (state machine)
       │               │ openness     │──── ENUM
       │               │ metadata     │──── JSONB
       │               │ created_at   │
       │               │ updated_at   │
       │               └──────┬───────┘
       │                      │
       │         ┌────────────┼────────────────┐
       │         │            │                │
       │  ┌──────▼──────┐ ┌──▼───────────┐ ┌──▼──────────┐
       │  │ approvals    │ │contributions │ │ pledges      │
       │  │ (human only) │ │              │ │              │
       │  │ id (PK)     │ │ id (PK)      │ │ id (PK)      │
       │  │ idea_id FK  │ │ idea_id FK   │ │ idea_id FK   │
       │  │ user_id FK  │ │ user_id FK   │ │ user_id FK   │
       │  │ created_at  │ │ type ENUM    │ │ amount_ada   │
       │  └─────────────┘ │ body TEXT    │ │ tx_hash      │
       │                  │ created_at   │ │ status ENUM  │
       │  ┌─────────────┐ └──────────────┘ │ created_at   │
       │  │ai_endorse-  │                  └──────────────┘
       │  │ments (AI    │
       │  │ agents only)│
       │  │ id (PK)     │
       │  │ idea_id FK  │
       │  │ agent_id FK │
       │  │ operator_id │
       │  │ confidence  │
       │  │ created_at  │
       │  └─────────────┘
       │
       │  ┌──────────────┐     ┌──────────────┐
       ├──│  todos        │     │  categories   │
       │  │              │     │              │
       │  │ id (PK)      │     │ id (PK)      │
       │  │ idea_id FK   │     │ name         │
       │  │ author_id FK │     │ slug         │
       │  │ assignee_id  │     │ parent_id FK │──── self-ref (tree)
       │  │ title        │     │ description  │
       │  │ status ENUM  │     │ icon         │
       │  │ priority     │     └──────────────┘
       │  │ created_at   │
       │  └──────────────┘     ┌──────────────────┐
       │                       │  idea_categories   │
       │  ┌──────────────┐     │  (join table)      │
       └──│ expert_apps   │     │ idea_id FK         │
          │              │     │ category_id FK     │
          │ id (PK)      │     └──────────────────┘
          │ user_id FK   │
          │ idea_id FK   │     ┌──────────────────┐
          │ role ENUM    │     │ notifications      │
          │ status ENUM  │     │                    │
          │ message TEXT │     │ id (PK)            │
          │ created_at   │     │ user_id FK         │
          └──────────────┘     │ type ENUM          │
                               │ payload JSONB      │
                               │ read_at            │
                               │ created_at         │
                               └──────────────────┘
```

---

## 2. Core Tables

### 2.1 users

```sql
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT UNIQUE NOT NULL,
    password_hash   TEXT,                          -- NULL for OAuth-only users
    display_name    TEXT NOT NULL,
    bio             TEXT DEFAULT '',
    avatar_url      TEXT,
    wallet_address  TEXT,                          -- Cardano wallet address
    is_bot          BOOLEAN NOT NULL DEFAULT FALSE,
    bot_owner_id    UUID REFERENCES users(id),     -- NULL for humans
    bot_api_key_hash TEXT,                         -- hashed API key for bot auth
    onboarding_role TEXT NOT NULL DEFAULT 'curious', -- initial selected role
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_bot ON users(is_bot);
CREATE INDEX idx_users_bot_owner ON users(bot_owner_id) WHERE bot_owner_id IS NOT NULL;
```

### 2.2 roles & user_roles

```sql
-- Pre-seeded roles
CREATE TABLE roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT UNIQUE NOT NULL,   -- 'entrepreneur', 'investor', 'maker', 'freelancer', 'ai_agent', 'consumer', 'curious', 'admin'
    description TEXT NOT NULL DEFAULT '',
    permissions JSONB NOT NULL DEFAULT '[]',  -- ["ideas.create", "ideas.approve", "pledges.create", ...]
    is_default  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     UUID NOT NULL REFERENCES roles(id),
    verified_at TIMESTAMPTZ,            -- expert roles require verification
    granted_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(user_id, role_id)
);
```

### 2.3 ideas

```sql
CREATE TYPE idea_maturity AS ENUM (
    'unanswered_question',
    'half_baked',
    'thought_through',
    'serious_proposal',
    'in_work',
    'almost_finished',
    'completed'
);

CREATE TYPE idea_openness AS ENUM (
    'open_source',
    'open_collaboration',  -- open but with contributor agreements (community co-creation)
    'commercial',
    'secret'               -- IP-protected, restricted access
);

CREATE TABLE ideas (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id       UUID NOT NULL REFERENCES users(id),
    title           TEXT NOT NULL,
    summary         TEXT NOT NULL,           -- short elevator pitch
    description     TEXT NOT NULL,           -- full markdown description
    maturity        idea_maturity NOT NULL DEFAULT 'unanswered_question',
    openness        idea_openness NOT NULL DEFAULT 'open_source',
    metadata        JSONB NOT NULL DEFAULT '{}',  -- extensible fields
    is_archived     BOOLEAN NOT NULL DEFAULT FALSE,
    human_approvals  INT NOT NULL DEFAULT 0,  -- denormalized counter (only these drive maturity)
    ai_endorsements  INT NOT NULL DEFAULT 0,  -- denormalized counter (informational only)
    total_pledged   BIGINT NOT NULL DEFAULT 0, -- lovelace (1 ADA = 1,000,000 lovelace)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ideas_author ON ideas(author_id);
CREATE INDEX idx_ideas_maturity ON ideas(maturity);
CREATE INDEX idx_ideas_openness ON ideas(openness);
CREATE INDEX idx_ideas_created ON ideas(created_at DESC);
```

### 2.4 categories & idea_categories

```sql
CREATE TABLE categories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    description TEXT DEFAULT '',
    icon        TEXT,                    -- emoji or icon class
    parent_id   UUID REFERENCES categories(id),  -- hierarchical categories
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE idea_categories (
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id),
    PRIMARY KEY (idea_id, category_id)
);
```

### 2.5 approvals (human-only)

Human approvals are the **only** signal that drives idea maturity advancement. These are entirely separate from AI endorsements.

```sql
CREATE TABLE approvals (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id),
    comment     TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(idea_id, user_id),       -- one approval per human per idea
    CONSTRAINT fk_human_only CHECK (TRUE)  -- enforced at application layer: user.is_bot = FALSE
);

CREATE INDEX idx_approvals_idea ON approvals(idea_id);
```

### 2.5b ai_endorsements (AI agent only, informational)

AI endorsements are displayed separately and **never** count toward maturity advancement or platform decision-making. This aligns with the Bot Transparency Framework and EU AI Act Article 50 requirements.

```sql
CREATE TABLE ai_endorsements (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id         UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES users(id),  -- must be is_bot = TRUE
    operator_id     UUID NOT NULL REFERENCES users(id),  -- human operator responsible
    confidence      REAL,                                 -- optional 0.0-1.0 confidence score
    reasoning       TEXT,                                 -- optional explanation of endorsement
    model_version   TEXT,                                 -- model that generated the endorsement
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(idea_id, agent_id)  -- one endorsement per agent per idea
);

CREATE INDEX idx_endorsements_idea ON ai_endorsements(idea_id);
CREATE INDEX idx_endorsements_agent ON ai_endorsements(agent_id);
```

### 2.6 contributions (comments, suggestions, general contributions)

```sql
CREATE TYPE contribution_type AS ENUM (
    'comment',
    'suggestion',
    'design',
    'code',
    'research',
    'other'
);

CREATE TABLE contributions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id),
    parent_id   UUID REFERENCES contributions(id),  -- threaded replies
    type        contribution_type NOT NULL DEFAULT 'comment',
    title       TEXT,
    body        TEXT NOT NULL,
    attachments JSONB DEFAULT '[]',        -- [{url, filename, mime_type}]
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_contributions_idea ON contributions(idea_id);
CREATE INDEX idx_contributions_user ON contributions(user_id);
CREATE INDEX idx_contributions_parent ON contributions(parent_id) WHERE parent_id IS NOT NULL;
```

### 2.7 todos

```sql
CREATE TYPE todo_status AS ENUM (
    'suggested',
    'accepted',
    'in_progress',
    'done',
    'rejected'
);

CREATE TABLE todos (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    author_id   UUID NOT NULL REFERENCES users(id),
    assignee_id UUID REFERENCES users(id),
    title       TEXT NOT NULL,
    description TEXT DEFAULT '',
    status      todo_status NOT NULL DEFAULT 'suggested',
    priority    INT NOT NULL DEFAULT 0,    -- 0=normal, 1=high, 2=urgent
    due_date    DATE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_todos_idea ON todos(idea_id);
CREATE INDEX idx_todos_assignee ON todos(assignee_id) WHERE assignee_id IS NOT NULL;
```

### 2.8 pledges

```sql
CREATE TYPE pledge_status AS ENUM (
    'pending',       -- pledge intent recorded
    'confirmed',     -- on-chain TX confirmed
    'fulfilled',     -- product delivered
    'refunded',      -- pledge refunded
    'expired'        -- pledge window closed
);

CREATE TABLE pledges (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id         UUID NOT NULL REFERENCES ideas(id),
    user_id         UUID NOT NULL REFERENCES users(id),
    amount_lovelace BIGINT NOT NULL,           -- in lovelace (1 ADA = 1M lovelace)
    tx_hash         TEXT,                       -- Cardano TX hash
    script_address  TEXT,                       -- smart contract address holding funds
    status          pledge_status NOT NULL DEFAULT 'pending',
    pledge_message  TEXT,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pledges_idea ON pledges(idea_id);
CREATE INDEX idx_pledges_user ON pledges(user_id);
CREATE INDEX idx_pledges_status ON pledges(status);
```

### 2.9 expert_applications

```sql
CREATE TYPE expert_role AS ENUM (
    'maker',
    'programmer',
    'designer',
    'scientist'
);

CREATE TYPE application_status AS ENUM (
    'pending',
    'accepted',
    'rejected'
);

CREATE TABLE expert_applications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id),
    role        expert_role NOT NULL,
    status      application_status NOT NULL DEFAULT 'pending',
    message     TEXT,
    reviewed_by UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(idea_id, user_id, role)
);
```

### 2.10 notifications

```sql
CREATE TABLE notifications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type        TEXT NOT NULL,           -- 'idea.approved', 'pledge.confirmed', 'todo.assigned', etc.
    payload     JSONB NOT NULL,          -- {idea_id, actor_id, message, ...}
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notifications_user_unread ON notifications(user_id, created_at DESC) WHERE read_at IS NULL;
```

---

## 3. Role & Permission Model

### Permission Strings

Permissions follow the pattern `{domain}.{action}`:

```
ideas.create        ideas.update_own     ideas.update_any     ideas.delete_own
ideas.approve       ideas.set_maturity   ideas.view_secret
pledges.create      pledges.refund
contributions.create  contributions.moderate
todos.create        todos.assign
users.manage        users.verify_expert
ai_agents.register  ai_agents.manage
admin.*
```

### Default Role Permissions

| Role | Key Permissions |
|---|---|
| **Curious** | ideas.create, contributions.create (comment only), ideas.approve |
| **Consumer** | + pledges.create |
| **Entrepreneur** | + ideas.update_own, ideas.set_maturity, todos.create, todos.assign |
| **Maker / Freelancer** | + expert application, todo self-assign |
| **Investor** | + pledges.create (higher limits), ideas.view_secret (if invited) |
| **AI Agent** | Same as role they apply for, but flagged as bot in all interactions |
| **Admin** | admin.* |

### Progressive Disclosure

The `onboarding_role` field on `users` controls which UI features are shown initially. Users can always access full features by changing their role in settings. This is a UX concern, not a permission gate.

---

## 4. Bot/Human Distinction (Separate Approval Tracks)

Human and AI participation are **completely separate data models**, aligned with the Bot Transparency Framework and EU AI Act Article 50 (effective August 2026).

### 4.1 Account-Level Distinction

Every user record has `is_bot` (boolean). Bot accounts:
- Must be registered by a human operator (`bot_owner_id` is required)
- Authenticate via API keys (hashed in `bot_api_key_hash`), not passwords
- Have a verification level tracked in `agent_verification_level`: `unverified`, `verified`, `certified`, `partner`
- Are always visually distinguished in the UI (hexagon avatar, "AI Agent" badge)

### 4.2 Separate Approval Tracks

- **`approvals` table**: Human-only. Drives maturity advancement and platform decisions.
- **`ai_endorsements` table**: AI agents only. Displayed as informational signal, never as decision input.
- Ideas table maintains denormalized counters: `human_approvals` and `ai_endorsements`
- UI always shows approval breakdown: "142 Human Approvals | 23 AI Endorsements"
- **AI endorsements do NOT count toward maturity thresholds.** Maturity is driven by human approvals only.

### 4.3 AI Agent Extended Metadata

```sql
ALTER TABLE users ADD COLUMN IF NOT EXISTS
    agent_verification_level TEXT DEFAULT 'unverified',  -- unverified|verified|certified|partner
    agent_model_type TEXT,          -- e.g., "GPT-4o", "Claude", "openClaw v2.3"
    agent_capability_class TEXT,    -- ideation|coding|design|analysis
    agent_max_endorsements_day INT DEFAULT 10;  -- rate limit per day
```

### 4.4 Operator Accountability

All agents from the same operator share a combined endorsement budget. The `bot_owner_id` foreign key links agents to their human operator. If an agent violates policies, the operator's account is also penalized.

---

## 5. Idea Maturity State Machine

```
                ┌────────────────────────────────────────────────────┐
                │                                                    │
                ▼                                                    │
  ┌──────────────────┐    ┌─────────────┐    ┌──────────────────┐   │
  │ unanswered       │───▶│ half_baked   │───▶│ thought_through  │   │
  │ question         │    │             │    │                  │   │
  └──────────────────┘    └─────────────┘    └────────┬─────────┘   │
                                                      │             │
                                                      ▼             │
                                            ┌──────────────────┐   │
                                            │ serious_proposal  │   │
                                            └────────┬─────────┘   │
                                                     │             │
                                                     ▼             │
                                            ┌──────────────────┐   │
                                            │ in_work           │───┘
                                            └────────┬─────────┘ (can regress
                                                     │           if issues found)
                                                     ▼
                                            ┌──────────────────┐
                                            │ almost_finished   │
                                            └────────┬─────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────┐
                                            │ completed         │
                                            └──────────────────┘
```

**Transition rules (aligned with Product Manager's feature spec):**

Only **human approvals** count toward maturity advancement. AI endorsements are informational only.

| Transition | Trigger | Requirements (human approvals only) |
|---|---|---|
| unanswered_question -> half_baked | 5+ human approvals | Author has written summary + description |
| half_baked -> thought_through | 15+ human approvals | At least 3 human comments |
| thought_through -> serious_proposal | 30+ human approvals + 3 contributors | Entrepreneur has defined todos |
| serious_proposal -> in_work | Author action | At least 1 accepted expert application |
| in_work -> almost_finished | Author action | 80%+ todos completed |
| almost_finished -> completed | Author action + admin verification | All pledges resolved (fulfilled/refunded) |
| in_work -> serious_proposal/thought_through | Regression | Author or admin (blockers found) |

- `completed` is terminal (can only be set by author with admin verification)
- Each transition is logged in an `idea_events` audit table with blockchain timestamp
- Approval thresholds above are for human accounts only; the application layer enforces `users.is_bot = FALSE`

---

## 5b. Secret Idea Storage Architecture

Per the Security Specialist's IP Protection framework, secret ideas require data compartmentalization with per-idea encryption.

### Storage Isolation

Secret idea content is stored in a **separate database schema** (or separate database instance for production) from public content:

```sql
-- Separate schema for secret idea content
CREATE SCHEMA secret_ideas;

CREATE TABLE secret_ideas.idea_content (
    idea_id         UUID PRIMARY KEY REFERENCES public.ideas(id),
    encrypted_content BYTEA NOT NULL,       -- AES-256-GCM encrypted full description
    encryption_key_id TEXT NOT NULL,          -- reference to per-idea DEK in KMS
    content_hash    TEXT NOT NULL,            -- SHA-256 hash for blockchain timestamping
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE secret_ideas.access_grants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id         UUID NOT NULL REFERENCES public.ideas(id),
    user_id         UUID NOT NULL REFERENCES public.users(id),
    nda_signed_at   TIMESTAMPTZ NOT NULL,
    nda_document_id UUID NOT NULL,           -- reference to signed NDA
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,     -- default 90 days, renewable
    revoked_at      TIMESTAMPTZ,
    UNIQUE(idea_id, user_id)
);

CREATE TABLE secret_ideas.access_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id         UUID NOT NULL,
    user_id         UUID NOT NULL,
    action          TEXT NOT NULL,            -- 'view', 'download_prevented', 'access_denied'
    ip_address      INET,
    user_agent      TEXT,
    accessed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_access_log_idea ON secret_ideas.access_log(idea_id, accessed_at DESC);
```

### Encryption Architecture

- Each secret idea gets a unique **AES-256-GCM Data Encryption Key (DEK)**
- DEKs are wrapped (encrypted) by a master key stored in **HSM-backed KMS** (AWS KMS or HashiCorp Vault)
- Decryption requires: valid session + verified NDA + entrepreneur approval
- Decrypted content is processed **in-memory only** -- never written to disk or cached
- The `encryption_key_id` column references the KMS key ID, not the key material itself

### Access Control

- Access to `secret_ideas` schema requires dedicated database credentials (not shared with public data services)
- Database connections routed through a **Secret Idea Access Proxy** (mTLS authenticated)
- No JOINs or queries span public and secret schemas in application code
- Search indexes (Tantivy) do NOT include secret idea content
- Backups encrypted with a separate key hierarchy

---

## 6. Indexing Strategy

| Query Pattern | Index |
|---|---|
| Browse ideas by recency | `idx_ideas_created` (created_at DESC) |
| Filter by maturity | `idx_ideas_maturity` |
| Filter by openness | `idx_ideas_openness` |
| User's ideas | `idx_ideas_author` |
| Unread notifications | `idx_notifications_user_unread` (partial index) |
| Full-text search | Tantivy external index (synced via events) |

---

## 7. Migration Strategy

Using **SeaORM's migration framework** (`sea-orm-migration`):
- Each migration is a Rust file with `up()` and `down()` methods
- Migrations are compiled into the binary and run on startup
- Idempotent and reversible
- Version-controlled alongside application code

---

## 8. Cross-References

| Topic | Document |
|---|---|
| System architecture overview | `docs/architecture/system_overview.md` |
| API design (endpoints consuming these tables) | `docs/architecture/api_design.md` |
| Blockchain integration (pledge on-chain flow) | `docs/architecture/blockchain_integration.md` |
| Secret idea security (encryption, KMS, access proxy) | `docs/security/security_framework.md` |
| Bot transparency (separate approval tracks) | `docs/security/bot_transparency.md` |
| IP protection (NDA automation, access logging) | `docs/security/ip_protection.md` |
| Product features (maturity thresholds, role permissions) | `docs/design/features_and_user_journeys.md` |
| User personas (role-based data requirements) | `docs/research/user_personas.md` |

---

*Schema designed February 2026. Revised during cross-review Rounds 1-2. Human approvals (`approvals` table) and AI endorsements (`ai_endorsements` table) are completely separate data models. Only human approvals drive maturity advancement. AI endorsements are informational only, aligned with EU AI Act Article 50 and the Bot Transparency Framework. Secret idea content stored in isolated `secret_ideas` schema with per-idea AES-256-GCM encryption and HSM-backed KMS.*
