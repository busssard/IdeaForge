# IdeaForge - System Architecture Overview

## Table of Contents

1. [1. Architecture Style: Modular Monolith in Rust](#1.%20Architecture%20Style:%20Modular%20Monolith%20in%20Rust)
2. [2. Component Diagram](#2.%20Component%20Diagram)
3. [3. Technology Stack](#3.%20Technology%20Stack)
4. [4. Deployment Strategy](#4.%20Deployment%20Strategy)
5. [5. Module Boundaries (Crate Map)](#5.%20Module%20Boundaries%20(Crate%20Map))
6. [6. Cross-References](#6.%20Cross-References)

## 1. Architecture Style: Modular Monolith in Rust

IdeaForge adopts a **modular monolith** architecture for the MVP, with clear module boundaries designed for future microservice extraction.

### Why Modular Monolith?

| Concern | Modular Monolith | Microservices |
|---|---|---|
| Deployment complexity | Single binary | Many services, orchestration |
| Development speed (MVP) | Fast | Slow (infra overhead) |
| Refactoring ease | Compiler-enforced module boundaries | Network boundary changes |
| Operational cost | Low | High (K8s, service mesh) |
| Future extraction | Designed for it | Already there |

Rust's module system and Cargo workspaces naturally enforce strong boundaries between domains. Each domain lives in its own crate with explicit public APIs, making future extraction to standalone services a matter of adding a network layer rather than restructuring code.

---

## 2. Component Diagram

```
                           ┌─────────────────────────────────┐
                           │         Load Balancer            │
                           │        (Nginx / Traefik)         │
                           └──────────┬──────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                  │
            ┌───────▼──────┐  ┌──────▼───────┐  ┌──────▼───────┐
            │  Web Client  │  │  Mobile App  │  │  AI Agent    │
            │  (Leptos SSR │  │  (Future)    │  │  Clients     │
            │   + WASM)    │  │              │  │  (REST/WS)   │
            └───────┬──────┘  └──────┬───────┘  └──────┬───────┘
                    │                │                  │
                    └────────────────┼──────────────────┘
                                     │ HTTPS / WSS
                    ┌────────────────▼────────────────────┐
                    │         API Gateway (Axum)           │
                    │  ┌──────────────────────────────┐   │
                    │  │  Auth Middleware (JWT/OAuth2) │   │
                    │  │  Rate Limiter (tower)         │   │
                    │  │  Request Tracing (tracing)    │   │
                    │  └──────────────────────────────┘   │
                    └────────────────┬────────────────────┘
                                     │
        ┌────────────────────────────┼────────────────────────────┐
        │                            │              MODULAR MONOLITH
        │  ┌─────────────┐  ┌───────▼──────┐  ┌──────────────┐  │
        │  │   Ideas      │  │   Users &    │  │  Pledges &   │  │
        │  │   Domain     │  │   Roles      │  │  Payments    │  │
        │  │              │  │              │  │              │  │
        │  │ - CRUD       │  │ - Signup     │  │ - Pledge     │  │
        │  │ - Maturity   │  │ - Profiles   │  │ - Escrow     │  │
        │  │ - Approval   │  │ - Roles      │  │ - Cardano TX │  │
        │  │ - Categories │  │ - Onboarding │  │              │  │
        │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
        │         │                 │                  │          │
        │  ┌──────▼───────┐  ┌─────▼────────┐  ┌─────▼────────┐ │
        │  │ Contributions│  │  AI Agents   │  │  Search &    │ │
        │  │ & Todos      │  │  Integration │  │  Discovery   │ │
        │  │              │  │              │  │              │ │
        │  │ - Comments   │  │ - Bot reg.   │  │ - Full-text  │ │
        │  │ - Suggestions│  │ - Endorsement│  │ - Categories │ │
        │  │ - Todo items │  │ - Workforce  │  │ - Filtering  │ │
        │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
        │         │                 │                  │          │
        └─────────┼─────────────────┼──────────────────┼──────────┘
                  │                 │                  │
        ┌─────────▼─────────────────▼──────────────────▼──────────┐
        │              Shared Infrastructure Layer                │
        │  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌────────┐│
        │  │PostgreSQL│  │  Redis     │  │ Tantivy  │  │ NATS   ││
        │  │(SeaORM)  │  │  (Cache +  │  │ (Search) │  │(Events)││
        │  │          │  │   Sessions)│  │          │  │        ││
        │  └──────────┘  └───────────┘  └──────────┘  └────────┘│
        └────────────────────┬────────────────────────────────────┘
                  │          │
        ┌─────────▼──────┐   │   ┌──────────────────────────────┐
        │ Cardano        │   │   │  Secret Ideas Zone (Isolated) │
        │ Blockchain     │   │   │                               │
        │ (Aiken Smart   │   └──▶│  ┌─────────────────────────┐ │
        │  Contracts)    │       │  │ Secret Idea Access Proxy │ │
        │ - Blockfrost   │       │  │ (mTLS, NDA verification) │ │
        │ - Wallets      │       │  └────────────┬────────────┘ │
        └────────────────┘       │  ┌────────────▼────────────┐ │
                                 │  │ Separate DB / Schema     │ │
                                 │  │ Per-idea AES-256-GCM     │ │
                                 │  │ HSM-backed KMS keys      │ │
                                 │  │ Isolated access logging  │ │
                                 │  └─────────────────────────┘ │
                                 └──────────────────────────────┘
```

---

## 3. Technology Stack

### Minimum Rust Version

**Rust 1.85+ (2024 edition) is required.** Several key dependencies (argon2, base64ct, and other modern crates) use the Rust 2024 edition. Install via `rustup update stable` or `rustup install 1.85.0`. The Dockerfile build stage uses `rust:1.85-bookworm`.

### Backend (Rust)

| Layer | Technology | Justification |
|---|---|---|
| **HTTP Framework** | **Axum 0.8+** | Tokio-native, tower middleware ecosystem, macro-free ergonomics, best-in-class async performance. Axum 0.8 achieves ~20% lower latency than earlier versions in high-throughput scenarios. |
| **ORM / DB** | **SeaORM 1.x** | Async-first (no r2d2 pool needed), active-record style for rapid CRUD development, migration tooling built-in. Better DX for MVP speed than Diesel. |
| **Database** | **PostgreSQL 16+** | JSONB for flexible idea metadata, full ACID, excellent Rust driver support, row-level security for multi-tenancy. |
| **Cache / Sessions** | **Redis 7+** | Session storage, rate-limit counters, real-time pub/sub for WebSocket fan-out. |
| **Search** | **Tantivy** (embedded) | Rust-native full-text search library. Embedded = no external service for MVP. Upgrade path to Meilisearch (built on Tantivy) when scaling. |
| **Event Bus** | **NATS** | Lightweight, Rust client available. Used for domain events (idea created, pledge made, etc.) and future service extraction. |
| **Auth** | **JWT (jsonwebtoken crate) + OAuth2 (oxide-auth)** | Stateless JWT for API auth, OAuth2 for social login (GitHub, Google). |
| **Blockchain** | **Blockfrost API + Aiken** | Aiken for smart contracts (Rust-inspired syntax), Blockfrost REST API for Cardano chain interaction from Rust backend. |
| **Fiat Payments** | **Stripe** (via `stripe-rust` crate) | Fiat on-ramp for subscriptions (Builder/Venture/Enterprise tiers) and optional fiat-to-ADA conversion for pledges. PCI DSS compliance via Stripe tokenization -- no card data stored on platform. |

### Frontend

| Layer | Technology | Justification |
|---|---|---|
| **Web Framework** | **Leptos 0.7+** | Full-stack Rust: SSR + hydration + WASM. Shares types with backend. Fine-grained reactivity for excellent performance. Eliminates JS/TS dependency for MVP. |
| **Styling** | **Tailwind CSS** | Utility-first, works well with Leptos component model. |
| **State Management** | **Leptos signals + server functions** | Built into framework, no extra library needed. |

### Infrastructure

| Layer | Technology | Justification |
|---|---|---|
| **Containerization** | **Docker** | Multi-stage builds: Rust builder -> slim Debian runtime. Single binary deployment. |
| **Orchestration** | **Docker Compose** (MVP) -> **Kubernetes** (scale) | Start simple, graduate to K8s when needed. |
| **CI/CD** | **GitHub Actions** | Rust caching, cargo-deny for license/vulnerability checks. |
| **Observability** | **tracing + OpenTelemetry** | Structured logging, distributed tracing, Prometheus metrics via tower middleware. |
| **Cloud** | **Hetzner / Fly.io** (MVP) | Cost-effective for early stage. Rust's low resource footprint means a single $20/mo server handles significant traffic. |

---

## 4. Deployment Strategy

### MVP Deployment (Single Server)

```
┌──────────────────────────────────┐
│  Docker Compose on Hetzner VPS   │
│                                  │
│  ┌────────────┐  ┌────────────┐  │
│  │ IdeaForge  │  │ PostgreSQL │  │
│  │ (single    │  │            │  │
│  │  binary)   │  └────────────┘  │
│  └────────────┘                  │
│  ┌────────────┐  ┌────────────┐  │
│  │ Redis      │  │ Nginx      │  │
│  │            │  │ (TLS term) │  │
│  └────────────┘  └────────────┘  │
└──────────────────────────────────┘
```

### Production Scale

```
┌──────────────────────────────────────────┐
│  Kubernetes Cluster                      │
│                                          │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │ API pods  │  │ Worker   │  │ Leptos │ │
│  │ (Axum)   │  │ pods     │  │ SSR    │ │
│  │ x3       │  │ x2       │  │ x2     │ │
│  └──────────┘  └──────────┘  └────────┘ │
│                                          │
│  ┌──────────┐  ┌──────────┐  ┌────────┐ │
│  │ PG (HA)  │  │ Redis    │  │ NATS   │ │
│  │ primary+ │  │ Sentinel │  │ cluster│ │
│  │ replica  │  │          │  │        │ │
│  └──────────┘  └──────────┘  └────────┘ │
└──────────────────────────────────────────┘
```

### Build Pipeline

```
Source -> cargo test -> cargo clippy -> cargo build --release
       -> docker build (multi-stage) -> push to registry
       -> deploy to staging -> smoke tests -> deploy to production
```

The Rust binary compiles to ~15-30MB, starts in <100ms, and uses ~20MB RAM at idle -- enabling aggressive scaling on minimal infrastructure.

---

## 5. Module Boundaries (Crate Map)

```
ideaforge/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── ideaforge-core/           # Domain types, traits, errors
│   │   └── domain/
│   │       ├── idea.rs           # Idea, maturity state machine, approval/endorsement separation
│   │       ├── user.rs           # User, roles, progressive disclosure tiers
│   │       ├── agent.rs          # AI agent types, verification levels, endorsement
│   │       └── ...
│   ├── ideaforge-db/             # SeaORM entities, migrations, repositories
│   ├── ideaforge-api/            # Axum routes, handlers, middleware
│   ├── ideaforge-auth/           # JWT, OAuth2, permissions
│   ├── ideaforge-search/         # Tantivy indexing and querying
│   ├── ideaforge-blockchain/     # Cardano/Blockfrost integration
│   ├── ideaforge-payments/      # Stripe fiat payments (subscriptions, fiat on-ramp)
│   ├── ideaforge-events/         # NATS event publishing/subscribing
│   └── ideaforge-web/            # Leptos frontend (future)
└── src/
    └── main.rs                   # Binary entry point, composes all crates
```

Each crate exposes only its public API. Inter-crate communication happens through shared types in `ideaforge-core` and the event system. This makes future extraction to microservices straightforward: replace in-process function calls with network calls at the crate boundary.

---

## 6. Cross-References

| Topic | Document |
|---|---|
| API design (REST endpoints, WebSocket, rate limiting) | `docs/architecture/api_design.md` |
| Database schema (tables, maturity state machine, secret ideas) | `docs/architecture/database_schema.md` |
| Blockchain integration (Aiken smart contracts, Cardano, CIP-30) | `docs/architecture/blockchain_integration.md` |
| Architecture decision records (ADRs) | `docs/architecture/tech_decisions.md` |
| Security framework (OWASP, mTLS, KMS, SOC 2) | `docs/security/security_framework.md` |
| Bot transparency (EU AI Act Article 50, separate approval tracks) | `docs/security/bot_transparency.md` |
| IP protection (secret idea encryption, NDA automation) | `docs/security/ip_protection.md` |
| Business model (pricing tiers, marketplace commissions) | `docs/business/business_model.md` |
| Unit economics (infrastructure cost basis, $30-60/1K users) | `docs/business/unit_economics.md` |
| Go-to-market strategy (launch phases, digital-first) | `docs/business/go_to_market.md` |
| Product vision and roadmap | `docs/design/product_vision.md` |
| Features and user journeys | `docs/design/features_and_user_journeys.md` |
| Brand identity (forge metaphor, Stoke vocabulary) | `docs/design/brand_identity.md` |
| UX philosophy (progressive disclosure, WCAG 2.2 AA) | `docs/design/ux_philosophy.md` |
| User personas | `docs/research/user_personas.md` |
| Community strategy (North Star Metric) | `docs/research/community_strategy.md` |
| Pitch deck | `deliverables/pitch-deck/generate_pitch_deck.py` |
| Whitepaper | `deliverables/whitepaper/ideaforge_whitepaper.tex` |

---

*Architecture designed February 2026. Revised during cross-review Rounds 1-2 with business, product, creative, security, and persona teams. Rust 1.85+ (2024 edition) required. Smart contracts: Aiken on Cardano. Fiat payments: Stripe via `ideaforge-payments` crate. Terminology: API uses stable technical names (`approval`/`endorsement`); UI maps to forge language (`Stoke`/`AI Endorsement`). Human approvals drive maturity advancement; AI endorsements are informational only. Accessibility target: WCAG 2.2 AA. Infrastructure cost: $30-60/1K users/month (Hetzner VPS, Rust modular monolith).*
