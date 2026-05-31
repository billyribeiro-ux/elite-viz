//! `/api/v1/patterns/*` — classic chart-pattern detection from bar history.
//!
//! Pattern detection itself lives in the pure `finviz-patterns` crate; this
//! module only loads the symbol's bars from [`AppState`] and hands them off.

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_patterns::Pattern;
use finviz_types::Interval;
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

#[derive(Debug, Deserialize)]
pub struct PatternQuery {
    /// Number of trailing daily bars to analyse (default 200).
    limit: Option<usize>,
}

/// `GET /api/v1/patterns/{symbol}` — detect classic chart patterns over the
/// symbol's recent daily bars. Returns `404` for an unknown symbol.
pub async fn detect(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<PatternQuery>,
) -> ApiResult<Json<Vec<Pattern>>> {
    let limit = q.limit.unwrap_or(200).clamp(1, 2_000);
    let bars = state
        .bars(&symbol, Interval::D1, limit)
        .ok_or_else(|| AppError::NotFound(format!("no bars for symbol `{symbol}`")))?;

    Ok(Json(finviz_patterns::detect(&bars)))
}
