# FINVIZ Elite+ (`elite-viz`)

A stock-screener / market-data visualization platform: a **Rust + Axum**
backend serving a **SvelteKit (Svelte 5)** dashboard.

> Status: **complete & running end-to-end** on a deterministic synthetic dataset
> — no credentials required. A 10-wave FINVIZ-parity build delivered 13 pages
> (screener, groups, map heatmap, markets, backtest, news, portfolio,
> watchlists, alerts, settings, login, symbol detail, ETF) over an 8-crate
> Rust + Axum workspace (~40 REST endpoints + 2 WebSocket streams), with JWT
> auth, CSV export, and a feature-gated Postgres layer. CI runs fmt, clippy
> `-D warnings`, tests, and the SvelteKit check/build on every push.
>
> ### ▶ The only step left to go live: **[`docs/GO_LIVE.md`](./docs/GO_LIVE.md)**
> Paste your market-data provider's API key (Finnhub / Polygon.io) or a
> webhook URL (Generic) into **Settings → Data Provider**, hit Test, and Save.
>
> See [`docs/GAP_ANALYSIS_AND_PLAN.md`](./docs/GAP_ANALYSIS_AND_PLAN.md) for the
> full FINVIZ feature scorecard and the wave-by-wave build log.

## What works today

- **Screener** — filter DSL (`price > 100 and pe < 30 and sector = "Technology"`,
  with `and`/`or`/`not`, parentheses, comparison + `~` substring) over **56
  fields** (descriptive, valuation, profitability, financial health, ownership,
  performance, technical); presets/signals, column sorting, **live mode** (WS),
  and **shareable deep-link URLs**. Hand-written lexer → parser → evaluator plus
  a parameterized SQL compiler for the Postgres path.
- **Groups & Map** — sector/industry/country aggregation and a FINVIZ-style
  heatmap (tile size ∝ market cap, color ∝ performance window).
- **Charts & indicators** — price chart with SMA/EMA/Bollinger overlays + RSI
  panel; SMA, EMA, RSI, MACD, Bollinger Bands, ATR endpoints.
- **Backtesting** — rule-based strategies (SMA cross / price-vs-SMA / RSI) with
  time-exit + stop-loss; metrics: total/avg return, win rate, max drawdown,
  Sharpe, Calmar; equity curve + trades table.
- **Symbol detail** — quote + chart + **detected chart patterns** + tabs for
  News / Insider / Ratings / Options chain; **live price** over WebSocket.
- **Markets** — Futures / Forex / Crypto boards with heat-strips.
- **News / Insider / Ratings** — aggregated market + per-ticker feeds.
- **Portfolio & watchlists** — valuation with unrealized P&L; CRUD.
- **Alerts** — screener-expression alerts with triggered evaluation.
- **Auth** — JWT register/login/me/refresh (Argon2).
- **Export** — CSV for screener / groups / portfolio; **saved screens**.
- **Pluggable data provider** — Settings screen to enter a provider API key
  (Finnhub / Polygon.io) or webhook URL (Generic), test, and go live. Keys are
  write-only (masked) and the platform falls back to demo data on any live error.
- **Persistence** — `finviz-db` crate + SQL migrations behind the `postgres`
  feature, materializing the full 56-field surface; default build stays
  in-memory and dependency-free.

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

Requires Rust 1.94+ (stable) and Node 24.16.0 LTS (see `.nvmrc`) with **pnpm 11+**.

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
| GET/POST | `/api/v1/alerts` | List / create price alerts |
| DELETE | `/api/v1/alerts/{id}` | Delete an alert |
| GET  | `/api/v1/alerts/check` | Evaluate alerts against live data |
| POST | `/api/v1/auth/register` · `/login` | Get a JWT |
| GET  | `/api/v1/auth/me` · POST `/refresh` | Current user / refresh token |
| GET/PUT | `/api/v1/settings/provider` | Read / update data-provider config |
| POST | `/api/v1/settings/provider/test` | Test provider connectivity |
| WS   | `/ws/quotes?symbols=AAPL,MSFT` | Realtime quote ticks |
| WS   | `/ws/screener-updates?query=...` | Live screener results |

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
