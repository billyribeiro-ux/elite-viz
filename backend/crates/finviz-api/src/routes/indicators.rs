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

fn load_bars(state: &AppState, symbol: &str, limit: usize) -> ApiResult<Vec<finviz_types::Bar>> {
    state
        .bars(symbol, Interval::D1, limit)
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

pub async fn ema(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<IndicatorSeries>> {
    let period = q.period.unwrap_or(20).max(1);
    let closes = load_closes(&state, &symbol, q.limit.unwrap_or(200))?;
    if closes.is_empty() {
        return Err(AppError::BadRequest(
            "need at least one bar to compute EMA".to_string(),
        ));
    }

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "ema",
        period,
        points: ema_series(&closes, period),
    }))
}

/// Exponential moving average over `(ts, close)` samples using the standard
/// `2/(period+1)` smoothing factor, seeded with the first close. Output has one
/// point per input sample (the first equals the seed). Empty input => empty.
fn ema_series(closes: &[(i64, f64)], period: usize) -> Vec<Point> {
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    closes
        .iter()
        .map(|&(ts, c)| {
            let ema = match prev {
                Some(p) => c * k + p * (1.0 - k),
                None => c,
            };
            prev = Some(ema);
            Point {
                ts,
                value: round2(ema),
            }
        })
        .collect()
}

