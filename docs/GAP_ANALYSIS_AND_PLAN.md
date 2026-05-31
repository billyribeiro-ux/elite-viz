# elite-viz vs FINVIZ — Gap Analysis & Implementation Plan

Compiled 2026-05-31. Measures the repo against
[`FINVIZ_FEATURE_REFERENCE.md`](./FINVIZ_FEATURE_REFERENCE.md). Nothing is
dropped — every gap is captured as a tracked wave below.

## Legend
✅ done · 🟡 partial · ❌ missing

---

## Scorecard

| Area | FINVIZ | elite-viz today | Status |
|------|--------|-----------------|--------|
| Screener filter DSL | 70+ filters, 3 categories | hand-written lexer/parser/eval + SQL compiler; **14 fields** | 🟡 |
| Screener signals (1-click) | ~20 prebuilt | 4 presets (read-only) | 🟡 |
| Screener views/tabs | 11 (Overview…TA) | 1 table view | 🟡 |
| Saved screens | save + 200 presets | presets only, no save | 🟡 |
| CSV/Excel export | yes | ❌ | ❌ |
| Charts | candles, intraday, studies, patterns | SVG line + SMA overlay | 🟡 |
| Indicators | SMA/EMA/RSI/MACD/BBands/Stoch/ATR | SMA, RSI | 🟡 |
| Pattern recognition | channels/wedges/triangles/H&S | ❌ | ❌ |
| Backtesting | 20y, win-rate, drawdown, Sharpe/Calmar | ❌ | ❌ |
| Maps / heatmaps | sector/theme/52w/drawdown | ❌ | ❌ |
| Groups | sector/industry/country/cap aggregation | ❌ | ❌ |
| Portfolios | 100×500, P&L, alerts | 1 in-memory, P&L ✅ | 🟡 |
| Watchlists | many | CRUD ✅ | ✅ |
| Alerts | price/news/ratings/insider/SEC/screener-match | symbol+expression + check | 🟡 |
| News | aggregated + per-ticker | ❌ | ❌ |
| Quote detail | quote+chart+insider+ratings+news+financials | quote+chart+stats | 🟡 |
| Insider trading | tables + map | ❌ | ❌ |
| Analyst ratings | tables | ❌ | ❌ |
| Options chain | view + export | ❌ | ❌ |
| ETF analysis | holdings + treemap | ❌ | ❌ |
| Futures/Forex/Crypto | prices + heatmaps | ❌ | ❌ |
| Real-time/premarket/AH | yes | jittered WS + pluggable providers | 🟡 |
| Auth (JWT) | account | ✅ | ✅ |
| Export API / tokens | yes | provider settings | 🟡 |
| Ad-free / layouts / deep links | yes | n/a / partial | 🟡 |

---

## Guiding constraints
- **Synthetic-data-first**: every feature must work on the bundled seed dataset
  with zero external keys, and light up further when a live provider is set.
- **Green gates always**: cargo build/fmt/clippy(-D warnings)/test + svelte-check
  (0/0) + pnpm build, after every wave.
- **Race-safe orchestration** (lesson learned): shared files
  (`finviz-types/lib.rs`, `core/state.rs`+`seed.rs`, `screener/lib.rs`,
  `routes/mod.rs`, `+layout.svelte`, `lib/api.ts`, `lib/types.ts`) are edited by
  the orchestrator at **wave boundaries only**. Parallel sub-agents are given
  **disjoint, mostly-additive** file sets (new route modules, new pages).

---

## Waves (ordered; each ends green & committed)

### Wave 1 — Data-model expansion (FOUNDATION) ⬅ unlocks everything
Expand `Fundamentals` + `ScreenerRow` and the seed to the full FINVIZ metric
set, and register them as screener fields.
- New fields: `forward_pe, peg, ps, pb, price_to_fcf, roa, roe, roic,
  current_ratio, quick_ratio, debt_equity, lt_debt_equity, gross_margin,
  oper_margin, profit_margin, payout_ratio, insider_own, inst_own, short_float,
  short_ratio, float_shares, avg_volume, rel_volume, target_price,
  perf_week, perf_month, perf_quarter, perf_half, perf_year, perf_ytd,
  volatility_w, volatility_m, rsi14, sma20_rel, sma50_rel, sma200_rel,
  high_52w_pct, low_52w_pct, atr, country, recom`.
- Update: `finviz-types`, `core/seed.rs` (realistic values for 26 tickers),
  `core/state.rs` (`ScreenerRow` build), `screener/lib.rs` (`Row` impl +
  `known_fields` + `canonical_field` aliases), tests.
- Frontend: screener **column views** (Overview/Valuation/Financial/Performance/
  Technical) + more preset signals.
**Owner: orchestrator (solo, shared files).**

### Wave 2 — Groups + Maps/Heatmap  ✅ DONE
- **R-Groups**: `routes/groups.rs` → aggregate `screener_rows` by
  sector/industry/country/cap (count, avg change, avg P/E, total mkt cap, perf).
  Frontend `/groups` page (sortable group table + bar viz).
- **R-Maps**: frontend `/map` heatmap page (treemap by sector, sized by mkt cap,
  colored by perf window) consuming existing screener rows; selectable window.
**Owners: 2 disjoint agents; orchestrator wires mod.rs/nav/api.**

### Wave 3 — Charts upgrade + more indicators  ✅ DONE
- **R-Indicators**: add EMA, MACD, Bollinger Bands, Stochastics, ATR endpoints
  (extend `indicators.rs` — single owner).
- **S-Charts**: candlestick `Chart` mode + indicator overlays + timeframe
  selector on the symbol page.
**Owners: 2 disjoint agents.**

### Wave 4 — Backtesting engine  [headline feature]
- `finviz-backtest` crate (or `backtest.rs`): run an indicator/screener rule over
  bar history; TIME-EXIT + STOP-LOSS; metrics: avg return, win rate, max
  drawdown, Sharpe, Calmar, equity curve, trade list. Pure + unit-tested.
- `routes/backtest.rs` + frontend `/backtest` page (rule builder, equity curve,
  metrics cards, trades table).

### Wave 5 — News + Quote-detail enrichment
- Synthetic news generator in core; `routes/news.rs` (market + per-ticker).
- Insider-trading + analyst-ratings synthetic tables; expose on quote detail.
- Frontend `/news` page + enrich `/symbol/[ticker]` (news, insider, ratings,
  financials tabs).

### Wave 6 — CSV export + saved screens + screener notifications
- Export endpoints (screener/groups/portfolio → CSV) + frontend download.
- Persist user-saved screens (in-memory store + CRUD + UI).
- Screener-match notifications surfaced in UI (poll `/alerts/check` model).

### Wave 7 — Options chain + ETF analysis
- Synthetic options chain (`routes/options.rs`) + `/symbol` options tab.
- ETF holdings + treemap (`routes/etf.rs`) + `/etf/[symbol]` page.

### Wave 8 — Futures / Forex / Crypto boards
- Synthetic boards + heatmaps + pages.

### Wave 9 — Pattern recognition
- Detect channels/wedges/triangles/H&S/double-top from bar series; annotate
  charts; add as screener signals.

### Wave 10 — Persistence cutover + polish
- Wire `finviz-db` Postgres store behind a `Store` trait (currently in-memory);
  seed loader; deep-link URL state for screens/maps; layout prefs.

---

## Execution note
Waves 1–3 are scheduled for immediate execution this session (foundation +
first parallel waves), each verified and committed independently. Waves 4–10 are
fully specified here so they are tracked and nothing is lost; they proceed in
subsequent sessions on the same green-gated, race-safe cadence.
