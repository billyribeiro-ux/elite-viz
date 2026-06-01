# Go Live — Connect Your Market-Data Provider

Everything in FINVIZ Elite+ is **built and running** on a deterministic synthetic
dataset (zero credentials required). The **only** step left to switch from demo
data to live quotes is pasting your provider's API key (or a webhook/endpoint URL)
into the Settings screen.

---

## 1. Start the app

```bash
# Terminal 1 — Rust API (http://localhost:8080)
pnpm backend:dev

# Terminal 2 — SvelteKit UI (http://localhost:5173)
pnpm install   # first run only
pnpm dev
```

Open http://localhost:5173 — the whole platform works immediately on demo data.

## 2. Add your data provider (the one remaining step)

1. Go to **Settings** (top nav) → **Data Provider**.
2. Pick your provider:
   - **Finnhub** — paste your API token. (Uses the `/quote` endpoint.)
   - **Polygon.io** — paste your API key. (Uses the stock snapshot endpoint.)
   - **Generic HTTP / Webhook** — paste your endpoint URL (e.g.
     `https://my-feed.example.com/quote`). The server calls
     `GET {url}?symbol=SYM` with an optional `Authorization: Bearer <key>` and
     accepts flexible JSON (`price`/`last`/`c`, `prev_close`/`pc`, `volume`, …).
3. Tick **“Use this provider for live quotes.”**
4. Click **Test connection** → you should see *“Connected to … provider.”*
5. Click **Save**.

That's it. Quote endpoints now serve live data, falling back to the demo dataset
automatically if a live call fails. Your key is **write-only** — it's masked
(`••••1234`) after saving and never returned in full by the API.

> The provider config lives in the server's in-memory state, so it resets when
> the API process restarts — re-enter it in Settings after a restart. (Persisting
> it to the `postgres` store is a small documented follow-up.)

---

## What's already done (no action needed)

- **13 pages**: Screener (56 filterable fields, live mode, deep-links), Groups,
  Map heatmap, Markets (futures/forex/crypto), Backtest, News, Portfolio,
  Watchlists, Alerts, Settings, Login, Symbol detail (chart + indicators +
  patterns + news/insider/ratings/options tabs), ETF.
- **Rust + Axum** backend: 8 crates, ~40 endpoints + 2 WebSocket streams,
  JWT auth, CSV export, optional Postgres persistence (`postgres` feature).
- Synthetic data is deterministic, so demos are reproducible.

## Provider notes

| Provider | Needs | What lights up today |
|----------|-------|----------------------|
| Mock (default) | nothing | Everything, on synthetic data |
| Finnhub | API token | Live **quotes** (price/change/day range) |
| Polygon.io | API key | Live **quotes** via snapshot |
| Generic / Webhook | endpoint URL (+ optional bearer) | Live **quotes** from your feed |

> Live wiring currently covers **quotes**. Bars/history, news, and options still
> render from the synthetic generators regardless of provider — extending the
> live adapters to those is the documented next step in
> `docs/GAP_ANALYSIS_AND_PLAN.md`.
