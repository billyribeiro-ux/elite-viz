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

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "sma",
        period,
        points: sma_series(&closes, period),
    }))
}

/// Simple moving average over `(ts, close)` samples. Each output point is
/// stamped with the timestamp of the most recent close in its window. Returns
/// an empty series when there are fewer than `period` samples.
fn sma_series(closes: &[(i64, f64)], period: usize) -> Vec<Point> {
    closes
        .windows(period)
        .filter_map(|window| {
            let avg = window.iter().map(|&(_, c)| c).sum::<f64>() / period as f64;
            window.last().map(|&(ts, _)| Point {
                ts,
                value: round2(avg),
            })
        })
        .collect()
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

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "rsi",
        period,
        points: rsi_series(&closes, period),
    }))
}

/// Wilder's RSI over `(ts, close)` samples. The caller guarantees
/// `closes.len() > period` (so the seed average has at least one delta and the
/// output is non-empty). A zero average loss yields an RSI of 100.
fn rsi_series(closes: &[(i64, f64)], period: usize) -> Vec<Point> {
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
        let (gain, loss) = if diff >= 0.0 {
            (diff, 0.0)
        } else {
            (0.0, -diff)
        };
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
    points
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[f64]) -> Vec<(i64, f64)> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as i64, v))
            .collect()
    }

    #[test]
    fn sma_matches_hand_computed_windows() {
        let closes = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let out = sma_series(&closes, 3);
        // windows: [1,2,3]=2, [2,3,4]=3, [3,4,5]=4, stamped with last ts.
        assert_eq!(out.len(), 3);
        assert_eq!((out[0].ts, out[0].value), (2, 2.0));
        assert_eq!((out[1].ts, out[1].value), (3, 3.0));
        assert_eq!((out[2].ts, out[2].value), (4, 4.0));
    }

    #[test]
    fn sma_empty_when_fewer_samples_than_period() {
        assert!(sma_series(&series(&[1.0, 2.0]), 3).is_empty());
    }

    #[test]
    fn rsi_is_100_when_prices_only_rise() {
        // No losses => avg_loss is 0 => RSI saturates at 100.
        let closes = series(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let out = rsi_series(&closes, 3);
        assert_eq!(out.len(), closes.len() - (3 + 1));
        assert!(out.iter().all(|p| (p.value - 100.0).abs() < 1e-9));
    }

    #[test]
    fn rsi_around_50_for_balanced_alternating_moves() {
        // Symmetric +1/-1 zig-zag keeps avg gain ~= avg loss, so RSI hovers
        // near 50 and always stays within (0, 100).
        let closes = series(&[10.0, 11.0, 10.0, 11.0, 10.0, 11.0, 10.0, 11.0]);
        let out = rsi_series(&closes, 3);
        assert!(!out.is_empty());
        for p in &out {
            assert!(p.value > 0.0 && p.value < 100.0, "rsi out of range: {p:?}");
        }
    }
}
