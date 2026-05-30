# FINVIZ Elite+ (`elite-viz`) — Implementation Plan

Status: **active build**. Last updated 2026-05-30.

## 0. Context & constraints

- The full 1,953-line source spec (`FINVIZ_ELITE_RUST_COMPLETE.md`) lives in an
  external Claude.ai sandbox and is **not reachable** from this repo's
  container (verified: external path, Google Drive, and the claude.ai chat URL
  all denied/empty). We therefore build from the pasted **overview**
  (`docs/FINVIZ_ELITE_RUST_COMPLETE.md`) and adjust to the user's direct
  instructions where they differ.
- Frontend = **Svelte 5 / SvelteKit** (user choice; supersedes the summary's
  "Next.js"). Backend = **Rust + Axum** (per spec).
- Package manager = **pnpm**; npm packages = **latest** as of 2026-05-30.
- Goal at each phase: code that actually **compiles and runs**, not pseudocode.

## 1. Architecture

```
elite-viz/
├── docs/                     # saved spec overview + plan
├── backend/                  # Rust + Axum (Cargo workspace)
│   ├── Cargo.toml            # [workspace]
│   └── crates/
│       ├── finviz-types/     # domain models (Quote, Bar, Instrument, …)
│       ├── finviz-screener/  # filter DSL: lexer → parser → evaluator (+SQL compile)
│       ├── finviz-core/      # config, error, AppState, in-memory seeded store
│       └── finviz-api/       # Axum server: routers, middleware, main.rs
│   └── migrations/           # PostgreSQL schema (sqlx migrate) — Phase 2
├── src/                      # SvelteKit (Svelte 5 runes)
│   ├── lib/                  # api client, types, components
│   └── routes/               # screener dashboard, market data, watchlist
├── docker-compose.yml        # Postgres + Redis for local dev — Phase 2
└── README.md
```

Runtime: SvelteKit talks to the Rust API over HTTP (`BACKEND_URL`,
default `http://localhost:8080`) and WebSocket for realtime. The backend runs
with an **in-memory seeded dataset by default** (no DB/API keys needed), and
swaps to Postgres + live adapters when configured.

## 2. API surface (from the overview)

```
/healthz                       liveness/readiness
/api/v1/market-data/*          quotes, bars (1m/5m/1h/1d), fundamentals
/api/v1/screener/*             run query, list/save presets
/api/v1/indicators/*           SMA/EMA/RSI/etc.
/api/v1/watchlists/*           CRUD
/api/v1/portfolio/*            positions, P&L
/api/v1/auth/*                 JWT login/register/refresh
/ws/quotes, /ws/screener-updates   realtime streams
```

## 3. Screener DSL (the core differentiator)

Filter expressions like:

```
price > 50 and pe < 25 and sector = "Technology" and (volume > 1e6 or marketcap > 1e10)
```

Pipeline: **lexer** (tokens) → **Pratt parser** (AST) → **evaluator** (against
in-memory rows) and a parallel **SQL compiler** (parameterized WHERE clause)
for the Postgres path. Fully unit-tested.

## 4. Phases (each ends compiling + runnable)

- **Phase 1 — Backend vertical slice** ✅ *done*
  Workspace + types + error + seeded in-memory store + screener
  lexer/parser/evaluator (tests) + Axum server (`/healthz`, market-data,
  screener, indicators). CORS, tracing, graceful shutdown.
- **Phase 2 — Frontend vertical slice** ✅ *done*
  SvelteKit (pnpm, latest, Svelte 5 runes): screener dashboard + symbol detail
  with SVG charts/SMA, portfolio, watchlists; same-origin proxy to the API.
- **Phase 3 — Persistence** 🟡 *scaffolded*
  PostgreSQL schema + migrations and a feature-gated `finviz-db` repository
  exist and compile; the SQL-compiler path is ready. **Not yet wired** into the
  running server (still in-memory) — next: a `Store` trait + DB seed loader.
- **Phase 4 — Realtime** ✅ *done* — `/ws/quotes` (live symbol page) and
  `/ws/screener-updates` (live screener results) both streaming.
- **Phase 5 — Auth & hardening** 🟡 *partial* — JWT register/login/me/refresh
  with Argon2 hashing and a bearer extractor; price alerts on the screener DSL.
  Rate limiting and per-user data scoping still to do.
- **Phase 6 — Data adapters** 🟡 *partial* — runtime-configurable providers
  (Finnhub, Polygon.io, generic HTTP/webhook) with a Settings UI, live-quote
  fetch with fallback, and write-only API keys. Bars/fundamentals adapters and
  WebSocket upstreams still to do.
- **Phase 7 — Ops** 🟡 *partial* — Dockerfiles, docker-compose, and CI
  (fmt/clippy/test/check/build) done; k8s manifests still to do.

## 5. Decisions / deviations (flag for review)

1. **Next.js → SvelteKit** per your instruction.
2. **No sqlx compile-time macros in Phase 1** — keeps the backend building with
   no database present; DB lands in Phase 3 with runtime-checked queries +
   offline metadata for CI.
3. **Mock data provider is the default** so the stack runs with zero secrets.
4. Single workspace, multiple small crates (not the summary's "7 crates" exactly
   — we add crates only as they earn their place).

If anything here conflicts with the full spec, say so and I'll realign.
