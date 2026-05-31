//! Core domain models for the FINVIZ Elite+ platform.
//!
//! These types are deliberately storage-agnostic: they are shared by the
//! in-memory dataset, the screener engine, the HTTP layer, and (in a later
//! phase) the PostgreSQL repositories.

use serde::{Deserialize, Serialize};

/// A ticker symbol, e.g. `AAPL`.
pub type Symbol = String;

/// Static, slow-changing reference data for a tradeable instrument.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instrument {
    pub symbol: Symbol,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub exchange: String,
}

/// A point-in-time market quote.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quote {
    pub symbol: Symbol,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: i64,
    pub prev_close: f64,
    pub day_high: f64,
    pub day_low: f64,
    /// Quote timestamp, Unix epoch seconds.
    pub ts: i64,
}

/// Fundamental metrics used heavily by the screener.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fundamentals {
    pub symbol: Symbol,
    pub market_cap: f64,
    pub pe: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub beta: Option<f64>,
    pub shares_outstanding: f64,
}

/// Candle resolution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Interval {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "1d")]
    D1,
}

impl Interval {
    /// Number of seconds covered by one bar of this interval.
    pub fn seconds(self) -> i64 {
        match self {
            Interval::M1 => 60,
            Interval::M5 => 300,
            Interval::H1 => 3_600,
            Interval::D1 => 86_400,
        }
    }
}

impl std::str::FromStr for Interval {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Interval::M1),
            "5m" => Ok(Interval::M5),
            "1h" => Ok(Interval::H1),
            "1d" => Ok(Interval::D1),
            other => Err(format!("unknown interval: {other}")),
        }
    }
}

/// An OHLCV candle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    /// Bar open time, Unix epoch seconds.
    pub ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

/// A fully merged screener row: everything the engine and UI need about one
/// symbol in a single flat record. The full FINVIZ-style metric surface the
/// screener DSL can filter on (fundamentals, technicals, descriptive fields).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerRow {
    // ---- Identity / descriptive ----
    pub symbol: Symbol,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub exchange: String,
    /// Country of domicile, e.g. `USA`.
    pub country: String,
    /// Mean analyst 12-month price target (absolute price), if covered.
    pub target_price: Option<f64>,
    /// 3-month average daily share volume.
    pub avg_volume: f64,
    /// Today's volume relative to `avg_volume` (1.0 == average).
    pub rel_volume: f64,
    /// Free-floating shares available to trade.
    pub float_shares: f64,
    /// Mean analyst recommendation, 1 (strong buy) .. 5 (strong sell).
    pub recom: Option<f64>,

    // ---- Market / quote ----
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: i64,

    // ---- Valuation ----
    pub market_cap: f64,
    pub pe: Option<f64>,
    pub forward_pe: Option<f64>,
    pub peg: Option<f64>,
    /// Price/Sales.
    pub ps: Option<f64>,
    /// Price/Book.
    pub pb: Option<f64>,
    /// Price/Free-cash-flow.
    pub price_to_fcf: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub beta: Option<f64>,

    // ---- Profitability (percentages) ----
    pub roa: Option<f64>,
    pub roe: Option<f64>,
    pub roic: Option<f64>,
    pub gross_margin: Option<f64>,
    pub oper_margin: Option<f64>,
    pub profit_margin: Option<f64>,
    pub payout_ratio: Option<f64>,

    // ---- Financial health ----
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub debt_equity: Option<f64>,
    pub lt_debt_equity: Option<f64>,

    // ---- Ownership (percentages) ----
    pub insider_own: Option<f64>,
    pub inst_own: Option<f64>,
    pub short_float: Option<f64>,
    pub short_ratio: Option<f64>,

    // ---- Performance (percentages) ----
    pub perf_week: f64,
    pub perf_month: f64,
    pub perf_quarter: f64,
    pub perf_half: f64,
    pub perf_year: f64,
    pub perf_ytd: f64,

    // ---- Technical ----
    /// Weekly volatility, percent.
    pub volatility_w: f64,
    /// Monthly volatility, percent.
    pub volatility_m: f64,
    /// 14-period Relative Strength Index, 0..100.
    pub rsi14: f64,
    /// Average True Range (absolute price units).
    pub atr: f64,
    /// Percent distance of price above/below its 20-day SMA.
    pub sma20_rel: f64,
    /// Percent distance of price above/below its 50-day SMA.
    pub sma50_rel: f64,
    /// Percent distance of price above/below its 200-day SMA.
    pub sma200_rel: f64,
    /// Percent from the 52-week high (<= 0).
    pub high_52w_pct: f64,
    /// Percent from the 52-week low (>= 0).
    pub low_52w_pct: f64,
}

/// A named collection of symbols a user is tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Watchlist {
    pub id: String,
    pub name: String,
    pub symbols: Vec<Symbol>,
}

/// A user-saved screener query for quick re-running.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedScreen {
    pub id: String,
    pub name: String,
    /// Filter DSL, e.g. `price > 100 and pe < 30`.
    pub query: String,
    /// Optional field to sort by.
    #[serde(default)]
    pub sort: Option<String>,
    /// Optional sort order, `"asc"` or `"desc"`.
    #[serde(default)]
    pub order: Option<String>,
}

/// A single open position in a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub symbol: Symbol,
    pub quantity: f64,
    /// Average cost basis per share.
    pub avg_price: f64,
}

/// A position enriched with current market value and unrealized P&L.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionValue {
    pub symbol: Symbol,
    pub quantity: f64,
    pub avg_price: f64,
    pub last_price: f64,
    pub market_value: f64,
    pub cost_basis: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
}

