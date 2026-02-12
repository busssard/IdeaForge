# IdeaForge - Architecture Decision Records (ADRs)

## Table of Contents

1. [ADR-001: Primary Language - Rust](#ADR-001:%20Primary%20Language%20-%20Rust)
2. [ADR-002: Database - PostgreSQL](#ADR-002:%20Database%20-%20PostgreSQL)
3. [ADR-003: Web Framework - Axum](#ADR-003:%20Web%20Framework%20-%20Axum)
4. [ADR-004: ORM - SeaORM](#ADR-004:%20ORM%20-%20SeaORM)
5. [ADR-005: Frontend - Leptos (Full-Stack Rust)](#ADR-005:%20Frontend%20-%20Leptos%20(Full-Stack%20Rust))
6. [ADR-006: Search Engine - Tantivy (Embedded)](#ADR-006:%20Search%20Engine%20-%20Tantivy%20(Embedded))
7. [ADR-007: Event System - NATS](#ADR-007:%20Event%20System%20-%20NATS)
8. [ADR-008: Authentication - JWT + OAuth2](#ADR-008:%20Authentication%20-%20JWT%20+%20OAuth2)
9. [ADR-009: Deployment - Docker + Compose (MVP)](#ADR-009:%20Deployment%20-%20Docker%20+%20Compose%20(MVP))
10. [ADR-010: Fiat Payments - Stripe](#ADR-010:%20Fiat%20Payments%20-%20Stripe)
11. [Cross-References](#Cross-References)

---

## ADR-001: Primary Language - Rust

**Status:** Accepted
**Date:** 2026-02-07

### Context

IdeaForge needs a primary language for backend, and potentially frontend development. The platform handles financial transactions (Cardano pledges), real-time collaboration, and AI agent interactions where reliability and performance are critical.

### Decision

Use **Rust** as the primary language for the entire stack.

### Rationale

1. **Memory safety without GC**: No null pointer exceptions, data races, or buffer overflows. Critical for a platform handling financial pledges.
2. **Performance**: Compiled, zero-cost abstractions. A single Rust binary on a $20/mo server handles traffic that would require multiple Node.js/Python instances.
3. **Type system**: Enums with data, pattern matching, and the ownership model catch entire classes of bugs at compile time. Idea maturity state machines, permission checks, and pledge flows are encoded in the type system.
4. **Ecosystem maturity**: Axum, SeaORM, Leptos, and tokio are production-ready. The Rust web ecosystem has matured significantly through 2024-2026.
5. **Cardano alignment**: Aiken (Cardano's modern smart contract language) is Rust-inspired. The Cardano tooling ecosystem has strong Rust support.
6. **Single-binary deployment**: `cargo build --release` produces one binary. No runtime dependencies, no dependency hell, no container bloat.
7. **Founder preference**: The founder explicitly prefers Rust.

### Consequences

- Steeper learning curve for new contributors (mitigated by good documentation and Rust's excellent compiler errors)
- Longer initial compilation times (mitigated by cargo workspaces, incremental compilation, and sccache)
- Smaller hiring pool (mitigated by Rust's growing community and the project's open-source nature attracting Rust enthusiasts)

---

## ADR-002: Database - PostgreSQL

**Status:** Accepted
**Date:** 2026-02-07

### Context

IdeaForge needs a primary database for storing users, ideas, pledges, approvals, and all platform data. The data model is relational with well-defined entities and relationships.

### Decision

Use **PostgreSQL 16+** as the primary database.

### Rationale

1. **Relational model fits**: Ideas, users, approvals, pledges -- the core domain is naturally relational with clear foreign key relationships.
2. **JSONB for flexibility**: Idea metadata, notification payloads, and role permissions use JSONB columns for schema-flexible fields without needing a separate document store.
3. **Mature Rust support**: SeaORM, Diesel, and SQLx all have excellent PostgreSQL drivers. `tokio-postgres` provides async native access.
4. **Advanced features**: Row-level security (for secret/IP-protected ideas), full-text search (as fallback), LISTEN/NOTIFY (for real-time), advisory locks, CTEs.
5. **Operational maturity**: Decades of production use, excellent tooling (pg_dump, pgBouncer, logical replication), wide hosting support.
6. **Cost**: Open source, runs anywhere, no license fees.

### Alternatives Considered

| Alternative | Why Not |
|---|---|
| MySQL | Weaker JSONB, no row-level security, less expressive SQL |
| MongoDB | Poor fit for relational data, no strong consistency guarantees needed for this workload |
| CockroachDB | Overkill for MVP, adds distributed complexity |
| SQLite | Single-writer bottleneck, no concurrent connections for a multi-user platform |

---

## ADR-003: Web Framework - Axum

**Status:** Accepted
**Date:** 2026-02-07

### Context

Need an HTTP framework for the REST API and WebSocket handling.

### Decision

Use **Axum 0.8+** as the HTTP framework.

### Rationale

1. **Tokio-native**: Built by the Tokio team, first-class async support, no impedance mismatch.
2. **Tower middleware**: Reuse the entire Tower ecosystem (rate limiting, tracing, compression, CORS, timeouts) without framework-specific abstractions.
3. **Macro-free**: Routes and handlers use plain Rust functions and types. No procedural macro magic, making the code easier to understand and debug.
4. **Type-safe extractors**: Request parsing (JSON body, path params, query params, headers) is type-safe and composable.
5. **WebSocket support**: Built-in WebSocket upgrade handling.
6. **Performance**: Benchmarks show Axum 0.8 achieves ~20% lower latency than earlier versions, competitive with Actix-web with better ergonomics.

### Alternatives Considered

| Alternative | Why Not |
|---|---|
| Actix-web | Slightly higher raw performance but macro-heavy, less ergonomic, separate actor runtime |
| Rocket | Sync by default (async added later), heavier macro usage, smaller middleware ecosystem |
| Warp | Filter-based API is harder to read, less active development |

---

## ADR-004: ORM - SeaORM

**Status:** Accepted
**Date:** 2026-02-07

### Context

Need a database abstraction layer for PostgreSQL interaction.

### Decision

Use **SeaORM 1.x** as the ORM.

### Rationale

1. **Async-first**: Built on SQLx with native async from day one. No need for r2d2 connection pool workaround.
2. **Active-record style**: Faster development for CRUD-heavy MVP. Less boilerplate than Diesel's DSL.
3. **Migration framework**: Built-in `sea-orm-migration` crate with Rust-based migrations (no separate SQL files).
4. **Code generation**: `sea-orm-cli` generates entity files from existing database, enabling rapid iteration.
5. **Dynamic queries**: Runtime query building is more natural than Diesel's compile-time approach, useful for search filtering with many optional parameters.

### Alternatives Considered

| Alternative | Why Not |
|---|---|
| Diesel | Synchronous core (diesel-async exists but is a separate crate), steeper learning curve, compile-time DSL adds complexity |
| SQLx | Too low-level for CRUD-heavy application, requires writing raw SQL for every query |
| Cornucopia | Interesting SQL-first approach but smaller ecosystem, less mature |

### Trade-offs Accepted

- Less compile-time query validation than Diesel (mitigated by comprehensive integration tests)
- Slightly higher runtime overhead than raw SQLx (acceptable for MVP)

---

## ADR-005: Frontend - Leptos (Full-Stack Rust)

**Status:** Accepted
**Date:** 2026-02-07

### Context

IdeaForge needs a web frontend with rich interactivity (real-time approvals, live comments, wallet integration).

### Decision

Use **Leptos 0.7+** for the web frontend, enabling full-stack Rust.

### Rationale

1. **Shared types**: Domain types (idea maturity enum, pledge status, etc.) are defined once in `ideaforge-core` and used in both backend and frontend. No TypeScript type duplication.
2. **Server functions**: `#[server]` macro enables seamless RPC from client to server without manually defining API endpoints for internal frontend calls.
3. **SSR + hydration**: Server-side rendering for SEO (idea pages should be indexable) with client-side hydration for interactivity.
4. **Fine-grained reactivity**: Signals system updates only the DOM nodes that change, without virtual DOM diffing overhead.
5. **Rust consistency**: One language, one toolchain, one CI pipeline. Reduces cognitive overhead and context switching.
6. **WASM performance**: Leptos compiles to WebAssembly, which is faster than JavaScript for compute-heavy UI operations.

### Alternatives Considered

| Alternative | Why Not |
|---|---|
| React/Next.js + TypeScript | Requires maintaining two languages, two build systems, type synchronization |
| Yew | Less mature SSR story, no server functions, heavier virtual DOM approach |
| Dioxus | Promising but less production-tested than Leptos for web |
| SvelteKit | Excellent DX but introduces JavaScript dependency and type duplication |

### Risks

- Leptos has a smaller ecosystem than React (fewer UI component libraries)
- CIP-30 wallet integration may require JavaScript interop for Cardano wallet connectors
- Mitigated by: Leptos supports JS interop via `wasm-bindgen`, and a thin JS bridge for wallet connectors is acceptable

---

## ADR-006: Search Engine - Tantivy (Embedded)

**Status:** Accepted
**Date:** 2026-02-07

### Context

IdeaForge needs full-text search for ideas, with support for typo tolerance and relevance ranking.

### Decision

Use **Tantivy** as an embedded search library for MVP, with a migration path to **Meilisearch** for production scale.

### Rationale

1. **Rust-native**: Tantivy is a pure Rust library, no external service to deploy or manage.
2. **Embedded simplicity**: For MVP, search runs in-process. No network hop, no service to monitor, no configuration.
3. **Performance**: Inspired by Apache Lucene, Tantivy provides fast full-text search with BM25 ranking.
4. **Migration path**: Meilisearch is built on Tantivy, so the indexing/query patterns translate directly when upgrading.
5. **Low operational cost**: No additional infrastructure for MVP.

### Upgrade Path

When the platform scales beyond single-server deployment:
1. Deploy Meilisearch as a separate service
2. Replace Tantivy indexing calls with Meilisearch SDK calls
3. The search crate (`ideaforge-search`) encapsulates this behind a trait, making the swap transparent

---

## ADR-007: Event System - NATS

**Status:** Accepted
**Date:** 2026-02-07

### Context

IdeaForge needs an event system for: (1) real-time updates via WebSocket, (2) async processing (notifications, search indexing), (3) future microservice communication.

### Decision

Use **NATS** as the message broker / event bus.

### Rationale

1. **Lightweight**: Single binary, minimal configuration, starts in milliseconds.
2. **Rust client**: `async-nats` crate is well-maintained and async-native.
3. **JetStream**: Persistent message streaming for reliable event delivery (notifications must not be lost).
4. **Pub/Sub + Request/Reply**: Supports both fire-and-forget events and request-response patterns.
5. **Future-proof**: When extracting microservices, NATS becomes the service mesh without changing the event model.

### Alternatives Considered

| Alternative | Why Not |
|---|---|
| RabbitMQ | Heavier, more complex configuration, Erlang runtime |
| Redis Pub/Sub | No persistence (messages lost if subscriber is down), limited routing |
| Apache Kafka | Massive overkill for MVP, complex operations |
| In-process channels | No persistence, no horizontal scaling, dead end for microservice extraction |

### Event Categories

```
ideaforge.ideas.created
ideaforge.ideas.maturity_changed
ideaforge.ideas.approved
ideaforge.pledges.confirmed
ideaforge.contributions.created
ideaforge.todos.assigned
ideaforge.users.registered
```

---

## ADR-008: Authentication - JWT + OAuth2

**Status:** Accepted
**Date:** 2026-02-07

### Context

The platform needs authentication for humans (email/password and social login) and bots (API keys).

### Decision

- **JWT** (jsonwebtoken crate) for stateless API authentication
- **OAuth2** (oxide-auth) for social login (GitHub, Google)
- **API keys** for bot authentication
- **Argon2** (argon2 crate) for password hashing

### Token Strategy

| Token | Lifetime | Storage | Purpose |
|---|---|---|---|
| Access token (JWT) | 15 min | Memory / Authorization header | API authentication |
| Refresh token | 7 days | httpOnly secure cookie | Token renewal |
| Bot API key | Until rotated | X-Api-Key header | Bot authentication |

---

## ADR-009: Deployment - Docker + Compose (MVP)

**Status:** Accepted
**Date:** 2026-02-07

### Context

Need a deployment strategy that balances simplicity (MVP) with production-readiness.

### Decision

- **MVP**: Docker Compose on a single VPS (Hetzner CX21, ~$10/mo)
- **Scale**: Kubernetes when needed (likely post-product-market-fit)

### Rationale

Rust's low resource footprint means a single $10-20/mo VPS handles significant traffic:
- Binary size: ~15-30MB
- Memory at idle: ~20MB
- Startup time: <100ms
- Concurrent connections: thousands (tokio async runtime)

A Kubernetes deployment at MVP stage would cost 5-10x more with no user-facing benefit.

### Docker Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/ideaforge /usr/local/bin/
EXPOSE 3000
CMD ["ideaforge"]
```

Final image: ~80MB (vs ~1.5GB for the build stage).

---

## ADR-010: Fiat Payments - Stripe

**Status:** Accepted
**Date:** 2026-02-07

### Context

Cross-review with business and persona teams identified that fiat payment support is non-negotiable for consumer adoption. The Consumer persona (driving 20% of Year 3 revenue via pledge-to-buy) may not adopt Cardano wallets.

### Decision

Use **Stripe** (via `stripe-rust` crate) for fiat payments in a dedicated `ideaforge-payments` crate.

### Scope

1. **Subscription billing**: Builder ($12/mo), Venture ($39/mo), Enterprise ($499+/mo) tiers via Stripe Checkout + Billing
2. **Fiat pledge on-ramp**: Optional fiat-to-ADA conversion for pledge campaigns (Stripe collects fiat, platform converts to ADA and locks in escrow)
3. **PCI DSS compliance**: Stripe tokenization ensures no card data is stored on platform

### Rationale

- `stripe-rust` is the official Rust SDK, actively maintained
- Stripe handles PCI DSS compliance, reducing security burden
- Fiat payments must be available from Phase 3 (Fueling the Fire), not as a later addition
- Crypto (Cardano) remains the primary rail for pledges; Stripe is the alternative for non-crypto users

---

## Cross-References

| Topic | Document |
|---|---|
| System architecture overview | `docs/architecture/system_overview.md` |
| API design | `docs/architecture/api_design.md` |
| Database schema | `docs/architecture/database_schema.md` |
| Blockchain integration | `docs/architecture/blockchain_integration.md` |
| Unit economics (infrastructure cost basis) | `docs/business/unit_economics.md` |

---

*ADRs documented February 2026. ADR-010 (Stripe) added during cross-review Round 2 based on persona analysis and business model alignment. Rust 1.85+ (2024 edition) required per ADR-001/ADR-009.*
