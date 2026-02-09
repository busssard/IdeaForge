# IdeaForge MVP Architecture (Phase 1)

> **This document describes what we build first.** The full architecture docs
> (system_overview.md, database_schema.md, etc.) represent the long-term vision.
> This MVP is scoped for **2-3 Rust engineers shipping in 4 months**.

---

## 1. Guiding Principles

| Principle | Implication |
|---|---|
| Ship fast, validate early | If it's not needed to validate the core loop, defer it |
| Team formation is the killer feature | Task boards, applications, and team assembly get the most design attention |
| Human-only platform at launch | No AI agents, no bot accounts, no AI endorsements |
| DB-backed everything | No NATS, no Redis. PostgreSQL handles notifications, sessions, and job queues |
| Auth is boring (on purpose) | JWT + password + email verification. No MFA, no OAuth, no social login for MVP |
| Cardano in Phase 2-3 | Pledges are deferred. No blockchain integration at launch |
| Stripe in Phase 2 | No payments at launch. Everyone is on the free tier |

---

## 2. MVP Scope Summary

### What We Build (Phase 1 -- 4 months)

| Domain | Scope |
|---|---|
| **Ideas** | CRUD, 3 maturity levels (Spark / Building / InWork), 3 openness modes (Open / Collaborative / Commercial -- no Secret) |
| **Users** | Registration, login, profiles, 3 roles (Entrepreneur / Maker / Curious) |
| **Approvals** | Human "Stokes" only (upvotes). Drive maturity advancement |
| **Contributions** | Comments and suggestions on ideas (threaded) |
| **Team Formation** | Task boards per idea, role applications, team member management -- **killer feature** |
| **Categories** | Hierarchical idea categorization |
| **Notifications** | DB-backed notification table, polled from API |
| **Search** | Tantivy embedded full-text search |
| **Auth** | JWT access/refresh tokens, Argon2 password hashing, email verification |

### What We Defer

| Feature | Deferred To | Reason |
|---|---|---|
| Cardano blockchain / pledges | Phase 2-3 | Requires smart contract audit, wallet integration |
| Stripe payments | Phase 2 | No revenue needed pre-PMF; everyone on free tier |
| NATS event bus | Phase 2+ | Over-engineering for MVP. DB-backed notifications suffice |
| AI agents / bot accounts | Phase 2+ | Human-only platform validates core loop first |
| AI endorsements | Phase 2+ | Comes with AI agent support |
| MFA (TOTP / WebAuthn) | Phase 2 | Security hardening after PMF |
| OAuth2 social login | Phase 2 | Nice-to-have, not blocking |
| Secret ideas / encryption | Phase 2+ | Complex IP protection infrastructure |
| WebSocket real-time | Phase 2 | Polling is fine for MVP traffic levels |

---

## 3. MVP Component Diagram

```
                       ┌──────────────────────────────┐
                       │       Web Client              │
                       │    (Leptos SSR + WASM)         │
                       └──────────────┬───────────────┘
                                      │ HTTPS
                       ┌──────────────▼───────────────┐
                       │      API Server (Axum)        │
                       │  ┌────────────────────────┐   │
                       │  │  Auth Middleware (JWT)  │   │
                       │  │  Rate Limiter (tower)   │   │
                       │  │  Tracing               │   │
                       │  └────────────────────────┘   │
                       └──────────────┬───────────────┘
                                      │
       ┌──────────────────────────────┼──────────────────────────┐
       │                              │            MODULAR MONOLITH
       │  ┌──────────────┐  ┌────────▼───────┐  ┌────────────┐  │
       │  │  Ideas        │  │  Users &       │  │  Team      │  │
       │  │  Domain       │  │  Auth          │  │  Formation │  │
       │  │               │  │                │  │  (KILLER)  │  │
       │  │ - CRUD        │  │ - Register     │  │            │  │
       │  │ - 3 Maturity  │  │ - Login/JWT    │  │ - Boards   │  │
       │  │ - Stokes      │  │ - 3 Roles      │  │ - Tasks    │  │
       │  │ - Categories  │  │ - Profiles     │  │ - Apply    │  │
       │  └───────┬───────┘  └────────┬───────┘  │ - Teams    │  │
       │          │                   │           └─────┬──────┘  │
       │  ┌───────▼───────┐  ┌───────▼────────┐  ┌─────▼──────┐  │
       │  │ Contributions  │  │ Notifications   │  │  Search    │  │
       │  │ & Comments     │  │ (DB-backed)     │  │ (Tantivy)  │  │
       │  └───────┬────────┘  └───────┬────────┘  └─────┬──────┘  │
       │          │                   │                  │          │
       └──────────┼───────────────────┼──────────────────┼──────────┘
                  │                   │                  │
       ┌──────────▼───────────────────▼──────────────────▼──────────┐
       │                    PostgreSQL 16+                           │
       │  (ideas, users, approvals, tasks, teams, notifications)    │
       └────────────────────────────────────────────────────────────┘
```

