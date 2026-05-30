# FINVIZ Elite+ (`elite-viz`)

A stock-screener / market-data visualization platform: a **Rust + Axum**
backend serving a **SvelteKit (Svelte 5)** dashboard.

> Status: **working end-to-end** — screener engine, full REST API, realtime
> WebSocket quotes, and a multi-page dashboard (screener, symbol detail with
> charts, portfolio, watchlists) on a synthetic in-memory dataset, plus a
> feature-gated Postgres layer, Docker, and CI. See [`PLAN.md`](./PLAN.md) for
> the roadmap and [`docs/`](./docs) for the spec overview this was built from.

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
- **Symbol detail** — OHLCV price chart with SMA overlay, key stats, and a
  **live price** streamed over WebSocket.
- **Portfolio & watchlists** — valuation with unrealized P&L; watchlist
  create/delete.
- **Pluggable data provider** — a Settings screen where you pick a provider
  (Finnhub, Polygon.io, or a generic HTTP/webhook endpoint), enter your API key
  or URL, test the connection, and go live. The API key is write-only (masked
  after saving) and the platform falls back to demo data if a live call fails.
- **Persistence (scaffolded)** — `finviz-db` crate + SQL migrations behind the
  `postgres` feature; the server still defaults to the in-memory store.

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
| GET/POST | `/api/v1/watchlists` | List / create watchlists |
| GET/PUT/DELETE | `/api/v1/watchlists/{id}` | Read / update / delete |
| GET/POST | `/api/v1/portfolio/positions` | List / upsert positions |
| DELETE | `/api/v1/portfolio/positions/{symbol}` | Remove a position |
| GET  | `/api/v1/portfolio/summary` | Valuation + unrealized P&L |
| GET/PUT | `/api/v1/settings/provider` | Read / update data-provider config |
| POST | `/api/v1/settings/provider/test` | Test provider connectivity |
| WS   | `/ws/quotes?symbols=AAPL,MSFT` | Realtime quote ticks |

Example:

```bash
curl -X POST localhost:8080/api/v1/screener/run \
  -H 'content-type: application/json' \
  -d '{"query":"market_cap > 1e12 and sector = \"Technology\"","sort":"market_cap","order":"desc"}'
```

## Docker

```bash
docker compose up --build   # web :3000, api :8080, postgres :5432, redis :6379
```

The `web` and `backend` services build from their respective Dockerfiles;
`db`/`redis` back the Postgres path. CI (GitHub Actions) runs rustfmt, clippy
(`-D warnings`), `cargo test`, the `postgres`-feature check, and
`pnpm check` + `pnpm build` on every push/PR.
