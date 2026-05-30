# FINVIZ Elite+ (`elite-viz`)

A stock-screener / market-data visualization platform: a **Rust + Axum**
backend serving a **SvelteKit (Svelte 5)** dashboard.

> Status: **Phase 1–2 complete** — a working vertical slice (screener engine,
> REST API, and dashboard) running on a synthetic in-memory dataset. See
> [`PLAN.md`](./PLAN.md) for the roadmap and [`docs/`](./docs) for the spec
> overview this was built from.

## What works today

- **Filter DSL** — `price > 100 and pe < 30 and sector = "Technology"` with
  `and` / `or` / `not`, parentheses, comparison + substring (`~`) operators.
  Compiled by a hand-written lexer → parser → evaluator (and a parameterized
  SQL compiler for the upcoming Postgres path).
- **REST API** — market-data (quotes, bars, fundamentals, instruments),
  screener (run / fields / presets), and indicators (SMA, RSI), with CORS,
  request tracing, and graceful shutdown.
- **Dashboard** — filter bar with one-click presets, sortable results table,
  trend coloring; server-rendered then interactive. Talks to the backend
  through a same-origin SvelteKit proxy.

## Architecture

```
SvelteKit (Svelte 5)  ──/api/v1 proxy──▶  Rust + Axum (finviz-api)
   src/                                      backend/crates/
   routes/, lib/components/                    finviz-types     domain models
                                               finviz-screener  filter DSL engine
                                               finviz-core      config + seeded store
                                               finviz-api       Axum server
```

The backend ships a **synthetic seed dataset**, so the whole stack runs with no
database and no market-data API keys. PostgreSQL + live data adapters land in
later phases (see `PLAN.md`).

## Running locally

Requires Rust (stable) and Node 22+ with **pnpm**.

```bash
# 1. Backend — http://localhost:8080
pnpm backend:dev          # cargo run -p finviz-api

# 2. Frontend — http://localhost:5173
pnpm install
pnpm dev
```

Configuration is via environment variables (see [`.env.example`](./.env.example)):
`BACKEND_URL` (frontend → backend), `BIND_ADDR` and `CORS_ORIGIN` (backend).

## Quality checks

```bash
cargo test --manifest-path backend/Cargo.toml   # screener engine unit tests
cargo clippy --manifest-path backend/Cargo.toml --workspace
pnpm run check                                   # svelte-check
pnpm run build
```

## API quick reference

| Method | Path | Description |
|--------|------|-------------|
| GET  | `/healthz` | Liveness probe |
| GET  | `/api/v1/market-data/instruments` | All instruments |
| GET  | `/api/v1/market-data/quote/{symbol}` | Latest quote |
| GET  | `/api/v1/market-data/fundamentals/{symbol}` | Fundamentals |
| GET  | `/api/v1/market-data/bars/{symbol}?interval=1d&limit=120` | OHLCV bars |
| POST | `/api/v1/screener/run` | Run a filter query |
| GET  | `/api/v1/screener/fields` | Filterable fields |
| GET  | `/api/v1/screener/presets` | Example screens |
| GET  | `/api/v1/indicators/sma/{symbol}?period=20` | Simple moving average |
| GET  | `/api/v1/indicators/rsi/{symbol}?period=14` | Relative strength index |

Example:

```bash
curl -X POST localhost:8080/api/v1/screener/run \
  -H 'content-type: application/json' \
  -d '{"query":"market_cap > 1e12 and sector = \"Technology\"","sort":"market_cap","order":"desc"}'
```
