//! `/api/v1/indicators/*` — technical indicators computed from bar history.

use axum::extract::{Path, Query, State};
use axum::Json;
use finviz_core::AppState;
use finviz_types::Interval;
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};

#[derive(Debug, Deserialize)]
pub struct IndicatorQuery {
    period: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Point {
    pub ts: i64,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct IndicatorSeries {
    pub symbol: String,
    pub indicator: &'static str,
    pub period: usize,
    pub points: Vec<Point>,
}

fn load_closes(state: &AppState, symbol: &str, limit: usize) -> ApiResult<Vec<(i64, f64)>> {
    state
        .bars(symbol, Interval::D1, limit)
        .map(|bars| bars.into_iter().map(|b| (b.ts, b.close)).collect())
        .ok_or_else(|| AppError::NotFound(format!("no bars for symbol `{symbol}`")))
}

pub async fn sma(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<IndicatorSeries>> {
    let period = q.period.unwrap_or(20).max(1);
    let closes = load_closes(&state, &symbol, q.limit.unwrap_or(200))?;

    let mut points = Vec::new();
    for window in closes.windows(period) {
        let avg = window.iter().map(|(_, c)| *c).sum::<f64>() / period as f64;
        points.push(Point {
            ts: window.last().unwrap().0,
            value: round2(avg),
        });
    }

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "sma",
        period,
        points,
    }))
}

pub async fn rsi(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<IndicatorSeries>> {
    let period = q.period.unwrap_or(14).max(1);
    let closes = load_closes(&state, &symbol, q.limit.unwrap_or(200))?;
    if closes.len() <= period {
        return Err(AppError::BadRequest(format!(
            "need more than {period} bars to compute RSI"
        )));
    }

    // Wilder's RSI.
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = closes[i].1 - closes[i - 1].1;
        if diff >= 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    let mut points = Vec::new();
    for i in (period + 1)..closes.len() {
        let diff = closes[i].1 - closes[i - 1].1;
        let (gain, loss) = if diff >= 0.0 { (diff, 0.0) } else { (0.0, -diff) };
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        points.push(Point {
            ts: closes[i].0,
            value: round2(rsi),
        });
    }

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "rsi",
        period,
        points,
    }))
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