No Redis. No NATS. No blockchain. One PostgreSQL database.

---

## 4. Simplified Domain Model

### 4.1 Maturity Levels (3, not 7)

```
  ┌─────────┐    5+ Stokes    ┌──────────┐    Author action    ┌──────────┐
  │  Spark   │───────────────▶│ Building  │──────────────────▶│  InWork  │
  └─────────┘                 └──────────┘                    └──────────┘
```

| Level | Old Name | Description | Transition |
|---|---|---|---|
| **Spark** | unanswered_question | New idea, just posted | Default for new ideas |
| **Building** | thought_through | Validated interest, developing | 5+ human Stokes |
| **InWork** | in_work | Active team, executing | Author action + at least 1 team member |

Rationale: The investor feedback says 3 levels. "Spark" captures the initial energy,
"Building" the validation phase, "InWork" the execution phase. These map cleanly to
the platform's core value proposition: ideas attract interest, form teams, and execute.

### 4.2 Roles (3, not 8)

| Role | Who | Key Capabilities |
|---|---|---|
| **Entrepreneur** | Idea authors, project leads | Create ideas, manage task boards, accept/reject team applications, advance maturity |
| **Maker** | Builders, designers, devs | Apply to join teams, claim tasks, contribute |
| **Curious** | Browsers, early supporters | Browse, Stoke ideas, comment |

No Investor, Consumer, Freelancer, or AI Agent roles at MVP. Simplified permission model.

### 4.3 Openness (3, not 4)

| Mode | Description |
|---|---|
| **Open** | Anyone can see and contribute |
| **Collaborative** | Open but team membership is curated |
| **Commercial** | Visible but contributions require approval |

No **Secret** mode at MVP. Secret ideas require encryption infrastructure that is deferred.

---

## 5. Team Formation -- The Killer Feature

Team formation is what differentiates IdeaForge from a simple idea board.
It answers: "I have an idea. How do I find people to build it?"

### 5.1 Data Model

```
┌──────────┐         ┌──────────────┐         ┌──────────────┐
│  ideas    │──1:N──▶│  task_boards  │──1:N──▶│  board_tasks  │
└──────────┘         │              │         │              │
                     │ idea_id FK   │         │ board_id FK  │
                     │ name         │         │ title        │
                     │ description  │         │ description  │
                     └──────────────┘         │ status       │
                                              │ assignee_id  │
                                              │ skill_tags   │
┌──────────┐         ┌──────────────┐         │ priority     │
│  users    │──1:N──▶│ team_members  │         └──────────────┘
└──────────┘         │              │
                     │ idea_id FK   │         ┌──────────────┐
                     │ user_id FK   │         │ team_apps     │
                     │ role         │         │              │
                     │ status       │         │ idea_id FK   │
                     │ joined_at    │         │ user_id FK   │
                     └──────────────┘         │ role         │
                                              │ pitch TEXT   │
                                              │ status       │
                                              └──────────────┘
```

### 5.2 Task Board Flow

1. **Entrepreneur creates idea** -> idea starts at Spark maturity
2. **Idea gets Stoked** -> reaches Building maturity at 5+ Stokes
3. **Entrepreneur creates task board** with categorized tasks
4. **Makers browse open tasks** and apply to join the team
5. **Entrepreneur reviews applications** and accepts/rejects
6. **Accepted makers join the team** and can self-assign tasks
7. **When team is formed** (1+ member), idea can advance to InWork
8. **Tasks are tracked** through suggested -> accepted -> in_progress -> done

### 5.3 Task Board API Endpoints

```
# Task Boards (one per idea for MVP, extensible to multiple later)
POST   /api/v1/ideas/{id}/board                Create task board for idea
GET    /api/v1/ideas/{id}/board                Get task board with all tasks
PUT    /api/v1/ideas/{id}/board                Update board metadata

# Board Tasks
POST   /api/v1/ideas/{id}/board/tasks          Create a task
GET    /api/v1/ideas/{id}/board/tasks           List tasks (filterable by status, assignee)
GET    /api/v1/ideas/{id}/board/tasks/{tid}     Get task details
PUT    /api/v1/ideas/{id}/board/tasks/{tid}     Update task (status, assignee, details)
DELETE /api/v1/ideas/{id}/board/tasks/{tid}     Remove task

# Team Applications
POST   /api/v1/ideas/{id}/team/apply            Apply to join idea's team
GET    /api/v1/ideas/{id}/team/applications      List applications (entrepreneur only)
PUT    /api/v1/ideas/{id}/team/applications/{aid} Accept/reject application

# Team Members
GET    /api/v1/ideas/{id}/team                   List team members
DELETE /api/v1/ideas/{id}/team/{uid}             Remove team member (entrepreneur only)
GET    /api/v1/users/me/teams                    List teams I'm part of
```

