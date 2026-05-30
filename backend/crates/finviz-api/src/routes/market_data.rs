//! `/api/v1/market-data/*` — quotes, bars, fundamentals, instruments.

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::{Bar, Fundamentals, Instrument, Interval, Quote};
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

pub async fn instruments(State(state): State<AppState>) -> Json<Vec<Instrument>> {
    Json(state.instruments().to_vec())
}

pub async fn quote(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<Quote>> {
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
