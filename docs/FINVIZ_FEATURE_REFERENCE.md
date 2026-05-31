# FINVIZ / FINVIZ Elite — Complete Feature Reference

> Compiled 2026-05-31 from FINVIZ's official help/elite pages and corroborating
> third-party reviews (sources at the bottom). This is the **target feature
> surface** we measure `elite-viz` against. Items marked **[Elite]** are paid-tier.

---

## 1. Stock Screener

### 1.1 Filter categories (~70 filters over 8,000+ US stocks & ETFs)

**Descriptive**
- Exchange (NASDAQ / NYSE / AMEX)
- Index membership (S&P 500, DJIA)
- Sector, Industry (with "all but" exclusions)
- Country / region
- Market Cap (mega → nano, with custom ranges)
- Dividend Yield
- Float / Shares Outstanding
- Average Volume, Relative Volume, Current Volume
- Price (ranges + custom)
- Target Price
- IPO Date
- Earnings Date (today/tomorrow/this week/next week/prev week/this month…)
- Average True Range (ATR)
- Optionable / Shortable
- Analyst Recommendation (1.0 Strong Buy → 5.0 Strong Sell)

**Fundamental**
- P/E, Forward P/E, PEG
- P/S, P/B, Price/Cash, Price/Free Cash Flow
- EPS (ttm), EPS growth this/next year, EPS growth past 5y, EPS growth qtr-over-qtr, EPS growth next 5y
- Sales growth past 5y, Sales growth qtr-over-qtr
- Dividend Yield
- ROA, ROE, ROIC
- Current Ratio, Quick Ratio, LT Debt/Equity, Debt/Equity
- Gross Margin, Operating Margin, Net Profit Margin
- Payout Ratio
- Insider Ownership, Insider Transactions
- Institutional Ownership, Institutional Transactions
- Short Float / Short Ratio

**Technical**
- Performance (multiple windows: 1d, week, month, quarter, half-y, year, YTD)
- Performance 2 (second window for cross-filtering)
- Volatility (week / month)
- RSI(14) ranges (overbought/oversold bands)
- Gap (up/down %)
- 20/50/200-day SMA relationships (price above/below, SMA crossovers)
- 52-Week High/Low proximity
- All-time High/Low proximity
- Change, Change from Open
- Candlestick patterns (doji, hammer, engulfing, etc.)
- Chart patterns (channel up/down, wedge, triangle, head & shoulders, double top/bottom) sorted by strength

### 1.2 Screener "Signals" (one-click prebuilt scans)
Top Gainers, Top Losers, New High, New Low, Most Volatile, Most Active,
Unusual Volume, Overbought, Oversold, Most Shorted, Insider Buying/Selling,
Analyst Upgrades/Downgrades, Earnings Before/After, Major News, Channel/Wedge/
Triangle/Head&Shoulders/Double-Top patterns.

### 1.3 Screener views / display tabs
Overview · Valuation · Financial · Ownership · Performance · Technical ·
Custom (pick columns) · Charts (mini-chart grid) · Basic (snapshot + chart) ·
News · Snapshot · TA · Tickers (compact list).

### 1.4 Screener UX
- **Saved screens / Presets** (200 preset slots **[Elite]**).
- Sortable, exportable columns (CSV/Excel **[Elite]**).
- Up to **100 rows/page [Elite]** (20 free).
- Pagination, column sorting, row density.
- Shareable screen URLs encoding the filter state.
- Screener **notifications** when new tickers match **[Elite]**.

---

## 2. Charts & Technical Analysis

- Interactive candlestick/line/area charts.
- **[Elite]** Intraday timeframes: 1m, 2m, 3m, 5m, 15m, 30m, 1h, 4h + daily/weekly/monthly.
- Technical indicators/overlays: SMA/EMA (multiple), Bollinger Bands, RSI, MACD,
  Stochastics, ATR, volume, and more — user-selectable studies.
- **Automatic pattern recognition** drawn on the chart (channels, wedges,
  triangles, head & shoulders, support/resistance trendlines).
- **Fundamental charts**: EPS, sales, shares outstanding over time.
- Saved chart layouts / preferences **[Elite]**.
- Multi-chart layouts.
- Drawing tools (trendlines, etc.).

---

