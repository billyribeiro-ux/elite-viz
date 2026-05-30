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
/// symbol in a single flat record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScreenerRow {
    pub symbol: Symbol,
    pub name: String,
    pub sector: String,
    pub industry: String,
    pub exchange: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub volume: i64,
    pub market_cap: f64,
    pub pe: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub beta: Option<f64>,
}

/// A named collection of symbols a user is tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Watchlist {
    pub id: String,
    pub name: String,
    pub symbols: Vec<Symbol>,
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
