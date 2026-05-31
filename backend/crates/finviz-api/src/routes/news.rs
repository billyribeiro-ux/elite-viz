//! `/api/v1/news` and `/api/v1/market-data/{insider,ratings}/{symbol}` —
//! synthetic, deterministic news feed plus per-ticker insider trades and
//! analyst ratings.

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::{AnalystRating, InsiderTrade, NewsItem};
use serde::Deserialize;

use crate::error::{ApiResult, AppError};

/// Default and maximum item counts for the news feed.
const NEWS_DEFAULT_LIMIT: usize = 30;
const NEWS_MAX_LIMIT: usize = 200;
/// Default and maximum counts for per-symbol enrichment endpoints.
const DETAIL_DEFAULT_LIMIT: usize = 20;
const DETAIL_MAX_LIMIT: usize = 50;

#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    symbol: Option<String>,
    limit: Option<usize>,
}

/// `GET /api/v1/news?symbol=&limit=` — per-ticker stream when `symbol` is
/// supplied, otherwise a merged broad-market feed. Always newest-first.
pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<NewsQuery>,
) -> ApiResult<Json<Vec<NewsItem>>> {
    let limit = q
        .limit
        .unwrap_or(NEWS_DEFAULT_LIMIT)
        .clamp(1, NEWS_MAX_LIMIT);
    match q.symbol.as_deref().filter(|s| !s.is_empty()) {
        Some(sym) => {
            // An explicit unknown ticker is a 404; the merged feed never 404s.
            if state.quote(sym).is_none() {
                return Err(AppError::NotFound(format!("unknown symbol `{sym}`")));
            }
            Ok(Json(state.news(Some(sym), limit)))
        }
        None => Ok(Json(state.news(None, limit))),
    }
}

#[derive(Debug, Deserialize)]
pub struct DetailQuery {
    limit: Option<usize>,
}

/// `GET /api/v1/market-data/insider/{symbol}` — synthetic insider trades.
pub async fn insider(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<DetailQuery>,
) -> ApiResult<Json<Vec<InsiderTrade>>> {
    if state.quote(&symbol).is_none() {
        return Err(AppError::NotFound(format!("unknown symbol `{symbol}`")));
    }
    let limit = q
        .limit
        .unwrap_or(DETAIL_DEFAULT_LIMIT)
        .clamp(1, DETAIL_MAX_LIMIT);
    Ok(Json(state.insider_trades(&symbol, limit)))
}

/// `GET /api/v1/market-data/ratings/{symbol}` — synthetic analyst ratings.
pub async fn ratings(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<DetailQuery>,
) -> ApiResult<Json<Vec<AnalystRating>>> {
    if state.quote(&symbol).is_none() {
        return Err(AppError::NotFound(format!("unknown symbol `{symbol}`")));
    }
    let limit = q
        .limit
        .unwrap_or(DETAIL_DEFAULT_LIMIT)
        .clamp(1, DETAIL_MAX_LIMIT);
    Ok(Json(state.analyst_ratings(&symbol, limit)))
}