### 5.4 Database Tables

```sql
-- Task board for an idea (one per idea for MVP)
CREATE TABLE task_boards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID UNIQUE NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    name        TEXT NOT NULL DEFAULT 'Main Board',
    description TEXT DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tasks on a board
CREATE TYPE board_task_status AS ENUM (
    'open',           -- available to claim
    'assigned',       -- someone is working on it
    'in_review',      -- work submitted, pending review
    'done'            -- completed
);

CREATE TABLE board_tasks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id    UUID NOT NULL REFERENCES task_boards(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT DEFAULT '',
    status      board_task_status NOT NULL DEFAULT 'open',
    assignee_id UUID REFERENCES users(id),
    skill_tags  TEXT[] DEFAULT '{}',        -- e.g., {'rust', 'design', 'marketing'}
    priority    INT NOT NULL DEFAULT 0,     -- 0=normal, 1=high, 2=urgent
    due_date    DATE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_board_tasks_board ON board_tasks(board_id);
CREATE INDEX idx_board_tasks_assignee ON board_tasks(assignee_id) WHERE assignee_id IS NOT NULL;
CREATE INDEX idx_board_tasks_status ON board_tasks(status);

-- Team members for an idea
CREATE TYPE team_member_role AS ENUM (
    'lead',           -- idea author / project lead
    'builder',        -- accepted maker
    'advisor'         -- non-building contributor
);

CREATE TYPE team_member_status AS ENUM (
    'active',
    'inactive',
    'removed'
);

CREATE TABLE team_members (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id),
    role        team_member_role NOT NULL DEFAULT 'builder',
    status      team_member_status NOT NULL DEFAULT 'active',
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(idea_id, user_id)
);

CREATE INDEX idx_team_members_idea ON team_members(idea_id);
CREATE INDEX idx_team_members_user ON team_members(user_id);

-- Team applications
CREATE TYPE team_app_status AS ENUM (
    'pending',
    'accepted',
    'rejected',
    'withdrawn'
);

CREATE TABLE team_applications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idea_id     UUID NOT NULL REFERENCES ideas(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id),
    role        team_member_role NOT NULL DEFAULT 'builder',
    pitch       TEXT NOT NULL,              -- why the applicant wants to join
    status      team_app_status NOT NULL DEFAULT 'pending',
    reviewed_by UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(idea_id, user_id)               -- one application per user per idea
);

CREATE INDEX idx_team_apps_idea ON team_applications(idea_id);
CREATE INDEX idx_team_apps_user ON team_applications(user_id);
```

---

## 6. MVP Crate Map

```
ideaforge/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── ideaforge-core/           # Domain types (MVP-scoped)
│   │   └── domain/
│   │       ├── idea.rs           # 3 maturity levels, 3 openness modes
│   │       ├── user.rs           # 3 roles: Entrepreneur, Maker, Curious
│   │       ├── team.rs           # NEW: task boards, team members, applications
│   │       ├── contribution.rs   # Comments and suggestions
│   │       ├── category.rs       # Hierarchical categories
│   │       ├── notification.rs   # DB-backed notification types
│   │       └── mod.rs
│   ├── ideaforge-db/             # SeaORM entities, migrations, repositories
│   ├── ideaforge-api/            # Axum routes, handlers, middleware
│   ├── ideaforge-auth/           # JWT + password only (no MFA, no OAuth)
│   ├── ideaforge-search/         # Tantivy indexing and querying
│   │
│   │   --- DEFERRED CRATES (Phase 2+, kept as stubs) ---
│   │
│   ├── ideaforge-blockchain/     # DEFERRED: Phase 2-3, Cardano pledges
│   ├── ideaforge-payments/       # DEFERRED: Phase 2, Stripe subscriptions
│   └── ideaforge-events/         # DEFERRED: Phase 2+, NATS event bus
```

### MVP Dependencies (ideaforge-api)

```
ideaforge-api
├── ideaforge-core      (domain types)
├── ideaforge-db        (database access)
├── ideaforge-auth      (JWT + password)
└── ideaforge-search    (Tantivy)
```

The API crate does NOT depend on blockchain, payments, or events at MVP.

---

## 7. MVP Auth Flow

```
POST /api/v1/auth/register     -> Create account (email + password)
POST /api/v1/auth/login        -> Returns JWT access + refresh token
POST /api/v1/auth/refresh      -> Refresh access token
POST /api/v1/auth/logout       -> Invalidate refresh token
POST /api/v1/auth/verify-email -> Email verification
```

No MFA. No OAuth. No social login. No bot API keys.

