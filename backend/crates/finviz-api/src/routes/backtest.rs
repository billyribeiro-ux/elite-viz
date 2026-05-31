//! `/api/v1/backtest` — run a strategy backtest over a symbol's daily history,
//! plus `/api/v1/backtest/rules` describing the available rule catalog for UIs.
//!
//! The heavy lifting lives in the pure [`finviz_backtest`] engine; this module
//! only validates input, loads bars, and serializes the result.

use axum::extract::State;
use axum::Json;
use finviz_backtest::{run_backtest, BacktestResult, Strategy};
use finviz_core::AppState;
use finviz_types::Interval;
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};

/// Default number of daily bars of history to load.
const DEFAULT_LIMIT: usize = 500;
/// Maximum number of bars a single request may pull.
const MAX_LIMIT: usize = 5_000;

#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
    pub symbol: String,
    pub strategy: Strategy,
    /// Bars of daily history to backtest over. Defaults to `DEFAULT_LIMIT`,
    /// clamped to `1..=MAX_LIMIT`.
    pub limit: Option<usize>,
}

/// `POST /api/v1/backtest`
pub async fn run(
    State(state): State<AppState>,
    Json(req): Json<BacktestRequest>,
) -> ApiResult<Json<BacktestResult>> {
    req.strategy
        .validate()
        .map_err(|e| AppError::BadRequest(format!("invalid strategy: {e}")))?;

    let limit = req.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    let bars = state
        .bars(&req.symbol, Interval::D1, limit)
        .ok_or_else(|| AppError::NotFound(format!("no bars for symbol `{}`", req.symbol)))?;

    Ok(Json(run_backtest(&bars, &req.strategy)))
}

// ---- static rule catalog for form-building UIs ----

#[derive(Debug, Serialize)]
pub struct ParamSpec {
    pub name: &'static str,
    /// One of `"integer"`, `"boolean"`, `"number"`.
    pub kind: &'static str,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RuleSpec {
    /// Matches the serde tag used by `finviz_backtest::Rule` (`kind`).
    pub kind: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub params: Vec<ParamSpec>,
}

#[derive(Debug, Serialize)]
pub struct ExitSpec {
    pub field: &'static str,
    pub kind: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RuleCatalog {
    pub rules: Vec<RuleSpec>,
    pub exits: Vec<ExitSpec>,
}

/// `GET /api/v1/backtest/rules`
pub async fn rules() -> Json<RuleCatalog> {
    Json(RuleCatalog {
        rules: vec![
            RuleSpec {
                kind: "price_vs_sma",
                label: "Price vs SMA",
                description: "Close above/below its simple moving average. Entry fires on the cross.",
                params: vec![
                    ParamSpec {
                        name: "period",
                        kind: "integer",
                        required: true,
                        description: "SMA lookback in bars (> 0).",
                    },
                    ParamSpec {
                        name: "above",
                        kind: "boolean",
                        required: true,
                        description: "true = close above SMA, false = below.",
                    },
                ],
            },
            RuleSpec {
                kind: "sma_cross",
                label: "SMA Cross",
                description: "Fast SMA crosses above (golden) or below (death) the slow SMA.",
                params: vec![
                    ParamSpec {
                        name: "fast",
                        kind: "integer",
                        required: true,
                        description: "Fast SMA period (> 0, < slow).",
                    },
                    ParamSpec {
                        name: "slow",
                        kind: "integer",
                        required: true,
                        description: "Slow SMA period (> fast).",
                    },
                    ParamSpec {
                        name: "above",
                        kind: "boolean",
                        required: false,
                        description: "true = golden cross (fast over slow), false = death cross. Defaults true.",
                    },
                ],
            },
            RuleSpec {
                kind: "rsi_threshold",
                label: "RSI Threshold",
                description: "RSI below and/or above a level (any-of). E.g. RSI < 30 for oversold entries.",
                params: vec![
                    ParamSpec {
                        name: "period",
                        kind: "integer",
                        required: true,
                        description: "RSI lookback in bars (> 0).",
                    },
                    ParamSpec {
                        name: "below",
                        kind: "number",
                        required: false,
                        description: "Fire when RSI is below this value (0..=100).",
                    },
                    ParamSpec {
                        name: "above",
                        kind: "number",
                        required: false,
                        description: "Fire when RSI is above this value (0..=100).",
                    },
                ],
            },
        ],
        exits: vec![
            ExitSpec {
                field: "time_exit",
                kind: "integer",
                description: "Hold at most N bars, then exit at that bar's close.",
            },
            ExitSpec {
                field: "stop_loss_pct",
                kind: "number",
                description: "Exit if close drops this percent below entry price (0 < pct < 100).",
            },
        ],
    })
}