## 3. Backtesting **[Elite]**

- Backtest technical-indicator strategies over universes: S&P 500, Russell
  1000 / 2000 / 3000, or a single stock.
- Multiple indicators combined with AND (all conditions true simultaneously).
- Entry conditions + **EXIT conditions**: TIME-EXIT (holding period) and STOP-LOSS.
- Up to **20 years** of historical data.
- Custom holding periods.
- Performance metrics: average return, **win rate**, **maximum drawdown**,
  Calmar ratio (annualized return / max drawdown), Sharpe ratio
  (annualized return / volatility), equity curve, trade list.

---

## 4. Maps / Heatmaps

- **S&P 500 map**: tiles grouped by sector, sized by market cap, colored by %
  change over a selectable window (1d → 1y).
- Variants: Dow Jones, Nasdaq 100, Russell 2000, World, ETFs, full US market.
- **Market Themes heatmap** (structural themes, not sectors).
- **52-Week High / Low maps**, **Drawdown-from-52w-High map**, **Insider-trading map**.
- Hover tooltips; click-through to ticker.

---

## 5. Groups
- Aggregated performance by **Sector / Industry / Country / Capitalization**.
- Group views: Overview, Valuation, Performance, etc. (same column families).
- Charts per group; sortable; exportable **[Elite]**.

---

## 6. Portfolios & Watchlists **[Elite tiers]**
- Up to **100 portfolios**, **500 tickers** each.
- Position tracking with P&L.
- Portfolio-based **alerts**.
- Custom layouts.
- Import/export.

---

## 7. Alerts & Notifications **[Elite]**
- Unlimited **email alerts + push notifications**.
- Price-movement alerts (above/below threshold, % change).
- News, analyst-ratings, insider-trading, SEC-filing alerts.
- **Screener-match** alerts and **portfolio** alerts.

---

## 8. News
- Aggregated headlines (Stocks / ETFs / Crypto) from many publishers.
- Per-ticker news stream.
- Blogs feed.
- Filterable; export **[Elite]**.

---

## 9. Quote / Ticker detail page
- Real-time **[Elite]** quote, premarket & after-hours.
- Fundamental snapshot table (~70 metrics).
- Interactive chart.
- Insider-trading table.
- Analyst ratings table.
- News stream.
- Income statement / financials (up to **8 years [Elite]**).
- Related/peer tickers; **correlated stocks [Elite]**.
- Options chain (export **[Elite]**).

---

## 10. ETF analysis **[Elite]**
- Full holdings breakdown (beyond top 10).
- Performance + structural data.
- Tree-map of holdings.

---

## 11. Futures / Forex / Crypto
- Futures prices + performance heatmap.
- Forex performance grid.
- Crypto prices.

---

## 12. Data export & API **[Elite]**
- Export Screener / Portfolios / Groups / Options Chain / News to CSV/Excel.
- **API access** with auth token + sample code (Google Sheets, Python, JS).

---

## 13. Platform / UX
- Ad-free **[Elite]**.
- Real-time data (15-min delayed on free).
- Customizable homepage & layouts.
- Shareable deep-link URLs for screens/charts/maps.
- Responsive web UI.

---

## Sources
- [Finviz Elite](https://finviz.com/elite)
- [Help — Screener](https://finviz.com/help/screener)
- [Help — Backtests](https://finviz.com/help/technical-analysis/backtests.ashx)
- [S&P 500 Map](https://finviz.com/map) · [Groups](https://finviz.com/groups) · [Futures](https://finviz.com/futures)
- [New Stock Market Maps (blog)](https://finviz.com/blog/new-stock-market-maps-for-market-cap-52-week-highs-lows-themes-and-insider-trading/)
- [Evolving the Heatmap (blog)](https://finviz.com/blog/evolving-the-heatmap-dow-jones-nasdaq-100-russell-2000-and-more/)
- Reviews: [liberatedstocktrader](https://www.liberatedstocktrader.com/finviz-review/), [stockbrokers.com](https://www.stockbrokers.com/review/tools/finviz), [traderhq](https://traderhq.com/finviz-elite-review-best-stock-screener-tool/), [stocksoftresearch — backtesting](https://stocksoftresearch.com/backtesting-with-finviz-elite/)
