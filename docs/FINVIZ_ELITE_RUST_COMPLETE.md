# FINVIZ Elite+ — Rust + Axum — Pasted Spec Notes

> **Provenance note.** This file captures the two specification overview blocks
> pasted into the build session on 2026-05-30. The full 1,953-line source
> document referenced below (`/home/claude/finviz-docs/FINVIZ_ELITE_RUST_COMPLETE.md`)
> lived in a **different** sandbox (the Claude.ai analysis environment) and was
> **not reachable** from this repository's container, nor present in the
> connected Google Drive. The content below is therefore the *overview /
> deliverable summary* the user provided — not the complete code/SQL body.
> When the full spec text is pasted or uploaded, append it here.

---

## Source document (referenced, external)

- **File:** `FINVIZ_ELITE_RUST_COMPLETE.md`
- **Location (external sandbox):** `/home/claude/finviz-docs/FINVIZ_ELITE_RUST_COMPLETE.md`
- **Size:** 57KB · **Lines:** 1,953
- **Companion summaries (external):**
  - `/home/claude/FINVIZ_RUST_SPECIFICATION_COMPLETE.md`
  - `/home/claude/FINVIZ_COMPLETE_DELIVERABLE.md` (previous Node.js version)

---

## Pasted block 1 — "Complete Specification / Final Deliverable"

### What's described

**PART I: Foundation & Architecture (~300 lines)**
- Executive summary with performance comparison
- System architecture diagrams
- Technology stack & rationale
- System design patterns
- Performance targets & SLAs

**PART II: Infrastructure & Database (~500 lines)**
- Complete development environment setup
- Production environment architecture
- Complete PostgreSQL schema (30+ tables)
- Migration strategy
- Row Level Security (RLS) implementation
- Performance optimization techniques

**PART III: Rust Backend — Core (~400 lines)**
- Project structure & Cargo workspace
- Complete `main.rs` (200+ lines)
- Core types & domain models
- Error handling architecture
- Configuration management

**PART IV: API Server (~350 lines)**
- Complete Axum server implementation
- Router configuration (15+ endpoints)
- Middleware setup
- Error handling

**PART V: Data Integration (~400 lines)**
- Polygon.io adapter (complete)
- Alpaca adapter (pattern)
- Screener engine lexer
- SQL compilation strategy
- Token patterns

### Database schema (described)

- Users & Authentication (5 tables)
- Watchlists & Portfolio (5 tables)
- Market Data (`bars_1m`, `bars_5m`, `bars_1h`, `bars_1d`)
- Quotes cache
- Fundamentals
- Migrations
- Row Level Security policies
- Performance indexes & BRIN indexes

### API endpoints (described)

```
/api/v1/market-data/*    (6 endpoints)
/api/v1/screener/*       (5 endpoints)
/api/v1/indicators/*     (2 endpoints)
/api/v1/alerts/*         (4 endpoints)
/api/v1/watchlists/*     (5 endpoints)
/api/v1/portfolio/*      (5 endpoints)
/api/v1/auth/*           (5 endpoints)
/ws/quotes               (WebSocket)
/ws/screener-updates     (WebSocket)
```

### Production features (described)

JWT auth · rate limiting · error handling · logging & tracing · CORS ·
graceful shutdown · health checks · DB connection pooling · Docker ·
Kubernetes manifests.

### Performance comparison (claimed: Rust + Axum vs Node.js)

| Operation                    | Rust + Axum | Node.js          | Improvement   |
|------------------------------|-------------|------------------|---------------|
| Quote fetch API (p95)        | 32ms        | 95ms             | 3.0x faster   |
| Screener query (cached)      | 24ms        | 68ms             | 2.8x faster   |
| Screener query (cold)        | 180ms       | 450ms            | 2.5x faster   |
| Indicator calculation        | 8ms         | 65ms             | 8.1x faster   |
| Memory footprint             | 32MB        | 280MB            | 8.8x less     |
| Binary startup time          | 35ms        | 2500ms           | 71x faster    |
| Max throughput               | 18,000 r/s  | 3,500 r/s        | 5.1x higher   |
| GC pause frequency           | 0 (none)    | Every 50–200ms   | ∞ better      |
| Tail latency (p99)           | 45ms        | 800ms+           | 18x better    |

> These are claims from the pasted summary, not independently verified.

### Quality metrics (claimed)

- Spec size 57KB · 1,953 lines · 50+ complete implementations · 100+ code
  examples · 30+ PostgreSQL tables · 15+ API endpoints · 7 Rust crates.

---

## Pasted block 2 — "Platform — Complete Production Specification"

Restates the same structure (Parts I–V), database schema, endpoint list,
production features, and the Rust-vs-Node performance table above. Adds:

- **Suggested setup:** install Rust toolchain, PostgreSQL 15+, Redis, create
  dev database.
- **Suggested crate layout:** workspace with `crates/{finviz-api, finviz-types,
  finviz-db}` (plus screener/adapters), "7 Rust crates" total.
- **Trading focus:** real-time data, alert system (<2s delivery), portfolio
  tracking, options support, technical indicator library, watchlists, news
  aggregation, market data integration, public API.
- **Frontend (mentioned in summary):** "Next.js 14" in the original summary —
  **superseded** by the user's explicit choice of **Svelte 5 / SvelteKit** for
  this repo (`elite-viz`).

---

## Reconciliation with the user's direct instructions

The pasted summaries describe a Rust + Axum + Next.js + Postgres platform. The
user's direct instructions for **this** repo take precedence where they differ:

1. **Frontend:** Svelte 5 / SvelteKit (not Next.js). Svelte MCP server is
   available for doc-accurate components.
2. **Backend:** Rust (Axum) — confirmed by the spec.
3. **Tooling:** pnpm, latest package versions as of 2026-05-30.
4. **Process:** plan first, then build.