/// Aggregate portfolio valuation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioSummary {
    pub positions: Vec<PositionValue>,
    pub market_value: f64,
    pub cost_basis: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
}

/// A streamed quote update (sent over the `/ws/quotes` WebSocket).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuoteTick {
    pub symbol: Symbol,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub ts: i64,
}

/// Which upstream market-data provider to use for live quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// Built-in synthetic dataset (no network). Default.
    #[default]
    Mock,
    /// Finnhub `/quote` endpoint.
    Finnhub,
    /// Polygon.io snapshot endpoint.
    Polygon,
    /// Any HTTP endpoint returning `{ "price": ... }`-ish JSON for a symbol.
    Generic,
}

/// Runtime-editable connection settings for the chosen provider.
///
/// Set via `PUT /api/v1/settings/provider`; the API key is write-only and is
/// never echoed back in full.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    #[serde(default)]
    pub api_key: Option<String>,
    /// Base URL / webhook endpoint (required for `generic`, optional override
    /// for the named providers).
    #[serde(default)]
    pub base_url: Option<String>,
    /// When false, the server always serves the built-in dataset.
    #[serde(default)]
    pub enabled: bool,
}

/// A public-facing user record (never includes the password hash).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub email: String,
}

/// A single synthetic news headline.
///
/// `category` is one of `"markets"`, `"earnings"`, `"analyst"`, `"insider"`,
/// or `"general"`. `symbol` is `None` for broad-market items.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsItem {
    pub id: String,
    /// Publication timestamp, Unix epoch seconds.
    pub ts: i64,
    pub symbol: Option<Symbol>,
    pub headline: String,
    pub source: String,
    pub url: String,
    pub category: String,
}

/// A synthetic insider transaction (Form 4 style).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InsiderTrade {
    pub symbol: Symbol,
    pub insider_name: String,
    /// Relationship to the issuer, e.g. `CEO`, `CFO`, `Director`, `10% Owner`.
    pub relation: String,
    /// `"Buy"` or `"Sell"`.
    pub transaction: String,
    pub shares: i64,
    pub price: f64,
    /// `shares as f64 * price`, in dollars.
    pub value: f64,
    /// Transaction timestamp, Unix epoch seconds.
    pub ts: i64,
}

/// A synthetic analyst rating action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalystRating {
    pub symbol: Symbol,
    /// Issuing research firm, e.g. `Morgan Stanley`.
    pub firm: String,
    /// `"Upgrade"`, `"Downgrade"`, `"Initiated"`, or `"Reiterated"`.
    pub action: String,
    /// Resulting rating, e.g. `Buy`, `Hold`, `Sell`, `Overweight`.
    pub rating: String,
    pub price_target: Option<f64>,
    /// Rating timestamp, Unix epoch seconds.
    pub ts: i64,
}

/// A single synthetic option contract (one strike / expiry / side).
///
/// Pricing is *illustrative only* — a crude intrinsic-plus-time-value sketch,
/// not a real options-pricing model. See `finviz_core::derivatives` for the
/// (approximate) formulas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptionContract {
    /// OCC-style contract id, e.g. `AAPL250117C00210000`.
    pub contract: String,
    /// `"call"` or `"put"`.
    pub kind: String,
    pub strike: f64,
    /// Expiry date, `YYYY-MM-DD`.
    pub expiry: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub open_interest: i64,
    /// Implied volatility as a fraction (e.g. `0.35` == 35%).
    pub implied_vol: f64,
    /// Rough option delta: calls in `0..1`, puts in `-1..0`, ATM ≈ ±0.5.
    pub delta: f64,
}

/// A synthetic option chain for one underlying across several expiries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptionChain {
    pub symbol: Symbol,
    pub underlying_price: f64,
    /// The distinct expiry dates present in `contracts`, ascending.
    pub expiries: Vec<String>,
    pub contracts: Vec<OptionContract>,
}

/// A single holding inside a synthetic ETF.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EtfHolding {
    pub symbol: Symbol,
    pub name: String,
    /// Portfolio weight, in percent (the holdings sum to ≈ 100).
    pub weight: f64,
}

/// A synthetic ETF profile: descriptive metadata plus its top holdings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EtfProfile {
    pub symbol: Symbol,
    pub name: String,
    pub holdings: Vec<EtfHolding>,
    /// Annual expense ratio, in percent (e.g. `0.09`).
    pub expense_ratio: f64,
    /// Assets under management, in dollars.
    pub aum: f64,
    /// Broad category label, e.g. `Large Blend` or `Technology`.
    pub category: String,
}

/// One row on a synthetic market board (futures, forex or crypto).
///
/// A single reusable shape covers all three boards: `group` carries the
/// board-specific bucket label (e.g. `Indices`, `Energy`, `Metals`,
/// `Agriculture` for futures; `Major`/`Minor` for forex; `Crypto` for crypto).
/// Prices are illustrative, not real market data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketAsset {
    pub symbol: Symbol,
    pub name: String,
    /// Board-specific bucket label, e.g. `Indices`, `Energy`, `Major`, `Crypto`.
    pub group: String,
    /// Last price (futures price, FX exchange rate, or crypto price in USD).
    pub price: f64,
    /// Absolute change versus the prior session, in price units.
    pub change: f64,
    /// Percent change versus the prior session.
    pub change_pct: f64,
    /// Trailing one-week performance, in percent.
    pub perf_week: f64,
    /// Trailing one-month performance, in percent.
    pub perf_month: f64,
}

/// A price/metric alert: a screener expression evaluated against one symbol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub id: String,
    pub symbol: Symbol,
    /// A screener filter expression, e.g. `price > 250`.
    pub query: String,
    #[serde(default)]
    pub note: String,
}