/// Raw (unrounded) EMA values, used internally by MACD to avoid compounding
/// rounding error across the chained EMAs.
fn ema_values(values: &[f64], period: usize) -> Vec<f64> {
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev: Option<f64> = None;
    values
        .iter()
        .map(|&c| {
            let ema = match prev {
                Some(p) => c * k + p * (1.0 - k),
                None => c,
            };
            prev = Some(ema);
            ema
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct MacdPoint {
    pub ts: i64,
    pub macd: f64,
    pub signal: f64,
    pub hist: f64,
}

#[derive(Debug, Serialize)]
pub struct MacdSeries {
    pub symbol: String,
    pub indicator: &'static str,
    pub points: Vec<MacdPoint>,
}

pub async fn macd(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<MacdSeries>> {
    let closes = load_closes(&state, &symbol, q.limit.unwrap_or(200))?;
    if closes.is_empty() {
        return Err(AppError::BadRequest(
            "need at least one bar to compute MACD".to_string(),
        ));
    }

    Ok(Json(MacdSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "macd",
        points: macd_series(&closes),
    }))
}

/// MACD over `(ts, close)` samples: line = EMA(12) - EMA(26), signal = EMA(9)
/// of the MACD line, histogram = line - signal. One point per input sample.
fn macd_series(closes: &[(i64, f64)]) -> Vec<MacdPoint> {
    let prices: Vec<f64> = closes.iter().map(|&(_, c)| c).collect();
    let ema12 = ema_values(&prices, 12);
    let ema26 = ema_values(&prices, 26);
    let macd_line: Vec<f64> = ema12.iter().zip(&ema26).map(|(a, b)| a - b).collect();
    let signal = ema_values(&macd_line, 9);

    closes
        .iter()
        .enumerate()
        .map(|(i, &(ts, _))| {
            let m = macd_line[i];
            let s = signal[i];
            MacdPoint {
                ts,
                macd: round2(m),
                signal: round2(s),
                hist: round2(m - s),
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct BandPoint {
    pub ts: i64,
    pub middle: f64,
    pub upper: f64,
    pub lower: f64,
}

#[derive(Debug, Serialize)]
pub struct BandSeries {
    pub symbol: String,
    pub indicator: &'static str,
    pub period: usize,
    pub points: Vec<BandPoint>,
}

pub async fn bbands(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<BandSeries>> {
    let period = q.period.unwrap_or(20).max(1);
    let closes = load_closes(&state, &symbol, q.limit.unwrap_or(200))?;
    if closes.len() < period {
        return Err(AppError::BadRequest(format!(
            "need at least {period} bars to compute Bollinger Bands"
        )));
    }

    Ok(Json(BandSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "bbands",
        period,
        points: bbands_series(&closes, period),
    }))
}

/// Bollinger Bands over `(ts, close)` samples: middle = SMA(period), upper /
/// lower = middle +/- 2 * population standard deviation of the window. Each
/// point is stamped with the most recent close in its window. Fewer than
/// `period` samples => empty series.
fn bbands_series(closes: &[(i64, f64)], period: usize) -> Vec<BandPoint> {
    closes
        .windows(period)
        .filter_map(|window| {
            let n = period as f64;
            let mean = window.iter().map(|&(_, c)| c).sum::<f64>() / n;
            let var = window.iter().map(|&(_, c)| (c - mean).powi(2)).sum::<f64>() / n;
            let sd = var.sqrt();
            window.last().map(|&(ts, _)| BandPoint {
                ts,
                middle: round2(mean),
                upper: round2(mean + 2.0 * sd),
                lower: round2(mean - 2.0 * sd),
            })
        })
        .collect()
}

pub async fn atr(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> ApiResult<Json<IndicatorSeries>> {
    let period = q.period.unwrap_or(14).max(1);
    let bars = load_bars(&state, &symbol, q.limit.unwrap_or(200))?;
    if bars.len() <= period {
        return Err(AppError::BadRequest(format!(
            "need more than {period} bars to compute ATR"
        )));
    }

    Ok(Json(IndicatorSeries {
        symbol: symbol.to_ascii_uppercase(),
        indicator: "atr",
        period,
        points: atr_series(&bars, period),
    }))
}

/// Wilder's Average True Range over OHLC bars. True range uses the prior close,
/// so the first bar is consumed as the seed reference. The caller guarantees
/// `bars.len() > period`, so the seed window exists and output is non-empty.
fn atr_series(bars: &[finviz_types::Bar], period: usize) -> Vec<Point> {
    // True ranges, indexed against bars[1..] (each needs a previous close).
    let trs: Vec<(i64, f64)> = bars
        .windows(2)
        .map(|w| {
            let prev_close = w[0].close;
            let b = &w[1];
            let tr = (b.high - b.low)
                .max((b.high - prev_close).abs())
                .max((b.low - prev_close).abs());
            (b.ts, tr)
        })
        .collect();

    if trs.len() < period {
        return Vec::new();
    }

    // Seed ATR = simple average of the first `period` true ranges.
    let mut atr = trs[..period].iter().map(|&(_, tr)| tr).sum::<f64>() / period as f64;
    let mut points = Vec::with_capacity(trs.len() - period + 1);
    points.push(Point {
        ts: trs[period - 1].0,
        value: round2(atr),
    });
    for &(ts, tr) in &trs[period..] {
        atr = (atr * (period as f64 - 1.0) + tr) / period as f64;
        points.push(Point {
            ts,
            value: round2(atr),
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
    fn ema_seeds_with_first_close_and_smooths() {
        // period=1 => k=1 => EMA tracks the close exactly.
        let closes = series(&[2.0, 4.0, 6.0]);
        let out = ema_series(&closes, 1);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].value, 2.0);
        assert_eq!(out[1].value, 4.0);
        assert_eq!(out[2].value, 6.0);

        // period=2 => k=2/3. First point is the seed (first close).
        let out2 = ema_series(&series(&[1.0, 2.0]), 2);
        assert_eq!(out2[0].value, 1.0);
        // 2*(2/3) + 1*(1/3) = 5/3 = 1.666.. -> rounded 1.67.
        assert_eq!(out2[1].value, 1.67);
    }

    #[test]
    fn macd_is_zero_on_constant_prices() {
        // Constant prices => EMA12 == EMA26 => macd line 0 => signal 0 => hist 0.
        let closes = series(&[5.0; 40]);
        let out = macd_series(&closes);
        assert_eq!(out.len(), 40);
        assert!(out
            .iter()
            .all(|p| p.macd == 0.0 && p.signal == 0.0 && p.hist == 0.0));
    }

    #[test]
    fn bbands_collapse_on_constant_prices() {
        // Zero variance => upper == middle == lower == the constant value.
        let closes = series(&[3.0; 5]);
        let out = bbands_series(&closes, 3);
        assert_eq!(out.len(), 3);
        for p in &out {
            assert_eq!((p.middle, p.upper, p.lower), (3.0, 3.0, 3.0));
        }
    }

    #[test]
    fn bbands_known_window() {
        // window [2,4,6]: mean=4, popvar = (4+0+4)/3 = 2.666.., sd=1.633..
        let closes = series(&[2.0, 4.0, 6.0]);
        let out = bbands_series(&closes, 3);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].middle, 4.0);
        // 4 +/- 2*1.63299 = 4 +/- 3.26599 -> 7.27 / 0.73.
        assert_eq!(out[0].upper, 7.27);
        assert_eq!(out[0].lower, 0.73);
    }

    fn bars(highs_lows_closes: &[(f64, f64, f64)]) -> Vec<finviz_types::Bar> {
        highs_lows_closes
            .iter()
            .enumerate()
            .map(|(i, &(high, low, close))| finviz_types::Bar {
                ts: i as i64,
                open: close,
                high,
                low,
                close,
                volume: 0,
            })
            .collect()
    }

    #[test]
    fn atr_known_input() {
        // Constant 2-wide range with closes mid-range => every TR == 2 => ATR 2.
        let b = bars(&[
            (11.0, 9.0, 10.0),
            (11.0, 9.0, 10.0),
            (11.0, 9.0, 10.0),
            (11.0, 9.0, 10.0),
            (11.0, 9.0, 10.0),
        ]);
        let out = atr_series(&b, 2);
        assert!(!out.is_empty());
        assert!(out.iter().all(|p| (p.value - 2.0).abs() < 1e-9));
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
