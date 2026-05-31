//! `/api/v1/options/{symbol}` — deterministic, synthetic option chains.
//!
//! Pricing is illustrative only (see `finviz_core::derivatives`), not a real
//! options-pricing model.

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::OptionChain;
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

/// Defaults for the chain shape (the generator clamps to its own maxima).
const DEFAULT_EXPIRIES: usize = 4;
const DEFAULT_STRIKES: usize = 8;

#[derive(Debug, Deserialize)]
pub struct ChainQuery {
    /// Number of monthly expiries to generate.
    expiries: Option<usize>,
    /// Number of strikes on each side of the money.
    strikes: Option<usize>,
}

/// `GET /api/v1/options/{symbol}?expiries=&strikes=` — synthetic option chain.
/// 404 for an unknown symbol; parameters are clamped by the generator.
pub async fn chain(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<ChainQuery>,
) -> ApiResult<Json<OptionChain>> {
    let expiries = q.expiries.unwrap_or(DEFAULT_EXPIRIES);
    let strikes = q.strikes.unwrap_or(DEFAULT_STRIKES);
    match state.option_chain(&symbol, expiries, strikes) {
        Some(chain) => Ok(Json(chain)),
        None => Err(AppError::NotFound(format!("unknown symbol `{symbol}`"))),
    }
}