JWT token:
```json
{
  "sub": "user-uuid",
  "email": "user@example.com",
  "roles": ["entrepreneur"],
  "exp": 1700000000,
  "iat": 1699996400
}
```

- Access token: 15-minute expiry
- Refresh token: 7-day expiry, httpOnly cookie

---

## 8. MVP API Endpoints (Complete)

### Auth
```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/logout
POST   /api/v1/auth/verify-email
```

### Users
```
GET    /api/v1/users/me
PUT    /api/v1/users/me
GET    /api/v1/users/{id}
GET    /api/v1/users/{id}/ideas
GET    /api/v1/users/me/teams
```

### Ideas
```
POST   /api/v1/ideas
GET    /api/v1/ideas                    (paginated, filterable)
GET    /api/v1/ideas/{id}
PUT    /api/v1/ideas/{id}
DELETE /api/v1/ideas/{id}               (soft delete)
PUT    /api/v1/ideas/{id}/maturity
```

### Stokes (Human Approvals)
```
POST   /api/v1/ideas/{id}/stokes
DELETE /api/v1/ideas/{id}/stokes
GET    /api/v1/ideas/{id}/stokes
```

### Contributions
```
POST   /api/v1/ideas/{id}/contributions
GET    /api/v1/ideas/{id}/contributions
PUT    /api/v1/ideas/{id}/contributions/{cid}
DELETE /api/v1/ideas/{id}/contributions/{cid}
```

### Team Formation (Killer Feature)
```
POST   /api/v1/ideas/{id}/board
GET    /api/v1/ideas/{id}/board
PUT    /api/v1/ideas/{id}/board
POST   /api/v1/ideas/{id}/board/tasks
GET    /api/v1/ideas/{id}/board/tasks
GET    /api/v1/ideas/{id}/board/tasks/{tid}
PUT    /api/v1/ideas/{id}/board/tasks/{tid}
DELETE /api/v1/ideas/{id}/board/tasks/{tid}
POST   /api/v1/ideas/{id}/team/apply
GET    /api/v1/ideas/{id}/team/applications
PUT    /api/v1/ideas/{id}/team/applications/{aid}
GET    /api/v1/ideas/{id}/team
DELETE /api/v1/ideas/{id}/team/{uid}
```

### Categories
```
GET    /api/v1/categories
GET    /api/v1/categories/{slug}
```

### Notifications
```
GET    /api/v1/notifications
PUT    /api/v1/notifications/{id}/read
PUT    /api/v1/notifications/read-all
GET    /api/v1/notifications/unread-count
```

### Search
```
GET    /api/v1/search?q=term
```

### Health
```
GET    /health
```

**Total: ~35 endpoints.** A focused, shippable surface area.

---

## 9. MVP Deployment

```
┌──────────────────────────────────┐
│  Docker Compose on Hetzner VPS   │
│  (~$10/mo CX21)                  │
│                                  │
│  ┌────────────┐  ┌────────────┐  │
│  │ IdeaForge  │  │ PostgreSQL │  │
│  │ (single    │  │            │  │
│  │  binary)   │  └────────────┘  │
│  └────────────┘                  │
│  ┌────────────┐                  │
│  │ Nginx      │                  │
│  │ (TLS term) │                  │
│  └────────────┘                  │
└──────────────────────────────────┘
```

No Redis. No NATS. Two containers + Nginx. Total infra cost: ~$10-20/mo.

---

## 10. Phase Roadmap (Architecture)

| Phase | Timeline | Architecture Additions |
|---|---|---|
| **Phase 1: MVP** | Months 1-4 | Core loop + team formation. PG only. |
| **Phase 2: Monetize** | Months 5-8 | Add Stripe payments, OAuth2 social login, MFA, Redis for sessions/cache |
| **Phase 3: Blockchain** | Months 9-12 | Cardano pledges, Aiken smart contracts, NATS event bus |
| **Phase 4: Scale** | Year 2+ | AI agents, secret ideas, Meilisearch, Kubernetes |

---

## 11. Cross-References

| Topic | Document |
|---|---|
| Full system architecture (long-term vision) | `docs/architecture/system_overview.md` |
| Full database schema (long-term) | `docs/architecture/database_schema.md` |
| Full API design (long-term) | `docs/architecture/api_design.md` |
| Blockchain integration (Phase 2-3) | `docs/architecture/blockchain_integration.md` |
| ADRs | `docs/architecture/tech_decisions.md` |

---

*MVP architecture designed February 2026. Scoped for 2-3 Rust engineers, 4-month delivery. Team formation is the killer feature. Simplified to 3 maturity levels (Spark/Building/InWork), 3 roles (Entrepreneur/Maker/Curious), human Stokes only. Deferred: blockchain, payments, NATS, AI agents, MFA, secret ideas. Infrastructure: single PG database, no Redis, no message broker.*
