//! `/api/v1/market-data/*` — quotes, bars, fundamentals, instruments.

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::{Bar, Fundamentals, Instrument, Interval, ProviderKind, Quote};
use serde::Deserialize;

use crate::error::{ApiResult, AppError};
use crate::providers;

pub async fn instruments(State(state): State<AppState>) -> Json<Vec<Instrument>> {
    Json(state.instruments().to_vec())
}

pub async fn quote(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<Quote>> {
    // When a live provider is configured, try it first and fall back to the
    // seeded dataset if the upstream call fails.
    let cfg = state.provider_config();
    if cfg.enabled && cfg.kind != ProviderKind::Mock {
        match providers::fetch_quote(&cfg, &symbol).await {
            Ok(q) => return Ok(Json(q)),
            Err(e) => {
                tracing::warn!(error = %e, symbol = %symbol, "live quote failed; serving seed");
            }
        }
    }
    state
        .quote(&symbol)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no quote for symbol `{symbol}`")))
}

pub async fn fundamentals(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<Fundamentals>> {
    state
        .fundamentals(&symbol)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no fundamentals for symbol `{symbol}`")))
}

#[derive(Debug, Deserialize)]
pub struct BarsQuery {
    interval: Option<String>,
    limit: Option<usize>,
}

pub async fn bars(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQuery>,
) -> ApiResult<Json<Vec<Bar>>> {
    let interval = match q.interval.as_deref() {
        Some(s) => Interval::from_str(s).map_err(AppError::BadRequest)?,
        None => Interval::D1,
    };
    let limit = q.limit.unwrap_or(120);
    state
        .bars(&symbol, interval, limit)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("no bars for symbol `{symbol}`")))
}
